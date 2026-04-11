# Skill Capability Manager Design

## Scope

This cycle implements the next host-first phase-6 slice in two linked areas:

1. expose a first-class skill capability manager through IPC and CLI
2. reduce prompt-injected skill inventory to the minimal turn-relevant set

## Ownership Boundaries

- `core/skill_capability_manager.rs`
  owns skill capability configuration, root discovery, dependency checks,
  disabled-skill filtering, and runtime summary generation
- `AgentCore`
  remains the composition root, consumes the enabled skill pool for turn
  routing, and exposes the capability summary through IPC
- `tizenclaw-cli`
  surfaces the same daemon-reported capability summary without duplicating
  skill scanning logic

## Persistence And Runtime Path Impact

- add `config/skill_capabilities.json` as the operator-owned configuration
  document for disabled skill names
- reuse existing managed skill roots, skill-hub roots, and registered skill
  roots; no migration to a new root format is required
- dependency checks stay runtime-only by resolving required executables from
  `metadata.openclaw.requires`

## Runtime Contract

`get_session_runtime` will gain a `skills` object with:

- resolved managed, hub, and registered roots
- external-root reporting
- configured disabled skill names
- per-skill enabled/disabled state
- dependency status and install guidance from textual skill metadata
- counts for total, enabled, disabled, and dependency-blocked skills

A dedicated IPC path and CLI surface will expose the same payload for
operator inspection.

## Prompt Inventory Strategy

- scan the full textual skill pool once per turn
- filter out disabled skills before selection
- prefetch only the highest-relevance enabled skills for the active prompt
- inject only that minimal prefetched skill inventory into the prompt
  builder instead of the full discovered skill catalog

## Verification Contract

- update `tests/system/basic_ipc_smoke.json` first to assert the new
  `skills` summary shape from `get_session_runtime`
- add focused unit coverage for disabled-skill filtering, dependency
  checks, and root classification
- validate through `./deploy_host.sh -b`, `./deploy_host.sh`,
  `~/.tizenclaw/bin/tizenclaw-tests scenario --file tests/system/basic_ipc_smoke.json`,
  and `./deploy_host.sh --test`
