//! Platform-resolved paths.
//!
//! Canonical mutable state lives under the runtime root:
//! - host: `~/.tizenclaw`
//! - Tizen: `/home/owner/.tizenclaw`
//!
//! Packaged read-only assets may live under `/opt/usr/share/tizenclaw` on
//! Tizen. Callers should consume this module instead of inferring paths from
//! `/opt/usr/share/tizenclaw` directly.

use std::path::{Path, PathBuf};

/// All resolved platform paths.
#[derive(Debug, Clone)]
pub struct PlatformPaths {
    /// Canonical mutable runtime root.
    pub runtime_root: PathBuf,
    /// Backward-compatible alias for the runtime root.
    pub data_dir: PathBuf,
    /// Optional packaged read-only asset root.
    pub packaged_root: Option<PathBuf>,
    /// Configuration files directory.
    pub config_dir: PathBuf,
    /// Tool scripts directory.
    pub tools_dir: PathBuf,
    /// Textual skills directory.
    pub skills_dir: PathBuf,
    /// Skill hub mount directory containing external OpenClaw-style roots.
    pub skill_hubs_dir: PathBuf,
    /// TizenClaw-owned embedded tool descriptor directory.
    pub embedded_tools_dir: PathBuf,
    /// Plugin shared objects directory.
    pub plugins_dir: PathBuf,
    /// Packaged or runtime reference docs directory.
    pub docs_dir: PathBuf,
    /// Web dashboard static files.
    pub web_root: PathBuf,
    /// Workflows directory.
    pub workflows_dir: PathBuf,
    /// Generated and reusable code directory.
    pub codes_dir: PathBuf,
    /// Log directory.
    pub logs_dir: PathBuf,
    /// Actions directory.
    pub actions_dir: PathBuf,
    /// Pipelines directory.
    pub pipelines_dir: PathBuf,
    /// LLM backend plugins directory.
    pub llm_plugins_dir: PathBuf,
    /// CLI plugins metadata directory.
    pub cli_plugins_dir: PathBuf,
}

const TIZEN_RUNTIME_DIR: &str = "/home/owner/.tizenclaw";
const TIZEN_PACKAGED_DIR: &str = "/opt/usr/share/tizenclaw";
const HOST_DATA_DIR_NAME: &str = ".tizenclaw";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeEnvironment {
    Host,
    Tizen,
}

impl PlatformPaths {
    /// Resolve platform paths from environment and runtime markers.
    pub fn resolve() -> Self {
        let env = detect_runtime_environment();
        let runtime_root = resolve_runtime_root(env);
        let packaged_root = resolve_packaged_root(env);
        let packaged_assets_root = packaged_root.clone().unwrap_or_else(|| runtime_root.clone());
        let plugins_default = packaged_root
            .as_ref()
            .map(|root| root.join("plugins"))
            .unwrap_or_else(|| runtime_root.join("plugins"));

        Self {
            runtime_root: runtime_root.clone(),
            data_dir: runtime_root.clone(),
            packaged_root,
            config_dir: resolve_path("TIZENCLAW_CONFIG_DIR", runtime_root.join("config")),
            tools_dir: resolve_path("TIZENCLAW_TOOLS_DIR", runtime_root.join("tools")),
            skills_dir: resolve_path("TIZENCLAW_SKILLS_DIR", runtime_root.join("workspace/skills")),
            skill_hubs_dir: resolve_path(
                "TIZENCLAW_SKILL_HUBS_DIR",
                runtime_root.join("workspace/skill-hubs"),
            ),
            embedded_tools_dir: resolve_path(
                "TIZENCLAW_EMBEDDED_TOOLS_DIR",
                packaged_assets_root.join("embedded"),
            ),
            plugins_dir: resolve_path("TIZENCLAW_PLUGINS_DIR", plugins_default),
            llm_plugins_dir: resolve_path(
                "TIZENCLAW_LLM_PLUGINS_DIR",
                runtime_root.join("plugins/llm"),
            ),
            cli_plugins_dir: resolve_path(
                "TIZENCLAW_CLI_PLUGINS_DIR",
                runtime_root.join("plugins/cli"),
            ),
            docs_dir: resolve_path("TIZENCLAW_DOCS_DIR", packaged_assets_root.join("docs")),
            web_root: resolve_path("TIZENCLAW_WEB_ROOT", packaged_assets_root.join("web")),
            workflows_dir: resolve_path("TIZENCLAW_WORKFLOWS_DIR", runtime_root.join("workflows")),
            codes_dir: resolve_path("TIZENCLAW_CODES_DIR", runtime_root.join("codes")),
            logs_dir: resolve_path("TIZENCLAW_LOGS_DIR", runtime_root.join("logs")),
            actions_dir: resolve_path("TIZENCLAW_ACTIONS_DIR", runtime_root.join("actions")),
            pipelines_dir: resolve_path("TIZENCLAW_PIPELINES_DIR", runtime_root.join("pipelines")),
        }
    }

