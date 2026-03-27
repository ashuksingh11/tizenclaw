//! Skill repository — scans directories for skill packages and loads manifests.

use super::skill_manifest::SkillManifest;
use std::collections::HashMap;

pub struct SkillRepository {
    skills: HashMap<String, (SkillManifest, std::path::PathBuf)>,
    skill_dirs: Vec<String>,
}

impl SkillRepository {
    pub fn new() -> Self {
        SkillRepository {
            skills: HashMap::new(),
            skill_dirs: vec![
                "/opt/usr/share/tizenclaw/skills".into(),
                "/opt/usr/share/tizenclaw/custom_skills".into(),
            ],
        }
    }

    pub fn add_skill_dir(&mut self, dir: &str) {
        self.skill_dirs.push(dir.to_string());
    }

    pub fn scan_all(&mut self) {
        self.skills.clear();
        for dir in self.skill_dirs.clone() {
            self.scan_dir(&dir);
        }
        log::info!("SkillRepository: scanned {} skills from {} directories",
            self.skills.len(), self.skill_dirs.len());
    }

    fn scan_dir(&mut self, dir: &str) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            // Try skill.md first, then skill.json
            let manifest = if let Ok(content) = std::fs::read_to_string(path.join("skill.md")) {
                SkillManifest::from_md(&content)
            } else if let Ok(content) = std::fs::read_to_string(path.join("skill.json")) {
                SkillManifest::from_json(&content)
            } else {
                None
            };

            if let Some(m) = manifest {
                self.skills.insert(m.name.clone(), (m, path));
            }
        }
    }

    pub fn get_skill(&self, name: &str) -> Option<&(SkillManifest, std::path::PathBuf)> {
        self.skills.get(name)
    }

    pub fn get_all_skills(&self) -> Vec<&SkillManifest> {
        self.skills.values().map(|(m, _)| m).collect()
    }

    pub fn remove_skill(&mut self, name: &str) -> bool {
        self.skills.remove(name).is_some()
    }
}
