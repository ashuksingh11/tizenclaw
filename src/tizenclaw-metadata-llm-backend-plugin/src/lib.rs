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

//! TizenClaw LLM Backend metadata parser plugin (Rust implementation).
//!
//! Exports the 9 `PKGMGR_MDPARSER_PLUGIN_*` C ABI symbols required by
//! Tizen's package manager parser plugin interface.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int};

use tizenclaw_plugin::ffi::GList;

/// Metadata key that this plugin monitors.
const METADATA_KEY: &[u8] = b"http://tizen.org/metadata/tizenclaw/llm-backend\0";

/// Plugin display name for log messages.
const PLUGIN_NAME: &str = "LLM backend";
const PLUGIN_INFO_JSON: &str = r#"{
  "plugin_id": "tizenclaw-llm-backend",
  "platform_name": "TizenClaw LLM Backend Metadata",
  "version": "1.0.0",
  "priority": 45,
  "capabilities": ["llm_backend_metadata", "backend_discovery"]
}"#;

#[no_mangle]
pub extern "C" fn claw_plugin_info() -> *const c_char {
    tizenclaw_plugin::plugin_info_raw(PLUGIN_INFO_JSON)
}

#[no_mangle]
pub unsafe extern "C" fn claw_plugin_free_string(s: *const c_char) {
    tizenclaw_plugin::plugin_free_string(s);
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_INSTALL(
    pkgid: *const c_char,
    _appid: *const c_char,
    metadata: *mut GList,
) -> c_int {
    tizenclaw_plugin::validate_metadata(pkgid, metadata, METADATA_KEY, PLUGIN_NAME)
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_UPGRADE(
    pkgid: *const c_char,
    _appid: *const c_char,
    metadata: *mut GList,
) -> c_int {
    tizenclaw_plugin::validate_metadata(pkgid, metadata, METADATA_KEY, PLUGIN_NAME)
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_UNINSTALL(
    _pkgid: *const c_char,
    _appid: *const c_char,
    _metadata: *mut GList,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_CLEAN(
    _pkgid: *const c_char,
    _appid: *const c_char,
    _metadata: *mut GList,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_UNDO(
    _pkgid: *const c_char,
    _appid: *const c_char,
    _metadata: *mut GList,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_REMOVED(
    _pkgid: *const c_char,
    _appid: *const c_char,
    _metadata: *mut GList,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_RECOVERINSTALL(
    pkgid: *const c_char,
    appid: *const c_char,
    metadata: *mut GList,
) -> c_int {
    PKGMGR_MDPARSER_PLUGIN_INSTALL(pkgid, appid, metadata)
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_RECOVERUPGRADE(
    pkgid: *const c_char,
    appid: *const c_char,
    metadata: *mut GList,
) -> c_int {
    PKGMGR_MDPARSER_PLUGIN_UPGRADE(pkgid, appid, metadata)
}

#[no_mangle]
pub unsafe extern "C" fn PKGMGR_MDPARSER_PLUGIN_RECOVERUNINSTALL(
    _pkgid: *const c_char,
    _appid: *const c_char,
    _metadata: *mut GList,
) -> c_int {
    0
}
