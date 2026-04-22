//! Runtime platform plugin discovery.
//!
//! Platform plugins are shared libraries that export:
//! - `claw_plugin_info() -> *const c_char`
//! - `claw_plugin_free_string(*const c_char)`

pub mod adapters;
pub mod logging;
pub mod pkgmgr_client;

use libloading::Library;
use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

type PluginInfoFn = unsafe extern "C" fn() -> *const c_char;
type PluginFreeStringFn = unsafe extern "C" fn(*const c_char);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PluginInfo {
    pub plugin_id: String,
    pub platform_name: String,
    pub version: String,
    pub priority: i32,
    pub capabilities: Vec<String>,
}

#[derive(Debug)]
pub struct PlatformPlugin {
    pub info: PluginInfo,
    pub library_path: PathBuf,
    _library: Library,
}

impl PlatformPlugin {
    pub fn has_capability(&self, capability: &str) -> bool {
        self.info
            .capabilities
            .iter()
            .any(|entry| entry == capability)
    }

    #[cfg(test)]
    pub(crate) fn from_test_parts(
        info: PluginInfo,
        library_path: PathBuf,
        library: Library,
    ) -> Self {
        Self {
            info,
            library_path,
            _library: library,
        }
    }
}

pub fn load_plugins(plugins_dir: &Path) -> Vec<PlatformPlugin> {
    let entries = match std::fs::read_dir(plugins_dir) {
        Ok(entries) => entries,
        Err(err) => {
            log::debug!(
                "Plugin directory {:?} unavailable, continuing without plugins: {}",
                plugins_dir,
                err
            );
            return Vec::new();
        }
    };

    let mut plugins = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_plugin_library(&path) {
            continue;
        }

        match unsafe { load_plugin(&path) } {
            Ok(plugin) => plugins.push(plugin),
            Err(err) => {
                log::warn!("Skipping invalid plugin {:?}: {}", path, err);
            }
        }
    }

    plugins.sort_by(|left, right| {
        right
            .info
            .priority
            .cmp(&left.info.priority)
            .then_with(|| left.library_path.cmp(&right.library_path))
    });

    plugins
}

fn is_plugin_library(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("lib") && name.ends_with(".so"))
            .unwrap_or(false)
}

/// # Safety
///
/// `path` must point to a shared library that follows the plugin ABI:
/// exported symbol names must match this loader and returned strings must stay
/// valid until `claw_plugin_free_string` is called.
unsafe fn load_plugin(path: &Path) -> Result<PlatformPlugin, String> {
    let library = Library::new(path).map_err(|err| format!("dlopen failed: {err}"))?;

    let info_fn = library
        .get::<PluginInfoFn>(b"claw_plugin_info\0")
        .map_err(|err| format!("missing claw_plugin_info: {err}"))?;
    let free_fn = library
        .get::<PluginFreeStringFn>(b"claw_plugin_free_string\0")
        .map_err(|err| format!("missing claw_plugin_free_string: {err}"))?;

    let raw_info = info_fn();
    if raw_info.is_null() {
        return Err("claw_plugin_info returned null".to_string());
    }

    let info_json = CStr::from_ptr(raw_info)
        .to_str()
        .map_err(|err| format!("plugin info was not valid UTF-8: {err}"))?
        .to_owned();

    free_fn(raw_info);

    let info: PluginInfo =
        serde_json::from_str(&info_json).map_err(|err| format!("invalid plugin JSON: {err}"))?;

    Ok(PlatformPlugin {
        info,
        library_path: path.to_path_buf(),
        _library: library,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_plugins_skips_missing_directory() {
        let missing = std::env::temp_dir().join(format!(
            "tizenclaw-missing-plugins-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        assert!(load_plugins(&missing).is_empty());
    }
}
