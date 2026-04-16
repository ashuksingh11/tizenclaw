# DASHBOARD

## Actual Progress

- Goal: Advance runtime flexibility and operator maintainability (tizenclaw_improve)
- Active roadmap focus: all five roadmap items completed
- Current workflow phase: evaluate (complete)
- Last completed workflow phase: evaluate
- Supervisor verdict: `approved`
- Escalation status: `none`

## Completed Work

All five roadmap targets have been implemented, tested, and committed:

1. **Provider-selection layer** — `src/tizenclaw/src/core/provider_selection.rs`
   - `ProviderRegistry` owns initialized backends with preference-ordered routing
   - `ProviderSelector` selects the first available provider at request time
   - Compatibility translation maps legacy `active_backend`/`fallback_backends` config
   - Admin/runtime status exposes `configured_provider_order`, `providers[]`, and
     `current_selection`
   - Failure records are written to `current_selection` when all providers fail,
     ensuring stale success state is not reported after an error

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
     tracked skills using the locked `source_base_url` (preserved verbatim so
     routine updates do not silently migrate skills to a different registry)
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

5. **Host validation** — 758 tests passed; 0 failed via `./deploy_host.sh --test`

## Validation Evidence

- `./deploy_host.sh` — PASS (daemon started, IPC ready)
- `./deploy_host.sh --test` — PASS (758 passed; 0 failed)
- Mock parity harness — PASS
- Documentation architecture verification — PASS

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
```

All phases: DONE

## Committed Changes

- `ea504cae` — Add provider registry, ClawHub update, skill cache
- `fd39c321` — Fix skill cache and expose full LLM runtime provider status
- `8eca9237` — Address reviewer findings from runtime flexibility sprint

## Risks And Watchpoints

- Provider init-time failures degrade gracefully to next available provider.
- ClawHub update failure for one entry does not abort the full batch.
- Snapshot cache fingerprint uses 1-second mtime resolution; edits within the
  same second may not be detected without an explicit `invalidate_snapshot_cache`
  call (acceptable tradeoff, documented in code).
- Telegram model choices are empty in builtins; operators must supply them via
  config if non-default model selection is needed.
