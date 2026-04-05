use std::path::PathBuf;

pub fn default_data_dir() -> PathBuf {
    if let Ok(path) = std::env::var("TIZENCLAW_DATA_DIR") {
        return PathBuf::from(path);
    }
    if std::path::Path::new("/etc/tizen-release").exists()
        || std::path::Path::new("/opt/usr/share/tizenclaw").exists()
    {
        return PathBuf::from("/opt/usr/share/tizenclaw");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".tizenclaw")
}

pub fn default_tools_dir() -> PathBuf {
    if let Ok(path) = std::env::var("TIZENCLAW_TOOLS_DIR") {
        return PathBuf::from(path);
    }
    default_data_dir().join("tools")
}
