//! Skill manifest — parses skill.md / skill.json manifest files.

use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub runtime: String,        // "python", "node", "native", "cli"
    pub entry_point: String,    // script or binary path
    pub parameters: Value,
    pub risk_level: String,
    pub category: String,
    pub author: String,
    pub permissions: Vec<String>,
}

impl Default for SkillManifest {
    fn default() -> Self {
        SkillManifest {
            name: String::new(),
            description: String::new(),
            version: "1.0.0".into(),
            runtime: "python".into(),
            entry_point: String::new(),
            parameters: json!({"type": "object", "properties": {}, "required": []}),
            risk_level: "low".into(),
            category: "general".into(),
            author: String::new(),
            permissions: vec![],
        }
    }
}

impl SkillManifest {
    /// Parse from YAML frontmatter in a Markdown file.
    pub fn from_md(content: &str) -> Option<Self> {
        let mut manifest = SkillManifest::default();
        let mut in_frontmatter = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if !in_frontmatter { in_frontmatter = true; continue; }
                else { break; }
            }
            if !in_frontmatter { continue; }

            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                match key {
                    "name" => manifest.name = value.to_string(),
                    "description" => manifest.description = value.to_string(),
                    "version" => manifest.version = value.to_string(),
                    "runtime" => manifest.runtime = value.to_string(),
                    "entry_point" => manifest.entry_point = value.to_string(),
                    "risk_level" => manifest.risk_level = value.to_string(),
                    "category" => manifest.category = value.to_string(),
                    "author" => manifest.author = value.to_string(),
                    _ => {}
                }
            }
        }

        if manifest.name.is_empty() { return None; }
        Some(manifest)
    }

    /// Parse from JSON.
    pub fn from_json(content: &str) -> Option<Self> {
        let v: Value = serde_json::from_str(content).ok()?;
        Some(SkillManifest {
            name: v["name"].as_str()?.to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
            version: v["version"].as_str().unwrap_or("1.0.0").to_string(),
            runtime: v["runtime"].as_str().unwrap_or("python").to_string(),
            entry_point: v["entry_point"].as_str().unwrap_or("").to_string(),
            parameters: v.get("parameters").cloned().unwrap_or(json!({"type": "object", "properties": {}})),
            risk_level: v["risk_level"].as_str().unwrap_or("low").to_string(),
            category: v["category"].as_str().unwrap_or("general").to_string(),
            author: v["author"].as_str().unwrap_or("").to_string(),
            permissions: v["permissions"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }
}
