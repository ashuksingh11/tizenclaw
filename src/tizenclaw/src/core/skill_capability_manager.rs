use crate::core::registration_store::RegisteredPaths;
use crate::core::skill_support::normalize_skill_name;
use crate::core::textual_skill_scanner::{scan_textual_skills_from_roots, TextualSkill};
use libtizenclaw_core::framework::paths::PlatformPaths;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

const SKILL_CAPABILITIES_CONFIG: &str = "skill_capabilities.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkillCapabilityConfig {
    #[serde(default)]
    pub disabled_skills: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillRoot {
    pub path: String,
    pub kind: String,
    pub external: bool,
}

#[derive(Clone, Debug)]
pub struct SkillCapabilityEntry {
    pub skill: TextualSkill,
    pub source_root: String,
    pub root_kind: String,
    pub enabled: bool,
    pub dependency_ready: bool,
    pub missing_requires: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SkillCapabilitySnapshot {
    pub config_path: String,
    pub disabled_skills: Vec<String>,
    pub roots: Vec<SkillRoot>,
    pub skills: Vec<SkillCapabilityEntry>,
}

impl SkillCapabilitySnapshot {
    pub fn enabled_skills(&self) -> Vec<TextualSkill> {
        self.skills
            .iter()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.skill.clone())
            .collect()
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled_skills.iter().any(|entry| entry == name)
    }

    pub fn find_skill(&self, name: &str) -> Option<&SkillCapabilityEntry> {
        self.skills.iter().find(|entry| entry.skill.file_name == name)
    }

    pub fn summary_json(&self) -> Value {
        let total_count = self.skills.len();
        let enabled_count = self.skills.iter().filter(|entry| entry.enabled).count();
        let disabled_count = total_count.saturating_sub(enabled_count);
        let dependency_blocked_count = self
            .skills
            .iter()
            .filter(|entry| !entry.dependency_ready)
            .count();
        let external_roots = self
            .roots
            .iter()
            .filter(|root| root.external)
            .map(|root| root.path.clone())
            .collect::<Vec<_>>();

        json!({
            "config_path": self.config_path,
            "disabled_skills": self.disabled_skills,
            "total_count": total_count,
            "enabled_count": enabled_count,
            "disabled_count": disabled_count,
            "dependency_blocked_count": dependency_blocked_count,
            "external_root_count": external_roots.len(),
            "external_roots": external_roots,
            "roots": {
                "managed": self.root_paths_for_kind("managed"),
                "hub": self.root_paths_for_kind("hub"),
                "registered": self.root_paths_for_kind("registered"),
            },
            "skills": self.skills.iter().map(|entry| json!({
                "name": entry.skill.file_name,
                "path": entry.skill.absolute_path,
                "description": entry.skill.description,
                "enabled": entry.enabled,
                "dependency_ready": entry.dependency_ready,
                "missing_requires": entry.missing_requires,
                "install_hints": entry.skill.openclaw_install,
                "requires": entry.skill.openclaw_requires,
                "prelude_excerpt": entry.skill.prelude_excerpt,
                "code_fence_languages": entry.skill.code_fence_languages,
                "shell_prelude": entry.skill.shell_prelude,
                "root_kind": entry.root_kind,
                "source_root": entry.source_root,
            })).collect::<Vec<_>>(),
        })
    }

    fn root_paths_for_kind(&self, kind: &str) -> Vec<String> {
        self.roots
            .iter()
            .filter(|root| root.kind == kind)
            .map(|root| root.path.clone())
            .collect()
    }
}

pub fn load_snapshot(paths: &PlatformPaths, registrations: &RegisteredPaths) -> SkillCapabilitySnapshot {
    let config_path = paths.config_dir.join(SKILL_CAPABILITIES_CONFIG);
    let config = load_config(&config_path);
    let disabled = normalized_disabled_skills(&config.disabled_skills);
    let roots = collect_skill_roots(paths, registrations);
    let root_kinds = roots
        .iter()
        .map(|root| (root.path.clone(), root.kind.clone()))
        .collect::<BTreeMap<_, _>>();
    let textual_skills =
        scan_textual_skills_from_roots(roots.iter().map(|root| root.path.as_str()));
    let mut skills = textual_skills
        .into_iter()
        .map(|skill| {
            let source_root = skill_source_root(&skill).unwrap_or_default();
            let root_kind = root_kinds
                .get(&source_root)
                .cloned()
                .unwrap_or_else(|| "managed".to_string());
            let missing_requires = missing_dependencies(&skill.openclaw_requires);
            let dependency_ready = missing_requires.is_empty();
            let enabled = dependency_ready && !disabled.contains(&skill.file_name);
            SkillCapabilityEntry {
                skill,
                source_root,
                root_kind,
                enabled,
                dependency_ready,
                missing_requires,
            }
        })
        .collect::<Vec<_>>();
    skills.sort_by(|left, right| left.skill.file_name.cmp(&right.skill.file_name));

    SkillCapabilitySnapshot {
        config_path: config_path.to_string_lossy().to_string(),
        disabled_skills: disabled.into_iter().collect(),
        roots,
        skills,
    }
}

fn load_config(path: &Path) -> SkillCapabilityConfig {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<SkillCapabilityConfig>(&content).ok())
        .unwrap_or_default()
}

