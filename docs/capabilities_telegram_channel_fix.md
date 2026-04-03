# Telegram Channel Integration Capability

## 1. Project Cognitive Requirements & Target Analysis
**Goal:** Restore multi-channel interaction capability by enabling the Telegram client to forward incoming messages to the AgentCore for autonomous cognitive processing.
**Context:** TizenClaw currently supports a web interface and A2A. The `telegram_client` module receives chat IDs and text via long-polling but lacks the systemic linkage to push observations into the Agent context engine. The lack of integration causes all incoming telegram messages to be dropped.
**Expected Fix:** Inject the `AgentCore` integration into the `telegram_client.rs` module, allowing it to invoke `AgentCore::process_prompt` asynchronously.

## 2. Agent Capabilities Listing and Resource Context
- **Goal of Cognitive Action:** Seamless text-based interaction from Telegram users.
- **Inputs Required:** Telegram JSON `getUpdates` payload containing `chat_id` and `text`.
- **Outputs Generated:** LLM response text pushed back to the Telegram API via `sendMessage`.
- **Mitigation of Constraints:** The HTTP polling must not block the main Tokio runtime or the `AgentCore` mutexes heavily. `tokio::task::spawn` will allow concurrent resolution of responses.

## 3. System Integration Planning
- **Module Convention:** `tizenclaw` application binary (`src/channel/telegram_client.rs` & `src/channel/channel_factory.rs`).
- **Execution Mode Classification:** **Streaming Event Listener**. The TelegramClient polls in a loop and propagates incoming messages into the `AgentCore` context state.
- **Subsystem Logic Path:**
  1. Initialize `TelegramClient` with `AgentCore` clone.
  2. Start standard std::thread polling.
  3. On message detection, use an async block or `tokio::runtime::Handle::current().spawn` if `AgentCore::process_prompt` requires async (wait, `TelegramClient` runs in a `std::thread`, so we need to pass a `tokio::runtime::Handle` or use `tokio::runtime::Runtime::new()` or block_on. Wait, `AgentCore::process_prompt` is async!).
  4. Once `process_prompt` returns, use `send_telegram_message` to dispatch the output back to the bot.
  
## Fallback Capabilities
If the token is invalid, log and abort listener cleanly. No hardware dependency on Tizen native functions.
