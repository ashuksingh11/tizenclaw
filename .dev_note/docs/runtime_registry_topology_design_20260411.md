# Runtime Registry And Topology Design

## Context

TizenClaw already contains strong runtime pieces, but their ownership model is
too fragmented for a predictable autonomous agent loop. Compared with
`openclaw`, `nanoclaw`, and `openclaude`, the main gaps are:

- runtime paths are available, but the topology contract is not exposed as a
  first-class daemon concept
- tool and skill registration is persisted as path lists rather than typed
  registry records
- IPC can report registered paths, but it cannot explain source, activation,
  or storage topology well enough for debugging
- logging exists, but registration lifecycle and runtime layout decisions are
  not visible enough during tests

## Selected Architecture

The chosen host-first design keeps the existing Rust daemon structure but adds
two explicit contracts.

### 1. Runtime topology contract

Introduce a runtime topology view in `core/runtime_paths.rs` that derives
canonical directories from the active data root and exposes:

- `config`
- `state`
- `state/registry`
- `sessions`
- `memory`
- `logs`
- `outbound`
- `plugins`
- `workspace/skills`
- `workspace/skill-hubs`
- `telegram_sessions`

This does not replace `PlatformPaths`. It acts as a daemon-facing projection
that removes ambiguity for storage, debugging, and IPC reporting.

### 2. Registry-first external capability contract

Replace the purely list-based registration persistence with typed entries:

- kind: tool or skill
- path: canonical directory path
- source: bundled, user, project, external, or unknown
- active flag
- created timestamp

Backward compatibility remains intact by preserving `tool_paths` and
`skill_paths` in IPC responses and on-disk compatibility behavior.

## Persistence Impact

The registry metadata will live under the runtime topology state root so that
it is clearly separated from user-editable config files.

- compatibility file:
  `config/registered_paths.json`
- normalized registry snapshot:
  `state/registry/registered_paths.v2.json`

## IPC-Observable Assertions

The runtime-visible contract will be verified through `tizenclaw-tests` by
updating `tests/system/basic_ipc_smoke.json` to assert:

- the existing `registrations.tool_paths` and `registrations.skill_paths`
  fields still exist
- `registrations.entries` exists
- `runtime_topology.state_dir` and `runtime_topology.registry_dir` exist

## Logging And Debugging

Registration lifecycle logging will emit:

- registry load path
- compatibility save path
- normalized registry snapshot path
- register and unregister operations with kind and canonical path

These logs create deterministic breadcrumbs for host-first debugging.