fn normalized_disabled_skills(raw: &[String]) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for value in raw {
        match normalize_skill_name(value) {
            Ok(name) => {
                normalized.insert(name);
            }
            Err(_) => {
                let trimmed = value.trim().to_lowercase();
                if !trimmed.is_empty() {
                    normalized.insert(trimmed);
                }
            }
        }
    }
    normalized.into_iter().collect()
}

fn collect_skill_roots(paths: &PlatformPaths, registrations: &RegisteredPaths) -> Vec<SkillRoot> {
    let mut roots = Vec::new();
    roots.push(SkillRoot {
        path: paths.skills_dir.to_string_lossy().to_string(),
        kind: "managed".to_string(),
        external: false,
    });
    for root in paths.discover_skill_hub_roots() {
        roots.push(SkillRoot {
            path: root.to_string_lossy().to_string(),
            kind: "hub".to_string(),
            external: true,
        });
    }
    for root in &registrations.skill_paths {
        roots.push(SkillRoot {
            path: root.clone(),
            kind: "registered".to_string(),
            external: true,
        });
    }

    let mut seen = HashSet::new();
    roots.retain(|root| seen.insert(root.path.clone()));
    roots.sort_by(|left, right| left.path.cmp(&right.path));
    roots
}

fn skill_source_root(skill: &TextualSkill) -> Option<String> {
    let path = PathBuf::from(&skill.absolute_path);
    let skill_dir = path.parent()?;
    let root = skill_dir.parent()?;
    Some(root.to_string_lossy().to_string())
}

fn missing_dependencies(requires: &[String]) -> Vec<String> {
    let mut missing = Vec::new();
    for requirement in requires {
        let trimmed = requirement.trim();
        if trimmed.is_empty() {
            continue;
        }
        if resolve_command(trimmed).is_none() {
            missing.push(trimmed.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

fn resolve_command(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() {
        return candidate.is_file().then(|| candidate.to_path_buf());
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(name);
        if full.is_file() {
            return Some(full);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{load_snapshot, SkillCapabilityConfig, SKILL_CAPABILITIES_CONFIG};
    use crate::core::registration_store::RegisteredPaths;
    use libtizenclaw_core::framework::paths::PlatformPaths;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn snapshot_marks_disabled_and_dependency_blocked_skills() {
        let temp = tempdir().unwrap();
        let paths = PlatformPaths::from_base(temp.path().join("runtime"));
        paths.ensure_dirs();

        let managed_skill = paths.skills_dir.join("managed-skill");
        fs::create_dir_all(&managed_skill).unwrap();
        fs::write(
            managed_skill.join("SKILL.md"),
            "---\ndescription: Managed\nmetadata:\n  openclaw:\n    requires:\n      - missing-cmd-for-test\n    install:\n      - sudo apt install missing-cmd-for-test\n---\n# Managed\n",
        )
        .unwrap();

        let external_root = temp.path().join("external-skills");
        fs::create_dir_all(external_root.join("external-skill")).unwrap();
        fs::write(
            external_root.join("external-skill").join("SKILL.md"),
            "---\ndescription: External\n---\n# External\n",
        )
        .unwrap();

        let config = SkillCapabilityConfig {
            disabled_skills: vec!["external skill".to_string()],
        };
        fs::write(
            paths.config_dir.join(SKILL_CAPABILITIES_CONFIG),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        let mut registrations = RegisteredPaths::default();
        registrations
            .skill_paths
            .push(external_root.to_string_lossy().to_string());

        let snapshot = load_snapshot(&paths, &registrations);
        assert_eq!(snapshot.disabled_skills, vec!["external-skill"]);
        assert_eq!(snapshot.skills.len(), 2);

        let managed = snapshot.find_skill("managed-skill").unwrap();
        assert!(!managed.dependency_ready);
        assert!(!managed.enabled);
        assert_eq!(managed.root_kind, "managed");

        let external = snapshot.find_skill("external-skill").unwrap();
        assert!(external.dependency_ready);
        assert!(!external.enabled);
        assert_eq!(external.root_kind, "registered");

        let summary = snapshot.summary_json();
        assert_eq!(summary["disabled_count"], 2);
        assert_eq!(summary["external_root_count"], 1);
    }

    #[test]
    fn snapshot_reports_hub_roots_separately() {
        let temp = tempdir().unwrap();
        let paths = PlatformPaths::from_base(temp.path().join("runtime"));
        paths.ensure_dirs();

        let hub_root = paths.skill_hubs_dir.join("openclaw");
        fs::create_dir_all(hub_root.join("battery-helper")).unwrap();
        fs::write(
            hub_root.join("battery-helper").join("SKILL.md"),
            "---\ndescription: Battery helper\n---\n# Skill\n",
        )
        .unwrap();

        let snapshot = load_snapshot(&paths, &RegisteredPaths::default());
        let entry = snapshot.find_skill("battery-helper").unwrap();
        assert_eq!(entry.root_kind, "hub");

        let summary = snapshot.summary_json();
        assert_eq!(
            summary["roots"]["hub"][0],
            json!(hub_root.to_string_lossy().to_string())
        );
    }
}
