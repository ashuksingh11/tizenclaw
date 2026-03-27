//! Telegram client — sends/receives messages via Telegram Bot API.

use super::{Channel, ChannelConfig};
use serde_json::{json, Value};

pub struct TelegramClient {
    name: String,
    bot_token: String,
    chat_id: String,
    enabled: bool,
}

impl TelegramClient {
    pub fn new(config: &ChannelConfig) -> Self {
        TelegramClient {
            name: config.name.clone(),
            bot_token: config.settings.get("bot_token")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            chat_id: config.settings.get("chat_id")
                .and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: config.enabled,
        }
    }
}

impl Channel for TelegramClient {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, msg: &str) -> Result<(), String> {
        if self.bot_token.is_empty() || self.chat_id.is_empty() {
            return Err("Telegram not configured".into());
        }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let body = json!({"chat_id": &self.chat_id, "text": msg}).to_string();
        crate::infra::http_client::HttpClient::new()
            .post(&url, &body)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn is_running(&self) -> bool { self.enabled }
}
