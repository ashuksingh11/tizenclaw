//! Voice channel — placeholder for voice-based interaction.

use super::{Channel, ChannelConfig};

pub struct VoiceChannel { name: String, enabled: bool }

impl VoiceChannel {
    pub fn new(config: &ChannelConfig) -> Self {
        VoiceChannel { name: config.name.clone(), enabled: config.enabled }
    }
}

impl Channel for VoiceChannel {
    fn name(&self) -> &str { &self.name }
    fn start(&mut self) -> bool { self.enabled = true; true }
    fn stop(&mut self) { self.enabled = false; }
    fn send_message(&self, _msg: &str) -> Result<(), String> {
        Err("Voice channel: TTS not yet implemented".into())
    }
    fn is_running(&self) -> bool { self.enabled }
}
