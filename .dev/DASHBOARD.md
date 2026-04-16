# DASHBOARD

## Actual Progress

- Goal: <!-- dormammu:goal_source=/home/hjhun/.dormammu/goals/tizenclaw_improve.md -->
- Prompt-driven scope: Runtime flexibility improvements — provider selection,
  Telegram model config, ClawHub update, skill snapshot cache, host validation.
- Active roadmap focus: all five roadmap targets delivered and committed.
- Current workflow phase: evaluate (complete)
- Last completed workflow phase: commit
- Supervisor verdict: `approved`
- Escalation status: `approved`

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

- All stages complete. Rework passes 1–10 resolved all reviewer findings.
- Current workflow phase: done
- Supervisor verdict: `approved`

## Rework Summary (rework pass 10 — reviewer findings addressed)

### Finding #1 — High: fallback executions recorded under primary backend

- Root: `process_prompt.rs` was using `primary_name()` to record token usage,
  so usage was attributed to the primary even when a fallback provider served.
- Fix: `process_prompt.rs` now reads `active_selection_provider_name()` from
  the registry, which `chat_with_fallback` sets via `set_active_selection`
  before returning. Token usage is now attributed to the actual serving backend.
- Files: `src/tizenclaw/src/core/agent_core/process_prompt.rs`.

### Finding #2 — Medium: plugin-discovered backends excluded from selection

- Root: `ProviderSelector::first_available` would skip providers absent from
  the routing config, breaking the legacy plugin-backend implicit-fallback path.
- Fix: `.unwrap_or(true)` on the routing config lookup so plugin-discovered
  backends are treated as enabled and eligible as last-resort fallbacks.
- Files: `src/tizenclaw/src/core/provider_selection.rs`.

## Validation Evidence

- `./deploy_host.sh -b`: succeeded, no warnings (rework pass 10).
- `./deploy_host.sh --test`: all test suites passed (603 unit tests in
  tizenclaw crate, plus canonical workspace and parity harness).

## Risks And Watchpoints

- `backends.*` scan order is non-deterministic (HashMap iteration) when
  multiple backends have the same default priority. Operators with
  deterministic ordering requirements should set explicit `priority` values.
- Env var mutation in ClawHub tests is not thread-safe across parallel test
  threads; `tokio::test` single-thread runtime limits risk.
- Plugin backends are now eligible as last-resort fallbacks but appear after
  all configured providers in preference order.
