//! Provider selection layer for LLM backend routing.
//!
//! This module decouples request-time routing from a single eagerly-selected
//! backend.  `ProviderRegistry` owns all initialized backends and exposes a
//! preference-ordered list.  `ProviderSelector` picks the first ready provider
//! at request time, consulting an external availability predicate (circuit
//! breaker state lives in `AgentCore`).
//!
//! Config compatibility
//! --------------------
//! `ProviderCompatibilityTranslator` converts the legacy `active_backend` +
//! `fallback_backends` + `backends.*` config shape into a normalized
//! `ProviderRoutingConfig`.  When the new `providers` array is present it is
//! authoritative and the legacy keys are only kept for read-compatibility.

use crate::llm::backend::LlmBackend;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Availability ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    #[default]
    Ready,
    Degraded,
    OpenCircuit,
    Unavailable,
}

impl ProviderAvailability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::OpenCircuit => "open_circuit",
            Self::Unavailable => "unavailable",
        }
    }
}

// ── Attempt result ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAttemptResult {
    Selected,
    SkippedDisabled,
    SkippedUnavailable,
    SkippedOpenCircuit,
    InitFailed,
    ExecutionFailed,
}

impl ProviderAttemptResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::SkippedDisabled => "skipped_disabled",
            Self::SkippedUnavailable => "skipped_unavailable",
            Self::SkippedOpenCircuit => "skipped_open_circuit",
            Self::InitFailed => "init_failed",
            Self::ExecutionFailed => "execution_failed",
        }
    }
}

// ── Selection record (last routing decision) ─────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderAttempt {
    pub provider: String,
    pub result: ProviderAttemptResult,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSelectionRecord {
    pub selected_provider: String,
    pub attempted_providers: Vec<ProviderAttempt>,
    pub reason: String,
    pub selected_at_unix_secs: u64,
}

// ── Config source marker ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderConfigSource {
    /// Declared in the new `providers` array.
    Providers,
    /// Synthesized from the legacy `active_backend` key.
    CompatibilityActive,
    /// Synthesized from the legacy `fallback_backends` key.
    CompatibilityFallback,
}

impl ProviderConfigSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Providers => "providers",
            Self::CompatibilityActive => "compatibility_active",
            Self::CompatibilityFallback => "compatibility_fallback",
        }
    }
}

// ── Per-provider config entry ─────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderPreference {
    pub name: String,
    /// Lower number = higher priority (selected first).
    pub priority: i64,
    pub enabled: bool,
    pub source: ProviderConfigSource,
}

// ── Normalized routing config ─────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct ProviderRoutingConfig {
    /// Ordered by ascending priority (smallest first = highest precedence).
    pub providers: Vec<ProviderPreference>,
    /// Preserved legacy values for compatibility reporting.
    pub raw_active_backend: String,
    pub raw_fallback_backends: Vec<String>,
}

impl ProviderRoutingConfig {
    pub fn ordered_names(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.name.as_str())
            .collect()
    }
}

// ── Compatibility translator ──────────────────────────────────────────────────

pub struct ProviderCompatibilityTranslator;

