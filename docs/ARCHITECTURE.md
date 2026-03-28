# System Architecture

TizenClaw has undergone a full paradigm shift from its legacy C++ containerized runtime (using `crun`) to a pure **Rust Asynchronous Daemon** powered by `tokio`. This design focuses on strict zero-cost abstractions, absolute memory safety without manual `malloc_trim` tricks, and high concurrency on embedded TVs and smart monitors.

## 1. High-Level Topology

The system operates across three tiers on the device:

### 1-1. The Main Daemon (`tizenclaw`)
The brain of the operation. It runs as a `systemd` background service and listens for queries on the `tizenclaw.sock` UNIX domain socket or WebDashboard SSE streams.
- **`AgentCore`**: Drives the agentic ReAct loop (Reason -> Act -> Return). It injects dynamic prompt contexts and issues payload commands to the tool runner.
- **`TextualSkillScanner`**: Watches for new Markdown intent files `SKILL.md` in the `/opt/usr/share/tizen-tools/skills/` hierarchy and bundles them into the LLM system prompt for dynamic runtime capability enhancement.
- **`ToolWatcher`**: A background task executing filesystem polling (fallback from inotify) that alerts `AgentCore` to newly added binary capabilities.

### 1-2. The Sandboxed Sandbox (`tizenclaw-tool-executor`)
Instead of bundling massive OCI container toolkits, TizenClaw routes dangerous external interactions (like evaluating Python blobs or running bash commands) over to `tizenclaw-tool-executor.sock`.
This companion daemon validates that the connection is authentically from the Main Daemon via **`SO_PEERCRED`**. This separation guarantees that the main `tizenclaw` daemon never blocks and cannot be brought down by a bad syscall from a tool.

### 1-3. The Tizen FFI Base (`tizen-sys` & `libtizenclaw`)
Native embedded OS commands (display, Bluetooth, package manager) must pass through the `tizen-sys` FFI layer using direct C bindings. To support legacy Tizen C-API consumers, the AI agent logic exposes itself backwards through the **`libtizenclaw`** C-ABI bridge, allowing native C-written UIs to request conversational outputs seamlessly via `TizenClaw_SendMessage()`.

## 2. Dynamic Workflow Lifecycle

1. **User Request**: "Turn off the TV and schedule a wake up memory at 6 AM." via `tizenclaw-cli`.
2. **IPC Dispatch**: The CLI payload streams over JSON-RPC 2.0 to the daemon socket listener.
3. **LLM Context Injection**: `AgentCore` asks the `PromptBuilder` to scrape available `SKILL.md` logic and Tizen environment globals.
4. **Execution Route**:
   - The LLM reasons it needs to execute a tool.
   - The daemon proxies `execute_cli` to the `tizenclaw-tool-executor`.
   - The executor shells out to native Tizen DBus / AppControl and returns JSON stdout.
5. **Streaming Return**: `AgentCore` evaluates the result and streams continuous output tokens back to `tizenclaw-cli` chunk by chunk for ultra-low latency interactivity.

## 3. Platform Dependencies

- **Hardware Acceleration**: Built-in ONNX runtime (`tizenclaw-assets`) offloaded for vector embeddings (RAG) directly on local Neural Processing Units (NPUs) or integrated GPUs.
- **SQLite / RAG**: Instead of relying heavily on massive in-memory storage, sessions and conversations are persistently spooled in SQLite using write-ahead logging (WAL mode), dynamically attaching vector-search graphs locally.
