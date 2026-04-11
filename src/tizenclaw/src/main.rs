//! TizenClaw Rust Daemon — full daemon entry point.
//!
//! Initializes platform detection, logging, AgentCore, IPC server,
//! and runs the main loop until SIGTERM/SIGINT is received.
//!
//! Build modes:
//!   cargo build          → Generic Linux (Ubuntu) — no Tizen libs needed
//!   deploy.sh (GBS)      → Tizen — libtizenclaw_plugin.so provides dlog, etc.

// Suppress unused warnings during migration.
// TODO: Remove once all modules are wired into the daemon.
#![allow(unused)]

pub mod channel;
pub mod common;
pub mod core;
pub mod generic;
pub mod infra;
pub mod llm;
pub mod network;
pub mod storage;
pub mod tizen;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Default)]
struct DaemonOptions {
    devel_mode: bool,
}

extern "C" fn signal_handler(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn print_usage() {
    eprintln!("Usage: tizenclaw [--devel]");
}

fn parse_daemon_options() -> DaemonOptions {
    let mut options = DaemonOptions::default();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--devel" => options.devel_mode = true,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_usage();
                std::process::exit(2);
            }
        }
    }

    options
}

fn setup_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGTERM, signal_handler as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, signal_handler as *const () as libc::sighandler_t);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

fn fix_tizen_tls() {
    const TIZEN_CA_BUNDLE: &str = "/etc/ssl/ca-bundle.pem";

    if std::path::Path::new(TIZEN_CA_BUNDLE).exists() {
        std::env::set_var("SSL_CERT_FILE", TIZEN_CA_BUNDLE);
        log::info!("Set SSL_CERT_FILE to {}", TIZEN_CA_BUNDLE);
    }
}

