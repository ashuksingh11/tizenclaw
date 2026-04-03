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

> [!IMPORTANT]
> **Language Rule**: You must plan and report in the same language as the user's input. (If asked in English, use English; if asked in Korean, use Korean / 사용자의 입력과 같은 언어로 plan과 보고를 합니다. 영어로 질의하면 영어로, 한국어로 질의하면 한국어로 답합니다).

> [!CAUTION]
> **Supervisor Gate Enforcement**: After completing each stage, the Supervisor Agent validates the stage's outputs against its SKILL.md requirements. If the Supervisor issues a FAIL verdict, you **must rollback** to the failed stage, re-read its SKILL.md, apply corrections, and re-execute. Maximum **3 retry attempts** per gate. On persistent failure, escalate to the user.

---

## Execution Order

> [!IMPORTANT]
> **Strict Sequential Execution**: The coding agent **MUST** strictly execute the full 6-stage sequence step-by-step for all development tasks, without skipping any stages or arbitrarily jumping between skill agents.

// turbo-all

### 1. Read AGENTS.md
Read the `.agent/rules/AGENTS.md` file to internalize the autonomous agent constraints and performance paradigms.

### 2. Planning
Refer to the `skills/planning-project/SKILL.md` skill to analyze the required AI capabilities, perception layers, or state-machine tasks defined in `<task description>`.
- Update `.dev_note/DASHBOARD.md`.

### 2.1 Supervisor Gate — Planning
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Planning stage outputs.
- Verify execution mode classification was performed.
- Verify DASHBOARD.md was updated.
- **On PASS**: proceed to Step 3. **On FAIL**: rollback to Step 2 with violation report.

### 3. Design
Refer to the `skills/designing-architecture/SKILL.md` skill to architect the Rust module. Aim for peak embedded performance utilizing `tokio` asynchronous components, fearless concurrency, and safe Tizen FFI data mapping.
- Update `.dev_note/DASHBOARD.md`.

### 3.1 Supervisor Gate — Design
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Design stage outputs.
- Verify FFI boundaries and `Send+Sync` specs are defined.
- Verify DASHBOARD.md was updated.
- **On PASS**: proceed to Step 4. **On FAIL**: rollback to Step 3 with violation report.

### 4. Development
Refer to the `skills/developing-code/SKILL.md` skill to program the core logic via Embedded TDD limits.
- **Local `cargo build/check/test` execution is prohibited.** Build via `./deploy.sh` to ensure target-environment integrity. Avoid using `cargo check` locally.
- Update `.dev_note/DASHBOARD.md`.

### 4.1 Supervisor Gate — Development
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Development stage outputs.
- Verify no local `cargo build/check/test` was used.
- Verify TDD cycle was followed (test-first methodology).
- Verify DASHBOARD.md was updated.
- **On PASS**: proceed to Step 5. **On FAIL**: rollback to Step 4 with violation report.

### 5. Build & Deploy
Refer to the `skills/building-deploying/SKILL.md` skill to compile the optimized daemon using `./deploy.sh`.
- **Building for x86_64 architecture** is mandatory.
- **[DISABLED]**: armv7l (ARM) architecture verification is currently disabled to prioritize execution speed but can be re-enabled if required.
- Update `.dev_note/DASHBOARD.md`.

### 5.1 Supervisor Gate — Build & Deploy
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Build & Deploy stage outputs.
- Verify x86_64 build was executed via `deploy.sh`.
- Verify no local `cargo build` or `cargo check` was used.
- Verify DASHBOARD.md was updated.
- **On PASS**: proceed to Step 6. **On FAIL**: rollback to Step 5 with violation report.

### 6. Test & Review
Refer to the `skills/reviewing-code/SKILL.md` skill to run integration tests and assess the running daemon.
- Perform continuous daemon execution assessment (`./deploy.sh --test`). Verify there are no deadlocks, panics, or unhandled states.
- Update `.dev_note/DASHBOARD.md`.

### 6.1 Supervisor Gate — Test & Review
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Test & Review stage outputs.
- Verify runtime logs from device were captured as evidence.
- Verify PASS/FAIL verdict was issued with concrete log proofs.
- Verify DASHBOARD.md was updated.
- **On PASS**: proceed to Step 7. **On FAIL**: rollback to Step 6 with violation report.

### 7. Commit & Push
Refer to the `skills/managing-versions/SKILL.md` skill to prepare the codebase.

> [!CAUTION]
> **Mandatory**: This step **MUST** invoke `skills/managing-versions/SKILL.md`
> without exception. Performing `git commit` or `git push` directly outside
> of this skill is a critical violation that triggers immediate Supervisor rollback.

- **Clean up unnecessary files before committing**: Remove `target/` remnants,
  `*.rpm` caches, and temp swap files.
- Command Git via `git commit -F .tmp/commit_msg.txt` strictly following the
  commit format rules below.
- Update `.dev_note/DASHBOARD.md`.

> [!IMPORTANT]
> **Commit Message Rules**:
> 1. **Language**: Write all commit messages in **English**.
> 2. **Line Length**: No line may exceed **80 characters**.
> 3. **Format**:
>    ```
>    <Title: concise imperative sentence, ≤50 chars>
>
>    <Body: clearly explain the purpose (Why) and the
>     specific changes made (What); each line ≤80 chars>
>    ```
> 4. The title must capture intent at a glance; the body must make
>    the motivation and exact changes unambiguous to future readers.

### 7.1 Supervisor Gate — Commit (Final)
Refer to the `skills/supervising-workflow/SKILL.md` skill to validate Commit stage outputs.
- Verify `managing-versions/SKILL.md` was used (mandatory skill enforcement).
- Verify `commit_msg.txt` was used (no `-m` flag).
- Verify commit message is written in **English**.
- Verify no line in the commit message exceeds **80 characters**.
- Verify commit format: title ≤50 chars, body explains purpose and changes.
- Verify workspace was cleaned (no build artifacts staged).
- Verify DASHBOARD.md was updated.
- **On PASS**: Cycle complete. **On FAIL**: rollback to Step 7 with violation report.