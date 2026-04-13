[O] Phase 1. Re-read `AGENTS.md`, confirm the host-default resume cycle, and
    record the current supervisor failure root cause in `.dev/DASHBOARD.md`.
[O] Phase 2. Define the resume-cycle design boundaries for the benchmark
    rerun and synchronize both `PLAN.md` copies with that design.
[O] Phase 3. Run `./deploy_host.sh` and execute a fresh full
    `python3 scripts/run_pinchbench_oauth.py --suite all --runs 1
    --no-stream-runtime-io` benchmark from scratch.
[O] Phase 4. Run `./deploy_host.sh --test`, review the host runtime
    evidence, and update `.dev/SCORE.md`, `docs/BENCHMARK.md`, and
    `.dev/DASHBOARD.md` from the verified results.
[O] Phase 5. Clean the workspace, commit the synchronized resume-cycle
    changes with `.tmp/commit_msg.txt`, and leave final verification
    evidence in `.dev/DASHBOARD.md`.
