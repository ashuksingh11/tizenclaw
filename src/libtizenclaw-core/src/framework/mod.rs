//! claw-platform: Platform abstraction layer for TizenClaw.
//!
//! Provides trait-based interfaces for platform-specific functionality
//! plus metadata-driven platform plugin discovery.
//!
//! Architecture:
//! - `PlatformPlugin`: Core trait every platform plugin must implement
//! - `GenericLinuxPlatform`: Built-in fallback for standard Linux/Ubuntu
//! - `PlatformContext`: Singleton holding the active platform + discovered plugins

pub mod generic_linux;
pub mod loader;
pub mod paths;

use serde_json::Value;
use std::sync::Arc;

// ─────────────────────────────────────────
// Core Traits
// ─────────────────────────────────────────

/// Log severity levels (platform-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Core platform plugin trait.
///
/// Every platform plugin (Tizen, Ubuntu, etc.) implements this trait.
/// The daemon loads plugins at runtime via `dlopen` and calls these methods.
pub trait PlatformPlugin: Send + Sync {
    /// Human-readable platform name (e.g., "Tizen", "Ubuntu", "Generic Linux").
    fn platform_name(&self) -> &str;

    /// Unique plugin identifier (e.g., "tizen", "ubuntu-desktop").
    fn plugin_id(&self) -> &str;

    /// Plugin version string.
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Priority for platform detection (higher = preferred).
    /// When multiple plugins claim to be compatible, the highest priority wins.
    fn priority(&self) -> u32 {
        0
    }

    /// Check if this plugin is compatible with the current environment.
    /// Called during plugin loading to determine which plugin to activate.
    fn is_compatible(&self) -> bool {
        true
    }

    /// Initialize the plugin. Called once after loading.
    fn initialize(&mut self) -> bool {
        true
    }

    /// Shutdown the plugin. Called once before unloading.
    fn shutdown(&mut self) {}
}

/// Platform-specific logging backend.
pub trait PlatformLogger: Send + Sync {
    /// Write a log message.
    fn log(&self, level: LogLevel, tag: &str, msg: &str);
}

/// Platform-specific system information provider.
pub trait SystemInfoProvider: Send + Sync {
    /// Get OS/platform version string.
    fn get_os_version(&self) -> Option<String>;

    /// Get full device profile as JSON.
    fn get_device_profile(&self) -> Value;

    /// Get battery level (0-100), if available.
    fn get_battery_level(&self) -> Option<u32> {
        None
    }

    /// Check if network is available.
    fn is_network_available(&self) -> bool {
        std::net::TcpStream::connect("8.8.8.8:53")
            .map(|_| true)
            .unwrap_or(false)
    }
}

/// Platform-specific package manager interface.
pub trait PackageManagerProvider: Send + Sync {
    /// List installed packages.
    fn list_packages(&self) -> Vec<PackageInfo>;

    /// Get info about a specific package.
    fn get_package_info(&self, pkg_id: &str) -> Option<PackageInfo>;

    /// Check if a package is installed.
    fn is_installed(&self, pkg_id: &str) -> bool {
        self.get_package_info(pkg_id).is_some()
    }

    /// Retrieve packages containing a specific metadata key.
    /// Default implementation returns an empty vector for generic environments.
    fn get_packages_by_metadata_key(&self, _key: &str) -> Vec<PackageInfo> {
        vec![]
    }

    /// Get a specific metadata value associated with a package.
    fn get_package_metadata_value(&self, _pkg_id: &str, _key: &str) -> Option<String> {
        None
    }

    /// Get the installation root path of a package.
    fn get_package_root_path(&self, _pkg_id: &str) -> Option<String> {
        None
    }

    /// Get the installation resource path of a package.
    fn get_package_res_path(&self, _pkg_id: &str) -> Option<String> {
        None
    }
}

/// Platform-specific application control.
pub trait AppControlProvider: Send + Sync {
    /// Launch an application by ID.
    fn launch_app(&self, app_id: &str) -> Result<(), String>;

    /// List running applications.
    fn list_running_apps(&self) -> Vec<String> {
        vec![]
    }
}

/// Platform-specific system event monitoring.
pub trait SystemEventProvider: Send + Sync {
    /// Start monitoring system events.
    fn start(&mut self) -> bool {
        true
    }

    /// Stop monitoring.
    fn stop(&mut self) {}
}

// ─────────────────────────────────────────
// Data Types
// ─────────────────────────────────────────

/// Basic info about an installed package.
#[derive(Debug, Clone, Default)]
pub struct PackageInfo {
    pub pkg_id: String,
    pub app_id: String,
    pub label: String,
    pub version: String,
    pub pkg_type: String,
    pub installed: bool,
}

// ─────────────────────────────────────────
// Platform Context (Singleton)
// ─────────────────────────────────────────

