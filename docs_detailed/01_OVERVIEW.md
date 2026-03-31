# TizenClaw Project Overview

## 1. What is TizenClaw?

TizenClaw is a **Rust-native async daemon** designed for Tizen OS that provides **agentic AI capabilities** to the platform. Rather than requiring users to navigate through traditional menus, buttons, and settings screens, TizenClaw enables **intent-driven natural language control** of the entire operating system.

A user says *"turn on Bluetooth and connect to my headphones"* and TizenClaw:

1. Receives the prompt via IPC socket or web dashboard.
2. Sends it to a cloud LLM (Gemini, OpenAI, Anthropic, Ollama, etc.).
3. The LLM decides which **tools** to call (e.g., `bluetooth_toggle`, `bluetooth_pair`).
4. TizenClaw executes those tools on the device.
5. Results flow back to the LLM for a final human-readable response.

This loop -- **Reason, Act, Observe, repeat** -- is known as a ReAct loop. TizenClaw implements it with multi-backend LLM fallback, circuit breakers, session persistence, and sandboxed tool execution.

### Key properties

| Property | Detail |
|---|---|
| Language | Rust (2021 edition) |
| Async runtime | Tokio (multi-thread) |
| IPC | Abstract Unix domain sockets, JSON-RPC 2.0 |
| LLM backends | Gemini, OpenAI/xAI, Anthropic, Ollama (pluggable) |
| Platform support | Tizen OS (via `libtizenclaw` plugin), Generic Linux fallback |
| Web UI | Built-in dashboard on port 9090 (Axum) |
| Tool execution | Subprocess-based with timeout, sandboxed executor daemon |


## 2. Why Rust Instead of C++?

TizenClaw targets resource-constrained Tizen devices (TVs, watches, appliances). The choice of Rust over C++ was deliberate and motivated by concrete engineering concerns:

### Ownership = RAII without the footguns

In C++, RAII works well -- until someone takes a raw pointer to a `unique_ptr`'s contents, or copies a `shared_ptr` into a lambda that outlives the object. Rust's ownership model enforces RAII at compile time. If code compiles, resources are freed exactly once, in the right order. No double-free, no use-after-free, no dangling pointers. Period.

### No malloc_trim hacks

Tizen C/C++ daemons frequently resort to `malloc_trim(0)` calls to return memory to the OS because `glibc` holds onto freed pages. Rust's allocator behavior, combined with `jemalloc` or the system allocator, produces tighter memory profiles. More importantly, Rust's ownership model means memory is freed deterministically at scope exit rather than accumulating in free lists.

### Tokio vs pthread pools

TizenClaw makes concurrent LLM API calls, tool executions, IPC handling, and web serving. In C++, this means hand-rolling a thread pool with `std::thread`, managing `std::mutex` hierarchies, and debugging deadlocks with `helgrind`. Rust's Tokio runtime provides M:N green-thread scheduling out of the box. The `async`/`await` syntax makes concurrent code read like sequential code, and the compiler rejects data races at build time.

### Memory safety guarantees

Every LLM response is untrusted input. Tool scripts return arbitrary data. IPC clients send arbitrary payloads. In C++, a single buffer overread in parsing code creates a CVE. In Rust, bounds checking is automatic, and `unsafe` blocks are explicit and auditable (TizenClaw uses them only for FFI calls to `libc` and signal handlers).

### Zero-cost abstractions

Rust traits, generics, and iterators compile to the same machine code as hand-written C. The `LlmBackend` trait in TizenClaw dispatches through a vtable (like a C++ virtual class), but the compiler inlines and devirtualizes where possible. Release builds use `opt-level = "s"` and `panic = "abort"` to produce binaries sized for embedded deployment.


## 3. Workspace at a Glance

TizenClaw is organized as a Cargo workspace with **6 member crates** and **2 excluded crates** (built separately for Tizen deployment via GBS).

