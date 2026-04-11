use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::lane_events::{LaneEvent, LaneEventKind, LaneEventPayload};
use crate::task_packet::{TaskAssignment, TaskFailure, TaskPacket, TaskStatus};
use crate::trust_resolver::TrustResolution;
use crate::worker_boot::WorkerBootState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TaskRegistrySnapshot {
    pub active_tasks: Vec<TaskPacket>,
    pub completed_tasks: Vec<String>,
    pub failed_tasks: Vec<String>,
    pub lane_events: Vec<LaneEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskRegistryError {
    DuplicateTask {
        task_id: String,
    },
    UnknownTask {
        task_id: String,
    },
    MissingLaneAssignment {
        task_id: String,
    },
    InvalidStatusTransition {
        task_id: String,
        from: TaskStatus,
        to: TaskStatus,
    },
}

#[derive(Debug, Default)]
struct TaskRegistryState {
    next_sequence: u64,
    tasks: BTreeMap<String, TaskPacket>,
    completed: BTreeSet<String>,
    failed: BTreeSet<String>,
    lane_events: Vec<LaneEvent>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskRegistry {
    inner: Arc<RwLock<TaskRegistryState>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, mut packet: TaskPacket) -> Result<TaskPacket, TaskRegistryError> {
        let mut state = self.inner.write().expect("task registry write lock");
        if state.tasks.contains_key(&packet.task_id) {
            return Err(TaskRegistryError::DuplicateTask {
                task_id: packet.task_id,
            });
        }

        packet.status = TaskStatus::Queued;
        let lane_id = packet
            .assignment
            .as_ref()
            .map(|assignment| assignment.lane_id.clone())
            .ok_or_else(|| TaskRegistryError::MissingLaneAssignment {
                task_id: packet.task_id.clone(),
            })?;

        push_lane_event(
            &mut state,
            lane_id,
            Some(packet.task_id.clone()),
            None,
            LaneEventKind::TaskQueued,
            format!("queued {}", packet.summary),
            LaneEventPayload::TaskQueued {
                summary: packet.summary.clone(),
                priority: packet.priority.clone(),
            },
        );

        state.tasks.insert(packet.task_id.clone(), packet.clone());
        Ok(packet)
    }

    pub fn get(&self, task_id: &str) -> Option<TaskPacket> {
        let state = self.inner.read().expect("task registry read lock");
        state.tasks.get(task_id).cloned()
    }

    pub fn lane_events(&self) -> Vec<LaneEvent> {
        let state = self.inner.read().expect("task registry read lock");
        state.lane_events.clone()
    }

    pub fn snapshot(&self) -> TaskRegistrySnapshot {
        let state = self.inner.read().expect("task registry read lock");
        TaskRegistrySnapshot {
            active_tasks: state
                .tasks
                .values()
                .filter(|task| {
                    !matches!(
                        task.status,
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                    )
                })
                .cloned()
                .collect(),
            completed_tasks: state.completed.iter().cloned().collect(),
            failed_tasks: state.failed.iter().cloned().collect(),
            lane_events: state.lane_events.clone(),
        }
    }

    pub fn record_trust_resolution(
        &self,
        task_id: &str,
        resolution: TrustResolution,
    ) -> Result<TaskPacket, TaskRegistryError> {
        let mut state = self.inner.write().expect("task registry write lock");
        let (task_snapshot, lane_event) = {
            let task = state.tasks.get_mut(task_id).ok_or_else(|| TaskRegistryError::UnknownTask {
                task_id: task_id.to_string(),
            })?;
            let lane_id = task_lane_id(task)?;

            match &mut task.trust {
                Some(gate) => gate.resolution = Some(resolution.clone()),
                None => {
                    return Err(TaskRegistryError::InvalidStatusTransition {
                        task_id: task_id.to_string(),
                        from: task.status.clone(),
                        to: TaskStatus::BlockedByTrust,
                    });
                }
            }

            let lane_event = if resolution.is_allowed() {
                task.status = TaskStatus::Queued;
                None
            } else {
                task.status = TaskStatus::BlockedByTrust;
                Some((
                    lane_id,
                    Some(task.task_id.clone()),
                    task.assignment
                        .as_ref()
                        .and_then(|assignment| assignment.worker_id.clone()),
                    resolution.reason.clone(),
                    LaneEventPayload::TrustBlocked {
                        resolution: resolution.clone(),
                    },
                ))
            };

            (task.clone(), lane_event)
        };

        if let Some((lane_id, task_id, worker_id, detail, payload)) = lane_event {
            push_lane_event(
                &mut state,
                lane_id,
                task_id,
                worker_id,
                LaneEventKind::TrustBlocked,
                detail,
                payload,
            );
        }

        Ok(task_snapshot)
    }

    pub fn assign_worker(
        &self,
        task_id: &str,
        worker_id: impl Into<String>,
        session_id: Option<String>,
    ) -> Result<TaskPacket, TaskRegistryError> {
        let worker_id = worker_id.into();
        let mut state = self.inner.write().expect("task registry write lock");
        let (task_snapshot, lane_id, event_task_id) = {
            let task = state.tasks.get_mut(task_id).ok_or_else(|| TaskRegistryError::UnknownTask {
                task_id: task_id.to_string(),
            })?;
            ensure_transition(task, TaskStatus::Assigned)?;

            let lane_id = task_lane_id(task)?;
            let assignment = task
                .assignment
                .get_or_insert_with(|| TaskAssignment {
                    lane_id: lane_id.clone(),
                    worker_id: None,
                    session_id: None,
                });
            assignment.worker_id = Some(worker_id.clone());
            assignment.session_id = session_id;
            task.status = TaskStatus::Assigned;

            (task.clone(), lane_id, task.task_id.clone())
        };

        push_lane_event(
            &mut state,
            lane_id,
            Some(event_task_id),
            Some(worker_id.clone()),
            LaneEventKind::WorkerAssigned,
            format!("assigned to {}", worker_id),
            LaneEventPayload::WorkerAssigned { worker_id },
        );

        Ok(task_snapshot)
    }

    pub fn start_task(&self, task_id: &str) -> Result<TaskPacket, TaskRegistryError> {
        let mut state = self.inner.write().expect("task registry write lock");
        let (task_snapshot, lane_id, event_task_id, worker_id) = {
            let task = state.tasks.get_mut(task_id).ok_or_else(|| TaskRegistryError::UnknownTask {
                task_id: task_id.to_string(),
            })?;
            ensure_transition(task, TaskStatus::Running)?;
            task.status = TaskStatus::Running;
            let lane_id = task_lane_id(task)?;
            let worker_id = task
                .assignment
                .as_ref()
                .and_then(|assignment| assignment.worker_id.clone());

            (task.clone(), lane_id, task.task_id.clone(), worker_id)
        };

        push_lane_event(
            &mut state,
            lane_id,
            Some(event_task_id),
            worker_id.clone(),
            LaneEventKind::TaskStarted,
            "task started",
            LaneEventPayload::TaskStarted { worker_id },
        );

        Ok(task_snapshot)
    }

    pub fn complete_task(&self, task_id: &str) -> Result<TaskPacket, TaskRegistryError> {
        let mut state = self.inner.write().expect("task registry write lock");
        let (task_snapshot, lane_id, event_task_id, worker_id) = {
            let task = state.tasks.get_mut(task_id).ok_or_else(|| TaskRegistryError::UnknownTask {
                task_id: task_id.to_string(),
            })?;
            ensure_transition(task, TaskStatus::Completed)?;
            task.status = TaskStatus::Completed;
            task.failure = None;
            let lane_id = task_lane_id(task)?;
            let worker_id = task
                .assignment
                .as_ref()
                .and_then(|assignment| assignment.worker_id.clone());

            (task.clone(), lane_id, task.task_id.clone(), worker_id)
        };

        push_lane_event(
            &mut state,
            lane_id,
            Some(event_task_id.clone()),
            worker_id.clone(),
            LaneEventKind::TaskCompleted,
            "task completed",
            LaneEventPayload::TaskCompleted { worker_id },
        );
        state.completed.insert(event_task_id);
        Ok(task_snapshot)
    }

    pub fn fail_task(
        &self,
        task_id: &str,
        failure: TaskFailure,
    ) -> Result<TaskPacket, TaskRegistryError> {
        let mut state = self.inner.write().expect("task registry write lock");
        let (task_snapshot, lane_id, event_task_id, worker_id) = {
            let task = state.tasks.get_mut(task_id).ok_or_else(|| TaskRegistryError::UnknownTask {
                task_id: task_id.to_string(),
            })?;
            ensure_transition(task, TaskStatus::Failed)?;
            task.status = TaskStatus::Failed;
            task.failure = Some(failure.clone());
            let lane_id = task_lane_id(task)?;
            let worker_id = task
                .assignment
                .as_ref()
                .and_then(|assignment| assignment.worker_id.clone());

            (task.clone(), lane_id, task.task_id.clone(), worker_id)
        };

        push_lane_event(
            &mut state,
            lane_id,
            Some(event_task_id.clone()),
            worker_id,
            LaneEventKind::TaskFailed,
            failure.message.clone(),
            LaneEventPayload::TaskFailed { failure },
        );
        state.failed.insert(event_task_id);
        Ok(task_snapshot)
    }

    pub fn record_worker_state(
        &self,
        lane_id: impl Into<String>,
        worker_id: impl Into<String>,
        state_name: WorkerBootState,
    ) {
        let lane_id = lane_id.into();
        let worker_id = worker_id.into();
        let mut state = self.inner.write().expect("task registry write lock");
        push_lane_event(
            &mut state,
            lane_id,
            None,
            Some(worker_id.clone()),
            LaneEventKind::WorkerStateChanged,
            format!("worker changed state to {:?}", state_name),
            LaneEventPayload::WorkerStateChanged {
                worker_id,
                state: state_name,
            },
        );
    }
}

fn task_lane_id(task: &TaskPacket) -> Result<String, TaskRegistryError> {
    task.assignment
        .as_ref()
        .map(|assignment| assignment.lane_id.clone())
        .ok_or_else(|| TaskRegistryError::MissingLaneAssignment {
            task_id: task.task_id.clone(),
        })
}

fn ensure_transition(task: &TaskPacket, next: TaskStatus) -> Result<(), TaskRegistryError> {
    if is_valid_transition(&task.status, &next) {
        Ok(())
    } else {
        Err(TaskRegistryError::InvalidStatusTransition {
            task_id: task.task_id.clone(),
            from: task.status.clone(),
            to: next,
        })
    }
}

fn is_valid_transition(current: &TaskStatus, next: &TaskStatus) -> bool {
    if current == next {
        return true;
    }

    match current {
        TaskStatus::Queued => matches!(
            next,
            TaskStatus::BlockedByTrust
                | TaskStatus::Assigned
                | TaskStatus::Cancelled
                | TaskStatus::Failed
        ),
        TaskStatus::BlockedByTrust => matches!(
            next,
            TaskStatus::Queued | TaskStatus::Cancelled | TaskStatus::Failed
        ),
        TaskStatus::Assigned => matches!(
            next,
            TaskStatus::Running | TaskStatus::Cancelled | TaskStatus::Failed
        ),
        TaskStatus::Running => matches!(
            next,
            TaskStatus::Completed | TaskStatus::Cancelled | TaskStatus::Failed
        ),
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => false,
    }
}

fn push_lane_event(
    state: &mut TaskRegistryState,
    lane_id: String,
    task_id: Option<String>,
    worker_id: Option<String>,
    kind: LaneEventKind,
    detail: impl Into<String>,
    payload: LaneEventPayload,
) {
    state.lane_events.push(LaneEvent::new(
        state.next_sequence,
        lane_id,
        task_id,
        worker_id,
        kind,
        detail,
        payload,
    ));
    state.next_sequence += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_packet::{TaskPacket, TaskPriority};
    use crate::trust_resolver::{
        TrustFailureReason, TrustLevel, TrustRequirement, TrustResolver, TrustSubject,
        TrustSubjectKind,
    };

    fn queued_task(task_id: &str) -> TaskPacket {
        TaskPacket::queued(task_id, "review worker output", TaskPriority::High)
            .with_lane("lane-a")
    }

    #[test]
    fn registry_records_lane_events_for_happy_path() {
        let registry = TaskRegistry::new();
        registry.register(queued_task("task-1")).expect("register task");
        registry
            .assign_worker("task-1", "worker-1", Some("session-1".to_string()))
            .expect("assign worker");
        registry.start_task("task-1").expect("start task");
        registry.complete_task("task-1").expect("complete task");

        let snapshot = registry.snapshot();
        assert!(snapshot.active_tasks.is_empty());
        assert_eq!(snapshot.completed_tasks, vec!["task-1".to_string()]);
        assert_eq!(snapshot.lane_events.len(), 4);
        assert_eq!(snapshot.lane_events[0].kind, LaneEventKind::TaskQueued);
        assert_eq!(snapshot.lane_events[1].kind, LaneEventKind::WorkerAssigned);
        assert_eq!(snapshot.lane_events[2].kind, LaneEventKind::TaskStarted);
        assert_eq!(snapshot.lane_events[3].kind, LaneEventKind::TaskCompleted);
    }

    #[test]
    fn registry_blocks_tasks_when_trust_resolution_denies_execution() {
        let registry = TaskRegistry::new();
        registry
            .register(
                queued_task("task-2")
                    .with_trust_requirement(TrustRequirement::at_least(TrustLevel::Trusted)),
            )
            .expect("register gated task");

        let resolution = TrustResolver::deny(
            TrustSubject::new(TrustSubjectKind::Task, "task-2"),
            TrustRequirement::at_least(TrustLevel::Trusted),
            TrustLevel::Restricted,
            TrustFailureReason::InsufficientLevel {
                required: TrustLevel::Trusted,
                actual: TrustLevel::Restricted,
            },
            "worker trust gate denied execution",
        );
        let task = registry
            .record_trust_resolution("task-2", resolution)
            .expect("record trust failure");

        assert_eq!(task.status, TaskStatus::BlockedByTrust);
        assert_eq!(
            registry
                .lane_events()
                .last()
                .map(|event| event.kind.clone()),
            Some(LaneEventKind::TrustBlocked)
        );
    }

    #[test]
    fn registry_records_failure_paths() {
        let registry = TaskRegistry::new();
        registry.register(queued_task("task-3")).expect("register task");
        registry
            .assign_worker("task-3", "worker-3", None)
            .expect("assign");
        registry.start_task("task-3").expect("start");
        let task = registry
            .fail_task(
                "task-3",
                TaskFailure {
                    message: "delegate crashed".to_string(),
                    retryable: true,
                },
            )
            .expect("fail task");

        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(registry.snapshot().failed_tasks, vec!["task-3".to_string()]);
    }

    #[test]
    fn registry_rejects_invalid_status_transition() {
        let registry = TaskRegistry::new();
        registry.register(queued_task("task-4")).expect("register task");

        let error = registry.complete_task("task-4").expect_err("complete should fail");
        assert_eq!(
            error,
            TaskRegistryError::InvalidStatusTransition {
                task_id: "task-4".to_string(),
                from: TaskStatus::Queued,
                to: TaskStatus::Completed,
            }
        );
    }
}
