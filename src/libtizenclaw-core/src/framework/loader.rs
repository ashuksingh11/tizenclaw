//! Compatibility helpers around plugin discovery.

use crate::framework::paths::PlatformPaths;
use crate::plugin_core::{self, PlatformPlugin};

/// List all discovered plugin metadata from the resolved plugins directory.
pub fn list_available_plugins(paths: &PlatformPaths) -> Vec<PlatformPlugin> {
    plugin_core::load_plugins(&paths.plugins_dir)
}
