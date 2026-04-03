# IPC Server Non-Blocking Fix Planning

## Step 1: Cognitive Requirements & Target Analysis
- **Goal**: Prevent Tokio worker thread stalls during Unix domain socket message streaming.
- **Analysis**: We identified that while standard socket communication blocks safely within an isolated `std::thread`, the token streaming loop uses `tokio::spawn` and executes `libc::write`. If the kernel send buffer fills up, this blocking syscall will stall a Tokio worker thread, violating Rust non-blocking concurrency paradigms.

## Step 2: Agent Capabilities Listing and Resource Context
- Wrap the raw `IpcServer::send_response` execution within `tokio::task::spawn_blocking` specifically inside the `stream` token streaming chunk loop.
- This isolates the potential blocking OS thread from the asynchronous Tokio reactor pool, resolving the potential jitter without requiring a full `tokio::net` refactoring that might disturb the precise abstract namespace binding (`\0tizenclaw.sock`).

## Step 3: Agent System Integration Planning
- **Module Convention**: Code updates reside purely internally within `tizenclaw` (`src/tizenclaw/src/core/ipc_server.rs`).
- **Mandatory Execution Mode**: Daemon Sub-task (Internal IPC handling).
- **Environmental Context**: Deploys via `deploy.sh -a x86_64` on Tizen emulator targets.
