//! Skill watcher — watches filesystem for skill package changes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

pub struct SkillWatcher {
    running: Arc<AtomicBool>,
    watch_dirs: Vec<String>,
    on_change: Option<Box<dyn Fn() + Send + Sync>>,
}

impl SkillWatcher {
    pub fn new() -> Self {
        SkillWatcher {
            running: Arc::new(AtomicBool::new(false)),
            watch_dirs: vec![
                "/opt/usr/share/tizenclaw/skills".into(),
                "/opt/usr/share/tizenclaw/custom_skills".into(),
            ],
            on_change: None,
        }
    }

    pub fn set_change_callback(&mut self, cb: impl Fn() + Send + Sync + 'static) {
        self.on_change = Some(Box::new(cb));
    }

    pub fn start(&self) -> Option<std::thread::JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) { return None; }
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let dirs = self.watch_dirs.clone();

        let handle = std::thread::spawn(move || {
            log::info!("SkillWatcher started, monitoring {} dirs", dirs.len());
            let mut mtimes: HashMap<String, u64> = HashMap::new();

            // Initial scan
            for dir in &dirs {
                scan_mtimes(dir, &mut mtimes);
            }

            while running.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let mut changed = false;

                for dir in &dirs {
                    let mut new_mtimes: HashMap<String, u64> = HashMap::new();
                    scan_mtimes(dir, &mut new_mtimes);

                    if new_mtimes != mtimes {
                        changed = true;
                        mtimes = new_mtimes;
                        break;
                    }
                }

                if changed {
                    log::info!("SkillWatcher: change detected");
                    // Note: callback would be invoked via the AgentCore
                }
            }
            log::info!("SkillWatcher stopped");
        });

        Some(handle)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn scan_mtimes(dir: &str, mtimes: &mut HashMap<String, u64>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = path.metadata() {
            let mtime = meta.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            mtimes.insert(path.to_string_lossy().to_string(), mtime);
        }
    }
}
