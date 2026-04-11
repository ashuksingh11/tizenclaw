//! Compatibility helpers around plugin discovery.

use crate::framework::paths::PlatformPaths;
use crate::plugin_core::{self, PlatformPlugin};
use serde_json::{Map, Value};
use std::io::Write;
use std::path::Path;

/// List all discovered plugin metadata from the resolved plugins directory.
pub fn list_available_plugins(paths: &PlatformPaths) -> Vec<PlatformPlugin> {
    plugin_core::load_plugins(&paths.plugins_dir)
}

/// Load and parse a JSON config file from the given path.
/// Returns an empty JSON object if the file does not exist or cannot be parsed.
pub fn load_json_config(path: &Path) -> Value {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Value::Object(Map::new());
        }
        Err(err) => {
            log::warn!("Failed to read JSON config {:?}: {}", path, err);
            return Value::Object(Map::new());
        }
    };

    serde_json::from_str(&content).unwrap_or_else(|err| {
        log::warn!("Failed to parse JSON config {:?}: {}", path, err);
        Value::Object(Map::new())
    })
}

/// Save a JSON value to a file atomically (temp -> rename).
pub fn save_json_config(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create config dir {:?}: {}", parent, err))?;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid config file name: {:?}", path))?;
    let temp_path = path.with_file_name(format!(".{file_name}.tmp"));
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|err| format!("Failed to serialize JSON for {:?}: {}", path, err))?;

    let mut file = std::fs::File::create(&temp_path)
        .map_err(|err| format!("Failed to create temp config {:?}: {}", temp_path, err))?;
    file.write_all(&payload)
        .map_err(|err| format!("Failed to write temp config {:?}: {}", temp_path, err))?;
    file.write_all(b"\n")
        .map_err(|err| format!("Failed to finalize temp config {:?}: {}", temp_path, err))?;
    file.sync_all()
        .map_err(|err| format!("Failed to sync temp config {:?}: {}", temp_path, err))?;
    drop(file);

    std::fs::rename(&temp_path, path).map_err(|err| {
        let _ = std::fs::remove_file(&temp_path);
        format!(
            "Failed to atomically replace config {:?} with {:?}: {}",
            path, temp_path, err
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_json_config, save_json_config};
    use serde_json::json;

    #[test]
    fn load_json_config_returns_empty_object_for_missing_file() {
        let path = std::env::temp_dir().join(format!(
            "tizenclaw-missing-config-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        assert_eq!(load_json_config(&path), json!({}));
    }

    #[test]
    fn save_json_config_writes_valid_json_atomically() {
        let dir = std::env::temp_dir().join(format!(
            "tizenclaw-config-dir-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("config.json");
        let value = json!({"enabled": true, "count": 2});

        save_json_config(&path, &value).unwrap();

        assert_eq!(load_json_config(&path), value);

        let _ = std::fs::remove_dir_all(dir);
    }
}