```mermaid
graph TD
    subgraph "Cargo Workspace (members)"
        LIB["libtizenclaw<br/><i>Platform abstraction &amp; plugin loader</i><br/>src/libtizenclaw/"]
        DAEMON["tizenclaw<br/><i>Main daemon (AgentCore, IPC, channels)</i><br/>src/tizenclaw/"]
        CLI["tizenclaw-cli<br/><i>CLI tool (connects via IPC socket)</i><br/>src/tizenclaw-cli/"]
        EXECUTOR["tizenclaw-tool-executor<br/><i>Sandboxed tool execution daemon</i><br/>src/tizenclaw-tool-executor/"]
        CLIENT["libtizenclaw-client<br/><i>C-ABI client library</i><br/>src/libtizenclaw-client/"]
        SDK["libtizenclaw-sdk<br/><i>SDK for plugin developers</i><br/>src/libtizenclaw-sdk/"]
    end

    subgraph "Excluded (built via GBS)"
        SYS["tizen-sys<br/><i>FFI bindings to Tizen C APIs</i><br/>src/tizen-sys/"]
        CORE["tizenclaw-core<br/><i>Core lib built separately</i><br/>src/tizenclaw-core/"]
    end

    DAEMON -->|"Cargo dependency"| LIB
    CLI -.->|"IPC socket<br/>(no library link)"| DAEMON
    EXECUTOR -.->|"IPC socket"| DAEMON
    CLIENT -.->|"IPC socket"| DAEMON
    SDK -->|"links at build time"| SYS
    CLI -->|"Cargo dependency"| SYS

    style LIB fill:#2d5016,color:#fff
    style DAEMON fill:#1a3a5c,color:#fff
    style CLI fill:#5c3a1a,color:#fff
    style EXECUTOR fill:#5c1a3a,color:#fff
    style CLIENT fill:#3a1a5c,color:#fff
    style SDK fill:#1a5c5c,color:#fff
    style SYS fill:#555,color:#fff
    style CORE fill:#555,color:#fff
```

### Crate responsibilities

| Crate | Type | Description |
|---|---|---|
| `libtizenclaw` | `cdylib` + `rlib` | Platform abstraction layer. Defines `PlatformPlugin`, `PlatformLogger`, and other traits. Loads `.so` plugins at runtime via `dlopen`. Falls back to `GenericLinuxPlatform` on non-Tizen hosts. See `src/libtizenclaw/src/lib.rs`. |
| `tizenclaw` | binary | The main daemon. Contains `AgentCore` (the agentic loop), `IpcServer` (JSON-RPC), `ToolDispatcher`, `SessionStore` (SQLite), `ChannelRegistry` (web dashboard, Telegram, Discord, etc.), `TaskScheduler`, and all LLM backends. See `src/tizenclaw/src/main.rs`. |
| `tizenclaw-cli` | binary | Command-line client. Connects to the daemon over the abstract Unix socket `\0tizenclaw.sock`. Does **not** link against `libtizenclaw` -- pure IPC. See `src/tizenclaw-cli/src/main.rs`. |
| `tizenclaw-tool-executor` | binary | Separate daemon that listens on `\0tizenclaw-tool-executor.sock`. Executes tool scripts in sandboxed subprocesses with timeouts. Validates peer credentials via `SO_PEERCRED`. See `src/tizenclaw-tool-executor/src/main.rs`. |
| `libtizenclaw-client` | `cdylib` + `rlib` | C-ABI wrapper so that C/C++ Tizen applications can call TizenClaw without writing Rust. Exports `extern "C"` functions matching `tizenclaw.h`. Thread-safe via `Arc<Mutex<...>>`. See `src/libtizenclaw-client/src/lib.rs`. |
| `libtizenclaw-sdk` | `cdylib` + `rlib` | SDK for plugin developers. Provides C FFI for LLM data types, HTTP helpers, and plugin interfaces. External `.so` plugins link against this. See `src/libtizenclaw-sdk/src/lib.rs`. |
| `tizen-sys` *(excluded)* | rlib | Raw FFI bindings to Tizen C APIs (`app_control`, `package_manager`, `dlog`, etc.). Built separately via GBS for cross-compilation. |
| `tizenclaw-core` *(excluded)* | rlib | Core library built separately via GBS to handle Tizen-specific cross-compilation constraints. |

### Why two excluded crates?

The Tizen build system (GBS/OBS) cross-compiles for ARM and uses its own sysroot. The `tizen-sys` crate wraps Tizen-specific C headers that only exist in the GBS sysroot, and `tizenclaw-core` depends on them. Including them in the workspace would break `cargo build` on developer laptops. Instead, they are built by `deploy.sh` during GBS packaging and linked into the final RPM.


## 4. How to Read These Docs

These documents are designed for different learning paths. Choose the one that matches your goal:

### "I want to understand the whole system"

1. **01 -- Overview** (this document) -- bird's-eye view
2. **03 -- Architecture Deep Dive** -- three-tier topology, boot sequence, concurrency model
3. **04 -- The Agentic Loop** -- ReAct cycle, tool calling, session management
4. **05 -- LLM Backends** -- Gemini/OpenAI/Anthropic/Ollama integration details

