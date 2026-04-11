# File Manager Bridge Observability Design

## Goal

Make `file_manager` behavior observable through daemon IPC so host-first
regression runs can prove both the Linux-utility path and the Rust
fallback path without ad-hoc local test harnesses.

## Ownership

- `AgentCore::execute_bridge_tool` becomes the bridge entry point for
  `file_manager`
- `file_manager_tool` remains the single execution owner for file
  operations and backend reporting
- `tests/system/file_manager_bridge.json` becomes the daemon-visible
  contract for this slice

## Contract Changes

- Allow bridge IPC callers to execute `file_manager` directly with an
  optional `session_id`
- Reuse session-scoped workdirs when a session store exists; otherwise
  fall back to a daemon-managed bridge workdir under `data/bridge_tool`
- Accept an optional `backend_preference` set to
  `linux_utility` or `rust_fallback`
- Keep the default behavior Linux-utility-first, but allow explicit
  fallback forcing for deterministic regression coverage

## Verification Shape

- `mkdir`, `read`, `list`, `copy`, and `remove` must remain observable
  with `backend=linux_utility`
- `read`, `stat`, `move`, and recursive `remove` must remain observable
  with `backend=rust_fallback` when forced
- The live daemon scenario must use `bridge_tool` only and avoid direct
  cargo-based helpers
