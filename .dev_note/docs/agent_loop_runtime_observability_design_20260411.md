# Agent Loop Runtime Observability Design

## Context

The runtime-topology and registry slice made storage paths explicit, but
the agent loop still lacked a first-class control-plane observation point.
Compared with the reference projects, the remaining gaps were:

- loop progress was only visible in logs, not in a durable daemon-facing
  contract
- resume readiness depended on session files that were not summarized
  through IPC
- failure state could terminate a turn without leaving a structured
  checkpoint behind

## Selected Architecture

Keep the existing `AgentLoopState` in-memory execution model and add a
host-first snapshot layer rather than redesigning the loop scheduler.

### 1. Loop snapshot contract

Persist a structured snapshot under the runtime topology state root:

- `state/loop/<session_id>.json`

Each snapshot captures:

- current loop phase
- plan-step counters
- round and retry counters
- last evaluation verdict
- last error
- workflow step ownership
- resumable flag

The snapshot is written at loop start, after self-inspection checkpoints,
and on terminal success or failure paths.

### 2. Session resume contract

Summarize the session persistence footprint directly from `SessionStore`:

- session directory path
- current day markdown path
- compacted snapshot paths
- transcript path
- workdir path
- message file count
- booleans for transcript and compaction existence
- `resume_ready`

This keeps resume behavior daemon-observable without forcing a prompt run.

### 3. IPC contract

Expose a new `get_session_runtime` method returning:

- `control_plane`
- `runtime_topology`
- `session`
- `loop_snapshot`

This keeps the phase-4 contract narrow, deterministic, and suitable for
`tizenclaw-tests`.
