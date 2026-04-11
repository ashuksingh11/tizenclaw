//! Boot status logger — appends simple boot progress markers.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct BootStatusLogger {
    log_path: PathBuf,
}

impl BootStatusLogger {
    pub fn new(log_path: PathBuf) -> Self {
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        Self { log_path }
    }

    pub fn write(&self, message: &str) {
        self.append_line(&format!("{}\n", message));
    }

    pub fn write_phase(&self, phase: u8, total: u8, message: &str) {
        self.append_line(&format!("[{}/{}] {}\n", phase, total, message));
    }

    fn append_line(&self, line: &str) {
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .and_then(|mut file| file.write_all(line.as_bytes()));
    }
}
