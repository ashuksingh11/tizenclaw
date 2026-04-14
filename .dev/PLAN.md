# PLAN

## Prompt-Derived Checklist

- [x] Refine the request into `.dev/REQUIREMENTS.md`
- [x] Regenerate `.dev/WORKFLOWS.md`, `.dev/PLAN.md`, and `.dev/DASHBOARD.md`
- [x] Inspect `agent_core` hotspots and define the target responsibility split
- [x] Refactor `AgentCore::process_prompt` and adjacent helpers into clearer
      modules with reusable interfaces
- [x] Remove or isolate accidental Korean-language and scenario-specific
      implementation leakage in production agent-loop code
- [x] Add or update deterministic `tests/system/` coverage for the touched
      daemon-visible behavior
- [x] Run `./deploy_host.sh` and the selected `tizenclaw-tests` scenarios
- [x] Record validation evidence and residual risks in `.dev/DASHBOARD.md`
- [x] Prepare commit artifacts if the cycle reaches a clean commit point
- [x] Produce the final evaluator report under `.dev/07-evaluator/`

## Resume Rule

Resume from the first unchecked item and route back to the earliest failed
stage if a supervisor gate fails.
