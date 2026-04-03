# Agent Loop V2: Architecture & Design

## Overview
This design document supplements the planning artifact by specifying the Rust structure required to implement Plan-and-Solve and Dynamic Fallback in `tizenclaw-core`.

## Core Subsystem Logic Engine Design
The logic engine operates within the `AgentCore` module (`src/tizenclaw/src/core/agent_core.rs`), processing `AgentLoopState` state transitions asynchronously via Tokio runtime.

### 1. Minimal FFI Principle & Zero-Cost Abstractions
This enhancement modifies pure cognitive logic within `tizenclaw-core`. It interacts safely with any Tizen APIs by simply injecting tool calls. No new `extern "C"` bindings or `libloading` dynamic loading fallbacks are required. All new state transitions are constrained within `Send + Sync` data structures. 

### 2. Async Topology Changes
The `Phase 3 (Planning)` step natively expands its continuous loop context to `await` a one-off `chat_with_fallback` call designed exclusively for Goal parsing. 
- **Tokio Safety**: `chat_with_fallback` manages its own internal locks without holding the core state over `.await` boundaries, preventing cross-task deadlocks.

### 3. Data Structure Extensions
```rust
// tizenclaw/src/core/agent_loop_state.rs modifications:
pub struct AgentLoopState {
    // ... existings
    pub stuck_retry_count: usize, // explicitly monitor fallbacks bounds
}
```

## Validation Protocol
All structural components are integrated via standard Rust idioms, enforcing memory and thread safety across the Tizen ARM/x86_64 target platforms.
