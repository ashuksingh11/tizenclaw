//! Safety guard — controls which tools are allowed based on safety policy.

use serde_json::{json, Value};
use std::collections::HashSet;

/// Side effect classification for tools.
#[derive(Clone, Debug, PartialEq)]
pub enum SideEffect {
    None,
    Reversible,
    Irreversible,
}

impl SideEffect {
    pub fn from_str(s: &str) -> Self {
        match s {
            "none" => SideEffect::None,
            "irreversible" => SideEffect::Irreversible,
            _ => SideEffect::Reversible,
        }
    }
}

/// Safety guard configuration.
pub struct SafetyGuard {
    blocked_tools: HashSet<String>,
    blocked_args: HashSet<String>,
    allow_irreversible: bool,
    max_tool_calls_per_session: usize,
}

impl SafetyGuard {
    pub fn new() -> Self {
        let mut blocked_args = HashSet::new();
        blocked_args.insert("rm -rf /".to_string());
        blocked_args.insert("mkfs".to_string());
        blocked_args.insert("dd if=".to_string());
        blocked_args.insert("shutdown".to_string());
        blocked_args.insert("reboot".to_string());

        SafetyGuard {
            blocked_tools: HashSet::new(),
            blocked_args,
            allow_irreversible: false,
            max_tool_calls_per_session: 50,
        }
    }

    /// Load safety policy from a JSON config file.
    pub fn load_config(&mut self, path: &str) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let config: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Some(blocked) = config["blocked_tools"].as_array() {
            for t in blocked {
                if let Some(s) = t.as_str() {
                    self.blocked_tools.insert(s.to_string());
                }
            }
        }
        if let Some(blocked) = config["blocked_args"].as_array() {
            for a in blocked {
                if let Some(s) = a.as_str() {
                    self.blocked_args.insert(s.to_string());
                }
            }
        }
        if let Some(allow) = config["allow_irreversible"].as_bool() {
            self.allow_irreversible = allow;
        }
        if let Some(max) = config["max_tool_calls_per_session"].as_u64() {
            self.max_tool_calls_per_session = max as usize;
        }
    }

    /// Check if a tool call is allowed.
    pub fn check_tool(&self, tool_name: &str, args: &str, side_effect: &SideEffect) -> Result<(), String> {
        if self.blocked_tools.contains(tool_name) {
            return Err(format!("Tool '{}' is blocked by safety policy", tool_name));
        }

        if *side_effect == SideEffect::Irreversible && !self.allow_irreversible {
            return Err(format!("Tool '{}' has irreversible side effects and is blocked", tool_name));
        }

        for blocked in &self.blocked_args {
            if args.contains(blocked.as_str()) {
                return Err(format!("Blocked argument pattern '{}' detected", blocked));
            }
        }

        Ok(())
    }

    /// Check if prompt contains injection attempts.
    pub fn check_prompt_injection(&self, prompt: &str) -> bool {
        let lower = prompt.to_lowercase();
        let patterns = [
            "ignore previous instructions",
            "disregard all previous",
            "you are now",
            "forget everything",
            "override your",
            "system prompt:",
        ];
        for p in &patterns {
            if lower.contains(p) {
                log::warn!("Potential prompt injection detected: '{}'", p);
                return true;
            }
        }
        false
    }
}