### "I want to extend TizenClaw with new tools or skills"

1. **01 -- Overview** (this document) -- understand what tools and skills are
2. **06 -- Tools and Skills** -- tool manifest format, skill scanner, writing your first tool
3. **04 -- The Agentic Loop** -- how tools are dispatched and results fed back

### "I know C++ but not Rust -- help me read this codebase"

1. **02 -- Rust for C++ Developers** -- ownership, traits, async, FFI mapped to C++ concepts
2. **01 -- Overview** (this document) -- project structure
3. **03 -- Architecture Deep Dive** -- how the pieces fit together

### "I want to integrate TizenClaw from my C/C++ Tizen app"

1. **02 -- Rust for C++ Developers**, section 7 (FFI) -- how Rust exposes C APIs
2. **08 -- C API Reference** -- `tizenclaw.h` function-by-function
3. **07 -- IPC Protocol Reference** -- JSON-RPC methods if you prefer socket-level integration

### "I need to build and deploy to a Tizen device"

1. **10 -- Build and Deploy** -- Cargo, GBS, RPM packaging, `deploy.sh`
2. **01 -- Overview** (this document) -- workspace structure for context


## 5. Key Terminology Glossary

| Term | Definition |
|---|---|
| **Agent** | An autonomous software entity that receives goals (natural language prompts), reasons about them, takes actions (tool calls), and iterates until the goal is achieved. TizenClaw's `AgentCore` (`src/tizenclaw/src/core/agent_core.rs`) is the central agent implementation. |
| **ReAct Loop** | **Re**ason + **Act**. The iterative cycle where the agent sends a prompt to the LLM, the LLM returns either a final answer or a tool call, the agent executes the tool and feeds results back, and the LLM reasons again. TizenClaw limits this to `MAX_TOOL_ROUNDS = 10` iterations to prevent runaway loops. |
| **Tool** | An executable (shell script, binary, Python script) registered with the daemon that the LLM can invoke by name. Each tool has a JSON manifest declaring its name, description, parameter schema, timeout, and side-effect classification. Tools live under `data/tools/`. |
| **Textual Skill** | A markdown or text file (under `data/skills/`) containing domain knowledge that is injected into the system prompt. Unlike tools, skills do not execute code -- they provide context to the LLM. Scanned at boot by `textual_skill_scanner.rs`. |
| **System Prompt** | The instruction text sent to the LLM before the conversation. Built dynamically by `PromptBuilder` from `system_prompt.txt`, `SOUL.md` (persona), available tool names, and runtime context (platform, model, data directory). |
| **Circuit Breaker** | A resilience pattern that tracks consecutive LLM backend failures. After 2 consecutive failures within 60 seconds, the backend is temporarily skipped and the next fallback is tried. Implemented in `AgentCore` via `CircuitBreakerState`. |
| **Session** | A conversation context identified by a `session_id` string. Messages are persisted in SQLite via `SessionStore`. The agent loads the last `MAX_CONTEXT_MESSAGES = 20` messages as conversation history for each LLM call. |
| **LLM Backend** | An implementation of the `LlmBackend` trait (`src/tizenclaw/src/llm/backend.rs:95`) that translates TizenClaw's internal message format to a specific LLM API (Gemini, OpenAI, Anthropic, Ollama). The daemon supports one primary and multiple fallback backends. |
| **Channel** | An external-facing communication interface. Implements the `Channel` trait (`src/tizenclaw/src/channel/mod.rs:23`). Examples: web dashboard (port 9090), Telegram bot, Discord bot, webhook receiver, voice input. Channels receive messages from users and forward them to `AgentCore`. |
| **Platform Plugin** | A `.so` shared library implementing the `PlatformPlugin` trait (`src/libtizenclaw/src/lib.rs:36`). Loaded at runtime via `dlopen`. Provides platform-specific logging (`dlog` on Tizen), package management, app control, and system info. Falls back to `GenericLinuxPlatform` when no plugin is found. |
| **IPC** | Inter-Process Communication. TizenClaw uses **abstract namespace Unix domain sockets** (socket paths starting with `\0`) for zero-filesystem-footprint communication between the daemon, CLI, tool executor, and client libraries. |
| **JSON-RPC 2.0** | The wire protocol over IPC sockets. Requests are JSON objects with `jsonrpc`, `method`, `params`, and `id` fields. Responses contain `result` or `error`. TizenClaw supports methods `"prompt"` and `"get_usage"`. Messages are framed with a 4-byte big-endian length prefix. |