    /// Backward-compatible alias for older callers.
    pub fn detect() -> Self {
        Self::resolve()
    }

    /// Build paths from a custom runtime root.
    pub fn from_base(base: PathBuf) -> Self {
        PlatformPaths {
            runtime_root: base.clone(),
            data_dir: base.clone(),
            packaged_root: None,
            config_dir: base.join("config"),
            tools_dir: base.join("tools"),
            skills_dir: base.join("workspace/skills"),
            skill_hubs_dir: base.join("workspace/skill-hubs"),
            embedded_tools_dir: base.join("embedded"),
            plugins_dir: base.join("plugins"),
            llm_plugins_dir: base.join("plugins/llm"),
            cli_plugins_dir: base.join("plugins/cli"),
            docs_dir: base.join("docs"),
            web_root: base.join("web"),
            workflows_dir: base.join("workflows"),
            codes_dir: base.join("codes"),
            logs_dir: base.join("logs"),
            actions_dir: base.join("actions"),
            pipelines_dir: base.join("pipelines"),
        }
    }

    /// Ensure writable runtime directories exist.
    pub fn ensure_dirs(&self) {
        for dir in self.runtime_dirs_to_create() {
            if let Err(err) = std::fs::create_dir_all(&dir) {
                log::warn!("Failed to create dir {:?}: {}", dir, err);
            }
        }

        for dir in [&self.embedded_tools_dir, &self.docs_dir, &self.web_root, &self.plugins_dir] {
            if self.is_writable_runtime_path(dir) {
                if let Err(err) = std::fs::create_dir_all(dir) {
                    log::warn!("Failed to create dir {:?}: {}", dir, err);
                }
            }
        }
    }

    pub fn is_tizen(&self) -> bool {
        detect_runtime_environment() == RuntimeEnvironment::Tizen
    }

    /// Get the session database path.
    pub fn sessions_db_path(&self) -> PathBuf {
        self.runtime_root.join("sessions/sessions.db")
    }

    /// Get the app data directory for file-based storage.
    pub fn app_data_dir(&self) -> PathBuf {
        self.runtime_root.clone()
    }

    /// Discover external skill roots mounted under `workspace/skill-hubs`.
    pub fn discover_skill_hub_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        for base in self.skill_hub_root_dirs() {
            let Ok(entries) = std::fs::read_dir(base) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    roots.push(path);
                }
            }
        }

        roots.sort();
        roots.dedup();
        roots
    }

    /// Compatibility skill roots for read-time discovery.
    pub fn skill_root_dirs(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.skills_dir.clone()];
        if let Some(legacy) = self.legacy_runtime_root() {
            roots.push(legacy.join("workspace/skills"));
        }
        roots.sort();
        roots.dedup();
        roots
    }

    /// Compatibility skill-hub roots for read-time discovery.
    pub fn skill_hub_root_dirs(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.skill_hubs_dir.clone()];
        if let Some(legacy) = self.legacy_runtime_root() {
            roots.push(legacy.join("workspace/skill-hubs"));
        }
        roots.sort();
        roots.dedup();
        roots
    }

    pub fn packaged_dir(&self) -> Option<&Path> {
        self.packaged_root.as_deref()
    }

    fn legacy_runtime_root(&self) -> Option<PathBuf> {
        let packaged = self.packaged_root.as_ref()?;
        (packaged != &self.runtime_root).then(|| packaged.clone())
    }

    fn is_writable_runtime_path(&self, path: &Path) -> bool {
        match self.packaged_root.as_deref() {
            Some(packaged_root) => !path.starts_with(packaged_root),
            None => true,
        }
    }

    fn runtime_dirs_to_create(&self) -> Vec<PathBuf> {
        let state_dir = self.runtime_root.join("state");
        let state_registry_dir = state_dir.join("registry");
        let state_loop_dir = state_dir.join("loop");
        let sessions_dir = self.runtime_root.join("sessions");
        let memory_dir = self.runtime_root.join("memory");
        let outbound_dir = self.runtime_root.join("outbound");
        let telegram_sessions_dir = self.runtime_root.join("telegram_sessions");

        vec![
            self.runtime_root.clone(),
            self.config_dir.clone(),
            self.tools_dir.clone(),
            self.skills_dir.clone(),
            self.skill_hubs_dir.clone(),
            self.llm_plugins_dir.clone(),
            self.cli_plugins_dir.clone(),
            self.workflows_dir.clone(),
            self.codes_dir.clone(),
            self.logs_dir.clone(),
            self.actions_dir.clone(),
            self.pipelines_dir.clone(),
            state_dir,
            state_registry_dir,
            state_loop_dir,
            sessions_dir,
            memory_dir,
            outbound_dir,
            telegram_sessions_dir,
        ]
    }
}