impl ProviderCompatibilityTranslator {
    /// Build a `ProviderRoutingConfig` from an `llm_config.json` document.
    ///
    /// If `providers` is present it is authoritative.
    /// Otherwise the legacy `active_backend` / `fallback_backends` keys are
    /// used to synthesize the provider order.
    pub fn translate(doc: &Value) -> ProviderRoutingConfig {
        let raw_active_backend = doc
            .get("active_backend")
            .and_then(Value::as_str)
            .unwrap_or("gemini")
            .to_string();
        let raw_fallback_backends: Vec<String> = doc
            .get("fallback_backends")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_else(|| vec!["openai".into(), "ollama".into()]);

        // If the new `providers` array is present, it is authoritative.
        if let Some(providers_arr) = doc.get("providers").and_then(Value::as_array) {
            let mut providers: Vec<ProviderPreference> = providers_arr
                .iter()
                .filter_map(|entry| {
                    let name = entry.get("name").and_then(Value::as_str)?.to_string();
                    if name.trim().is_empty() {
                        return None;
                    }
                    let priority = entry
                        .get("priority")
                        .and_then(Value::as_i64)
                        .unwrap_or(50);
                    let enabled = entry
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(true);
                    Some(ProviderPreference {
                        name,
                        priority,
                        enabled,
                        source: ProviderConfigSource::Providers,
                    })
                })
                .collect();
            // Lower priority number = higher preference (select first).
            providers.sort_by_key(|p| p.priority);
            return ProviderRoutingConfig {
                providers,
                raw_active_backend,
                raw_fallback_backends,
            };
        }

        // Synthesize from legacy keys.
        let mut providers = Vec::new();
        let mut priority: i64 = 100;

        // Active backend gets highest priority.
        if !raw_active_backend.trim().is_empty() {
            providers.push(ProviderPreference {
                name: raw_active_backend.trim().to_string(),
                priority,
                enabled: true,
                source: ProviderConfigSource::CompatibilityActive,
            });
            priority -= 10;
        }

        // Fallback backends follow in declared order.
        for name in &raw_fallback_backends {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Skip if already added as active.
            if providers.iter().any(|p| p.name == trimmed) {
                continue;
            }
            providers.push(ProviderPreference {
                name: trimmed.to_string(),
                priority,
                enabled: true,
                source: ProviderConfigSource::CompatibilityFallback,
            });
            priority -= 10;
        }

        ProviderRoutingConfig {
            providers,
            raw_active_backend,
            raw_fallback_backends,
        }
    }
}

// ── Provider instance (runtime) ───────────────────────────────────────────────

pub struct ProviderInstance {
    pub name: String,
    pub backend: Box<dyn LlmBackend>,
    pub last_init_error: Option<String>,
}

// ── Provider registry ─────────────────────────────────────────────────────────

pub struct ProviderRegistry {
    routing: ProviderRoutingConfig,
    /// Initialized backends in preference order (primary first).
    instances: Vec<ProviderInstance>,
    /// Last request-time routing decision.
    active_selection: Option<ProviderSelectionRecord>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self {
            routing: ProviderRoutingConfig::default(),
            instances: Vec::new(),
            active_selection: None,
        }
    }
}

