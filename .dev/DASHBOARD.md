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

## 2026-04-16 — Reviewer NEEDS_WORK resolution (third rework cycle)

**Stage**: Test/Review rework — all three reviewer findings addressed.

### Changes applied

| Finding | Severity | File | Fix |
|---------|----------|------|-----|
| `deploy_host.sh` swallows `cargo test` failures | High | `deploy_host.sh:548` | Changed `warn` to `fail` so test failures cause a non-zero exit |
| `/select` accepted `coding-agent`, `coding_agent`, `agent` aliases | Medium | `types.rs:18` | Removed those aliases; retained `"coding"` as the internal canonical value and added `"backend"` as the new user-facing alias |
| User-facing fallback said "Choose [chat] or [coding]." | Medium | `commands.rs:136` | Changed to "Unknown mode. Choose [chat] or [backend]." |
| Backend prompts described session as "Telegram coding mode" | Medium | `commands.rs:852,855` | Changed to "AI agent via TizenClaw" |
| Session history block labelled "Telegram coding session history" | Medium | `commands.rs:869` | Changed to "Current session history" |
| Tests asserted `coding-agent` alias was valid | Medium | `tests.rs:36-48` | Updated `parse_mode_aliases_work` to assert old aliases return `None` and `"backend"` returns `Coding` |
| ClawHub tests lacked extraction, path-sanitization, atomic-replace coverage | Medium | `clawhub_client.rs` | Added 5 new tests: `extract_zip_archive_writes_files_to_dest`, `extract_zip_archive_strips_slug_prefix_when_present`, `extract_zip_archive_rejects_path_traversal`, `extract_zip_archive_rejects_absolute_paths`, `atomic_install_staging_then_rename` |

### Validation

`cargo test -p tizenclaw` (offline, locked):
- **566 passed / 6 failed** (same 6 pre-existing `agent_core` content-scoring failures)
- All 42 Telegram client tests pass
- All 10 ClawHub tests pass (5 new + 5 existing)
- No regressions introduced

**Status**: RESOLVED — all reviewer findings addressed.

---

## 2026-04-16 — Fourth rework cycle (supervisor rework_required)

**Stage**: Develop — addressing supervisor finding that PLAN.md items were still
`[ ]` and reviewer finding that `"coding"` was still accepted as a mode alias.

### Changes applied

| Finding | Severity | File | Fix |
|---------|----------|------|-----|
| PLAN.md items still `[ ]` | High | `.dev/PLAN.md` | Marked all 5 prompt-derived items `[O]` |
| `/select coding` still accepted | Medium | `types.rs:16-18` | Removed `"coding"` from `parse()`; only `"backend"` maps to `Coding` mode |
| `as_str()` returned `"coding"` | Medium | `types.rs:23-26` | Changed `Coding.as_str()` to return `"backend"`; session labels now show `backend-0001` |
| Test expected `"coding-0001"` session label | Medium | `tests.rs:67-69` | Updated to expect `"backend-0001"` |
| No test asserting `"coding"` parse returns `None` | Low | `tests.rs:41-44` | Added `assert_eq!(TelegramInteractionMode::parse("coding"), None)` |

### Validation

`./deploy_host.sh --test`:
- **566 passed / 6 failed** (same 6 pre-existing `agent_core` content-scoring failures)
- All Telegram client tests pass including `select_with_valid_arg_removes_reply_keyboard`
  and `coding_usage_report_includes_actual_cli_tokens`
- No regressions introduced

**Status**: RESOLVED — all supervisor and reviewer findings addressed.