fn resolve_path(env_key: &str, default: PathBuf) -> PathBuf {
    std::env::var_os(env_key)
        .map(PathBuf::from)
        .unwrap_or(default)
}

fn resolve_runtime_root(env: RuntimeEnvironment) -> PathBuf {
    std::env::var_os("TIZENCLAW_HOME")
        .or_else(|| std::env::var_os("TIZENCLAW_DATA_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| match env {
            RuntimeEnvironment::Tizen => PathBuf::from(TIZEN_RUNTIME_DIR),
            RuntimeEnvironment::Host => dirs_or_home().join(HOST_DATA_DIR_NAME),
        })
}

fn resolve_packaged_root(env: RuntimeEnvironment) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("TIZENCLAW_PACKAGED_DIR") {
        return Some(PathBuf::from(path));
    }

    match env {
        RuntimeEnvironment::Tizen => Some(PathBuf::from(TIZEN_PACKAGED_DIR)),
        RuntimeEnvironment::Host => None,
    }
}

fn detect_runtime_environment() -> RuntimeEnvironment {
    match std::env::var("TIZENCLAW_RUNTIME_ENV") {
        Ok(value) if value.eq_ignore_ascii_case("tizen") => RuntimeEnvironment::Tizen,
        Ok(value) if value.eq_ignore_ascii_case("host") => RuntimeEnvironment::Host,
        _ => {
            if Path::new("/etc/tizen-release").exists() {
                RuntimeEnvironment::Tizen
            } else {
                RuntimeEnvironment::Host
            }
        }
    }
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn from_base_places_openclaw_style_paths_under_runtime_root() {
        let base = PathBuf::from("/tmp/tizenclaw-paths");
        let paths = PlatformPaths::from_base(base.clone());

        assert_eq!(paths.runtime_root, base);
        assert_eq!(paths.tools_dir, PathBuf::from("/tmp/tizenclaw-paths/tools"));
        assert_eq!(
            paths.skills_dir,
            PathBuf::from("/tmp/tizenclaw-paths/workspace/skills")
        );
        assert_eq!(
            paths.skill_hubs_dir,
            PathBuf::from("/tmp/tizenclaw-paths/workspace/skill-hubs")
        );
        assert_eq!(
            paths.embedded_tools_dir,
            PathBuf::from("/tmp/tizenclaw-paths/embedded")
        );
        assert_eq!(paths.codes_dir, PathBuf::from("/tmp/tizenclaw-paths/codes"));
    }

    #[test]
    fn ensure_dirs_creates_writable_runtime_directories() {
        let unique = format!(
            "tizenclaw-paths-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let base = std::env::temp_dir().join(unique);
        let paths = PlatformPaths::from_base(base.clone());

        paths.ensure_dirs();

        assert!(paths.skill_hubs_dir.exists());
        assert!(paths.embedded_tools_dir.exists());
        assert!(paths.codes_dir.exists());
        assert!(base.join("state").exists());
        assert!(base.join("state/registry").exists());
        assert!(base.join("state/loop").exists());
        assert!(base.join("sessions").exists());
        assert!(base.join("memory").exists());
        assert!(base.join("outbound").exists());
        assert!(base.join("telegram_sessions").exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn resolve_prefers_runtime_overrides() {
        let _guard = env_lock().lock().unwrap();
        let original_runtime_dir = std::env::var_os("TIZENCLAW_DATA_DIR");
        let original_home_dir = std::env::var_os("TIZENCLAW_HOME");
        let original_tools_dir = std::env::var_os("TIZENCLAW_TOOLS_DIR");
        let base = std::env::temp_dir().join(format!(
            "tizenclaw-env-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tools = base.join("custom-tools");

        unsafe {
            std::env::remove_var("TIZENCLAW_HOME");
            std::env::set_var("TIZENCLAW_DATA_DIR", &base);
            std::env::set_var("TIZENCLAW_TOOLS_DIR", &tools);
        }

        let paths = PlatformPaths::resolve();

        assert_eq!(paths.runtime_root, base);
        assert_eq!(paths.data_dir, paths.runtime_root);
        assert_eq!(paths.tools_dir, tools);
        assert_eq!(paths.llm_plugins_dir, base.join("plugins/llm"));

        unsafe {
            restore_env_var("TIZENCLAW_DATA_DIR", original_runtime_dir);
            restore_env_var("TIZENCLAW_HOME", original_home_dir);
            restore_env_var("TIZENCLAW_TOOLS_DIR", original_tools_dir);
        }
    }

    #[test]
    fn resolve_uses_home_based_host_paths_without_overrides() {
        let _guard = env_lock().lock().unwrap();
        let original_home = std::env::var_os("HOME");
        let original_runtime_dir = std::env::var_os("TIZENCLAW_DATA_DIR");
        let original_runtime_env = std::env::var_os("TIZENCLAW_RUNTIME_ENV");
        let home = std::env::temp_dir().join(format!(
            "tizenclaw-home-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        unsafe {
            std::env::set_var("TIZENCLAW_RUNTIME_ENV", "host");
            std::env::remove_var("TIZENCLAW_DATA_DIR");
            std::env::set_var("HOME", &home);
        }

        let paths = PlatformPaths::resolve();

        assert_eq!(paths.runtime_root, home.join(".tizenclaw"));
        assert_eq!(paths.config_dir, home.join(".tizenclaw/config"));
        assert!(paths.packaged_root.is_none());

        unsafe {
            restore_env_var("HOME", original_home);
            restore_env_var("TIZENCLAW_DATA_DIR", original_runtime_dir);
            restore_env_var("TIZENCLAW_RUNTIME_ENV", original_runtime_env);
        }
    }

    #[test]
    fn resolve_uses_owner_home_and_packaged_assets_on_tizen() {
        let _guard = env_lock().lock().unwrap();
        let original_runtime_dir = std::env::var_os("TIZENCLAW_DATA_DIR");
        let original_runtime_env = std::env::var_os("TIZENCLAW_RUNTIME_ENV");

        unsafe {
            std::env::set_var("TIZENCLAW_RUNTIME_ENV", "tizen");
            std::env::remove_var("TIZENCLAW_DATA_DIR");
        }

        let paths = PlatformPaths::resolve();

        assert_eq!(paths.runtime_root, PathBuf::from("/home/owner/.tizenclaw"));
        assert_eq!(paths.skills_dir, PathBuf::from("/home/owner/.tizenclaw/workspace/skills"));
        assert_eq!(paths.docs_dir, PathBuf::from("/opt/usr/share/tizenclaw/docs"));
        assert_eq!(paths.web_root, PathBuf::from("/opt/usr/share/tizenclaw/web"));
        assert_eq!(
            paths.packaged_root,
            Some(PathBuf::from("/opt/usr/share/tizenclaw"))
        );

        unsafe {
            restore_env_var("TIZENCLAW_DATA_DIR", original_runtime_dir);
            restore_env_var("TIZENCLAW_RUNTIME_ENV", original_runtime_env);
        }
    }

    #[test]
    fn skill_root_dirs_include_legacy_packaged_root_for_compatibility() {
        let _guard = env_lock().lock().unwrap();
        let original_runtime_env = std::env::var_os("TIZENCLAW_RUNTIME_ENV");

        unsafe {
            std::env::set_var("TIZENCLAW_RUNTIME_ENV", "tizen");
        }

        let paths = PlatformPaths::resolve();
        let roots = paths.skill_root_dirs();

        assert_eq!(
            roots,
            vec![
                PathBuf::from("/home/owner/.tizenclaw/workspace/skills"),
                PathBuf::from("/opt/usr/share/tizenclaw/workspace/skills"),
            ]
        );

        unsafe {
            restore_env_var("TIZENCLAW_RUNTIME_ENV", original_runtime_env);
        }
    }

    #[test]
    fn discover_skill_hub_roots_lists_child_directories() {
        let unique = format!(
            "tizenclaw-skill-hubs-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let base = std::env::temp_dir().join(unique);
        let paths = PlatformPaths::from_base(base.clone());
        std::fs::create_dir_all(&paths.skill_hubs_dir).unwrap();
        std::fs::create_dir_all(paths.skill_hubs_dir.join("openclaw")).unwrap();
        std::fs::write(paths.skill_hubs_dir.join("README.md"), "ignore").unwrap();

        let roots = paths.discover_skill_hub_roots();

        assert_eq!(roots, vec![paths.skill_hubs_dir.join("openclaw")]);

        let _ = std::fs::remove_dir_all(base);
    }

    unsafe fn restore_env_var(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
