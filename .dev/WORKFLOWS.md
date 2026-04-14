# Workflows

## Task: Improve agent loop

Host-default refactor cycle for decomposing the oversized `AgentCore`
agent-loop implementation into smaller modules, removing accidental
language- or scenario-specific leakage from common paths, and strengthening
daemon-visible regression coverage.

[x] Phase 0. Refine requirements - `refining-requirements`
[x] Phase 1. Plan refactor cycle - `planning-project`
[x] Phase 2. Design agent-loop module boundaries - `designing-architecture`
[x] Phase 3. Refactor agent-loop implementation - `developing-code`
[x] Phase 4. Refresh daemon-visible system scenarios - `testing-with-tizenclaw-tests`
[x] Phase 5. Run host build and deploy validation - `building-deploying`
[x] Phase 6. Review results and residual risks - `reviewing-code`
[x] Phase 7. Prepare commit and repository state - `managing-versions`
[x] Phase 8. Write the final evaluator report - `evaluating-outcomes`

## Notes

- Execution class: `host-default`
- Shell path: direct `bash` on Linux/WSL, no `wsl.exe` wrapper
- Validation path: `./deploy_host.sh` plus deterministic `tizenclaw-tests`
  coverage for affected `tests/system/` scenarios
- The refactor must preserve `AgentCore::process_prompt` as the external
  entry point while reducing its responsibility
- Final evaluation under `.dev/07-evaluator/` is mandatory for this cycle
