# TizenClaw LLM Cache Telemetry Design

## Scope

This design improves token-saving observability for Anthropic and Gemini
backends. The goal is to expose whether prompt caching is actually being
used and how many prompt tokens were written into or read from cache,
without changing the existing conversation flow.

## Ownership

- `src/tizenclaw/src/llm/backend.rs`
  extends the normalized response model with cache-related token fields.
- `src/tizenclaw/src/llm/anthropic.rs`
  parses Anthropic usage cache fields and emits cache-aware logs.
- `src/tizenclaw/src/llm/gemini.rs`
  normalizes Gemini cached token metadata into the same response model.
- `src/tizenclaw/src/storage/session_store.rs`
  persists cache token counters and loads aggregated totals.
- `src/tizenclaw/src/core/agent_core.rs`
  records round and cumulative cache usage in runtime logs.
- `src/tizenclaw/src/core/ipc_server.rs`
  extends `get_usage` output with cache token totals.

## Normalized Cache Usage Model

Each LLM round may carry two optional counters:

- `cache_creation_input_tokens`
  prompt tokens newly written into server-side cache
- `cache_read_input_tokens`
  prompt tokens served from cache instead of full prompt re-send

These fields are normalized at the `LlmResponse` layer so different
provider-specific payloads can be reported consistently.

## Provider Mapping

### Anthropic

- Read from `usage.cache_creation_input_tokens` when present.
- Read from `usage.cache_read_input_tokens` when present.
- Continue sending the existing prompt-caching request hints:
  `anthropic-beta: prompt-caching-2024-07-31` and
  `cache_control: {"type":"ephemeral"}` in the system prompt block.

### Gemini

- Continue using `CachedContent`.
- Map `usageMetadata.cachedContentTokenCount` to
  `cache_read_input_tokens`.
- `cache_creation_input_tokens` remains zero because the current Gemini
  cache creation call does not return an equivalent normalized prompt
  write count in the chat response path.

## Persistence and Reporting

`token_usage` gains two additive counters:

- `cache_creation_input_tokens`
- `cache_read_input_tokens`

The store must handle existing SQLite databases by adding the new
columns lazily during initialization if they do not exist yet.

`get_usage` returns:

- `prompt_tokens`
- `completion_tokens`
- `cache_creation_input_tokens`
- `cache_read_input_tokens`
- `total_requests`

## Safety and Runtime Notes

- No new FFI boundary is introduced.
- No `libloading` behavior changes.
- The change is limited to pure Rust response parsing, SQLite storage,
  and IPC reporting.
- Existing prompt flow remains intact; only telemetry depth improves.

## Design Checklist

- [x] Normalize Anthropic and Gemini cache counters in one response model
- [x] Persist cache usage across sessions in SQLite
- [x] Extend IPC usage reporting without changing prompt execution
- [x] Keep the implementation inside pure Rust with no new FFI surface
