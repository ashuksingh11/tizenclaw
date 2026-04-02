//! Task scheduler — manages recurring and one-shot scheduled agent tasks.

use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A scheduled task definition.
#[derive(Clone, Debug)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub session_id: String,
    pub interval_secs: u64,
    pub one_shot: bool,
    pub enabled: bool,
}

pub struct TaskScheduler {
    running: Arc<AtomicBool>,
    tasks: Arc<std::sync::Mutex<Vec<ScheduledTask>>>,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            running: Arc::new(AtomicBool::new(false)),
            tasks: Arc::new(std::sync::Mutex::new(vec![])),
        }
    }

    pub fn add_task(&self, task: ScheduledTask) {
        if let Ok(mut tasks) = self.tasks.lock() {
            log::info!("Scheduler: added task '{}' (interval={}s)", task.name, task.interval_secs);
            tasks.push(task);
        }
    }

    pub fn remove_task(&self, task_id: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.retain(|t| t.id != task_id);
        }
    }

    pub fn load_config(&self, dir_path: &str) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() { continue; }
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext != "md" { continue; }
                
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                
                let id = path.file_stem().unwrap().to_string_lossy().to_string();
                let mut name = id.clone();
                let mut interval_secs = 3600;
                let mut one_shot = false;
                let mut enabled = true;
                let mut session_id = "scheduler".to_string();
                let mut prompt = String::new();
                let mut in_frontmatter = false;
                
                for line in content.lines() {
                    let text = line.trim();
                    if text == "---" {
                        in_frontmatter = !in_frontmatter;
                        continue;
                    }
                    if in_frontmatter {
                        if let Some((k, v)) = text.split_once(':') {
                            let val = v.trim().trim_matches(|c| c == '\'' || c == '"');
                            match k.trim() {
                                "interval" | "interval_secs" => interval_secs = val.parse().unwrap_or(3600),
                                "one_shot" => one_shot = val == "true",
                                "enabled" => enabled = val != "false",
                                "name" => name = val.to_string(),
                                "session_id" => session_id = val.to_string(),
                                _ => {}
                            }
                        }
                    } else if !text.is_empty() {
                        prompt.push_str(text);
                        prompt.push('\n');
                    }
                }
                
                let task = ScheduledTask { id, name, prompt: prompt.trim().to_string(), session_id, interval_secs, one_shot, enabled };
                self.add_task(task);
            }
        }
    }

    pub fn start(&self) -> Option<tokio::task::JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) {
            return None;
        }
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let tasks = self.tasks.clone();

        let handle = tokio::spawn(async move {
            log::info!("TaskScheduler started");
            let mut last_run: std::collections::HashMap<String, std::time::Instant> =
                std::collections::HashMap::new();
            
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            while running.load(Ordering::SeqCst) {
                interval.tick().await;

                let task_list = match tasks.lock() {
                    Ok(t) => t.clone(),
                    Err(_) => continue,
                };

                let now = std::time::Instant::now();
                for task in &task_list {
                    if !task.enabled { continue; }

                    let should_run = match last_run.get(&task.id) {
                        Some(last) => now.duration_since(*last).as_secs() >= task.interval_secs,
                        None => true,
                    };

                    if should_run {
                        log::info!("Scheduler: executing task '{}'", task.name);
                        last_run.insert(task.id.clone(), now);

                        if task.one_shot {
                            if let Ok(mut ts) = tasks.lock() {
                                ts.retain(|t| t.id != task.id);
                            }
                        }
                    }
                }
            }
            log::info!("TaskScheduler stopped");
        });

        Some(handle)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task(id: &str) -> ScheduledTask {
        ScheduledTask {
            id: id.into(),
            name: format!("Task {}", id),
            prompt: "Check status".into(),
            session_id: "sched".into(),
            interval_secs: 60,
            one_shot: false,
            enabled: true,
        }
    }

    #[test]
    fn test_add_task() {
        let scheduler = TaskScheduler::new();
        scheduler.add_task(sample_task("t1"));
        let tasks = scheduler.tasks.lock().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "t1");
    }

    #[test]
    fn test_remove_task() {
        let scheduler = TaskScheduler::new();
        scheduler.add_task(sample_task("t1"));
        scheduler.add_task(sample_task("t2"));
        scheduler.remove_task("t1");
        let tasks = scheduler.tasks.lock().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "t2");
    }

    #[test]
    fn test_remove_nonexistent_task() {
        let scheduler = TaskScheduler::new();
        scheduler.add_task(sample_task("t1"));
        scheduler.remove_task("nonexistent");
        let tasks = scheduler.tasks.lock().unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_empty_scheduler() {
        let scheduler = TaskScheduler::new();
        let tasks = scheduler.tasks.lock().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_task_fields() {
        let t = sample_task("t1");
        assert_eq!(t.interval_secs, 60);
        assert!(!t.one_shot);
        assert!(t.enabled);
    }
}

