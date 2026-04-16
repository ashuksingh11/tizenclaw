//! ClawHub skill hub client.
//!
//! Provides install, search, and list operations against the ClawHub public registry
//! at <https://clawhub.ai>.  Skills are extracted into the runtime
//! `workspace/skill-hubs/clawhub/<slug>/` directory and are picked up automatically
//! by the next `skill_capability_manager::load_snapshot` call.
//!
//! The lock file at `workspace/.clawhub/lock.json` tracks installed skills so a
//! future update command can re-fetch from the same registry.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_CLAWHUB_BASE_URL: &str = "https://clawhub.ai";
const REQUEST_TIMEOUT_SECS: u64 = 30;
const LOCK_SUBPATH: &str = ".clawhub/lock.json";

// ── Lock file types ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClawHubLockEntry {
    pub slug: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub install_path: String,
    pub installed_at_secs: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClawHubLock {
    #[serde(default)]
    pub skills: Vec<ClawHubLockEntry>,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Search ClawHub for skills matching `query`.
///
/// Returns the raw registry JSON response so the caller can surface it directly
/// via IPC without having to re-serialize an intermediate struct.
pub async fn clawhub_search(query: &str) -> Result<Value, String> {
    let url = format!("{}/api/v1/search", resolve_base_url());
    let client = build_client()?;
    let resp = client
        .get(&url)
        .query(&[("q", query), ("limit", "20")])
        .send()
        .await
        .map_err(|err| format!("ClawHub search request failed: {}", err))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|err| format!("ClawHub search body read failed: {}", err))?;
    if !status.is_success() {
        return Err(format!("ClawHub search failed ({}): {}", status, body));
    }
    serde_json::from_str::<Value>(&body)
        .map_err(|err| format!("ClawHub search parse failed: {}", err))
}

/// Download and install a skill from ClawHub into the runtime skill-hubs tree.
///
/// `source` may be either `clawhub:<slug>` or just `<slug>`.
/// The skill is extracted to `skill_hubs_dir/clawhub/<slug>/` and the lock file
/// is written to `skill_hubs_dir/../.clawhub/lock.json`.
pub async fn clawhub_install(skill_hubs_dir: &Path, source: &str) -> Result<Value, String> {
    let slug = parse_clawhub_slug(source)?;
    let base_url = resolve_base_url();
    let client = build_client()?;

    // Fetch skill metadata to get display name and current version.
    let detail_url = format!("{}/api/v1/skills/{}", base_url, slug);
    let detail_resp = client
        .get(&detail_url)
        .send()
        .await
        .map_err(|err| format!("ClawHub skill detail request failed: {}", err))?;
    let detail_status = detail_resp.status();
    let detail_body = detail_resp
        .text()
        .await
        .unwrap_or_else(|_| String::new());
    if !detail_status.is_success() {
        return Err(format!(
            "ClawHub skill '{}' not found ({}): {}",
            slug, detail_status, detail_body
        ));
    }
    let detail: Value = serde_json::from_str(&detail_body)
        .map_err(|err| format!("ClawHub skill detail parse failed: {}", err))?;

    let display_name = detail
        .pointer("/skill/displayName")
        .and_then(Value::as_str)
        .unwrap_or(&slug)
        .to_string();
    let version = detail
        .pointer("/latestVersion/version")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    // Download the zip archive.
    let download_url = format!("{}/api/v1/download", base_url);
    let download_resp = client
        .get(&download_url)
        .query(&[("slug", slug.as_str())])
        .send()
        .await
        .map_err(|err| format!("ClawHub download request failed: {}", err))?;
    let download_status = download_resp.status();
    if !download_status.is_success() {
        let err_body = download_resp.text().await.unwrap_or_default();
        return Err(format!(
            "ClawHub download for '{}' failed ({}): {}",
            slug, download_status, err_body
        ));
    }
    let archive_bytes = download_resp
        .bytes()
        .await
        .map_err(|err| format!("ClawHub archive read failed: {}", err))?;

    // Extract into a staging directory first, then atomically replace the
    // final install path.  This prevents retries or concurrent updates from
    // leaving a partially-extracted skill in place.
    let install_dir = skill_hubs_dir.join("clawhub").join(&slug);
    let staging_dir = skill_hubs_dir
        .join("clawhub")
        .join(format!("{}.__installing__", slug));

    // Remove any leftover staging directory from a previous failed attempt.
    if staging_dir.exists() {
        std::fs::remove_dir_all(&staging_dir).map_err(|err| {
            format!(
                "Failed to remove stale staging directory '{}': {}",
                staging_dir.display(),
                err
            )
        })?;
    }
    std::fs::create_dir_all(&staging_dir).map_err(|err| {
        format!(
            "Failed to create staging directory '{}': {}",
            staging_dir.display(),
            err
        )
    })?;
    extract_zip_archive(&archive_bytes, &staging_dir, &slug)?;

    // Replace the live install directory with the staging directory.
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir).map_err(|err| {
            format!(
                "Failed to remove existing install directory '{}': {}",
                install_dir.display(),
                err
            )
        })?;
    }
    std::fs::rename(&staging_dir, &install_dir).map_err(|err| {
        format!(
            "Failed to move staging directory to '{}': {}",
            install_dir.display(),
            err
        )
    })?;

    // Record the install in the lock file.
    let workspace_dir = skill_hubs_dir
        .parent()
        .unwrap_or(skill_hubs_dir);
    update_lock_file(
        workspace_dir,
        &slug,
        &display_name,
        version.as_deref(),
        &install_dir,
    )?;

    Ok(json!({
        "status": "installed",
        "slug": slug,
        "display_name": display_name,
        "version": version,
        "install_path": install_dir.to_string_lossy().as_ref(),
    }))
}

