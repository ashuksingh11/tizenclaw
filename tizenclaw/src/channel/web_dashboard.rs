//! Web dashboard channel — serves web UI (libsoup HTTP server stub).

use super::{Channel, ChannelConfig};

pub struct WebDashboard {
    name: String,
    port: u16,
    running: bool,
}

impl WebDashboard {
    pub fn new(config: &ChannelConfig) -> Self {
        WebDashboard {
            name: config.name.clone(),
            port: config.settings.get("port")
                .and_then(|v| v.as_u64())
                .unwrap_or(8080) as u16,
            running: false,
        }
    }
}

impl Channel for WebDashboard {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool {
        log::info!("WebDashboard: would start on port {} (libsoup FFI required)", self.port);
        self.running = true;
        true
    }
    fn stop(&mut self) { self.running = false; }
    fn send_message(&self, _msg: &str) -> Result<(), String> {
        Ok(()) // WebSocket broadcast would go here
    }
    fn is_running(&self) -> bool { self.running }
}
