# Multi-Agent Coordination Capability (Phase 3)

## 1. Goal
Provide autonomous device-to-device task delegation and dynamic execution discovery via mDNS and A2A JSON-RPC. This enables multiple TizenClaw daemons within the same local network to discover each other and collaborate seamlessly entirely without user intervention.

## 2. Capability Details
- **mDNS Network Scanner (Zero-Config)**
  - **Description**: Discover and register other TizenClaw daemons dynamically on the local network (e.g., `_tizenclaw._tcp.local`).
  - **Inputs**: Service announcements and queries via mDNS broadcast in the local subnet.
  - **Outputs**: A live registry table containing accessible peer agent URLs, identifiers, and capabilities.
  
- **A2A Task Dispatcher (Agent-to-Agent)**
  - **Description**: Wire the existing `A2aHandler` stub into the real `AgentCore` via an asynchronous channel. Real `tasks/send` requests from other agents will be appended to the agent's active memory context.
  - **Inputs**: JSON-RPC 2.0 request payloads on the A2A endpoints (`tasks/send`, `tasks/get`).
  - **Outputs**: Asynchronous update propagation on `tasks/get`, yielding the LLM's true execution result to the requesting agent.

## 3. Execution Mode Classification
- **mDNS Network Scanner**: **Daemon Sub-task** (An internal persistent asynchronous background loop monitoring UDP multicast port for announcements and keeping a heartbeat cache).
- **A2A Task Dispatcher**: **Streaming Event Listener** (Listens to the configured network interface, decodes A2A JSON-RPC payloads, and injects tasks into the `AgentCore` channel).

## 4. Resource & Power Mitigation in Embedded Tizen
- **mDNS Network Hibernation**: mDNS service broadcasts will apply a controlled sleep-wake cycle. When no other peers are active, the broadcast interval is extended exponentially, reducing Wi-Fi radio wakeups.
- **Zero-Allocation Passthrough**: The A2A handler limits heap allocations when buffering task messages, dropping overly large requests at the network edge to prevent OOM scenarios.
- **Dynamic Fallback**: If network restrictions block multicast packets or the Wi-Fi physical interface is down, the module silently falls back to operating strictly in isolated local mode without crashing the daemon.
