# DASHBOARD

## Active Goal

Compare tizenclaw against openclaw and openclaude, identify gaps, author ROADMAP.md,
implement ClawHub-ready host path, and clean up Telegram coding-agent UX.

## Current Stage

**Stage 6. Commit** — in progress

## Stage Completion Status

| Stage | Status |
|---|---|
| 0. Refine | DONE |
| 1. Plan | DONE |
| 2. Design | DONE |
| 3. Develop | DONE |
| 4. Build/Deploy | DONE |
| 5. Test/Review | DONE |
| 6. Commit | IN PROGRESS |
| 7. Evaluate | PENDING |

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

## Test Results (Stage 5)

| Suite | Pass | Fail |
|---|---|---|
| Telegram client | 42 | 0 |
| All tizenclaw tests | 561 | 6 (pre-existing, unrelated) |
| ClawHub live search | ✓ live response from clawhub.ai | — |
| ClawHub list | ✓ returns empty lock | — |
| Parity harness | PASS | — |
| Doc architecture | PASS | — |

## Risks and Watchpoints

- ClawHub live download requires network access at runtime; offline or rate-limited
  hosts will need the lock file pre-populated.
- 6 pre-existing test failures in `agent_core::tests` (prediction market / news
  summarization) are unrelated to this sprint and were present before these changes.

## Supervisor Verdict

Plan stage: `approved` — proceed to develop.

## Last Updated

2026-04-16 — Stage 5 done, entering Stage 6.
