# WORKFLOWS

## Planning Classification

- Request class source: `.dev/workflow_state.json`
- `intake.request_class`: `full_workflow`
- `workflow_policy.required_phases`:
  `refine -> plan -> design -> develop -> build_deploy -> test_review ->
  commit -> evaluate`
- `workflow_policy.skipped_phases`: none
- `workflow_policy.skip_rationale`: none
- `refinement.mode`: `clarify`
- `refinement.blocked`: `false`
- Effective planning depth: `full_workflow`
- Rationale: this goal requires comparison, roadmap authoring, concrete
  implementation, host validation through `./deploy_host.sh`, test or review
  follow-up, and the mandatory final evaluator stage.

## Active Stage Sequence

```text
refine(resolved) -> plan -> design -> develop -> build/deploy
-> test/review -> commit -> evaluate
```

## Stage Plan

### Stage 0. Refine
Purpose:
- keep the resolved requirements as the authoritative scope for execution

Outputs:
- `.dev/REQUIREMENTS.md`
- synchronized `.dev/workflow_state.json`

Gate to exit:
- baselines, ClawHub definition, runnable host proof, and Telegram UX policy
  are explicit and no refinement blocker remains

### Stage 1. Plan
Purpose:
- define the downstream execution path from the resolved requirements and the
  synced workflow policy

Outputs:
- `.dev/WORKFLOWS.md`
- `.dev/PLAN.md`
- `.dev/TASKS.md`
- `.dev/DASHBOARD.md`

Gate to exit:
- workflow, plan, tasks, and dashboard match the resolved requirements and
  machine state

### Stage 2. Design
Purpose:
- pin down the ClawHub skill install or mount path and the Telegram UX
  migration before code changes begin

Outputs:
- design notes under `.dev/docs/` when needed
- refreshed `.dev/DASHBOARD.md`

Gate to exit:
- implementation no longer depends on guesswork around skill source handling,
  runtime discovery, or Telegram-visible behavior

### Stage 3. Develop
Purpose:
- compare `tizenclaw` against the pinned references
- rewrite `.dev/ROADMAP.md`
- implement the ClawHub path and Telegram UX cleanup

Required skills:
- `developing-code`
- `testing-with-tizenclaw-tests`

Outputs:
- scoped code and config changes
- rewritten `.dev/ROADMAP.md`
- refreshed `.dev/DASHBOARD.md`

Gate to exit:
- intended files are changed and the repository state matches the planned scope

### Stage 4. Build/Deploy
Purpose:
- validate the host path using the repository-approved script only

Outputs:
- scripted host evidence from `./deploy_host.sh`

Gate to exit:
- `./deploy_host.sh` succeeds and records usable host-path evidence

### Stage 5. Test/Review
Purpose:
- validate runtime-visible and Telegram-visible behavior changes
- confirm the ClawHub path is observable through runtime discovery
- record residual risks or missing coverage

Required skills:
- `reviewing-code`
- `testing-with-tizenclaw-tests`

Outputs:
- executed validation results
- refreshed `.dev/DASHBOARD.md`

Gate to exit:
- test and review evidence is recorded with clear findings or residual risks

### Stage 6. Commit
Purpose:
- prepare the commit stage if the implementation reaches a clean handoff

Outputs:
- `.tmp/commit_msg.txt`
- final commit

Gate to exit:
- diff scope is correct and commit formatting follows `AGENTS.md`

### Stage 7. Evaluate
Purpose:
- produce the mandatory final evaluator verdict

Outputs:
- evaluator report under `.dev/07-evaluator/`

Gate to exit:
- the final assessment is recorded with an explicit verdict

## Skipped Phases

- None.
- `workflow_policy.skipped_phases` is an empty list in
  `.dev/workflow_state.json`.
- No skip rationale applies for this run.

## Current Gate Status

- Active phase: `complete`
- Current verdict: `PASS`
- Blocking source: none
- Release condition: all seven stages completed and committed (cfa3c43d, c85cad34, develRust)
- Rework completed: TC-06 (CLI help now lists skill-hub) and TC-07 (setup wizard
  no longer shows coding mode) verified against installed binary after full deploy

## Phase Completion Record

- [O] refine — `.dev/REQUIREMENTS.md` produced
- [O] plan — `.dev/WORKFLOWS.md`, `.dev/PLAN.md`, `.dev/TASKS.md`, `.dev/DASHBOARD.md` produced
- [O] design — ClawHub API endpoints and Telegram cleanup scope pinned
- [O] develop — `clawhub_client.rs`, IPC, CLI, Telegram UX cleanup implemented
- [O] build/deploy — `./deploy_host.sh -b` passed
- [O] test/review — live validation and Telegram help verified
- [O] commit — committed with `.tmp/commit_msg.txt`
- [O] evaluate — `.dev/07-evaluator/2026-04-16-clawhub-telegram-cleanup.md`
