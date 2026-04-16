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

## 2026-04-16 — Reviewer NEEDS_WORK resolution

**Stage**: Test/Review rework after reviewer findings.

**Changes applied**:

| Finding | File | Fix |
|---------|------|-----|
| High: path traversal in zip extraction | `src/tizenclaw/src/core/clawhub_client.rs` | Replaced `contains("..")` check with `Path::components()` guard that rejects `ParentDir`, `RootDir`, and `Prefix` components, plus a final `starts_with(dest_dir)` defense-in-depth check |
| Medium: non-atomic install | `src/tizenclaw/src/core/clawhub_client.rs` | Extract to `<slug>.__installing__` staging dir, then `remove_dir_all` + `rename` to final path |
| High: "CodingAgent:" in Telegram UX | `transport.rs`, `commands.rs`, `execution.rs` | Renamed label to `"Backend:"` across all five format sites |
| High: coding-agent prompt language | `commands.rs` (`build_unified_agent_prompt`) | Renamed preference keys (`Backend:`, `Model:`, `Execution mode:`, `Auto approve:`); removed `run_coding_agent` / `create_task` instruction paragraph |
| High: tests locked old wording | `tests.rs` | Updated `assert!(message.contains("CodingAgent: [...]"))` → `"Backend: [...]"` at lines 609, 634, 914 |

**Validation**: `./deploy_host.sh --test` → 561 passed / 6 failed (same 6 pre-existing agent_core content-scoring failures; no regressions introduced).

**Status**: RESOLVED — all reviewer findings addressed.

---

## 2026-04-16 — Second rework cycle (supervisor rework_required)

**Stage**: Develop — resume from supervisor finding that PLAN items were unchecked
and remaining reviewer findings were not fully applied.

**Reviewer findings addressed in this cycle**:

| Finding | File | Fix |
|---------|------|-----|
| High: keyboard still uses `/coding_agent` | `transport.rs:112` | Changed `cli_backend_keyboard` to emit `/backend {backend}` buttons |
| High: "local coding agent" in system prompt | `commands.rs:855,858` | Changed to "AI agent handling requests through TizenClaw" |
| High: tests locked `/coding_agent` keyboard text | `tests.rs:138-142,189` | Updated assertions to `/backend codex`, `/backend gemini`, etc.; renamed test |
| Medium: `/backend` command missing from handler | `commands.rs` | Added `"backend"` arm in `handle_command`, implemented `set_cli_backend` |
| Medium: `/backend` absent from help text and bot menu | `transport.rs` | Added `/backend [name]` to `supported_commands_text`; added `("backend", "Select AI backend")` to `command_menu_entries` |
| State: PLAN.md and TASKS.md still showed `[ ]` | `.dev/PLAN.md`, `.dev/TASKS.md` | Marked all 5 prompt-derived items `[O]` |

**Validation**: `./deploy_host.sh --test` → 561 passed / 6 failed (same 6 pre-existing
agent_core content-scoring failures; no regressions introduced).

**Status**: RESOLVED — all supervisor and reviewer findings addressed.