#[tokio::main]
async fn main() {
    let options = parse_daemon_options();

    const TOTAL_BOOT_PHASES: u8 = 7;

    // Phase 1: Detect platform & initialize paths
    let platform = libtizenclaw_core::framework::PlatformContext::detect();
    platform.paths.ensure_dirs();
    let boot_log_path = platform.paths.logs_dir.join("tizenclaw.log");
    let boot_logger = common::boot_status_logger::BootStatusLogger::new(boot_log_path);
    boot_logger.write_phase(1, TOTAL_BOOT_PHASES, "Detected platform and initialized paths");

    // Phase 2: Initialize logging
    common::logging::FileLogBackend::init(&platform.paths.logs_dir, 10 * 1024 * 1024);
    common::logging::init_with_logger();
    boot_logger.write_phase(2, TOTAL_BOOT_PHASES, "Initialized logging backend");

    // Phase 3: Fix Tizen TLS
    fix_tizen_tls();
    boot_logger.write_phase(3, TOTAL_BOOT_PHASES, "Applied TLS environment fix");

    setup_signal_handlers();
    infra::http_client::default_client();

    log::info!("TizenClaw daemon starting");
    log::info!("Platform: {}", platform.platform_name());
    log::info!("Data dir: {}", platform.paths.data_dir.display());

    // Phase 4: Initialize AgentCore
    let agent = Arc::new(core::agent_core::AgentCore::new(platform.clone()));
    if !agent.initialize().await {
        boot_logger.write_phase(4, TOTAL_BOOT_PHASES, "AgentCore initialization failed");
        log::error!("AgentCore initialization failed");
        std::process::exit(1);
    }
    boot_logger.write_phase(4, TOTAL_BOOT_PHASES, "Initialized AgentCore");

    // Register pkgmgr listener for runtime plugin injection
    use libtizenclaw_core::plugin_core::pkgmgr_client::{
        PkgmgrClient, PkgmgrEventArgs, PkgmgrListener,
    };
    struct AgentPkgmgrListener(Arc<core::agent_core::AgentCore>);
    impl PkgmgrListener for AgentPkgmgrListener {
        fn on_pkgmgr_event(&self, args: Arc<PkgmgrEventArgs>) {
            if args.event_status == "end" {
                let agent_clone = self.0.clone();
                let event_name = args.event_name.clone();
                let pkgid = args.pkgid.clone();
                tokio::spawn(async move {
                    agent_clone.handle_pkgmgr_event(&event_name, &pkgid).await;
                });
            }
        }
    }
    PkgmgrClient::global().add_listener(Arc::new(AgentPkgmgrListener(agent.clone())));

    log::info!("Starting task scheduler");
    let task_scheduler = core::task_scheduler::TaskScheduler::with_agent(agent.clone());
    let task_dir = platform.paths.data_dir.join("tasks");
    let _ = std::fs::create_dir_all(&task_dir);
    let seeded_tasks = task_scheduler.seed_default_tasks_if_empty(&task_dir.to_string_lossy());
    task_scheduler.load_config(&task_dir.to_string_lossy());
    let scheduler_started = task_scheduler.start().is_some();
    log::info!(
        "Task scheduler started: {} (seeded {} default task(s))",
        scheduler_started,
        seeded_tasks
    );

    log::info!("Initializing channels");
    let channel_registry = Arc::new(Mutex::new(channel::ChannelRegistry::new()));

    // Load from channel_config.json
    let channel_config_path = platform.paths.config_dir.join("channel_config.json");
    {
        let mut reg = channel_registry.lock().unwrap();
        reg.load_config(&channel_config_path.to_string_lossy(), Some(agent.clone()));

        // Ensure web_dashboard is always registered for manual CLI startup.
        // If not present in config, keep it disabled by default.
        if !reg.has_channel("web_dashboard") {
            let web_root = platform.paths.web_root.to_string_lossy().to_string();
            let dashboard_config = channel::ChannelConfig {
                name: "web_dashboard".into(),
                channel_type: "web_dashboard".into(),
                enabled: false,
                settings: serde_json::json!({
                    "port": core::runtime_paths::default_dashboard_port(),
                    "localhost_only": false,
                    "web_root": web_root
                }),
            };
            if let Some(ch) =
                channel::channel_factory::create_channel(&dashboard_config, Some(agent.clone()))
            {
                reg.register(ch, false);
                log::info!(
                    "Web dashboard registered (port {}, auto_start=false)",
                    core::runtime_paths::default_dashboard_port()
                );
            }
        }

        reg.start_all();
    }
    log::info!("Channel registry initialized");

    // Phase 5: Start IPC server
    let ipc_server = core::ipc_server::IpcServer::new();
    let ipc_socket_path = std::env::var("TIZENCLAW_SOCKET_PATH").unwrap_or_default();
    let ipc_handle = ipc_server.start(&ipc_socket_path, agent.clone(), channel_registry.clone());
    boot_logger.write_phase(5, TOTAL_BOOT_PHASES, "Started IPC server");

    log::info!("Starting mDNS network scanner");
    let mdns_scanner = network::mdns_discovery::MdnsScanner::new();
    mdns_scanner.start();

    // Phase 6: Run startup indexing
    agent.run_startup_indexing().await;
    boot_logger.write_phase(6, TOTAL_BOOT_PHASES, "Completed startup indexing");

    // Phase 7: Run devel mode (optional)
    if options.devel_mode {
        boot_logger.write_phase(7, TOTAL_BOOT_PHASES, "Running devel mode sequence");
        core::devel_mode::run(&agent).await;
        log::info!("Shutting down...");
        channel_registry.lock().unwrap().stop_all();
        task_scheduler.stop();
        mdns_scanner.stop();
        agent.shutdown().await;
        ipc_server.stop();
        let _ = ipc_handle.join();
        return;
    }

    boot_logger.write_phase(7, TOTAL_BOOT_PHASES, "Daemon ready");
    log::info!("TizenClaw daemon ready");

    // Main loop
    while RUNNING.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }

    log::info!("Shutting down...");
    boot_logger.write("Shutting down...");
    channel_registry.lock().unwrap().stop_all();
    task_scheduler.stop();
    mdns_scanner.stop();
    agent.shutdown().await;
    ipc_server.stop();
    let _ = ipc_handle.join();
    log::info!("TizenClaw daemon stopped.");
}
