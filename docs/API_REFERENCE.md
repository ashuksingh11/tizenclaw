# API & Integration Reference

TizenClaw acts as the primary AI hub on a Tizen Device. Thus, interacting with the daemon is supported via several programmatic paths mapping back into the asynchronous IPC loop. The Rust transformation ensures optimal safety for parsing all IPC messages, preventing legacy C++ segmentation-fault style crashing under malformed payloads.

## 1. `tizenclaw-cli` - The System Gateway

If you are invoking conversational capabilities natively over the bash commandline, or automating shell responses, the IPC bridge application is the simplest interface to the backend.

```bash
tizenclaw-cli [options] [prompt]
```

**Common Flags**
- `-s <id>`: Define a separate working session ID (state isolation from background streams).
- `--stream`: Yield chunks sequentially in real-time (optimal for UI displays).
- `--usage`: Output JSON describing LLM token performance (e.g. `{"completion_tokens": 120, "prompt_tokens": 8000}`).

Calling interactive endpoints defaults to interactive repl loops locking stdio buffer polling contexts efficiently.

## 2. `libtizenclaw` - The C-ABI Bindings

Because Tizen utilizes legacy EFL and modern Native C/C++ libraries, Rust naturally exposes C-Linkable `.so` shared symbols under the `libtizenclaw` crate.

A developer constructing a GUI native Tizen App can include our Headers and statically map to Rust symbols without knowing anything about internal LLM mechanisms.

**Rust Implementation Details**

The `libtizenclaw::api` dynamically allocates a cross-runtime asynchronous handler utilizing memory-safe `CString` structures for pointer interactions:
```rust
use tizenclaw::api::TizenClaw;
// TizenClaw::new("App_Context", ...) initializes internal thread wrappers allowing C to call
// `.send_message("prompt", callback_fn_ptr)`
```

## 3. WebDashboard REST (`axum`)

Administrative panels, memory state modifications, or `llm_config.json` rewrites run efficiently over HTTP/1.1 ports (Default `9090`) exposed securely inside firewall bounds. These connect to local or browser networks depending on Tizen IP Routing permissions.

- `POST /api/chat`: Forward chat payloads.
- `GET /api/metrics`: Internal prometheus metrics payload tracking total IPC calls, active memory contexts, and LLM throughput averages.

Integration paths support full SSE (Server-Sent Event) data channels for front-end stream tracking via standard HTTP logic frameworks.
