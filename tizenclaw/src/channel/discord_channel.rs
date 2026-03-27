//! Discord channel — sends messages via Discord webhook.

use super::{Channel, ChannelConfig};
use serde_json::json;

pub struct DiscordChannel {
    name: String,
    webhook_url: String,
    enabled: bool,
}

impl DiscordChannel {
    pub fn new(config: &ChannelConfig) -> Self {
        DiscordChannel {
            name: config.name.clone(),
            webhook_url: config.settings.get("webhook_url")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: config.enabled,
        }
    }
}

impl Channel for DiscordChannel {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, msg: &str) -> Result<(), String> {
        if self.webhook_url.is_empty() { return Err("Discord webhook not configured".into()); }
        let body = json!({"content": msg}).to_string();
        crate::infra::http_client::HttpClient::new()
            .post(&self.webhook_url, &body)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn is_running(&self) -> bool { self.enabled }
}
