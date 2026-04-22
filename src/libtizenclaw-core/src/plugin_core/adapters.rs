//! Capability adapter contracts exposed by platform plugins.

use serde_json::Value;

pub trait LoggingAdapter: Send + Sync {
    fn log(&self, level: &str, tag: &str, message: &str);
}

pub trait PackageManagerAdapter: Send + Sync {
    fn list_packages(&self) -> Vec<String>;
    fn get_package_info(&self, pkg_id: &str) -> Option<Value>;
}
