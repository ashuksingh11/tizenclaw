# DASHBOARD

## Actual Progress

- Goal: Advance runtime flexibility and operator maintainability (tizenclaw_improve)
- Active roadmap focus: all five roadmap items completed + rework pass complete
- Current workflow phase: develop (rework pass — fixing reviewer findings)
- Last completed workflow phase: develop
- Supervisor verdict: `rework_required` (addressing now)
- Escalation status: `none`

## Rework Pass — Reviewer Findings

Three correctness gaps identified in the prior sprint's code review.
All three are now addressed:

### Finding 1 (High): startup path missing `providers_authoritative` filter

**Root cause**: The daemon's startup initialization path (`init()` in
`runtime_core_impl.rs` around line 1087) sorted initialized instances by
configured preference order but did not drop disabled providers when
`providers[]` was the authoritative config source. The `reload_backends()`
path (added in commit 3c1e39e1) had the filter but the startup path did not.

**Fix**: Applied the same `providers_authoritative` retain-filter to the
startup path, matching the behavior of `reload_backends()` exactly.

### Finding 2 (High): set_llm_config("providers", ...) did not trigger reload

**Status**: Already fixed in commit 3c1e39e1 (`runtime_admin_impl.rs` line 23
shows `providers` in `llm_config_path_affects_backends`). Verified in current
source.

### Finding 3 (Medium): Telegram model precedence reversed

**Status**: Already fixed in commit 3c1e39e1. `read_backend_models_from_llm_
config()` is called before the `telegram_config.json` merge block at line 110,
so `telegram_config.json` wins (later merge takes precedence). Comment at
line 108–109 documents this. Verified in current source.

## Completed Work

All five roadmap targets have been implemented, tested, and committed:

1. **Provider-selection layer** — `src/tizenclaw/src/core/provider_selection.rs`
   - `ProviderRegistry` owns initialized backends with preference-ordered routing
   - `ProviderSelector` selects the first available provider at request time
   - Compatibility translation maps legacy `active_backend`/`fallback_backends` config
   - Admin/runtime status exposes `configured_provider_order`, `providers[]`, and
     `current_selection`
   - `providers[]` is now authoritative in both startup and reload paths —
     disabled providers are filtered out before the registry is stored

2. **Telegram model configuration externalized**
   - All three builtin backends (codex, gemini, claude) have `model_choices: vec![]`
     in source; no curated lists are baked in
   - Operators configure model choices via
     `telegram_config.json.cli_backends.backends.<id>.model_choices`
   - Precedence chain: `telegram_config.json` > `llm_config.json.telegram` >
     `llm_config.json.backends.<provider>.model` > builtin empty list
   - Loading path is tested and backwards-compatible

3. **ClawHub update flow** — `src/tizenclaw/src/core/clawhub_client.rs`
   - `clawhub_update()` reads `workspace/.clawhub/lock.json` and re-installs
     tracked skills using the locked `source_base_url`
   - Reports `updated`, `skipped`, and `failed` entries
   - `tizenclaw-cli skill-hub update` exposes the flow consistently with
     install/list
   - IPC method `clawhub_update` added in `ipc_server.rs`
   - One update failure does not abort the full batch

4. **Skill snapshot caching** — `src/tizenclaw/src/core/skill_capability_manager.rs`
   - `SkillSnapshotCache` with `SkillSnapshotFingerprint` tracks root mtimes,
     registration, and capability-config changes
   - `load_snapshot()` returns cached value on fingerprint match; rebuilds on miss
   - `invalidate_snapshot_cache()` available for forced refresh

5. **Host validation** — tests passed via `./deploy_host.sh --test`

## Workflow Phases

```mermaid
flowchart LR
    refine([Refine]) --> plan([Plan])
    plan --> design([Design])
    design --> develop([Develop])
    develop --> build([Build/Deploy])
    build --> test([Test/Review])
    test --> commit([Commit])
    commit --> evaluate([Evaluate])
    evaluate -->|rework| develop
```

- [O] Stage 0. Refine — DONE
- [O] Stage 1. Plan — DONE
- [O] Stage 2. Design — DONE
- [O] Stage 3. Develop — DONE (rework pass: startup providers_authoritative filter added)
- [O] Stage 4. Build/Deploy — DONE (`./deploy_host.sh -b` PASS)
- [O] Stage 5. Test/Review — DONE (`./deploy_host.sh --test` PASS: 590+others; 0 failed)
- [ ] Stage 6. Commit — pending
- [ ] Stage 7. Evaluate — pending re-evaluation after rework commit

## Risks And Watchpoints

- Provider init-time failures degrade gracefully to next available provider.
- ClawHub update failure for one entry does not abort the full batch.
- Snapshot cache fingerprint uses 1-second mtime resolution; edits within the
  same second may not be detected without an explicit `invalidate_snapshot_cache`
  call (acceptable tradeoff, documented in code).
- Telegram model choices are empty in builtins; operators must supply them via
  config if non-default model selection is needed.
- The startup-path `providers_authoritative` filter gap is now closed; disabled
  providers can no longer slip through on daemon start.