/// List skills recorded in the ClawHub lock file.
pub fn clawhub_list(skill_hubs_dir: &Path) -> Result<Value, String> {
    let workspace_dir = skill_hubs_dir
        .parent()
        .unwrap_or(skill_hubs_dir);
    let lock_path = workspace_dir.join(LOCK_SUBPATH);
    let lock = load_lock_file(&lock_path);
    Ok(json!({
        "skills": lock.skills,
        "lock_path": lock_path.to_string_lossy().as_ref(),
    }))
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn resolve_base_url() -> String {
    std::env::var("TIZENCLAW_CLAWHUB_URL")
        .or_else(|_| std::env::var("CLAWHUB_URL"))
        .unwrap_or_else(|_| DEFAULT_CLAWHUB_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("Failed to build HTTP client: {}", err))
}

fn parse_clawhub_slug(source: &str) -> Result<String, String> {
    let trimmed = source.trim();
    let slug = if let Some(rest) = trimmed.strip_prefix("clawhub:") {
        rest.trim()
    } else {
        trimmed
    };
    if slug.is_empty() {
        return Err("ClawHub skill slug cannot be empty.".to_string());
    }
    // Basic sanity check: slugs are lowercase alphanumeric with hyphens.
    if slug
        .chars()
        .any(|char| !char.is_alphanumeric() && char != '-' && char != '_' && char != '.')
    {
        return Err(format!(
            "Invalid ClawHub slug '{}': only alphanumeric characters, hyphens, underscores, and dots are allowed.",
            slug
        ));
    }
    Ok(slug.to_string())
}

fn extract_zip_archive(bytes: &[u8], dest_dir: &Path, slug: &str) -> Result<(), String> {
    use std::io::Cursor;

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|err| format!("Failed to open zip archive: {}", err))?;

    // Determine whether all entries share a common prefix that matches the slug
    // (e.g., `<slug>/SKILL.md`).  If so, strip it when writing.
    let prefix = format!("{}/", slug);
    let all_have_prefix = (0..archive.len()).all(|index| {
        archive
            .by_index(index)
            .map(|entry| {
                let name = entry.name().to_string();
                name.starts_with(&prefix) || name == slug
            })
            .unwrap_or(false)
    });

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("Failed to read archive entry {}: {}", index, err))?;

        if entry.is_dir() {
            continue;
        }

        let raw_name = entry.name().to_string();
        let relative = if all_have_prefix {
            raw_name
                .strip_prefix(&prefix)
                .unwrap_or(&raw_name)
        } else {
            raw_name.as_str()
        };

        if relative.is_empty() {
            continue;
        }

        // Reject path-traversal and absolute-path entries.
        // Checking just for ".." misses absolute entries like "/etc/passwd"
        // which Path::join() would treat as a new root. We inspect every
        // component individually and also verify the final path stays inside
        // dest_dir as a defense-in-depth guard.
        {
            use std::path::Component;
            let unsafe_component = Path::new(relative).components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            });
            if unsafe_component {
                log::warn!(
                    "ClawHub: skipping unsafe archive entry '{}'",
                    raw_name
                );
                continue;
            }
        }

        let out_path = dest_dir.join(relative);

        // Defense-in-depth: after joining, confirm the path is still rooted
        // inside dest_dir (catches any edge case the component check missed).
        if !out_path.starts_with(dest_dir) {
            log::warn!(
                "ClawHub: skipping archive entry '{}' that would escape install dir",
                raw_name
            );
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create directory '{}': {}",
                    parent.display(),
                    err
                )
            })?;
        }

        let mut buf = Vec::new();
        entry
            .read_to_end(&mut buf)
            .map_err(|err| format!("Failed to read archive entry '{}': {}", raw_name, err))?;

        let mut out_file = std::fs::File::create(&out_path).map_err(|err| {
            format!("Failed to create '{}': {}", out_path.display(), err)
        })?;
        out_file.write_all(&buf).map_err(|err| {
            format!("Failed to write '{}': {}", out_path.display(), err)
        })?;
    }

    Ok(())
}

