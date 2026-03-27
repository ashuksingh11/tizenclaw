//! Agent Core — the brain of TizenClaw.
//!
//! Manages LLM interaction, tool calling, session management,
//! and the agentic loop (prompt → LLM → tool call → result → LLM → ...).

use serde_json::{json, Value};

use crate::infra::key_store::KeyStore;
use crate::llm::backend::{self, LlmBackend, LlmMessage};
use crate::storage::session_store::SessionStore;
use crate::core::tool_dispatcher::ToolDispatcher;

const APP_DATA_DIR: &str = "/opt/usr/share/tizenclaw";
const MAX_TOOL_ROUNDS: usize = 10;
const MAX_CONTEXT_MESSAGES: usize = 20;

pub struct AgentCore {
    backend: Option<Box<dyn LlmBackend>>,
    session_store: Option<SessionStore>,
    tool_dispatcher: ToolDispatcher,
    key_store: KeyStore,
    system_prompt: String,
    backend_name: String,
}

impl AgentCore {
    pub fn new() -> Self {
        AgentCore {
            backend: None,
            session_store: None,
            tool_dispatcher: ToolDispatcher::new(),
            key_store: KeyStore::new(),
            system_prompt: String::new(),
            backend_name: String::new(),
        }
    }

    pub fn initialize(&mut self) -> bool {
        log::info!("AgentCore initializing...");

        // Load API keys
        let key_path = format!("{}/config/keys.json", APP_DATA_DIR);
        self.key_store.load(&key_path);

        // Load system prompt
        let prompt_path = format!("{}/config/system_prompt.txt", APP_DATA_DIR);
        self.system_prompt = std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| "You are TizenClaw, an AI assistant for Tizen devices. You can execute tools to help users interact with the device.".into());

        // Load agent config
        let config_path = format!("{}/config/agent_config.json", APP_DATA_DIR);
        let config: Value = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({"backend": "gemini"}));

        self.backend_name = config["backend"].as_str().unwrap_or("gemini").to_string();

        // Initialize LLM backend
        let mut backend_config = json!({});
        if let Some(key) = self.key_store.get(&format!("{}_API_KEY", self.backend_name.to_uppercase())) {
            backend_config["api_key"] = json!(key);
        }
        if let Some(model) = config["model"].as_str() {
            backend_config["model"] = json!(model);
        }
        if let Some(endpoint) = config["endpoint"].as_str() {
            backend_config["endpoint"] = json!(endpoint);
        }

        if let Some(mut be) = backend::create_backend(&self.backend_name) {
            if be.initialize(&backend_config) {
                log::info!("LLM backend '{}' initialized", self.backend_name);
                self.backend = Some(be);
            } else {
                log::error!("LLM backend '{}' failed to initialize", self.backend_name);
            }
        } else {
            log::error!("Unknown LLM backend: {}", self.backend_name);
        }

        // Initialize session store
        let db_path = format!("{}/sessions.db", APP_DATA_DIR);
        match SessionStore::new(&db_path) {
            Ok(store) => {
                log::info!("Session store initialized");
                self.session_store = Some(store);
            }
            Err(e) => log::error!("Session store failed: {}", e),
        }

        // Load tools from tools directories
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/cli");
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/embedded");
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/system_cli");
        log::info!("Tools loaded");

        true
    }

    /// Process a user prompt through the agentic loop.
    pub fn process_prompt(
        &self,
        session_id: &str,
        prompt: &str,
        on_chunk: Option<&dyn Fn(&str)>,
    ) -> String {
        log::info!("Processing prompt for session '{}' ({} chars)", session_id, prompt.len());

        let backend = match &self.backend {
            Some(b) => b,
            None => return "Error: No LLM backend configured".into(),
        };

        // Store user message
        if let Some(store) = &self.session_store {
            store.add_message(session_id, "user", prompt);
        }

        // Build conversation history
        let history = self.session_store.as_ref()
            .map(|s| s.get_messages(session_id, MAX_CONTEXT_MESSAGES))
            .unwrap_or_default();

        let mut messages: Vec<LlmMessage> = history.iter().map(|m| {
            LlmMessage {
                role: m.role.clone(),
                text: m.text.clone(),
                ..Default::default()
            }
        }).collect();

        // If history is empty or doesn't end with user message, add it
        if messages.is_empty() || messages.last().map(|m| m.role.as_str()) != Some("user") {
            messages.push(LlmMessage::user(prompt));
        }

        let tools = self.tool_dispatcher.get_tool_declarations();

        // Agentic loop
        for round in 0..MAX_TOOL_ROUNDS {
            let response = backend.chat(&messages, &tools, on_chunk, &self.system_prompt);

            if !response.success {
                let err = format!("LLM error (HTTP {}): {}", response.http_status, response.error_message);
                log::error!("{}", err);
                return err;
            }

            // Record token usage
            if let Some(store) = &self.session_store {
                store.record_usage(
                    session_id,
                    response.prompt_tokens,
                    response.completion_tokens,
                    backend.get_name(),
                );
            }

            if response.has_tool_calls() {
                log::info!("Round {}: {} tool call(s)", round, response.tool_calls.len());

                // Add assistant message with tool calls
                messages.push(LlmMessage {
                    role: "assistant".into(),
                    text: response.text.clone(),
                    tool_calls: response.tool_calls.clone(),
                    ..Default::default()
                });

                // Execute each tool call and add results
                for tc in &response.tool_calls {
                    log::info!("Executing tool: {} (id: {})", tc.name, tc.id);
                    let result = self.tool_dispatcher.execute(&tc.name, &tc.args);
                    messages.push(LlmMessage::tool_result(&tc.id, &tc.name, result));
                }
                // Continue loop for next LLM response
            } else {
                // Final text response
                let text = response.text.clone();
                if let Some(store) = &self.session_store {
                    store.add_message(session_id, "assistant", &text);
                }
                return text;
            }
        }

        "Error: Maximum tool call rounds exceeded".into()
    }

    pub fn shutdown(&mut self) {
        log::info!("AgentCore shutting down");
        if let Some(be) = &mut self.backend {
            be.shutdown();
        }
    }

    pub fn get_session_store(&self) -> &Option<SessionStore> {
        &self.session_store
    }

    pub fn reload_tools(&mut self) {
        self.tool_dispatcher = ToolDispatcher::new();
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/cli");
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/embedded");
        self.tool_dispatcher.load_tools_from_dir("/opt/usr/share/tizen-tools/system_cli");
        log::info!("Tools reloaded");
    }
}

