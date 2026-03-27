//! Webhook channel — HTTP webhook endpoint for receiving/sending messages.

use super::{Channel, ChannelConfig};
use serde_json::{json, Value};

pub struct WebhookChannel {
    name: String,
    webhook_url: String,
    secret: String,
    enabled: bool,
}

impl WebhookChannel {
    pub fn new(config: &ChannelConfig) -> Self {
        WebhookChannel {
            name: config.name.clone(),
            webhook_url: config.settings.get("webhook_url")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            secret: config.settings.get("secret")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: config.enabled,
        }
    }

    pub fn send_webhook(&self, payload: &Value) -> Result<(), String> {
        if self.webhook_url.is_empty() {
            return Err("No webhook URL configured".into());
        }
        let body = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        let resp = crate::infra::http_client::HttpClient::new()
            .post(&self.webhook_url, &body)
            .map_err(|e| e.to_string())?;
        if resp.status_code >= 400 {
            return Err(format!("Webhook returned {}", resp.status_code));
        }
        Ok(())
    }
}

impl Channel for WebhookChannel {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, msg: &str) -> Result<(), String> {
        self.send_webhook(&json!({"text": msg}))
    }
    fn is_running(&self) -> bool { self.enabled }
}
