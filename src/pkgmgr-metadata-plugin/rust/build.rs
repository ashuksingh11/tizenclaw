fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Let pkg-config find the necessary Tizen libraries
    // We only need these for the Rust code that calls their APIs:
    // pkgmgr-installer (pkgmgr_installer_info_get_privilege_level)
    // dlog (dlog_print)
    // glib-2.0 is needed for GList definition and handling but we just use raw pointers in Rust
    
    // We don't necessarily need to link them strictly here because the final CMake target
    // links the shared library with these dependencies, but it's good practice for staticlib
    // generation to express its requirements. The actual linking happens when building the .so.
    
    // For Tizen GBS build, pkg-config will find these
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=dlog");
        println!("cargo:rustc-link-lib=pkgmgr-installer");
        println!("cargo:rustc-link-lib=glib-2.0");
    }
}
