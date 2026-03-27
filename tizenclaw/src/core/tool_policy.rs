//! Tool policy — controls tool execution limits, loop detection, risk levels.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum RiskLevel { Low, Normal, High }

impl RiskLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "low" => RiskLevel::Low,
            "high" => RiskLevel::High,
            _ => RiskLevel::Normal,
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Normal => "normal",
            RiskLevel::High => "high",
        }
    }
}

struct PolicyConfig {
    max_repeat_count: usize,
    max_iterations: usize,
    blocked_skills: HashSet<String>,
    risk_levels: HashMap<String, RiskLevel>,
    aliases: HashMap<String, String>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        PolicyConfig {
            max_repeat_count: 3,
            max_iterations: 15,
            blocked_skills: HashSet::new(),
            risk_levels: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
}

pub struct ToolPolicy {
    config: PolicyConfig,
    call_history: Mutex<HashMap<String, HashMap<String, usize>>>,
    idle_history: Mutex<HashMap<String, Vec<String>>>,
}

const IDLE_WINDOW_SIZE: usize = 3;

impl ToolPolicy {
    pub fn new() -> Self {
        ToolPolicy {
            config: PolicyConfig::default(),
            call_history: Mutex::new(HashMap::new()),
            idle_history: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_config(&mut self, path: &str) -> bool {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => { log::info!("No tool policy config at {}, using defaults", path); return true; }
        };
        let j: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => { log::error!("Failed to parse tool policy: {}", e); return false; }
        };

        if let Some(v) = j.get("max_repeat_count").and_then(|v| v.as_u64()) {
            self.config.max_repeat_count = v as usize;
        }
        if let Some(v) = j.get("max_iterations").and_then(|v| v.as_u64()) {
            self.config.max_iterations = v as usize;
        }
        if let Some(arr) = j.get("blocked_skills").and_then(|v| v.as_array()) {
            for s in arr {
                if let Some(name) = s.as_str() {
                    self.config.blocked_skills.insert(name.to_string());
                }
            }
        }
        if let Some(obj) = j.get("risk_overrides").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    self.config.risk_levels.insert(k.clone(), RiskLevel::from_str(s));
                }
            }
        }
        if let Some(obj) = j.get("aliases").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    self.config.aliases.insert(k.clone(), s.to_string());
                }
            }
        }
        log::info!("Tool policy loaded: max_repeat={}, blocked={}, aliases={}",
            self.config.max_repeat_count, self.config.blocked_skills.len(), self.config.aliases.len());
        true
    }

    pub fn check_policy(&self, session_id: &str, skill_name: &str, args: &Value) -> Result<(), String> {
        if self.config.blocked_skills.contains(skill_name) {
            return Err(format!("Tool '{}' is blocked by security policy.", skill_name));
        }

        let hash = Self::hash_call(skill_name, args);
        if let Ok(mut history) = self.call_history.lock() {
            let session = history.entry(session_id.to_string()).or_default();
            let count = session.entry(hash).or_insert(0);
            *count += 1;
            if *count > self.config.max_repeat_count {
                return Err(format!(
                    "Tool '{}' with identical arguments called {} times (limit: {}). Blocked to prevent infinite loop.",
                    skill_name, count, self.config.max_repeat_count
                ));
            }
        }
        Ok(())
    }

    pub fn check_idle_progress(&self, session_id: &str, output: &str) -> bool {
        if let Ok(mut history) = self.idle_history.lock() {
            let entries = history.entry(session_id.to_string()).or_default();
            entries.push(output.to_string());
            while entries.len() > IDLE_WINDOW_SIZE { entries.remove(0); }
            if entries.len() < IDLE_WINDOW_SIZE { return false; }
            let first = &entries[0];
            entries.iter().all(|e| e == first)
        } else {
            false
        }
    }

    pub fn reset_session(&self, session_id: &str) {
        if let Ok(mut h) = self.call_history.lock() { h.remove(session_id); }
        if let Ok(mut h) = self.idle_history.lock() { h.remove(session_id); }
    }

    pub fn reset_idle_tracking(&self, session_id: &str) {
        if let Ok(mut h) = self.idle_history.lock() { h.remove(session_id); }
    }

    pub fn get_max_iterations(&self) -> usize { self.config.max_iterations }
    pub fn get_aliases(&self) -> &HashMap<String, String> { &self.config.aliases }
    pub fn get_risk_level(&self, name: &str) -> RiskLevel {
        self.config.risk_levels.get(name).cloned().unwrap_or(RiskLevel::Normal)
    }

    fn hash_call(name: &str, args: &Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let input = format!("{}:{}", name, args);
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
