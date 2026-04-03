# Telegram Channel Integration Architecture Blueprint

## Overview
This document outlines the zero-cost async integration of the Telegram Client module within the TizenClaw daemon, enabling dynamic LLM processing of incoming messages without blocking the primary continuous execution loops.

## FFI Boundaries & Dynamic Loading
- **FFI Principle**: No Tizen C-API FFI boundaries are required for the Telegram integration, adhering to the minimal FFI principle. Network communication is handled via embedded Rust HTTP clients operating independently of `libcurl` C structures where possible.
- **Dynamic Loading**: `TelegramClient` operates as an attachable Channel configuration. If network access fails, it will back off without aborting the daemon.

## Asynchronous Topologies
- **Threading Model**: 
  - The `TelegramClient` long-polling loop currently operates inside an isolated `std::thread`. To bridge into the `AgentCore` async ecosystem, a local `tokio::runtime::Runtime` will be instantiated inside the polling thread.
  - This allows the blocking `HttpClient::get_sync` polls to continue sequentially, while incoming messages are offloaded to `rt.spawn` tasks for concurrent processing.
- **Message Dispatch**:
  - `AgentCore::process_prompt(session_id, text, None)` is invoked asynchronously.
  - The `session_id` is derived from the Telegram `chat_id` (e.g., `tg_{chat_id}`) to maintain distinct persistent Context Engine histories per user.
- **Resource Constraints**:
  - The number of concurrent handlers will be artificially limited or gracefully degraded depending on configuration to prevent OOM on embedded devices.

## Module State Machines
1. **Init**: `channel_factory::create_channel` instantiates `TelegramClient::new(config, agent.clone())`.
2. **Polling Event Loop**: Executes inside a background thread.
3. **Dispatch**: On message, spawns an async task -> `AgentCore` evaluates -> Response returned.
4. **Responder**: `send_telegram_message` delivers the final agentic output.

## Code Path Modifications
- `src/tizenclaw/src/channel/telegram_client.rs`: Update `new` signature, incorporate `AgentCore` injection, update polling loop block.
- `src/tizenclaw/src/channel/channel_factory.rs`: Pass `agent` to `TelegramClient::new`.
