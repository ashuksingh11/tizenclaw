# Autonomous Agent TDD & Test Guide

This document is the detailed drafting convention that the `developing-code` agent skill must reference when following the TDD procedure for the TizenClaw daemon.

## 1. Test Strategy by Layer

### 1-1. Unit Test
- External dependencies (e.g., Tizen Device API, Native Sensors, Event Loops, File System) MUST be mocked (utilizing `mockall` or standard FFI trait mocking).
- Verify the integrity of autonomous logic loops (sensor data parsing, state machine transitions, event prioritization, capability triggers). Ensure no panic unrolls.

### 1-2. Integration Test
- Test the asynchronous data flow when two or more components (e.g., IPC Communication Module + Native Tizen FFI Adapter) are heavily loaded concurrently.
- By referring to the existing TCT test code within the `/home/hjhun/tizen/` platform sources, model the expected Tizen API blocking behavior or callback cadences and guarantee the `tokio` runtime gracefully handles backpressure.

### 1-3. System Daemon Test
- Apply when necessary, and define the End-to-End behavior of whether the final `tizenclaw` binary initializes perfectly, attaches to systemd/startup without fatal crashes, and reacts asynchronously to stimuli (e.g. valid DBus/IPC requests) under actual emulator environments.

## 2. Detailed Execution of the TDD Cycle (Red-Green-Refactor)

1. **Write Asynchronous Test Skeleton (Red)**:
   - Create a hollow async trait execution or daemon component structure without any logic, and establish a robust `#[tokio::test]` unit test case that invokes this component. Verify that this structural boundary "Fails" explicitly.
2. **Principle of Minimal Implementation (Green)**:
   - Only implement the necessary asynchronous logic inside `src/` to clear the failed test code above. Do not prematurely optimize or induce `unsafe` memory blocks; keep it purely logical and strictly typed.
3. **Safety & Concurrency Driven Refactoring (Refactor)**:
   - With test coverage secured, refactor the `Arc<Mutex<T>>` or `tokio::sync::mpsc` lifetimes. Fortify object lifetimes against memory leaks and ensure cross-architectural compatibility (pointer mappings). Validate the Tizen CAPI style parameters against strict Rust `const` semantics.

## 3. Test Code Naming and Structuring Rules

```rust
// Example: Naming rules when utilizing #[tokio::test] for the Agent
#[tokio::test]
async fn agent_state_machine_should_ignore_malformed_sensor_data() {
    // Arrange: Initialize mocked inputs and valid async Context
    
    // Act: Invoke the autonomous perception task with garbage data
    
    // Assert: Verify the Agent transitions elegantly to Recover/Ignore state, returning Ok() or defined core standard Error.
}
```

Every autonomous logic generated through this guide will possess flawless test coverage ensuring no production AI agent crashes exist, and must be delivered to the Release Engineer as uncompromised software.

//turbo-all
