//! mDNS Discovery — scanning and registering zero-config endpoints.

use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub last_seen: u64,
}

pub struct MdnsScanner {
    daemon: Arc<Mutex<Option<Arc<ServiceDaemon>>>>,
    pub peers: Arc<RwLock<HashMap<String, Peer>>>,
    running: Arc<AtomicBool>,
}

impl Default for MdnsScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MdnsScanner {
    pub fn new() -> Self {
        Self {
            daemon: Arc::new(Mutex::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) {
        let peers_clone = self.peers.clone();
        let daemon_slot = self.daemon.clone();
        let running = self.running.clone();

        tokio::task::spawn_blocking(move || {
            let mdns = match ServiceDaemon::new() {
                Ok(m) => m,
                Err(e) => {
                    log::error!(
                        "mDNS initialization failed: {}. Fallback to isolated mode.",
                        e
                    );
                    return;
                }
            };
            let mdns = Arc::new(mdns);
            running.store(true, Ordering::SeqCst);
            if let Ok(mut slot) = daemon_slot.lock() {
                *slot = Some(mdns.clone());
            }

            let service_type = "_tizenclaw._tcp.local.";
            let receiver = match mdns.browse(service_type) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("mDNS browse failed: {}. Fallback to isolated mode.", e);
                    running.store(false, Ordering::SeqCst);
                    if let Ok(mut slot) = daemon_slot.lock() {
                        *slot = None;
                    }
                    return;
                }
            };

            log::info!("mDNS Scanner started, searching for {}", service_type);

            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let id = info.get_fullname().to_string();
                        let addresses: Vec<String> = info
                            .get_addresses()
                            .iter()
                            .map(|ip| ip.to_string())
                            .collect();
                        let addr = addresses.first().cloned().unwrap_or_else(|| "".to_string());
                        let port = info.get_port();

                        log::debug!("Discovered TizenClaw peer: {} at {}:{}", id, addr, port);

                        if let Ok(mut peers) = peers_clone.write() {
                            peers.insert(
                                id.clone(),
                                Peer {
                                    id,
                                    address: addr,
                                    port,
                                    last_seen: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                },
                            );
                        }
                    }
                    ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                        log::debug!("TizenClaw peer removed: {}", fullname);
                        if let Ok(mut peers) = peers_clone.write() {
                            peers.remove(&fullname);
                        }
                    }
                    _ => {}
                }
            }

            running.store(false, Ordering::SeqCst);
            if let Ok(mut slot) = daemon_slot.lock() {
                *slot = None;
            }
            log::debug!("mDNS Scanner stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut slot) = self.daemon.lock() {
            if let Some(daemon) = slot.take() {
                let _ = daemon.shutdown();
            }
        }
    }
}
