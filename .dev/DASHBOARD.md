# DASHBOARD

## Actual Progress

- Goal: runtime-flexibility roadmap (provider selection, Telegram model
  config, ClawHub update, skill snapshot cache)
- Current workflow phase: COMPLETE — all plan items closed
- Last completed workflow phase: develop rework pass 8 → all reviewer
  findings resolved; PLAN/TASKS/WORKFLOWS synchronized
- Supervisor verdict: `approved` (all rework items addressed and marked [O])
- Escalation status: none

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
- [O] Stage 3. Develop — DONE (rework passes 5–8: all reviewer findings resolved)
- [O] Stage 4. Build/Deploy — DONE (`./deploy_host.sh -b` PASS)
- [O] Stage 5. Test/Review — DONE (`./deploy_host.sh --test` PASS: 597; 0 failed)
- [O] Stage 6. Commit — DONE (ce70f4b4, b6c8b3d8)
- [O] Stage 7. Evaluate — DONE (see .dev/07-evaluator/20260416-tizenclaw-improve.md)

## Completed Work

All five roadmap targets have been implemented, tested, and committed.
Eight rework passes have addressed all reviewer findings.

1. **Provider-selection layer** — `src/tizenclaw/src/core/provider_selection.rs`
   - `ProviderRegistry` owns initialized backends with preference-ordered routing
   - `ProviderSelector` selects the first available provider at request time
   - Compatibility translation respects `backends.<name>.priority` from
     legacy config, mirroring `build_backend_candidates` sort semantics
   - Admin/runtime status exposes `configured_provider_order`, `providers[]`, and
     `current_selection`; availability field reflects circuit-breaker state
   - Fallback path (write-locked registry) populates `providers[]` from routing
     config with `"availability": "unknown"`

2. **Telegram model configuration externalized**
   - All three builtin backends (codex, gemini, claude) have `model_choices: vec![]`
   - Operators configure model choices via `telegram_config.json`
   - Precedence chain: `telegram_config.json` wins over `llm_config.json`
   - `load_coding_agent_runtime` applies llm_config first so telegram_config wins

3. **ClawHub update flow** — `src/tizenclaw/src/core/clawhub_client.rs`
   - `clawhub_update()` reads `workspace/.clawhub/lock.json` and re-installs skills
   - Reports `updated`, `skipped`, and `failed` entries
   - One failure does not abort the full batch

4. **Skill snapshot caching** — `src/tizenclaw/src/core/skill_capability_manager.rs`
   - `SkillSnapshotCache` with `SkillSnapshotFingerprint` tracks root mtimes,
     registration, tool directory file-level changes, and capability-config changes
   - `tool_dir_signatures` field catches installs inside existing tool roots
   - `invalidate_snapshot_cache` called on all clawhub install/update paths

5. **Host validation** — all tests passed via `./deploy_host.sh --test`
   (597 passed, 0 failed after rework pass 8)

## Rework Pass 8 — Changes Applied

### High: Skill snapshot cache — tool directory file changes not fingerprinted
**File**: `src/tizenclaw/src/core/skill_capability_manager.rs`
**Fix**: Added `tool_dir_signatures: Vec<SkillRootSignature>` to
`SkillSnapshotFingerprint`. `compute()` now builds `SkillRootSignature`s for
all three tool directory sources so any file-level change triggers a cache miss.

### Medium: Planning preflight bypassed provider fallback
**File**: `src/tizenclaw/src/core/agent_core/process_prompt_loop.rs`
**Fix**: Replaced the single-backend call with `self.chat_with_fallback(...)`.
The planning preflight now participates in the full provider-selection loop.

## Rework Pass 7 — Changes Applied

### Medium: Telegram coding-agent config precedence reversed
**File**: `src/tizenclaw/src/channel/telegram_client/client_impl.rs`
**Fix**: Moved `read_backend_models_from_llm_config` call before the
`telegram_config.json` merge block, so Telegram config wins over llm_config
— matching the precedence in `TelegramClient::new()`.

### Medium: Prompt preparation used primary_name() regardless of availability
**File**: `src/tizenclaw/src/core/agent_core/process_prompt.rs`
**Fix**: `model_name` for `prompt_mode_from_doc` / `reasoning_policy_from_doc`
is now derived from `ProviderSelector::first_available`, falling back to
`primary_name()` only when no provider is available.

## Test Evidence

`./deploy_host.sh --test` — all suites PASS (rework pass 8)
- libtizenclaw api suite: 3 passed, 0 failed
- libtizenclaw_core suite: 30 passed, 0 failed
- Legacy tizenclaw suite: 597 passed, 0 failed
- Admin/runtime suite: 21 passed, 0 failed
- Canonical Rust workspace: 7 crates, all green
- Mock parity harness: PASS
- Doc-architecture verification: PASS

## Risks And Watchpoints

- No new risks introduced; all reviewer findings are closed.
- Provider init-time failures degrade gracefully to next available provider.
- ClawHub update failure for one entry does not abort the full batch.
- Snapshot cache fingerprint includes tool directory file-level signatures;
  same-second writes are covered by explicit `invalidate_snapshot_cache` calls.
- Telegram model choices are empty in builtins; operators supply them via config.
- Legacy config compatibility: `backends.<name>.priority` is respected in
  the routing layer; existing `active_backend`/`fallback_backends` configs
  are unaffected (positional defaults preserved).
