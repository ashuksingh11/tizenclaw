# Tool Execution Audit Design

## Scope

This cycle resumes the rework-required follow-up run with a repository-visible
slice that makes wrapper and prelude trust observable before execution:

1. audit shell and runtime wrappers inside `ToolDispatcher`
2. expose tool audit summaries through daemon IPC and the CLI
3. attach skill prelude metadata to capability summaries before prompt
   prefetch injects the selected skills

## Ownership Boundaries

- `core/tool_dispatcher.rs`
  owns descriptor parsing, wrapper/runtime classification, and execution-time
  audit logging for external textual tools
- `core/textual_skill_scanner.rs`
  owns textual skill prelude extraction and fenced-command language detection
- `core/skill_capability_manager.rs`
  exposes the skill prelude audit fields through daemon-visible capability
  summaries
- `AgentCore`
  remains the composition root, records prefetched skill audit logs, and
  returns tool audit summaries through session/runtime IPC
- `tizenclaw-cli`
  surfaces the daemon-reported tool audit payload through `tools status`

## Persistence And Runtime Impact

- no new persistent state file is required for this slice
- tool audit data is derived from loaded tool descriptors in memory
- skill prelude audit data is derived from textual `SKILL.md` bodies at scan
  time and reported read-only through existing capability IPC

## Runtime Contract

`get_session_runtime` gains a `tool_audit` object with:

- total tool count
- runtime-wrapper and shell-wrapper counts
- inline-command-carrier count
- missing-binary count
- per-tool wrapper/runtime/trust metadata

`get_tool_audit` and `tizenclaw-cli tools status` expose the same audit payload
without requiring a session id.

Skill capability summaries now include:

- `prelude_excerpt`
- `code_fence_languages`
- `shell_prelude`

## Verification Contract

- align `tests/system/basic_ipc_smoke.json` with the new tool-audit fields and
  IPC method
- add focused unit coverage for wrapper classification and skill prelude
  extraction
- validate through `./deploy_host.sh -b`, `./deploy_host.sh`,
  `~/.tizenclaw/bin/tizenclaw-tests scenario --file tests/system/basic_ipc_smoke.json`,
  `~/.tizenclaw/bin/tizenclaw-cli tools status`, and `./deploy_host.sh --test`
