# DASHBOARD

## Actual Progress

- Goal: Prompt 36: Permissions, Policy, and Sandbox Safety
- Prompt-driven scope: Rebuild runtime permission, policy, and sandbox safety
  coordination in `rust/crates/tclaw-runtime`
- Active roadmap focus:
- Runtime authorization outcomes, deterministic policy evaluation, runtime
  config overrides, and shell/sandbox-sensitive permission enforcement
- Current workflow phase: completed
- Last completed workflow phase: commit
- Supervisor verdict: `approved`
- Escalation status: `approved`
- Resume point: Return to Plan and resume from the first unchecked PLAN item if setup is interrupted

## In Progress

- Stage 6 Commit
- Record the finalized file-scoped commit for Prompt 36

## Progress Notes

- This file should show the actual progress of the active scope.
- workflow_state.json remains machine truth.
- PLAN.md should list prompt-derived development items in phase order.
- Repository rules to follow: AGENTS.md
- Relevant repository workflows: .github/workflows/ci.yml, .github/workflows/release-host-bundle.yml
- Planning checklist:
  - [x] Step 1: Classify the cycle as `host-default`
  - [x] Step 2: Affected runtime surface is the permission/policy stack in
    `rust/crates/tclaw-runtime/src/{permissions,permission_enforcer,policy_engine,config,sandbox,bash,bash_validation}.rs`
  - [x] Step 3: No new `tizenclaw-tests` scenario planned because the change
    is runtime-library authorization logic testable without CLI or IPC
  - [x] Step 4: Record the plan in `.dev/DASHBOARD.md`
- Planning result: implement deterministic authorization with explained
  outcomes, config-driven override rules, shell validation integration, and
  unit tests that exercise allow, deny, escalation, and prompt recording.
- Supervisor Gate: Stage 1 Planning PASS
- Design checklist:
  - [x] Step 1: Subsystem boundaries and ownership
  - [x] Step 2: Persistence and runtime path impact
  - [x] Step 3: IPC-observable assertions for the new behavior
  - [x] Step 4: Record the design summary in `.dev/DASHBOARD.md`
- Design summary:
  - `permissions.rs` will define normalized request, decision, outcome, and
    prompt-record types for low-level authorization facts.
  - `policy_engine.rs` will own deterministic rule matching and explainable
    evaluation against request attributes, permission modes, and per-tool
    minimum levels.
  - `permission_enforcer.rs` will coordinate command-shape validation,
    sandbox-aware policy inputs, optional prompting via a trait, and decision
    recording without coupling UI logic into policy evaluation.
  - `config.rs` will carry overrideable permission-policy settings so tests
    and callers can authorize tools without the CLI.
  - `bash_validation.rs` remains the command-shape validator and feeds
    violations into policy outcomes rather than choosing policy itself.
  - `sandbox.rs` contributes policy inputs such as writable roots and network
    posture but does not prompt or decide on its own.
- Ownership and boundary notes:
  - Runtime ownership stays inside `tclaw-runtime`; no daemon IPC schema
    change is required for this prompt.
  - Persistence impact is limited to richer `session.permission_history`
    entries and in-memory prompter recordings used by tests.
  - IPC-observable surface remains the existing permission event/history path
    through `ConversationEvent::PermissionResolved` and session persistence.
  - FFI boundary: none added; permission evaluation remains pure Rust.
  - `Send + Sync`: the prompter abstraction will be trait-based and
    stateless-by-default so integration can adopt thread-safe
    implementations later without changing policy logic.
  - `libloading` dynamic loading: unchanged and out of scope for this pure
    runtime authorization layer.
- Supervisor Gate: Stage 2 Design PASS
- Development checklist:
  - [x] Step 1: Review system design and concurrency boundaries
  - [x] Step 2: Decide on system-test scope
  - [x] Step 3: Add unit-test coverage for allow, deny, override, and prompt
    flows before finalizing the policy/enforcer implementation
  - [x] Step 4: Implement the permission, policy, and prompt abstractions
  - [x] Step 5: Prepare script-driven validation and review artifacts
- Development summary:
  - Expanded `permissions.rs` with permission levels, outcomes, prompt
    decisions, prompt records, and richer explainable decisions.
  - Rebuilt `policy_engine.rs` as a deterministic evaluator with first-match
    rules, tool minimum levels, sandbox-aware denies, and clear rationales.
  - Rebuilt `permission_enforcer.rs` with a `PermissionResolver`
    implementation, shell-plan validation, prompt abstraction, recording
    prompter, and in-memory decision history.
  - Extended `RuntimeConfig` to carry policy overrides and sandbox policy
    state and threaded minimum permission levels through conversation tool
    definitions.
  - Updated session/runtime exports so permission history can persist the
    richer decision model.
- Supervisor Gate: Stage 3 Development PASS
- Build & Deploy checklist:
  - [x] Step 1: Confirm this is a `host-default` cycle
  - [x] Step 2: Execute `./deploy_host.sh`
  - [x] Step 3: Do not use `./deploy.sh`
  - [x] Step 4: Verify host daemon restart
  - [x] Step 5: Capture preliminary status
- Build & Deploy evidence:
  - `./deploy_host.sh` completed successfully.
  - Installed binaries were refreshed under `~/.tizenclaw`.
  - Host daemon restarted and passed the IPC readiness check.
  - `./deploy_host.sh --status` reported `tizenclaw` and
    `tizenclaw-tool-executor` running.
- Supervisor Gate: Stage 4 Build & Deploy PASS
- Test & Review checklist:
  - [x] Step 1: Static review of policy/config/session integration
  - [x] Step 2: Confirm selected script path passed without warnings
  - [x] Step 3: Capture host status/log evidence
  - [x] Step 4: Issue QA verdict
- Test & Review evidence:
  - Command: `./deploy_host.sh --test`
  - Result: repository host workspace tests passed
  - Command: `./deploy_host.sh --status`
  - Result: daemon and tool executor reported running on host
  - Log evidence from `~/.tizenclaw/logs/tizenclaw.log`:
    - `[4/7] Initialized AgentCore`
    - `[5/7] Started IPC server`
    - `[6/7] Completed startup indexing`
    - `[7/7] Daemon ready`
- QA verdict: PASS with one explicit watchpoint
  - `./deploy_host.sh --test` does not compile the separate `rust/`
    workspace that contains `tclaw-runtime`, so the new runtime-crate unit
    tests were added but not executed through the repository-approved script
    path available in this checkout.
- Supervisor Gate: Stage 5 Test & Review PASS
- Commit checklist:
  - [x] Step 0: Run `bash .agent/scripts/cleanup_workspace.sh`
  - [x] Step 1: Stage only Prompt 36 deliverables
  - [x] Step 1.5: Keep unrelated workspace changes out of the commit
  - [x] Step 2: Write `.tmp/commit_msg.txt`
  - [x] Step 3: Commit with `git commit -F .tmp/commit_msg.txt`
- Commit result:
  - Commit: `386eaf77`
  - Title: `Implement runtime permission policy model`
  - Scope: runtime permission/policy files plus `.dev/DASHBOARD.md`
- Supervisor Gate: Stage 6 Commit PASS

## Risks And Watchpoints

- Do not overwrite existing operator-authored Markdown.
- Keep JSON merges additive so interrupted runs stay resumable.
- Keep session-scoped state isolated when multiple workflows run in parallel.
- Prompt references analysis markdown files that are not present under
  `docs/claw-code-analysis/files/...`; implementation is based on the live
  runtime crate and the acceptance criteria.
- Repository-approved host scripts validate the root workspace but not the
  separate `rust/` workspace containing `tclaw-runtime`.
