//! TizenClaw pkgmgr metadata parser plugin — Rust core logic.
//!
//! This staticlib provides `tizenclaw_metadata_check_privilege()`, which is called
//! by thin C plugin shims to verify that packages declaring TizenClaw metadata
//! are signed with platform-level certificates.
//!
//! The 3 C shim `.so` files (llm-backend, skill, cli) each export the 9 required
//! `PKGMGR_MDPARSER_PLUGIN_*` symbols and delegate privilege checking here.

use libc::{c_char, c_int, c_void};
use std::ffi::CStr;

// ─────────────────────────────────────────────────────
//  GLib FFI types
// ─────────────────────────────────────────────────────

/// GLib GList node — opaque linked list used by pkgmgr to pass metadata.
#[repr(C)]
pub struct GList {
    pub data: *mut c_void,
    pub next: *mut GList,
    pub prev: *mut GList,
}

// ─────────────────────────────────────────────────────
//  pkgmgr-parser FFI types
// ─────────────────────────────────────────────────────

/// `__metadata_t` from pkgmgr_parser.h — key/value pair in the metadata GList.
#[repr(C)]
pub struct MetadataT {
    pub key: *mut c_char,
    pub value: *mut c_char,
}

// ─────────────────────────────────────────────────────
//  pkgmgr-installer FFI
// ─────────────────────────────────────────────────────

/// Platform privilege level — matches `pkgmgr_privilege_level` enum.
const PM_PRIVILEGE_PLATFORM: c_int = 4;

extern "C" {
    fn pkgmgr_installer_info_get_privilege_level(level: *mut c_int) -> c_int;
}

// ─────────────────────────────────────────────────────
//  dlog FFI
// ─────────────────────────────────────────────────────

/// dlog priority levels
const DLOG_INFO: c_int = 4;
const DLOG_ERROR: c_int = 6;

extern "C" {
    fn dlog_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
}

// ─────────────────────────────────────────────────────
//  Internal helpers
// ─────────────────────────────────────────────────────

/// Log tag for all metadata plugin messages.
const LOG_TAG: &[u8] = b"TIZENCLAW_METADATA_PLUGIN\0";

/// Safe wrapper for dlog INFO.
fn log_info(msg: &[u8]) {
    // msg must be null-terminated
    unsafe {
        dlog_print(
            DLOG_INFO,
            LOG_TAG.as_ptr() as *const c_char,
            b"%s\0".as_ptr() as *const c_char,
            msg.as_ptr() as *const c_char,
        );
    }
}

/// Safe wrapper for dlog ERROR.
fn log_error(msg: &[u8]) {
    unsafe {
        dlog_print(
            DLOG_ERROR,
            LOG_TAG.as_ptr() as *const c_char,
            b"%s\0".as_ptr() as *const c_char,
            msg.as_ptr() as *const c_char,
        );
    }
}

/// Check if the current installer context has platform-level privilege.
fn has_platform_privilege() -> bool {
    let mut level: c_int = 0;
    let ret = unsafe { pkgmgr_installer_info_get_privilege_level(&mut level) };
    if ret != 0 {
        log_error(b"Failed to get privilege level\0");
        return false;
    }
    level == PM_PRIVILEGE_PLATFORM
}

/// Convert a C string pointer to a byte slice (without allocation).
/// Returns an empty slice if the pointer is null.
unsafe fn cstr_bytes(ptr: *const c_char) -> &'static [u8] {
    if ptr.is_null() {
        return b"";
    }
    CStr::from_ptr(ptr).to_bytes()
}

// ─────────────────────────────────────────────────────
//  Exported C ABI function
// ─────────────────────────────────────────────────────

/// Check if a package has platform-level privilege for a TizenClaw metadata key.
///
/// Called by the C plugin shims during INSTALL/UPGRADE lifecycle hooks.
///
/// # Arguments
/// * `pkgid` - Package ID (C string, may be null)
/// * `metadata` - GList of `__metadata_t` entries (may be null)
/// * `metadata_key` - The metadata URI to match (C string, must not be null)
/// * `plugin_name` - Plugin variant name for logging (C string, must not be null)
///
/// # Returns
/// * `0` — allow installation (no matching key found, or privilege check passed)
/// * `-1` — reject installation (matching key found but package lacks platform privilege)
#[no_mangle]
pub extern "C" fn tizenclaw_metadata_check_privilege(
    pkgid: *const c_char,
    metadata: *mut GList,
    metadata_key: *const c_char,
    plugin_name: *const c_char,
) -> c_int {
    // Guard against null key/name — should never happen, but be defensive.
    if metadata_key.is_null() || plugin_name.is_null() {
        return 0;
    }

    let key_bytes = unsafe { cstr_bytes(metadata_key) };
    let _plugin_bytes = unsafe { cstr_bytes(plugin_name) };
    let pkgid_bytes = unsafe { cstr_bytes(pkgid) };

    // Build log message for package ID
    let mut log_buf = [0u8; 256];
    let prefix = b"package=";
    let mut pos = 0usize;
    for &b in prefix.iter().chain(pkgid_bytes.iter()) {
        if pos >= log_buf.len() - 1 {
            break;
        }
        log_buf[pos] = b;
        pos += 1;
    }
    log_buf[pos] = 0;
    log_info(&log_buf[..pos + 1]);

    // Walk the GList looking for a matching metadata key
    let mut iter = metadata;
    while !iter.is_null() {
        let node = unsafe { &*iter };
        let md = node.data as *const MetadataT;

        if !md.is_null() {
            let md_ref = unsafe { &*md };
            let md_key = unsafe { cstr_bytes(md_ref.key) };

            if md_key == key_bytes {
                // Found matching metadata key — check privilege
                if !has_platform_privilege() {
                    // Build rejection log message
                    let mut err_buf = [0u8; 512];
                    let err_prefix = b"Package(";
                    let err_suffix = b") was not signed by platform level certificate";
                    let mut epos = 0usize;
                    for &b in err_prefix
                        .iter()
                        .chain(pkgid_bytes.iter())
                        .chain(err_suffix.iter())
                    {
                        if epos >= err_buf.len() - 1 {
                            break;
                        }
                        err_buf[epos] = b;
                        epos += 1;
                    }
                    err_buf[epos] = 0;
                    log_error(&err_buf[..epos + 1]);

                    return -1; // Reject installation
                }

                // Build success log message
                let mut ok_buf = [0u8; 512];
                let ok_prefix = b"Package(";
                let ok_suffix = b") has valid platform signature for TizenClaw plugin";
                let mut opos = 0usize;
                for &b in ok_prefix
                    .iter()
                    .chain(pkgid_bytes.iter())
                    .chain(ok_suffix.iter())
                {
                    if opos >= ok_buf.len() - 1 {
                        break;
                    }
                    ok_buf[opos] = b;
                    opos += 1;
                }
                ok_buf[opos] = 0;
                log_info(&ok_buf[..opos + 1]);

                return 0; // Allow — privilege verified
            }
        }

        iter = node.next;
    }

    0 // No matching metadata key found — allow installation
}
