# PLAN

## Active Sprint: ClawHub Integration + Telegram UX Cleanup

### Stage 0. Refine — DONE
- `.dev/REQUIREMENTS.md` exists with baselines, ClawHub definition, host proof, and
  Telegram UX policy resolved.

### Stage 1. Plan — DONE
- `.dev/WORKFLOWS.md`, `.dev/PLAN.md`, `.dev/DASHBOARD.md` exist and reflect the
  resolved requirements.

### Stage 2. Design — DONE
- ClawHub entry points pinned to `https://clawhub.ai/api/v1/*` from openclaw source.
- Install path: `workspace/skill-hubs/clawhub/<slug>/` (auto-discovered by runtime).
- Lock file: `workspace/.clawhub/lock.json`.
- Telegram cleanup scope: remove `/coding_agent`, `/devel`, `/devel_result`, `/mode`,
  `/auto_approve` commands; remove CodingAgent/CodingMode/AutoApprove from status,
  startup, and connected messages.

### Stage 3. Develop — DONE

#### ClawHub Integration
- [x] Write `.dev/ROADMAP.md`
- [x] Add `zip` crate to `src/tizenclaw/Cargo.toml` and vendor it
- [x] Create `src/tizenclaw/src/core/clawhub_client.rs`
- [x] Register `clawhub_client` in `src/tizenclaw/src/core/mod.rs`
- [x] Add `clawhub_install`, `clawhub_search`, `clawhub_list` to `AgentCore`
      (`agent_core/runtime_admin_impl.rs`)
- [x] Wire IPC methods in `src/tizenclaw/src/core/ipc_server.rs`
- [x] Add `skill-hub` subcommand to `src/tizenclaw-cli/src/main.rs`

#### Telegram UX Cleanup
- [x] Update `transport.rs`: `command_menu_entries()`, `select_keyboard()`,
      `supported_commands_text()`
- [x] Update `commands.rs`: `pending_menu_command()`, `handle_command()`,
      `format_status_text()`, `build_startup_message()`, `build_connected_message()`
- [x] Update `types.rs`: remove `CodingAgent`, `ExecutionMode`, `AutoApprove` from
      `TelegramPendingMenu`
- [x] Update `tests.rs`: fix assertions for new menu/help/status behavior

### Stage 4. Build/Deploy — DONE
- [x] Run `./deploy_host.sh -b` — succeeded
- [x] Fixed zip vendor and type annotation issues; 561 tests pass, 6 pre-existing failures
      unrelated to this sprint

### Stage 5. Test/Review — IN PROGRESS
- [ ] Verify ClawHub path reachable in host runtime
- [ ] Verify Telegram help/status output
- [ ] Record findings in `.dev/DASHBOARD.md`

### Stage 6. Commit — PENDING
- [ ] Write `.tmp/commit_msg.txt`
- [ ] Commit with `git commit -F .tmp/commit_msg.txt`

### Stage 7. Evaluate — PENDING
- [ ] Write evaluator report under `.dev/07-evaluator/`

## Resume Checkpoint

Resume from Stage 5: Test/Review.
