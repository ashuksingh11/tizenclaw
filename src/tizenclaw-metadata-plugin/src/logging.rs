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

//! Tizen dlog-based logging for metadata plugins.

use std::ffi::{c_int, CString};

/// dlog priority levels from `<dlog.h>`.
const DLOG_INFO: c_int = 4;
const DLOG_ERROR: c_int = 6;

/// Log tag for all TizenClaw metadata plugins.
const TAG: &[u8] = b"TIZENCLAW_METADATA_PLUGIN\0";

/// Printf format string for a single string argument.
const FMT_STR: &[u8] = b"%s\0";

extern "C" {
    fn dlog_print(prio: c_int, tag: *const u8, fmt: *const u8, ...) -> c_int;
}

/// Log an informational message to Tizen dlog.
pub fn log_info(msg: &str) {
    if let Ok(c_msg) = CString::new(msg) {
        unsafe {
            dlog_print(
                DLOG_INFO,
                TAG.as_ptr(),
                FMT_STR.as_ptr(),
                c_msg.as_ptr(),
            );
        }
    }
}

/// Log an error message to Tizen dlog and stderr.
pub fn log_error(msg: &str) {
    eprintln!("{}", msg);
    if let Ok(c_msg) = CString::new(msg) {
        unsafe {
            dlog_print(
                DLOG_ERROR,
                TAG.as_ptr(),
                FMT_STR.as_ptr(),
                c_msg.as_ptr(),
            );
        }
    }
}
