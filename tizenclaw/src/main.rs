//! TizenClaw Rust Daemon — full daemon entry point.
//!
//! Initializes logging, AgentCore, IPC server, and runs
//! the main loop until SIGTERM/SIGINT is received.

pub mod common;
pub mod infra;
pub mod storage;
pub mod llm;
pub mod core;
pub mod channel;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn signal_handler(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn main() {
    // Initialize logging (dlog backend)
    common::logging::init();
    log::info!("═══════════════════════════════════════");
    log::info!("  TizenClaw Rust Daemon v1.0.0");
    log::info!("═══════════════════════════════════════");

    // Set up signal handlers
    unsafe {
        libc::signal(libc::SIGINT, signal_handler as libc::sighandler_t);
        libc::signal(libc::SIGTERM, signal_handler as libc::sighandler_t);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

    // Initialize AgentCore
    log::info!("[Boot] Initializing AgentCore...");
    let mut agent = core::agent_core::AgentCore::new();
    if !agent.initialize() {
        log::error!("AgentCore initialization failed");
    }
    let agent = Arc::new(Mutex::new(agent));

    // Start IPC server
    log::info!("[Boot] Starting IPC server...");
    let ipc = core::ipc_server::IpcServer::new();
    let ipc_handle = ipc.start(agent.clone());

    log::info!("[Boot] TizenClaw daemon ready.");

    // Main loop — sleep until signal received
    while RUNNING.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Shutdown
    log::info!("TizenClaw daemon shutting down...");
    ipc.stop();
    let _ = ipc_handle.join();

    if let Ok(mut a) = agent.lock() {
        a.shutdown();
    }

    log::info!("TizenClaw daemon stopped.");
}
