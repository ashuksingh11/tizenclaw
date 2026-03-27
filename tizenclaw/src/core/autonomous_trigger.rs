//! Autonomous trigger — triggers agent actions based on system events.

use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TriggerRule {
    pub id: String,
    pub event_type: String,
    pub condition: String,
    pub action_prompt: String,
    pub session_id: String,
    pub enabled: bool,
}

pub struct AutonomousTrigger {
    rules: HashMap<String, TriggerRule>,
}

impl AutonomousTrigger {
    pub fn new() -> Self {
        AutonomousTrigger { rules: HashMap::new() }
    }

    pub fn load_config(&mut self, path: &str) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let config: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Some(rules) = config["triggers"].as_array() {
            for r in rules {
                let id = r["id"].as_str().unwrap_or("").to_string();
                if id.is_empty() { continue; }
                self.rules.insert(id.clone(), TriggerRule {
                    id,
                    event_type: r["event_type"].as_str().unwrap_or("").to_string(),
                    condition: r["condition"].as_str().unwrap_or("").to_string(),
                    action_prompt: r["action_prompt"].as_str().unwrap_or("").to_string(),
                    session_id: r["session_id"].as_str().unwrap_or("autonomous").to_string(),
                    enabled: r["enabled"].as_bool().unwrap_or(true),
                });
            }
        }
        log::info!("AutonomousTrigger: loaded {} rules", self.rules.len());
    }

    pub fn check_event(&self, event_type: &str, data: &Value) -> Vec<&TriggerRule> {
        self.rules.values()
            .filter(|r| r.enabled && r.event_type == event_type)
            .collect()
    }

    pub fn add_rule(&mut self, rule: TriggerRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    pub fn remove_rule(&mut self, id: &str) -> bool {
        self.rules.remove(id).is_some()
    }

    pub fn list_rules(&self) -> Vec<Value> {
        self.rules.values().map(|r| json!({
            "id": r.id, "event_type": r.event_type,
            "condition": r.condition, "enabled": r.enabled
        })).collect()
    }
}
