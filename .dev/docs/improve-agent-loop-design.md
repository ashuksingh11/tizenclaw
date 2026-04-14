# Improve Agent Loop Design

## Objective

Reduce the size and responsibility of
`src/tizenclaw/src/core/agent_core/process_prompt.rs` without changing the
public entry point `AgentCore::process_prompt`.

## Scope Of This Refactor

- Keep `process_prompt()` as the phase-oriented orchestrator.
- Extract task- and scenario-specific prompt contract injection into a focused
  helper module.
- Extract the main LLM/tool execution loop and completion routing into a
  focused helper module.
- Keep existing helper functions in `content_and_workspace.rs`,
  `news_and_grounding.rs`, `research.rs`, and `prompt_completion.rs`
  unchanged unless small adjustments are needed for the new boundaries.

## Target File Boundaries

### `process_prompt.rs`

Own only the high-level sequence:

1. initialize loop state and early guards
2. load conversation history and base context
3. build tool declarations and the system prompt
4. apply prompt contracts and synthetic prefetch/setup helpers
5. hand off to the execution loop helper

### `process_prompt_contracts.rs`

Own prompt-specific context and contract injection that currently clutters the
common orchestration path, including:

- working-directory and explicit-file grounding messages
- direct-tool hints and research/file-writing contracts
- long-form writing, concise summary, briefing, email, humanization, and
  prediction-market guidance
- prefetch-driven context such as Polymarket snapshots
- any scenario-specific setup that should remain isolated from the generic loop

### `process_prompt_loop.rs`

Own the runtime execution loop after prompt preparation, including:

- pre-loop planning/compaction setup
- workflow-mode handling
- LLM dispatch and fallback parsing
- tool execution, result budgeting, and loop-state updates
- completion checks for file outputs, grounded answers, and round limits

## Intentional Multilingual Handling

The production agent loop still needs a small amount of multilingual matching
for supported prompts and task fixtures. Korean literals that remain after the
refactor should be limited to localized parsing or keyword matching, with
English comments explaining why they are retained.

## Validation Seam

The refactor touches daemon-visible prompt orchestration, so validation must
cover:

- `./deploy_host.sh`
- at least one deterministic `tests/system/` runtime-contract scenario for the
  affected agent loop behavior

## Stop Condition

This iteration is complete when `process_prompt.rs` reads mainly as
orchestration over named helpers and the host-default validation path passes.
