/*
 * Copyright (c) 2026 Samsung Electronics Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! TizenClaw metadata plugin shared library.
//!
//! Provides FFI bindings and common validation logic for the 3 pkgmgr
//! metadata parser plugins (llm-backend, skill, cli).

pub mod ffi;
pub mod logging;

use std::ffi::{c_char, c_int, CStr, CString};

#[cfg(any(
    test,
    all(feature = "platform-plugin-exports", not(feature = "helper-only"))
))]
const PLUGIN_INFO_JSON: &str = r#"{
  "plugin_id": "tizen",
  "platform_name": "Tizen",
  "version": "1.0.0",
  "priority": 100,
  "capabilities": [
    "logging",
    "system_info",
    "package_manager",
    "app_control",
    "system_events"
  ]
}"#;

/// Validate metadata entries against a specific TizenClaw metadata key.
///
/// Iterates through the GLib linked list of metadata entries and checks
/// whether the package has the required metadata key. If found, validates
/// that the installer has platform-level privilege.
///
/// # Safety
///
/// - `pkgid` must be a valid C string pointer or null.
/// - `metadata` must be a valid GList pointer or null.
/// - `metadata_key` must be a null-terminated byte string.
///
/// # Returns
///
/// - `0` if the package is allowed (no matching key, or key found with platform privilege).
/// - `-1` if the package has the metadata key but lacks platform privilege.
pub unsafe fn validate_metadata(
    pkgid: *const c_char,
    metadata: *mut ffi::GList,
    metadata_key: &[u8],
    plugin_name: &str,
) -> c_int {
    let pkgid_str = if pkgid.is_null() {
        "<null>"
    } else {
        CStr::from_ptr(pkgid).to_str().unwrap_or("<invalid>")
    };

    crate::plugin_log_info!("{} plugin: package={}", plugin_name, pkgid_str);

    // Strip null terminator for comparison
    let key_bytes = if metadata_key.last() == Some(&0) {
        &metadata_key[..metadata_key.len() - 1]
    } else {
        metadata_key
    };

    let mut iter = metadata;
    while !iter.is_null() {
        let node = &*iter;
        let md = node.data as *const ffi::MetadataT;
        if !md.is_null() && !(*md).key.is_null() {
            let key = CStr::from_ptr((*md).key);
            if key.to_bytes() == key_bytes {
                if !has_platform_privilege() {
                    crate::plugin_log_error!(
                        "Package({}) was not signed by platform level certificate",
                        pkgid_str
                    );
                    return -1;
                }

                let value = if (*md).value.is_null() {
                    "(empty)"
                } else {
                    CStr::from_ptr((*md).value).to_str().unwrap_or("(invalid)")
                };
                crate::plugin_log_info!(
                    "Package({}) has valid platform signature for {}: {}",
                    pkgid_str,
                    plugin_name,
                    value
                );
                break;
            }
        }
        iter = node.next;
    }

    0 // Allow installation
}

pub fn plugin_info_raw(json: &str) -> *const c_char {
    match CString::new(json) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null(),
    }
}

pub unsafe fn plugin_free_string(s: *const c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s as *mut c_char);
    }
}

#[cfg(all(feature = "platform-plugin-exports", not(feature = "helper-only")))]
#[no_mangle]
pub extern "C" fn claw_plugin_info() -> *const c_char {
    plugin_info_raw(PLUGIN_INFO_JSON)
}

#[cfg(all(feature = "platform-plugin-exports", not(feature = "helper-only")))]
#[no_mangle]
pub unsafe extern "C" fn claw_plugin_free_string(s: *const c_char) {
    plugin_free_string(s);
}

/// Check whether the current installer has platform-level privilege.
unsafe fn has_platform_privilege() -> bool {
    let mut level: c_int = ffi::PM_PRIVILEGE_UNKNOWN;
    let ret = ffi::pkgmgr_installer_info_get_privilege_level(&mut level);
    if ret != 0 {
        crate::plugin_log_error!("Failed to get privilege level");
        return false;
    }
    level == ffi::PM_PRIVILEGE_PLATFORM
}

#[cfg(test)]
mod tests {
    use crate::PLUGIN_INFO_JSON;

    // Tests are executed on-device via deploy.sh, not locally.
    // Unit tests here validate pure-Rust logic only (no FFI calls).

    #[test]
    fn metadata_key_strip_null_terminator() {
        let key = b"http://tizen.org/metadata/tizenclaw/llm-backend\0";
        let stripped = if key.last() == Some(&0) {
            &key[..key.len() - 1]
        } else {
            &key[..]
        };
        assert_eq!(stripped, b"http://tizen.org/metadata/tizenclaw/llm-backend");
    }

    #[test]
    fn plugin_info_json_is_valid() {
        let info: serde_json::Value = serde_json::from_str(PLUGIN_INFO_JSON).unwrap();
        assert_eq!(info["plugin_id"], "tizen");
        assert_eq!(info["priority"], 100);
    }
}
