//! A2A handler — Agent-to-Agent protocol handler.

use super::{Channel, ChannelConfig};
use serde_json::{json, Value};

pub struct A2aHandler { name: String, enabled: bool }

impl A2aHandler {
    pub fn new(config: &ChannelConfig) -> Self {
        A2aHandler { name: config.name.clone(), enabled: config.enabled }
    }
}

impl Channel for A2aHandler {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, msg: &str) -> Result<(), String> {
        log::info!("A2A: sending message to peer agent");
        Ok(())
    }
    fn is_running(&self) -> bool { self.enabled }
}
