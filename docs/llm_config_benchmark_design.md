# TizenClaw LLM Config Benchmark Design

## Scope

This design adds a `tizenclaw-cli` configuration surface for
PinchBench-oriented runtime setup using the existing
`<config_dir>/llm_config.json` file. The goal is to let operators manage
Anthropic and Gemini settings in a workflow similar to OpenClaw or
ZeroClaw without introducing a second benchmark-only config file.

## Ownership

- `src/tizenclaw-cli/src/main.rs`
  adds `config get`, `config set`, `config unset`, and `config reload`
  commands.
- `src/tizenclaw/src/core/ipc_server.rs`
  exposes new JSON-RPC methods for reading and mutating `llm_config.json`
  and for reloading live backends.
- `src/tizenclaw/src/core/agent_core.rs`
  owns config-file resolution, atomic persistence, and backend reload
  orchestration.
- `src/tizenclaw/src/llm/anthropic.rs`
  and `src/tizenclaw/src/llm/gemini.rs`
  accept new provider defaults from `llm_config.json`, including
  `temperature` and `max_tokens`.
- `data/sample/llm_config.json.sample`
  documents the supported schema, including PinchBench metadata.

## Config Model

`llm_config.json` keeps the existing top-level backend selection fields
and gains an optional benchmark section:

```json
{
  "active_backend": "anthropic",
  "fallback_backends": ["gemini"],
  "benchmark": {
    "pinchbench": {
      "actual_tokens": {
        "prompt": 0,
        "completion": 0,
        "total": 0
      },
      "target": {
        "score": 0.8,
        "summary": "match the OpenClaw comparison run",
        "suite": "all"
      }
    }
  },
  "backends": {
    "anthropic": {
      "api_key": "",
      "model": "claude-sonnet-4-20250514",
      "temperature": 0.7,
      "max_tokens": 4096
    },
    "gemini": {
      "api_key": "",
      "model": "gemini-2.5-flash",
      "temperature": 0.7,
      "max_tokens": 4096
    }
  }
}
```

Rules:

- `active_backend` and `fallback_backends` remain the live selection
  inputs.
- `backends.<name>.api_key` may be set directly in `llm_config.json`.
  Existing `keys.json` fallback support remains valid for compatibility.
- `backends.<name>.temperature` and `backends.<name>.max_tokens` become
  provider defaults used when a request does not override them at call
  time.
- `benchmark.pinchbench.*` is daemon-readable metadata intended for CLI
  management and operator workflows; it does not introduce a new daemon
  execution loop.

## CLI Contract

The new CLI surface is intentionally close to OpenClaw-style config
management:

- `tizenclaw-cli config get`
- `tizenclaw-cli config get <path>`
- `tizenclaw-cli config set <path> <value>`
- `tizenclaw-cli config set <path> <json> --strict-json`
- `tizenclaw-cli config unset <path>`
- `tizenclaw-cli config reload`

Behavior:

- Dot-separated paths such as `backends.anthropic.model` address nested
  JSON objects.
- Default `set` behavior stores the value as a JSON string.
- `--strict-json` parses the provided value as JSON so numbers, booleans,
  arrays, and objects can be written safely.
- `config get` without a path returns the full parsed document.
- `config reload` asks the daemon to re-read `llm_config.json` and
  reinitialize live LLM backends.

## IPC Topology

New JSON-RPC methods:

- `get_llm_config`
- `set_llm_config`
- `unset_llm_config`
- `reload_llm_backends`

Flow:

1. `tizenclaw-cli` sends a config mutation request.
2. The daemon updates `llm_config.json` under `config_dir`.
3. The daemon returns the updated document or selected value.
4. `config reload` triggers `AgentCore::reload_backends()`.

The mutation path stays daemon-owned so the CLI does not need to know
the runtime config directory on Tizen versus host Linux.

## Persistence and Safety

- `llm_config.json` writes are synchronous one-shot file operations.
- The persistence helper creates `config_dir` if it does not exist.
- Writes use JSON pretty-printing for stable operator review.
- Path traversal is not involved because callers may mutate JSON values
  only inside the fixed `llm_config.json` location resolved by
  `PlatformPaths`.

## Async and Concurrency Notes

- No new long-lived worker is introduced.
- Config reads and writes are one-shot operations invoked through the
  existing IPC request path.
- Live backend replacement remains inside the current `RwLock`-guarded
  backend ownership in `AgentCore`, so the design preserves the existing
  `Send + Sync` model.
- FFI boundaries remain unchanged because the work is confined to pure
  Rust JSON persistence, IPC, and HTTP payload shaping.

## libloading Strategy

No new `libloading` behavior is required. Plugin backend loading keeps
the current strategy already used by `PluginManager` and
`PluginLlmBackend`.

## Failure Handling

- Missing `llm_config.json`: `config get` returns the default document
  shape rather than failing hard.
- Invalid JSON path: mutation requests return a structured IPC error.
- Reload failure: the daemon returns a failure response while preserving
  the last successfully initialized backend set.
- Missing `api_key`: initialization continues to follow the current
  backend rules, allowing `keys.json` or environment fallbacks.

## Design Checklist

- [x] Assign ownership for CLI, IPC, config storage, and backend use
- [x] Preserve the current `Send + Sync` backend reload model
- [x] Confirm no new FFI boundary and no new `libloading` requirement
- [x] Keep benchmark metadata in `llm_config.json` instead of a separate
  file
