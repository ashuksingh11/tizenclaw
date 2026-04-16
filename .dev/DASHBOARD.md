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

- Review the prompt-derived goal and success criteria for <!-- dormammu:goal_source=/home/hjhun/.dormammu/goals/tizenclaw_improve.md -->.
- Review repository guidance from AGENTS.md, .github/workflows/ci.yml, .github/workflows/release-host-bundle.yml
- Generate DASHBOARD.md and PLAN.md from the active prompt before implementation continues.

## Progress Notes

- This file should show the actual progress of the active scope.
- workflow_state.json remains machine truth.
- PLAN.md should list prompt-derived development items in phase order.
- Repository rules to follow: AGENTS.md
- Relevant repository workflows: .github/workflows/ci.yml, .github/workflows/release-host-bundle.yml

## Risks And Watchpoints

- Do not overwrite existing operator-authored Markdown.
- Keep JSON merges additive so interrupted runs stay resumable.
- Keep session-scoped state isolated when multiple workflows run in parallel.

---

## 2026-04-16 — Fifth rework cycle (reviewer NEEDS_WORK resolution)

**Stage**: Test/Review rework — both reviewer findings resolved.

### Changes applied

| Finding | Severity | File | Fix |
|---------|----------|------|-----|
| `select_keyboard()` only showed `/select chat` | Medium | `transport.rs:107` | Added `/select backend` as second button in the same row |
| `pending_menu_command()` only mapped `1` to chat | Medium | `commands.rs:95-97` | Added `2 => Some("/select backend")` arm |
| Tests locked in broken single-button behavior | Medium | `tests.rs:243-252, 307-314` | Updated to assert 2-button row and `"2"` → `/select backend` |
| Regex lookahead `(?!\s*%)` in `output_lacks_numeric_market_fact` | High | `foundation.rs:1906` | Removed lookahead — `regex` crate does not support it; `Regex::new()` was failing silently, making `has_numeric_price` always `false` |
| Four date fixtures hardcoded to 2026-04-12/13/14 | Medium | `tests.rs` (4 tests) | Added `iso_date_days_ago()` helper; fixtures now compute dates relative to wall-clock time |
| `project_email_summary_shortcut` test unreachable | Medium | `process_prompt.rs:60-109` | Moved `session_workdir` + `try_process_prompt_shortcuts` before the no-backend early return; shortcuts are pure local transforms |

### Validation

`./deploy_host.sh --test`:
- **572 passed / 0 failed** (was 566 passed / 6 failed)
- All Telegram client tests pass including 2-button keyboard assertions
- All content-scoring tests pass with dynamic date fixtures
- `project_email_summary_shortcut_records_transcript_and_completes` now passes
- Host validation gate exit code: 0

**Status**: RESOLVED — all reviewer findings addressed, `deploy_host.sh --test` passes.
