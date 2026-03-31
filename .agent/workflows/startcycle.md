---
description: Start the TizenClaw Autonomous AI Agent development cycle with a new module or capability
---

# /startcycle — Start TizenClaw Development Cycle

When a user types `/startcycle <task description>`, you **must first read `.agent/rules/AGENTS.md`** and start the development cycle according to its autonomous engineering rules.

> [!IMPORTANT]
> **Mandatory Reference to AGENTS.md**: Before starting a cycle, you must read the `.agent/rules/AGENTS.md` document.
> AGENTS.md defines all high-performance embedded Rust procedures, including the 6-step progression logic, zero-cost abstraction standards, and TDD feedback loops essential for a continuously running AI daemon.

> [!IMPORTANT]
> **Auto-Approval Execution Mode**: All actions requiring approval during development are automatically approved.
> Do not halt simply at writing system design documents. Physically execute terminal commands (`./deploy.sh`, `git commit`) to construct the actual agent capabilities. Local `cargo` runs are prohibited; use `deploy.sh` exclusively.

---

## Execution Order

// turbo-all

### 1. Read AGENTS.md
Read the `.agent/rules/AGENTS.md` file to internalize the autonomous agent constraints and performance paradigms.

### 2. Planning
Refer to the `skills/planning-project/SKILL.md` skill to analyze the required AI capabilities, perception layers, or state-machine tasks defined in `<task description>`.
- Save the deliverables in `.dev_note/01-planning/`.
- Update `.dev_note/DASHBOARD.md`.

### 3. Design
Refer to the `skills/designing-architecture/SKILL.md` skill to architect the Rust module. Aim for peak embedded performance utilizing `tokio` asynchronous components, fearless concurrency, and safe Tizen FFI data mapping.
- Save the deliverables in `.dev_note/03-design/`.
- Update `.dev_note/DASHBOARD.md`.

### 4. Development
Refer to the `skills/developing-code/SKILL.md` skill to program the core logic via Embedded TDD limits.
- **Local `cargo build/test` execution is prohibited.** Build via `./deploy.sh` to ensure target-environment integrity.
- Save the deliverables in `.dev_note/04-development/`.
- Update `.dev_note/DASHBOARD.md`.

### 5. Build & Deploy
Refer to the `skills/building-deploying/SKILL.md` skill to compile the optimized daemon using `./deploy.sh`.
- **Building for both x86_64 and armv7l architectures** is mandatory.
- Save the deliverables in `.dev_note/05-build-and-deploy/`.
- Update `.dev_note/DASHBOARD.md`.

### 6. Test & Review
Refer to the `skills/reviewing-code/SKILL.md` skill to run integration tests and assess the running daemon.
- Perform continuous daemon execution assessment (`./deploy.sh --test`). Verify there are no deadlocks, panics, or unhandled states.
- Save the deliverables in `.dev_note/06-test-and-code-review/`.
- Update `.dev_note/DASHBOARD.md`.

### 7. Commit & Push
Refer to the `skills/managing-versions/SKILL.md` skill to prepare the codebase.
- **Clean up unnecessary files before committing**: Remove `target/` remnants, `*.rpm` caches, and temp swap files.
- Command Git via `git commit -F .dev_note/commit_msg.txt` strictly following Gerrit style protocols.
- Save the deliverables in `.dev_note/07-commit-and-push/`.
- Update `.dev_note/DASHBOARD.md`.
- Cycle complete.