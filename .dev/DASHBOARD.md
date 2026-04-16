# DASHBOARD

## Actual Progress

- Goal: <!-- dormammu:goal_source=/home/hjhun/.dormammu/goals/tizenclaw_improve.md -->
- Prompt-driven scope: Phase 4. Supervisor Validation, Continuation Loop, and Resume prompt-driven setup for Follow the guidance files below before making changes.
- Active roadmap focus:
- Phase 4. Supervisor Validation, Continuation Loop, and Resume
- Current workflow phase: plan
- Last completed workflow phase: none
- Supervisor verdict: `approved`
- Escalation status: `approved`
- Resume point: Return to Plan and resume from the first unchecked PLAN item if setup is interrupted

## Workflow Phases

```mermaid
flowchart LR
    plan([Plan]) --> design([Design])
    design --> develop([Develop])
    design --> test_author([Test Author])
    develop --> test_review([Test & Review])
    test_author --> test_review
    test_review --> final_verify([Final Verify])
    final_verify -->|approved| commit([Commit])
    final_verify -->|rework| develop
```

## In Progress

- Rework pass 10 complete: both reviewer findings resolved, build verified clean.
- Current workflow phase: commit
- Supervisor verdict: `approved`

## Rework Summary (reviewer findings addressed)

### Finding #1 — High: `backends.*`-only entries excluded from routing

- Root: `ProviderCompatibilityTranslator::translate()` only added backends
  named in `active_backend` or `fallback_backends` to `routing.providers`.
  Backends defined only under `backends.<name>` could initialize but never
  be selected by `ProviderSelector::ordered_enabled_names()`.
- Fix: Added a second scan of all `backends.*` keys after the positional
  loop. Keys not yet in the `seen` set are added as `CompatibilityBackends`
  entries, using explicit `priority` if present or a positional default
  below the fallback range (800 - index).
- New source variant: `ProviderConfigSource::CompatibilityBackends`.
- Tests added: `backends_only_entry_included_in_routing_config`,
  `backends_only_high_priority_sorts_before_positional_defaults`.
- Files: `src/tizenclaw/src/core/provider_selection.rs`.

### Finding #2 — Medium: legacy ClawHub lock entries use hardcoded URL

- Root: `clawhub_update()` fell back to `DEFAULT_CLAWHUB_BASE_URL` when
  `source_base_url` was absent in a lock entry, ignoring
  `TIZENCLAW_CLAWHUB_URL` / `CLAWHUB_URL` env vars.
- Fix: Replaced `unwrap_or(DEFAULT_CLAWHUB_BASE_URL)` with
  `unwrap_or(&resolved)` where `resolved = resolve_base_url()`, so
  operator env-var overrides are respected for pre-migration entries.
- Test added: `clawhub_update_missing_source_url_falls_back_to_env_var`.
- Files: `src/tizenclaw/src/core/clawhub_client.rs`.

## Validation Evidence

- `./deploy_host.sh -b`: succeeded, no warnings.
- `./deploy_host.sh --test`: all test suites passed (603 unit tests in
  tizenclaw crate, plus canonical workspace and parity harness).

## Risks And Watchpoints

- `backends.*` scan order is non-deterministic (HashMap iteration) when
  multiple backends have the same default priority. Operators with
  deterministic ordering requirements should set explicit `priority` values.
- Env var mutation in the new ClawHub test is not thread-safe across
  parallel test threads; `tokio::test` single-thread runtime limits risk.
