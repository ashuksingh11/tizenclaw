# DASHBOARD

## Actual Progress

- Goal: Advance runtime flexibility and operator maintainability (tizenclaw_improve)
- Active roadmap focus: all five roadmap items completed + two rework passes complete
- Current workflow phase: evaluate (complete)
- Last completed workflow phase: evaluate
- Supervisor verdict: `approved`
- Escalation status: `none`

## Second Rework Pass — Reviewer Findings

Four correctness gaps identified in the second review. All four are now addressed
in commit 8ab1a6ef.

### Finding 1 (High): snapshot cache not invalidated after clawhub install/update

**Root cause**: `clawhub_install` and `clawhub_update` in `runtime_admin_impl.rs`
(lines 599 and 617) did not call `invalidate_snapshot_cache` on success. A skill
install or update that completed within the same clock second as the last snapshot
would leave the cache stale until the mtime rolled over, violating the acceptance
criterion that callers receive correct results immediately after installs or updates.

**Fix**: Added explicit `invalidate_snapshot_cache(ClawHubInstall)` and
`invalidate_snapshot_cache(ClawHubUpdate)` calls after each successful operation.
The update handler invalidates unconditionally because even a partial success
writes skill directories.

### Finding 2 (Medium): get_llm_runtime() fallback path reported wrong order

**Root cause**: The write-lock fallback branch in `runtime_admin_impl.rs` (line 250)
set `configured_provider_order` to `configured_fallback_backends` only, dropping
the active backend from the list. This made the status surface misleading to
operators inspecting routing state during a reload.

**Fix**: Rebuilt the full ordered list (active backend first, then deduplicated
fallbacks) to match `ProviderRegistry::status_json()`.

### Finding 3 (Medium): chat_with_fallback did not route through ProviderSelector

**Root cause**: `chat_with_fallback` in `runtime_core_impl.rs` (line 1416) iterated
`rg.instances()` directly, which does not filter disabled providers. `ProviderSelector`
existed but was not the authority for the production request path, so future routing
policy changes in `ProviderSelector` would not have taken effect without a manual
duplicate.

**Fix**: Added `ProviderSelector::ordered_enabled_names(registry)` to
`provider_selection.rs` and replaced the direct `instances()` iteration with it.
Disabled providers are now excluded from the fallback loop via the centralized
selection layer.

### Finding 4 (Medium): ClawHub update tests covered only the empty-lock case

**Root cause**: The only update test was `clawhub_update_returns_skipped_for_empty_lock`.
The partial-failure accumulation loop and source-identity reuse branches in
`clawhub_update` had no test coverage despite writing installs and lock state.

**Fix**: Added two tests to `clawhub_client.rs`:
- `clawhub_update_uses_source_base_url_from_lock_entry` — verifies source identity
  is read from the lock entry by checking the error message references the custom
  URL (not the default clawhub.ai address).
- `clawhub_update_accumulates_failures_for_multiple_entries` — verifies the loop
  does not abort on first failure; both entries must appear in `failed`.

## Completed Work

All five roadmap targets have been implemented, tested, and committed.
Two rework passes have addressed all reviewer findings.

1. **Provider-selection layer** — `src/tizenclaw/src/core/provider_selection.rs`
   - `ProviderRegistry` owns initialized backends with preference-ordered routing
   - `ProviderSelector` selects the first available provider at request time
   - `ProviderSelector::ordered_enabled_names` is the authoritative source for the
     provider iteration order in `chat_with_fallback`
   - Compatibility translation maps legacy `active_backend`/`fallback_backends` config
   - Admin/runtime status exposes `configured_provider_order`, `providers[]`, and
     `current_selection` on both normal and fallback (write-locked) paths

2. **Telegram model configuration externalized**
   - All three builtin backends (codex, gemini, claude) have `model_choices: vec![]`
   - Operators configure model choices via `telegram_config.json`
   - Precedence chain documented and tested

3. **ClawHub update flow** — `src/tizenclaw/src/core/clawhub_client.rs`
   - `clawhub_update()` reads `workspace/.clawhub/lock.json` and re-installs skills
   - Reports `updated`, `skipped`, and `failed` entries
   - One failure does not abort the full batch
   - Tests cover empty lock, source-identity reuse, and partial-failure accumulation

4. **Skill snapshot caching** — `src/tizenclaw/src/core/skill_capability_manager.rs`
   - `SkillSnapshotCache` with `SkillSnapshotFingerprint` tracks root mtimes,
     registration, and capability-config changes
   - `invalidate_snapshot_cache` is now called on all paths that change the
     skill filesystem: clawhub install, clawhub update

5. **Host validation** — all tests passed via `./deploy_host.sh --test`

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
- [O] Stage 3. Develop — DONE (rework pass 2: all four findings addressed)
- [O] Stage 4. Build/Deploy — DONE (`./deploy_host.sh -b` PASS)
- [O] Stage 5. Test/Review — DONE (`./deploy_host.sh --test` PASS: 592+others; 0 failed)
- [O] Stage 6. Commit — DONE (8ab1a6ef); PLAN.md sync commit pending
- [O] Stage 7. Evaluate — DONE

## Risks And Watchpoints

- Provider init-time failures degrade gracefully to next available provider.
- ClawHub update failure for one entry does not abort the full batch.
- Snapshot cache fingerprint uses 1-second mtime resolution; same-second writes
  are now covered by explicit `invalidate_snapshot_cache` calls on all clawhub
  operation handlers.
- Telegram model choices are empty in builtins; operators must supply them via config.
- The startup-path `providers_authoritative` filter gap was closed in rework pass 1.
- `chat_with_fallback` now routes through `ProviderSelector::ordered_enabled_names`;
  disabled providers can no longer slip into the fallback loop.
- The `get_llm_runtime()` write-lock fallback now reports the full provider order.
