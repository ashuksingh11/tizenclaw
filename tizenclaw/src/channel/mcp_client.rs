//! MCP client — Model Context Protocol client for external tool servers.

use serde_json::{json, Value};
use crate::llm::backend::LlmToolDecl;

pub struct McpClient {
    pub name: String,
    pub url: String,
    tools: Vec<LlmToolDecl>,
    connected: bool,
}

impl McpClient {
    pub fn new(name: &str, url: &str) -> Self {
        McpClient { name: name.to_string(), url: url.to_string(), tools: vec![], connected: false }
    }

    pub fn connect(&mut self) -> bool {
        log::info!("MCP: connecting to {} at {}", self.name, self.url);
        // Fetch tool list from MCP server
        match crate::infra::http_client::HttpClient::new().get(&format!("{}/tools", self.url)) {
            Ok(resp) if resp.status_code < 400 => {
                if let Ok(tools) = serde_json::from_str::<Value>(&resp.body) {
                    if let Some(arr) = tools["tools"].as_array() {
                        self.tools = arr.iter().filter_map(|t| {
                            Some(LlmToolDecl {
                                name: format!("mcp_{}_{}", self.name, t["name"].as_str()?),
                                description: t["description"].as_str().unwrap_or("").to_string(),
                                parameters: t.get("inputSchema").cloned().unwrap_or(json!({"type": "object"})),
                            })
                        }).collect();
                    }
                }
                self.connected = true;
                log::info!("MCP: connected to {} ({} tools)", self.name, self.tools.len());
                true
            }
            _ => { log::warn!("MCP: failed to connect to {}", self.name); false }
        }
    }

    pub fn get_tools(&self) -> &[LlmToolDecl] { &self.tools }
    pub fn is_connected(&self) -> bool { self.connected }

    pub fn call_tool(&self, tool_name: &str, args: &Value) -> Value {
        let body = json!({"name": tool_name, "arguments": args}).to_string();
        match crate::infra::http_client::HttpClient::new()
            .post(&format!("{}/call", self.url), &body) {
            Ok(resp) => serde_json::from_str(&resp.body).unwrap_or(json!({"error": "parse error"})),
            Err(e) => json!({"error": e.to_string()}),
        }
    }
}

pub struct McpClientManager {
    clients: Vec<McpClient>,
}

impl McpClientManager {
    pub fn new() -> Self { McpClientManager { clients: vec![] } }

    pub fn load_config_and_connect(&mut self, path: &str) -> bool {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let config: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if let Some(servers) = config["servers"].as_array() {
            for s in servers {
                let name = s["name"].as_str().unwrap_or("").to_string();
                let url = s["url"].as_str().unwrap_or("").to_string();
                if !name.is_empty() && !url.is_empty() {
                    let mut client = McpClient::new(&name, &url);
                    client.connect();
                    self.clients.push(client);
                }
            }
        }
        !self.clients.is_empty()
    }

    pub fn get_all_tools(&self) -> Vec<LlmToolDecl> {
        self.clients.iter().flat_map(|c| c.get_tools().to_vec()).collect()
    }

    pub fn call_tool(&self, full_name: &str, args: &Value) -> Option<Value> {
        for client in &self.clients {
            let prefix = format!("mcp_{}_", client.name);
            if full_name.starts_with(&prefix) {
                let tool_name = full_name.strip_prefix(&prefix)?;
                return Some(client.call_tool(tool_name, args));
            }
        }
        None
    }
}
