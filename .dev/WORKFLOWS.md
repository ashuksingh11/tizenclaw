# WORKFLOWS

## Task
Standardize TizenClaw's runtime directory structure to the OpenClaw-style
layout, using `~/.tizenclaw` on host systems and `/home/owner/.tizenclaw` on
Tizen, while strengthening compatibility handling, packaging behavior, and
test coverage.

[x] Phase 0. Refine the runtime-layout scope and acceptance criteria -
    `refining-requirements`
[x] Phase 1. Plan the adaptive stage sequence, validation path, and supervisor
    gates - `planning-project`
[x] Phase 2. Design the canonical runtime topology, packaged-asset boundary,
    and legacy compatibility strategy - `designing-architecture`
[x] Phase 3. Develop the unified path resolution, packaging changes, and test
    updates required by the new layout - `developing-code` plus
    `testing-with-tizenclaw-tests` when daemon-visible behavior changes
[x] Phase 4. Run the scripted host and explicit Tizen build/deploy paths to
    verify provisioning behavior - `building-deploying`
[x] Phase 5. Review the executed evidence, confirm acceptance criteria, and
    record residual risks - `reviewing-code` plus
    `testing-with-tizenclaw-tests` when applicable
[x] Phase 6. Commit the validated change set with the repository commit policy
    if commit execution is requested for this cycle - `managing-versions`
[x] Phase 7. Record the final evaluator verdict and follow-up actions -
    `evaluating-outcomes`

## Supervisor Gates

- Refine -> `.dev/REQUIREMENTS.md` explicitly captures the OpenClaw alignment
  goal, the host and Tizen mutable roots, the packaged-read-only boundary, the
  compatibility expectation, and the required validation paths.
- Plan -> `.dev/WORKFLOWS.md`, `.dev/PLAN.md`, and `.dev/DASHBOARD.md`
  consistently describe the same runtime-layout scope, active phase, next
  action, and risks.
- Design -> the canonical directory contract, packaged-vs-mutable split, and
  migration or compatibility policy are explicit enough that implementation
  files do not need to invent structure.
- Develop -> the intended path-bearing code, scripts, packaging files, and
  tests reflect one coherent layout model and the dashboard is synchronized
  with the real file state.
- Build/Deploy -> `./deploy_host.sh` completes in the foreground for host
  validation, and `./deploy.sh` is executed for the explicit Tizen packaging or
  deployment requirement.
- Test/Review -> executed evidence covers host defaults, Tizen defaults, at
  least one legacy compatibility or migration scenario, and any daemon-visible
  behavior updates.
- Commit -> `.tmp/commit_msg.txt` exists, the commit message follows the
  repository format, and the diff scope matches the validated runtime-layout
  change.
- Evaluate -> a final report exists under `.dev/07-evaluator/` with an
  explicit verdict, residual risks, and any follow-up recommendations.

## Notes

- Execution class: `explicit-tizen`
- Shell path: direct `bash`
- Host-first rule: use `./deploy_host.sh` by default, with `./deploy.sh`
  required later because the user explicitly asked for Tizen installation
  behavior.
- `testing-with-tizenclaw-tests` is expected during development and
  test/review if daemon-visible skill or workspace behavior changes.
- Final evaluator stage is mandatory for this cycle.
- Current design target:
  - host runtime root: `~/.tizenclaw`
  - Tizen runtime root: `/home/owner/.tizenclaw`
  - packaged read-only root: `/opt/usr/share/tizenclaw`
  - canonical skills path: `<runtime_root>/workspace/skills/<skill>/SKILL.md`