/// Holds the active platform configuration and all loaded plugin capabilities.
///
/// Created once at daemon boot via `PlatformContext::detect()`.
pub struct PlatformContext {
    /// Active platform plugin.
    pub platform: Box<dyn PlatformPlugin>,
    /// Platform logger (from active plugin or generic stderr).
    pub logger: std::sync::Arc<dyn PlatformLogger>,
    /// System info provider.
    pub system_info: Box<dyn SystemInfoProvider>,
    /// Package manager (optional — may be no-op).
    pub package_manager: Box<dyn PackageManagerProvider>,
    /// App controller (optional — may be no-op).
    pub app_control: Box<dyn AppControlProvider>,
    /// Platform-resolved paths.
    pub paths: paths::PlatformPaths,
    /// Discovered platform plugins loaded from the plugins directory.
    pub plugins: Vec<crate::plugin_core::PlatformPlugin>,
    /// True when Tizen filesystem markers are present.
    pub is_tizen: bool,
    /// Current CPU architecture string.
    pub arch: String,
}

impl PlatformContext {
    /// Detect and load the appropriate platform.
    ///
    /// 1. Resolve `PlatformPaths`
    /// 2. Discover metadata plugins from `paths.plugins_dir`
    /// 3. Return an `Arc<PlatformContext>` that remains valid with zero plugins
    pub fn detect() -> Arc<Self> {
        let platform_paths = paths::PlatformPaths::resolve();
        let plugins = crate::plugin_core::load_plugins(&platform_paths.plugins_dir);
        let is_tizen = platform_paths.is_tizen();
        let arch = generic_linux::get_arch();
        let platform: Box<dyn PlatformPlugin> = plugins
            .first()
            .map(|plugin| {
                Box::new(DiscoveredPlatform::new(plugin.info.clone())) as Box<dyn PlatformPlugin>
            })
            .unwrap_or_else(|| Box::new(generic_linux::GenericLinuxPlatform::new()));

        Arc::new(PlatformContext {
            logger: Arc::new(generic_linux::StderrLogger),
            system_info: Box::new(generic_linux::LinuxSystemInfo),
            package_manager: Box::new(generic_linux::GenericPackageManager),
            app_control: Box::new(generic_linux::GenericAppControl),
            platform,
            paths: platform_paths,
            plugins,
            is_tizen,
            arch,
        })
    }

    /// Get the platform name.
    pub fn platform_name(&self) -> &str {
        self.platform.platform_name()
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.plugins
            .iter()
            .any(|plugin| plugin.has_capability(capability))
    }

    pub fn os_info_string(&self) -> String {
        if self.is_tizen {
            match detect_tizen_version() {
                Some(version) => format!("Tizen {} {}", version, self.arch),
                None => format!("Tizen {}", self.arch),
            }
        } else {
            match generic_linux::get_os_pretty_name().or_else(|| {
                let name = generic_linux::get_os_name();
                (name != "Linux").then_some(name)
            }) {
                Some(name) => format!("Linux {} ({})", self.arch, name),
                None => format!("Linux {}", self.arch),
            }
        }
    }
}

fn detect_tizen_version() -> Option<String> {
    let content = std::fs::read_to_string("/etc/tizen-release").ok()?;

    content.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
        if cleaned.chars().any(|c| c.is_ascii_digit()) {
            Some(cleaned.to_string())
        } else {
            None
        }
    })
}

struct DiscoveredPlatform {
    info: crate::plugin_core::PluginInfo,
}

impl DiscoveredPlatform {
    fn new(info: crate::plugin_core::PluginInfo) -> Self {
        Self { info }
    }
}

impl PlatformPlugin for DiscoveredPlatform {
    fn platform_name(&self) -> &str {
        &self.info.platform_name
    }

    fn plugin_id(&self) -> &str {
        &self.info.plugin_id
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn priority(&self) -> u32 {
        self.info.priority.max(0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libloading::Library;
    use std::path::PathBuf;

    fn test_library() -> Library {
        unsafe {
            Library::new("libc.so.6")
                .or_else(|_| Library::new("libc.so"))
                .expect("host test library should load")
        }
    }

    #[test]
    fn has_capability_matches_loaded_plugin_metadata() {
        let plugin = crate::plugin_core::PlatformPlugin::from_test_parts(
            crate::plugin_core::PluginInfo {
                plugin_id: "tizen".to_string(),
                platform_name: "Tizen".to_string(),
                version: "1.0.0".to_string(),
                priority: 100,
                capabilities: vec!["logging".to_string(), "package_manager".to_string()],
            },
            PathBuf::from("/tmp/libtizenclaw_plugin.so"),
            test_library(),
        );

        let context = PlatformContext {
            platform: Box::new(generic_linux::GenericLinuxPlatform::new()),
            logger: Arc::new(generic_linux::StderrLogger),
            system_info: Box::new(generic_linux::LinuxSystemInfo),
            package_manager: Box::new(generic_linux::GenericPackageManager),
            app_control: Box::new(generic_linux::GenericAppControl),
            paths: paths::PlatformPaths::from_base(std::env::temp_dir().join("tizenclaw-test")),
            plugins: vec![plugin],
            is_tizen: false,
            arch: "x86_64".to_string(),
        };

        assert!(context.has_capability("logging"));
        assert!(!context.has_capability("system_events"));
    }

    #[test]
    fn os_info_string_is_non_empty_on_host_context() {
        let context = PlatformContext::detect();

        assert!(!context.os_info_string().trim().is_empty());
    }
}
