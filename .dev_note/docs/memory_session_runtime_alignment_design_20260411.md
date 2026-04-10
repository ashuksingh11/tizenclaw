# Memory And Session Runtime Alignment Design

## Scope

Phase 5 extends the runtime-topology contract into memory persistence
and session-context flow so the daemon can expose enough disk-first
state for host debugging without reading internal files manually.

## Ownership Boundaries

- `SessionStore` remains the owner of transcript, compaction, workdir,
  and session resume artifacts under the runtime data root.
- `MemoryStore` remains the owner of persisted memory markdown files,
  category directories, and the SQLite-backed entry index.
- `AgentCore` owns the daemon-facing composition step that merges both
  stores into a single IPC summary for one session.

## Persistence Impact

- No new persistence format is introduced for phase 5.
- The IPC summary must expose the existing `memory.md` path, category
  directories, and entry counts so operators can debug prompt context
  assembly from runtime metadata alone.
- Session summaries continue to expose transcript and compaction paths.

## IPC Contract

`get_session_runtime` is expanded with:

- `memory`: persisted-memory paths, category counts, total entries, and
  prompt-readiness metadata
- `context_flow`: session resume readiness plus memory prompt-readiness
  derived from disk-backed session and memory summaries

## Verification Path

- Update `tests/system/basic_ipc_smoke.json` before implementation.
- Add unit coverage for the new memory runtime summary contract.
- Validate through `./deploy_host.sh -b`, `./deploy_host.sh`,
  `~/.tizenclaw/bin/tizenclaw-tests scenario --file tests/system/basic_ipc_smoke.json`,
  and `./deploy_host.sh --test`.
