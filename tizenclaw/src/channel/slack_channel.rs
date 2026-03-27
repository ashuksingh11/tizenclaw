//! Slack channel — sends messages via Slack webhook.

use super::{Channel, ChannelConfig};
use serde_json::json;

pub struct SlackChannel {
    name: String,
    webhook_url: String,
    enabled: bool,
}

impl SlackChannel {
    pub fn new(config: &ChannelConfig) -> Self {
        SlackChannel {
            name: config.name.clone(),
            webhook_url: config.settings.get("webhook_url")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: config.enabled,
        }
    }
}

impl Channel for SlackChannel {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, msg: &str) -> Result<(), String> {
        if self.webhook_url.is_empty() { return Err("Slack webhook not configured".into()); }
        let body = json!({"text": msg}).to_string();
        crate::infra::http_client::HttpClient::new()
            .post(&self.webhook_url, &body)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn is_running(&self) -> bool { self.enabled }
}
