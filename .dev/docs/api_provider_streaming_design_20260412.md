# Prompt 33 Design: API Provider and Streaming Layer

## Scope

Reconstruct `rust/crates/tclaw-api` as the canonical Rust workspace crate for
provider-agnostic model communication.

## Ownership Boundaries

- `types.rs`
  - Stable request/response and streaming contracts used by runtime, CLI, and
    tests.
- `error.rs`
  - Typed failure surface shared by provider implementations and downstream
    callers.
- `http_client.rs`
  - Mockable request/response transport seam with no baked-in network runtime.
- `sse.rs`
  - Provider-neutral SSE frame parsing.
- `prompt_cache.rs`
  - Stable prompt cache request/response metadata.
- `client.rs`
  - Public provider-agnostic client traits and wrapper API.
- `providers/anthropic.rs`
  - Anthropic request/response translation only.
- `providers/openai_compat.rs`
  - OpenAI-compatible request/response translation only.

## Runtime and Persistence Impact

- No daemon persistence changes.
- No host/Tizen path split inside this crate.
- The crate stays pure Rust and transport-agnostic so it can be reused by
  runtime code and offline tests without live network dependencies.

## Concurrency and Trait Rules

- `HttpClient` and `ProviderClient` are `Send + Sync`.
- Test doubles store request history behind `Mutex`, keeping the public API
  re-entrant for parallel test harnesses.
- Streaming is explicit through typed `StreamEvent` values, not concatenated
  strings.

## FFI and Dynamic Loading Boundaries

- This crate has no FFI boundary.
- No `libloading` usage is introduced here.
- Provider differences are isolated in provider adapters rather than dynamic
  loading or platform-specific linkage.

## IPC-Observable Assertions

- No direct daemon IPC assertion applies yet because this prompt targets the
  canonical `rust/` workspace crate only.
- Verification instead relies on crate-local tests for:
  - SSE parsing correctness
  - error decoding surfaces
  - one end-to-end streaming path via a mock `HttpClient`
