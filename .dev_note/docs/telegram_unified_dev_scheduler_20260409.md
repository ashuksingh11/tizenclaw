# Telegram Unified Development Scheduler Plan

## Scope

- Keep the default host-first cycle.
- Remove the requirement to switch Telegram into a separate coding mode
  before development work can start.
- Let ordinary Telegram messages reach TizenClaw first, then allow the
  daemon to invoke a local coding agent CLI as a tool when development
  work is actually needed.
- Reuse the same development execution path for scheduled tasks.

## Recommended Structure

### 1. Unified development entry

- Add a built-in `run_coding_agent` tool to `AgentCore`.
- The tool executes the configured local coding-agent CLI (`codex`,
  `gemini`, `claude`, or config-defined custom backends).
- Telegram chat mode remains the primary user entrypoint. Chat requests
  are wrapped with coding preferences such as project directory, backend,
  model, execution mode, and auto-approve state.
- The agent decides whether to answer directly, call normal tools, or
  call `run_coding_agent`.

### 2. Shared coding-agent executor

- Extract a reusable coding-agent execution helper from the Telegram CLI
  runtime path instead of keeping the CLI invocation as Telegram-only
  behavior.
- The helper must load backend definitions from `telegram_config.json`
  and support:
  - default backend resolution
  - backend-specific invocation templates
  - model overrides
  - project directory overrides
  - auto-approve flags
  - response and usage extraction

### 3. Scheduler integration

- Extend scheduled task definitions so they can store development
  context:
  - `project_dir`
  - `coding_backend`
  - `coding_model`
  - `auto_approve`
  - `execution_mode`
- When a task becomes due, `TaskScheduler` should execute the task
  through `AgentCore::process_prompt()` instead of only logging that the
  task is due.
- The scheduler should prepend the stored development context to the
  scheduled prompt so the agent can call `run_coding_agent` with the same
  defaults used by Telegram.

## Safety rails

- Keep TizenClaw as the first decision-maker. The local coding CLI is a
  tool, not the top-level Telegram mode.
- Reuse existing Telegram-configured backend allowlists and invocation
  definitions instead of hardcoding a new execution stack.
- Preserve project-directory resolution and explicit path validation.
- Preserve auto-approve as an explicit opt-in state.
- Scheduled tasks should run under dedicated session IDs to avoid mixing
  unrelated periodic work into ad-hoc user chat sessions.

## User-visible behavior target

- Telegram users can ask for development work directly in the normal
  chat flow.
- `/coding_agent`, `/project`, `/mode`, and `/auto_approve` become
  preference-setting commands for future development requests rather than
  a separate execution mode switch.
- Periodic development work can be registered through normal agent
  requests that call `create_task` with the same coding context.
