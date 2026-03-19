---
description: Main Development Workflow (Plan -> Develop -> Verify -> Review)
---

# TizenClaw Main Development Workflow

This workflow defines the core development process (Plan → Develop → Verify → Review → Commit) for the TizenClaw project. The AGENT must always follow this process when performing tasks.

## 1. Plan
- Accurately understand the objectives and requirements.
- Analyze existing code and check applicable workflows (e.g., `/coding_rules`, `/commit_guidelines`). See `README.md` for the full workflow index. **CRITICAL**: The Agent MUST strictly adhere to the project's coding style as defined in `coding_rules.md` (e.g., Google C++ Style, 2-space indentation, trailing underscore `_` for members). Do not introduce or mimic inconsistent styles found in older legacy parts of the codebase.
- **CRITICAL BRANCH POLICY**: Do not create or switch to new branches for development or feature work. Always apply patches, make commits, and push changes directly to the **current branch** you are currently on. Maintain this single-branch development policy at all times.
- **WORKFLOW DOC POLICY**: Workflow documents (.md) must only be created or modified after the corresponding feature has been fully verified (build, deploy, and runtime validation) on an actual device. Writing workflow documents for unverified features is prohibited. When adding a new workflow, you must also update the `README.md` index.
- Write a work unit (`task.md`) and establish a detailed plan before implementation.

## 2. Develop & Deploy
- Modify source code and add/modify unit tests.
- After writing code, use the `deploy.sh` script to build, deploy, and restart the daemon via a single command.
  - Run: `./deploy.sh`
  - The script will automatically trigger a `gbs build`, locate the built rpm packages, install them on the device, and restart the `tizenclaw` service.
  - **IMPORTANT**: Do NOT run raw `gbs build` commands directly. Always use `deploy.sh` for build and deployment. Raw GBS commands should only be executed when explicitly requested by the user.

## 3. Verify
Once `deploy.sh` successfully finishes:
- Check the log output of the TizenClaw daemon to verify correct startup and runtime execution:
  - Command: `sdb shell dlogutil TIZENCLAW TIZENCLAW_WEBVIEW`
- **Functional Testing via `tizenclaw-cli`**:
  - Use the CLI to send natural language prompts to the daemon and verify new features work end-to-end.
  - Single-shot mode: `sdb shell tizenclaw-cli "your prompt here"`
  - With session ID: `sdb shell tizenclaw-cli -s <session_id> "your prompt here"`
  - Streaming: `sdb shell tizenclaw-cli --stream "your prompt here"`
  - Interactive mode: `sdb shell tizenclaw-cli` (type prompts, Ctrl+D to exit)
  - Example (workflow tools): `sdb shell tizenclaw-cli "Use the list_workflows tool to show the workflow list"`
- Verify the Web Dashboard is accessible:
  - Dashboard Port: `9090` (e.g., `http://<device-ip>:9090`)
- If you need a more advanced component test, refer to `/gtest_integration.md`.

> [!TIP]
> If a crash occurs after deployment, refer to the `crash_debug.md` workflow to analyze the crash dump.

## 4. Code Review
After verification passes, perform a code review on all changed files using the `code_review.md` workflow checklist:
1. **Coding Style** — `coding_rules.md` compliance
2. **Correctness** — logic errors, boundary conditions, missing error handling
3. **Memory Issues** — memory leaks, dangling pointer, use-after-free
4. **Performance** — unnecessary copies, inefficient loops, lock contention
5. **Logic Issues** — dead code, unreachable branches, variable shadowing
6. **Security** — missing input validation, buffer overflow, injection vulnerabilities
7. **Thread Safety** — race condition, deadlock, GLib callback safety
8. **Resource Management** — fd/socket/D-Bus release, GLib resources, container cleanup
9. **Test Coverage** — gtest additions/modifications, unit tests for new functions
10. **Error Propagation & Logging** — dlog usage, error propagation paths, silent failure prevention

### Review-Fix Loop (max 5 iterations)
- **PASS**: All items pass → proceed to Commit stage
- **FAIL**: Issues found → return to **Develop** stage to fix → `deploy.sh` → **Verify** → re-**Review**
- This loop repeats up to **5 times**. If exceeded, escalate to the user.

> [!CAUTION]
> If the Review-Fix loop exceeds 5 iterations, you must report to the user to prevent an infinite loop.

Refer to `code_review.md` for the detailed checklist and procedures.

## 5. Commit (Completion of Work)
When all review passes, perform a `git commit` to finalize the work according to the `commit_guidelines.md` workflow.
Refer to the detailed rules in the respective workflow, but the core points are as follows.

### Basic Structure of a Commit Message
Write in the Conventional Commits style. **The commit message MUST be written in English.**

```text
Title (Under 50 chars, clear and concise English)

Provide a detailed explanation of the implemented features, bug fixes,
or structural changes. Describe 'Why' and 'What' was done extensively
but clearly. (Wrap text at 72 characters)
```

### Writing Example (Good)
```text
Switch from LXC to lightweight runc for ContainerEngine

Refactored the ContainerEngine implementation to use the lightweight
`runc` CLI via `std::system` instead of relying on `liblxc` APIs.
This change was necessary because the Tizen 10 GBS build environment
does not provide the `pkgconfig(lxc)` dependency.
```

### Prohibitions
- Mechanical text, such as Verification/Testing Results blocks, must **NEVER be included** in the commit message.
- Do not add unnecessary, verbose phrases generated by a bot.

### Commit Timing
1. One unit feature specified in the document is implemented.
2. `gbs build` (including `%check` gtests internally) passes without errors.
3. Perform `git commit` formatted as above after `git add .`.

---

## Workflow Reference List
This is a list of detailed workflow files referenced in this AGENTS workflow.

| Workflow | File | Referenced Stage |
|---|---|---|
| Coding Rules | `coding_rules.md` | Plan |
| Code Review | `code_review.md` | Code Review |
| Commit Guidelines | `commit_guidelines.md` | Commit |
| GTest Unit Testing | `gtest_integration.md` | Verify |
| Crash Dump Debugging | `crash_debug.md` | Verify |
| CLI Functional Testing | `cli_testing.md` | Verify |