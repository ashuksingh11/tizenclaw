# Multi-Agent Architecture Blueprint (Phase 3)

## 1. Async Topologies & Channel Mapping
To seamlessly link the `A2aHandler` with the main cognitive engine, the system will employ a non-blocking asynchronous mapping to bridge the JSON-RPC endpoints to the `AgentCore` tasks.
- **`A2aHandler`**: Must be modified to accept an `Arc<AgentCore>`. When `tasks/send` is hit, it will update task status to `Working` and kick off a detached background task using `tokio::spawn` that executes `AgentCore::process_prompt(session_id, msg)`.
- Upon future completion, the background task updates the internal task registry to `Completed` and assigns the LLM's raw output string to the `artifacts` field of the JSON-RPC representation.

## 2. mDNS Network Discovery
- **Dependency Strategy**: Use the pure Rust `mdns-sd` crate. This complies strictly with the *Minimal FFI Principle*, avoiding brittle Tizen native C-API dependencies and simplifying `Send + Sync` semantics.
- **Topology**: A dedicated `tokio::spawn` loop (Daemon Sub-task) will continuously probe and listen for `_tizenclaw._tcp.local` services dynamically across the subnet.
- **State Handling**: Actively discovered peers will be stored in an `Arc<RwLock<HashMap<String, Peer>>>` which `AgentCore` can consult when performing Multi-Agent delegatory tool invocations.

## 3. Threat-Safe Fallbacks (Zero-Cost Abstractions)
- If UDP multicasting is restricted by the Tizen Network Firewall (e.g., standard profile restrictions) or the port is in use, the error is intercepted and suppressed via `thiserror` mapping. The mDNS polling loop degrades into an exponentially backed-off sleep cycle rather than crashing/spinning.
- `A2aHandler` JSON-RPC extraction remains fully zero-allocation on the HTTP path until validation passes, preventing network-based OOM DOS vectors on embedded profiles.

## 4. FFI Boundaries & Ownership
- **Minimal FFI**: Multi-Agent communication runs solely on Tokio async runtime over TCP/UDP. No `extern "C"` `libloading` transitions are required. 
- **Safe Traits**: Closure captures for the background execution are rigidly bounded by `Send + Sync + 'static`, keeping the HTTP listener thread free to receive immediate polling requests (`tasks/get`).
