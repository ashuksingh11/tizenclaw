# DASHBOARD

## Actual Progress

- Goal: Runtime flexibility and operator maintainability improvements
- Active roadmap focus: Provider selection, Telegram config externalization,
  ClawHub update flow, skill snapshot caching
- Current workflow phase: evaluate
- Last completed workflow phase: commit
- Supervisor verdict: `approved`
- Escalation status: none

## Workflow Phases

```mermaid
flowchart LR
    refine([Refine]) --> plan([Plan])
    plan --> design([Design])
    design --> develop([Develop])
    develop --> build_deploy([Build/Deploy])
    build_deploy --> test_review([Test/Review])
    test_review --> commit([Commit])
    commit --> evaluate([Evaluate])
```

## Phase Status

| Phase | Status | Evidence |
|---|---|---|
| 0. Refine | complete | `.dev/REQUIREMENTS.md` covers all four feature areas |
| 1. Plan | complete | `.dev/WORKFLOWS.md`, `.dev/PLAN.md`, `.dev/DASHBOARD.md` updated |
| 2. Design | complete | Design in prompt; all ambiguities resolved |
| 3. Develop | complete | All four subsystems implemented |
| 4. Build/Deploy | complete | `./deploy_host.sh --test` all pass |
| 5. Test/Review | complete | 603+ unit tests pass, 0 failures |
| 6. Commit | complete | dev tracking state committed |
| 7. Evaluate | complete | see evaluator section below |

## Feature Implementation Status

### Provider Selection (provider_selection.rs)
- `ProviderCompatibilityTranslator` normalizes legacy and new config
- `ProviderRoutingConfig` owns preference-ordered provider list
- `ProviderRegistry` runtime catalog with status JSON
- `ProviderSelector` pure selection policy with enabled/circuit checks
- `AgentCore` uses registry for all request routing
- Tests: 15+ unit tests covering all routing scenarios

### Telegram Model Configuration (types.rs, client_impl.rs)
- `read_backend_models_from_llm_config` merges from `llm_config.json`
- `telegram_config.json.cli_backends` takes precedence
- `llm_config.json.telegram.cli_backends` as secondary source
- `llm_config.json.backends.<provider>.model` as fallback
- Model choices operator-managed without rebuild

### ClawHub Update Flow (clawhub_client.rs, ipc_server.rs, main.rs)
- `clawhub_update()` reads lock file and re-installs all tracked skills
- `update_one_skill()` handles one entry with staging/validate/atomic-replace
- Result buckets: `updated`, `skipped`, `failed`
- IPC: `clawhub_update` method exposed via daemon
- CLI: `clawhub update` command exposed

### Skill Snapshot Cache (skill_capability_manager.rs)
- `SNAPSHOT_CACHE` process-global cache with `Mutex` protection
- `SkillSnapshotFingerprint` deterministic invalidation key
- `load_snapshot()` returns cache hit when fingerprint matches
- `invalidate_snapshot_cache()` explicit hook for install/update/reload
- `SkillRootSignature` tracks SKILL.md mtimes for in-place edit detection
- Tests: cache hit, invalidation, SKILL.md edit detection

## Validation Evidence

- `./deploy_host.sh --test`: all test suites pass
  - `tizenclaw` crate: 603 passed, 0 failed
  - `tizenclaw-cli` crate: 21 passed, 0 failed
  - All other crates: pass
  - Mock parity harness: PASS
  - Doc architecture verification: PASS

## Risks And Watchpoints

- Provider routing is ordered-preference only (request-aware routing deferred)
- ClawHub update always refreshes (no speculative skip if registry shows same version)
- Snapshot cache uses process-global state; test isolation requires explicit clear
- Telegram model config precedence: telegram_config.json > llm_config.json.telegram > llm_config.json.backends.<provider>.model > built-in empty
