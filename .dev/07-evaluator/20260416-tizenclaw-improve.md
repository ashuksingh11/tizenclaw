# Evaluator Report — tizenclaw_improve (2026-04-16)

## Verdict: APPROVED

## Goal Summary

Advance five deferred roadmap items that improve runtime flexibility and
operator maintainability in `tizenclaw`:

1. Provider-selection layer for multi-backend routing
2. Telegram model configuration externalized to operator config
3. ClawHub skill update flow backed by lock file
4. Skill snapshot caching with safe invalidation
5. Host-first validation via `./deploy_host.sh`

## Implementation Record

### 1. Provider-Selection Layer

**File**: `src/tizenclaw/src/core/provider_selection.rs`

- `ProviderRoutingConfig` normalizes config from `llm_config.json`
- `ProviderCompatibilityTranslator` maps legacy `active_backend` /
  `fallback_backends` into the new routing model
- `ProviderRegistry` owns initialized backends with preference-ordered routing
- `ProviderSelector` selects the first available provider at request time
- Admin/runtime status exposes `configured_provider_order`, `providers[]`,
  and `current_selection` on both normal and write-locked paths
- Write-locked fallback path builds a populated `providers[]` from routing
  config (rework 4 fix, commit `f69aa1c3`)

### 2. Telegram Model Configuration

- All three builtin backends (codex, gemini, claude) have `model_choices: vec![]`
- Operators configure model choices via `telegram_config.json`
- Precedence chain documented and tested

### 3. ClawHub Update Flow

**File**: `src/tizenclaw/src/core/clawhub_client.rs`

- `clawhub_update()` reads `workspace/.clawhub/lock.json`
- Re-installs skills using recorded source identity
- Reports `updated`, `skipped`, and `failed` entries
- One failure does not abort the full batch

### 4. Skill Snapshot Caching

**File**: `src/tizenclaw/src/core/skill_capability_manager.rs`

- `SkillSnapshotCache` with `SkillSnapshotFingerprint` tracks root mtimes,
  registration, and capability-config changes
- `invalidate_snapshot_cache` called on all clawhub install/update paths

### 5. Host Validation

All rework passes validated via `./deploy_host.sh --test`.

Final result: **597 passed, 0 failed** (rework pass 5+6).

## Rework History

| Pass | Finding | Fix | Commit |
|------|---------|-----|--------|
| 1 | Startup path filtered providers incorrectly | Added `providers_authoritative` filter | (included in initial impl) |
| 2 | Second reviewer findings | Various fixes | `8ab1a6ef` |
| 3 | Fallback path misreported provider order for `providers[]` configs | Added `raw_doc` to `LlmConfig`; fallback uses `ProviderCompatibilityTranslator::translate()` | `cb3c1153` |
| 4 | Write-locked fallback hard-coded `"providers": []` | Fallback builds populated `providers[]` from `routing.providers` | `f69aa1c3` |
| 5 | `backends.*.priority` ignored; circuit-breaker not reflected in status | Rewrote legacy synthesis path; added `is_available` predicate to `status_json` | `ce70f4b4` |
| 6 | PLAN.md tracking items left unchecked after implementation was done | Marked all 5 PLAN.md items `[O]`; updated WORKFLOWS.md/DASHBOARD.md | `23ea5ac7` |

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| Provider routing no longer tied to one primary + static fallback | PASS |
| Dedicated provider-selection layer with preference/availability/routing | PASS |
| Legacy `active_backend`/`fallback_backends` compatibility preserved | PASS |
| Legacy `backends.*.priority` respected in routing order | PASS |
| Admin/runtime status exposes routing state on both normal and fallback paths | PASS |
| Circuit-breaker state reflected as `open_circuit` in provider status | PASS |
| Telegram model lists externalized to operator config | PASS |
| ClawHub update backed by lock file | PASS |
| ClawHub update preserves install safety guarantees | PASS |
| Skill snapshot cache avoids redundant rescans | PASS |
| Cache invalidation on root/registration/config changes | PASS |
| Host validation via `./deploy_host.sh` | PASS |
| Regression validation via `./deploy_host.sh --test` | PASS |

## Residual Risks and Follow-Up Items

- Snapshot cache fingerprint uses 1-second mtime resolution; same-second
  writes are covered by explicit `invalidate_snapshot_cache` calls.
- Telegram model choices are empty in builtins; operators must supply them.
- Write-locked fallback reports availability as `"unknown"` since live
  instance state is inaccessible during reload — this is expected behavior.
- The `providers[]` config schema in `llm_config.json` is additive; legacy
  operators who do not migrate remain on the compatibility path.

## Final State

- All five roadmap targets implemented and committed.
- No open reviewer findings remain.
- All PLAN.md tracking items marked `[O]`.
- All tests pass (597 passed, 0 failed).
- Repository state is synchronized with plan and dashboard (commit `23ea5ac7`).
