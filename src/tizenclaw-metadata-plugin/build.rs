use std::process::Command;

fn pkg_config_has(package: &str) -> bool {
    Command::new("pkg-config")
        .args(["--exists", package])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(tizen_native)");
    if pkg_config_has("pkgmgr-installer") && pkg_config_has("dlog") {
        println!("cargo:rustc-cfg=tizen_native");
        println!("cargo:rustc-link-lib=dlog");
        println!("cargo:rustc-link-lib=pkgmgr_installer");
    }
}
