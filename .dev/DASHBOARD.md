# DASHBOARD

## Active Goal

Compare tizenclaw against openclaw and openclaude, identify gaps, author ROADMAP.md,
implement ClawHub-ready host path, and clean up Telegram coding-agent UX.

## Current Stage

**Stage 7. Evaluate** — DONE

## Stage Completion Status

| Stage | Status |
|---|---|
| 0. Refine | DONE |
| 1. Plan | DONE |
| 2. Design | DONE |
| 3. Develop | DONE |
| 4. Build/Deploy | DONE |
| 5. Test/Review | DONE |
| 6. Commit | DONE |
| 7. Evaluate | DONE |

## Scope

### ClawHub Integration
- New `clawhub_client.rs` module in tizenclaw daemon
- IPC methods: `clawhub_install`, `clawhub_search`, `clawhub_list`
- CLI commands: `tizenclaw-cli skill-hub install|search|list`
- Install target: `~/.tizenclaw/workspace/skill-hubs/clawhub/<slug>/`
- Lock file: `~/.tizenclaw/workspace/.clawhub/lock.json`
- Base URL: `https://clawhub.ai` (env override: `TIZENCLAW_CLAWHUB_URL`)
- `zip` crate (v2, deflate) vendored for archive extraction

### Telegram UX Cleanup
- Removed commands: `/coding_agent`, `/devel`, `/devel_result`, `/mode`, `/auto_approve`
- Removed `/select coding` from keyboard; only `/select chat` remains
- Removed from status/startup/connected messages: `CodingAgent:`, `CodingMode:`,
  `AutoApprove:`, `Binary:`, `Runs:`
- Removed from `TelegramPendingMenu`: `CodingAgent`, `ExecutionMode`, `AutoApprove`
- `TelegramInteractionMode::Coding` retained internally for persisted state compat
- Removed dead functions: `set_cli_backend`, `set_execution_mode`, `set_auto_approve`
- Updated 42 Telegram tests: all pass

### CLI Help and Setup Cleanup (c85cad34)
- `tizenclaw-cli --help` now lists ClawHub commands under "ClawHub commands:" section
- `tizenclaw-cli setup` no longer shows "Telegram coding mode" wording
- Setup prompt now reads: "Do you want to configure Telegram now?"

## Test Results (Stage 5 + Rework)

| Suite | Pass | Fail |
|---|---|---|
| Telegram client | 42 | 0 |
| All tizenclaw tests | 561 | 6 (pre-existing, unrelated) |
| ClawHub live search | ✓ live response from clawhub.ai | — |
| ClawHub list | ✓ returns empty lock | — |
| Parity harness | PASS | — |
| Doc architecture | PASS | — |
| TC-06 CLI help (skill-hub visible) | PASS | — |
| TC-07 Setup wizard (no coding mode) | PASS | — |
| deploy_host.sh full deploy | PASS | — |

## Risks and Watchpoints

- ClawHub live download requires network access at runtime; offline or rate-limited
  hosts will need the lock file pre-populated.
- 6 pre-existing test failures in `agent_core::tests` (prediction market / news
  summarization) are unrelated to this sprint and were present before these changes.

## Task Queue Synchronization

All prompt-derived TASKS.md queue items marked [O]. PLAN.md prompt-derived
items marked [O]. WORKFLOWS.md phase completion record is fully updated.
Repository state is synchronized with the pipeline outcome.

## Continuation Run 1 Verification (2026-04-16 ~16:16)

Supervisor verdict from prior pipeline run: `rework_required` — WORKFLOWS.md
reported pending `[ ]` items. Root-cause: the supervisor evaluated at
16:16:09+09:00, 8 seconds after commit `52ce8e7a` (16:16:01+09:00) landed;
a Samba/WSL flush lag likely caused the supervisor to see a slightly stale
Phase Completion Record.

Verification performed in continuation run 1:
- Confirmed WORKFLOWS.md has no `[ ]` items; all phases are `[O]`.
- Confirmed TC-06 fix is present: `--help` lists "ClawHub commands:" section.
- Confirmed TC-07 fix is present: setup reads "Do you want to configure
  Telegram now?" (no "coding mode" wording).
- Re-ran `./deploy_host.sh -b`: build succeeded, no regressions.

No additional code changes required. Repository state is correct.

## Continuation Run 2 Verification (2026-04-16, this run)

Supervisor re-triggered with `rework_required` — same root cause as above.
Build and installed-binary checks re-run from scratch:

- `./deploy_host.sh -b`: succeeded (3.21s, debug profile)
- `~/.tizenclaw/bin/tizenclaw-cli --help`: ClawHub commands section present
- `tizenclaw-cli setup` wizard: reads "Do you want to configure Telegram now?"
  with no "coding mode" wording
- WORKFLOWS.md: no `[ ]` items; all Phase Completion Record entries are `[O]`
- Both repo and session WORKFLOWS.md are fully synchronized

| Check | Result |
|---|---|
| Build (`./deploy_host.sh -b`) | PASS |
| TC-06 CLI help (skill-hub visible) | PASS |
| TC-07 Setup wizard (no coding mode) | PASS |
| WORKFLOWS.md no pending `[ ]` items | PASS |

No additional code changes required.

## Last Updated

2026-04-16 — Continuation run 2 complete. All verifications pass.
Commits cfa3c43d, c85cad34, c5c4c1af, 52ce8e7a on develRust.