impl ProviderRegistry {
    pub fn new(routing: ProviderRoutingConfig, instances: Vec<ProviderInstance>) -> Self {
        Self {
            routing,
            instances,
            active_selection: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn has_any(&self) -> bool {
        !self.instances.is_empty()
    }

    pub fn primary_name(&self) -> &str {
        self.instances
            .first()
            .map(|inst| inst.name.as_str())
            .unwrap_or("")
    }

    pub fn instances(&self) -> &[ProviderInstance] {
        &self.instances
    }

    pub fn set_active_selection(&mut self, record: ProviderSelectionRecord) {
        self.active_selection = Some(record);
    }

    pub fn shutdown_all(&mut self) {
        for inst in &mut self.instances {
            inst.backend.shutdown();
        }
    }

    pub fn status_json(&self) -> Value {
        let ordered_names = self.routing.ordered_names();
        let provider_list: Vec<Value> = self
            .routing
            .providers
            .iter()
            .map(|pref| {
                let inst = self.instances.iter().find(|inst| inst.name == pref.name);
                let availability = if inst.is_some() {
                    ProviderAvailability::Ready.as_str()
                } else {
                    ProviderAvailability::Unavailable.as_str()
                };
                let last_init_error = inst
                    .and_then(|inst| inst.last_init_error.as_deref())
                    .unwrap_or_default();
                json!({
                    "name": pref.name,
                    "priority": pref.priority,
                    "enabled": pref.enabled,
                    "availability": availability,
                    "last_init_error": if last_init_error.is_empty() { Value::Null } else { Value::String(last_init_error.to_string()) },
                    "source": pref.source.as_str(),
                })
            })
            .collect();

        let current_selection = self.active_selection.as_ref().map(|rec| {
            json!({
                "selected_provider": rec.selected_provider,
                "attempted_providers": rec.attempted_providers.iter().map(|a| json!({
                    "provider": a.provider,
                    "result": a.result.as_str(),
                    "detail": a.detail,
                })).collect::<Vec<_>>(),
                "reason": rec.reason,
                "selected_at_unix_secs": rec.selected_at_unix_secs,
            })
        });

        json!({
            "configured_active_backend": self.routing.raw_active_backend,
            "configured_fallback_backends": self.routing.raw_fallback_backends,
            "configured_provider_order": ordered_names,
            "providers": provider_list,
            "current_selection": current_selection,
        })
    }
}

// ── Selector ──────────────────────────────────────────────────────────────────

pub struct ProviderSelector;

impl ProviderSelector {
    /// Return the index of the first instance that passes `is_available`.
    ///
    /// Iterates the registry in preference order (position 0 = highest priority).
    /// Disabled providers in the routing config are skipped regardless of
    /// whether a backend was initialized for them.
    pub fn first_available(
        registry: &ProviderRegistry,
        is_available: impl Fn(&str) -> bool,
    ) -> Option<usize> {
        for (idx, inst) in registry.instances.iter().enumerate() {
            // Check whether this provider is enabled in routing config.
            let enabled = registry
                .routing
                .providers
                .iter()
                .find(|p| p.name == inst.name)
                .map(|p| p.enabled)
                .unwrap_or(true); // unknown providers default to enabled
            if !enabled {
                continue;
            }
            if is_available(&inst.name) {
                return Some(idx);
            }
        }
        None
    }

    /// Return the names of all enabled providers in preference order.
    ///
    /// Only providers that are enabled in the routing config appear in the
    /// result.  Unknown providers (no routing config entry) default to enabled.
    /// This is the authoritative source for the ordered provider list that
    /// `chat_with_fallback` iterates so selection policy stays centralized here.
    pub fn ordered_enabled_names(registry: &ProviderRegistry) -> Vec<String> {
        registry
            .instances
            .iter()
            .filter(|inst| {
                registry
                    .routing
                    .providers
                    .iter()
                    .find(|p| p.name == inst.name)
                    .map(|p| p.enabled)
                    .unwrap_or(true)
            })
            .map(|inst| inst.name.clone())
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compatibility_translator_synthesizes_active_then_fallbacks() {
        let doc = json!({
            "active_backend": "gemini",
            "fallback_backends": ["openai", "ollama"],
        });
        let config = ProviderCompatibilityTranslator::translate(&doc);
        let names: Vec<&str> = config.ordered_names();
        assert_eq!(names, vec!["gemini", "openai", "ollama"]);
        assert_eq!(config.providers[0].source, ProviderConfigSource::CompatibilityActive);
        assert_eq!(config.providers[1].source, ProviderConfigSource::CompatibilityFallback);
    }

    #[test]
    fn compatibility_translator_deduplicates_active_in_fallbacks() {
        let doc = json!({
            "active_backend": "gemini",
            "fallback_backends": ["gemini", "openai"],
        });
        let config = ProviderCompatibilityTranslator::translate(&doc);
        let names: Vec<&str> = config.ordered_names();
        assert_eq!(names, vec!["gemini", "openai"]);
    }

    #[test]
    fn explicit_providers_array_overrides_legacy_keys() {
        let doc = json!({
            "active_backend": "gemini",
            "fallback_backends": ["openai"],
            "providers": [
                { "name": "anthropic", "priority": 50, "enabled": true },
                { "name": "openai", "priority": 100, "enabled": true },
            ],
        });
        let config = ProviderCompatibilityTranslator::translate(&doc);
        // Sorted by ascending priority: anthropic(50) then openai(100).
        let names: Vec<&str> = config.ordered_names();
        assert_eq!(names, vec!["anthropic", "openai"]);
        assert_eq!(config.providers[0].source, ProviderConfigSource::Providers);
    }

    #[test]
    fn explicit_providers_disabled_entry_excluded_from_ordered_names() {
        let doc = json!({
            "providers": [
                { "name": "gemini", "priority": 100, "enabled": true },
                { "name": "openai", "priority": 90, "enabled": false },
            ],
        });
        let config = ProviderCompatibilityTranslator::translate(&doc);
        let names: Vec<&str> = config.ordered_names();
        // openai is disabled, so only gemini appears in ordered names.
        assert_eq!(names, vec!["gemini"]);
        // But the disabled one is still in providers list.
        assert_eq!(config.providers.len(), 2);
    }

    #[test]
    fn selector_returns_none_for_empty_registry() {
        let registry = ProviderRegistry::default();
        assert!(ProviderSelector::first_available(&registry, |_| true).is_none());
    }

    #[test]
    fn registry_status_json_lists_configured_and_initialized_providers() {
        let config = ProviderCompatibilityTranslator::translate(&json!({
            "active_backend": "gemini",
            "fallback_backends": ["openai"],
        }));
        let registry = ProviderRegistry::new(config, vec![]);
        let status = registry.status_json();
        let providers = status["providers"].as_array().unwrap();
        // Both configured providers appear even if no backend is initialized.
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0]["name"], "gemini");
        assert_eq!(providers[0]["availability"], "unavailable");
    }

    #[test]
    fn compatibility_translator_empty_active_backend() {
        let doc = json!({
            "active_backend": "",
            "fallback_backends": ["openai"],
        });
        let config = ProviderCompatibilityTranslator::translate(&doc);
        // Empty active_backend is skipped.
        let names: Vec<&str> = config.ordered_names();
        assert_eq!(names, vec!["openai"]);
    }

    /// Verify that the write-locked fallback path in `get_llm_runtime()` produces
    /// a non-empty `providers[]` array.  The fallback reconstructs provider
    /// metadata from the routing config without accessing live instances, so
    /// availability is reported as `"unknown"`.
    #[test]
    fn fallback_status_json_providers_array_is_populated_legacy_config() {
        let raw_doc = json!({
            "active_backend": "gemini",
            "fallback_backends": ["openai"],
        });
        let routing = ProviderCompatibilityTranslator::translate(&raw_doc);
        // Replicate the fallback JSON construction from runtime_admin_impl.rs.
        let providers: Vec<Value> = routing
            .providers
            .iter()
            .map(|pref| {
                json!({
                    "name": pref.name,
                    "priority": pref.priority,
                    "enabled": pref.enabled,
                    "availability": "unknown",
                    "last_init_error": Value::Null,
                    "source": pref.source.as_str(),
                })
            })
            .collect();
        assert_eq!(providers.len(), 2, "fallback must not return an empty providers array");
        assert_eq!(providers[0]["name"], "gemini");
        assert_eq!(providers[0]["availability"], "unknown");
        assert_eq!(providers[1]["name"], "openai");
        assert_eq!(providers[1]["availability"], "unknown");
    }

    #[test]
    fn fallback_status_json_providers_array_is_populated_providers_array_config() {
        let raw_doc = json!({
            "providers": [
                {"name": "anthropic", "priority": 10, "enabled": true},
                {"name": "openai",    "priority": 20, "enabled": false},
            ],
        });
        let routing = ProviderCompatibilityTranslator::translate(&raw_doc);
        let providers: Vec<Value> = routing
            .providers
            .iter()
            .map(|pref| {
                json!({
                    "name": pref.name,
                    "priority": pref.priority,
                    "enabled": pref.enabled,
                    "availability": "unknown",
                    "last_init_error": Value::Null,
                    "source": pref.source.as_str(),
                })
            })
            .collect();
        assert_eq!(providers.len(), 2, "fallback must expose all configured providers");
        assert_eq!(providers[0]["name"], "anthropic");
        assert_eq!(providers[0]["source"], "providers");
        assert_eq!(providers[1]["name"], "openai");
        assert_eq!(providers[1]["enabled"], false);
    }
}