fn load_lock_file(path: &Path) -> ClawHubLock {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<ClawHubLock>(&content).ok())
        .unwrap_or_default()
}

fn update_lock_file(
    workspace_dir: &Path,
    slug: &str,
    display_name: &str,
    version: Option<&str>,
    install_dir: &Path,
) -> Result<(), String> {
    let lock_path = workspace_dir.join(LOCK_SUBPATH);
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!("Failed to create lock directory '{}': {}", parent.display(), err)
        })?;
    }

    let mut lock = load_lock_file(&lock_path);
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Some(entry) = lock.skills.iter_mut().find(|entry| entry.slug == slug) {
        entry.display_name = display_name.to_string();
        entry.version = version.map(ToString::to_string);
        entry.install_path = install_dir.to_string_lossy().to_string();
        entry.installed_at_secs = now_secs;
    } else {
        lock.skills.push(ClawHubLockEntry {
            slug: slug.to_string(),
            display_name: display_name.to_string(),
            version: version.map(ToString::to_string),
            install_path: install_dir.to_string_lossy().to_string(),
            installed_at_secs: now_secs,
        });
    }

    let serialized = serde_json::to_string_pretty(&lock)
        .map_err(|err| format!("Failed to serialize lock file: {}", err))?;
    std::fs::write(&lock_path, &serialized).map_err(|err| {
        format!(
            "Failed to write lock file '{}': {}",
            lock_path.display(),
            err
        )
    })?;

    Ok(())
}

// ── Path helper ──────────────────────────────────────────────────────────────

/// Return the `workspace/skill-hubs` path from a `PlatformPaths` reference.
pub fn skill_hubs_dir_from_paths(
    paths: &libtizenclaw_core::framework::paths::PlatformPaths,
) -> PathBuf {
    paths.skill_hubs_dir.clone()
}

#[cfg(test)]
mod tests {
    use super::{load_lock_file, parse_clawhub_slug, update_lock_file, ClawHubLock};
    use tempfile::tempdir;

    #[test]
    fn parse_clawhub_slug_strips_clawhub_prefix() {
        assert_eq!(
            parse_clawhub_slug("clawhub:battery-helper").unwrap(),
            "battery-helper"
        );
        assert_eq!(
            parse_clawhub_slug("battery-helper").unwrap(),
            "battery-helper"
        );
    }

    #[test]
    fn parse_clawhub_slug_rejects_empty() {
        assert!(parse_clawhub_slug("clawhub:").is_err());
        assert!(parse_clawhub_slug("").is_err());
    }

    #[test]
    fn update_and_load_lock_file_round_trips() {
        let dir = tempdir().unwrap();
        update_lock_file(
            dir.path(),
            "test-skill",
            "Test Skill",
            Some("1.0.0"),
            &dir.path().join("skill-hubs/clawhub/test-skill"),
        )
        .unwrap();

        let lock = load_lock_file(&dir.path().join(".clawhub/lock.json"));
        assert_eq!(lock.skills.len(), 1);
        assert_eq!(lock.skills[0].slug, "test-skill");
        assert_eq!(lock.skills[0].version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn update_lock_file_upserts_existing_entry() {
        let dir = tempdir().unwrap();
        let install_dir = dir.path().join("skill-hubs/clawhub/test-skill");

        update_lock_file(dir.path(), "test-skill", "Test Skill", Some("1.0.0"), &install_dir)
            .unwrap();
        update_lock_file(dir.path(), "test-skill", "Test Skill", Some("1.1.0"), &install_dir)
            .unwrap();

        let lock = load_lock_file(&dir.path().join(".clawhub/lock.json"));
        assert_eq!(lock.skills.len(), 1);
        assert_eq!(lock.skills[0].version.as_deref(), Some("1.1.0"));
    }

    #[test]
    fn clawhub_list_returns_empty_when_no_lock_file() {
        let dir = tempdir().unwrap();
        let skill_hubs_dir = dir.path().join("workspace/skill-hubs");
        std::fs::create_dir_all(&skill_hubs_dir).unwrap();
        let result = super::clawhub_list(&skill_hubs_dir).unwrap();
        let skills = result["skills"].as_array().unwrap();
        assert!(skills.is_empty());
    }
}
