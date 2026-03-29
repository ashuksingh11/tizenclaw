# Architecture Design: Dynamic FFI Binding

## Step 1: Review Planner Artifacts
The planner determined all static bindings (like `cargo:rustc-link-lib`) must be removed to allow the single binary to survive OS boot without Tizen OS presence natively.

## Step 2: Mapping Tizen C-APIs to Safe Rust Abstractions
To avoid static linking resolution issues, we will adopt the `libloading` crate pattern universally across `tizen-sys` or core capability modules.
Instead of declaring `extern "C" { fn dlog_print(...); }`, we will:
1. Manage a dynamic `libloading::Library::new("libdlog.so.0")` resolution at runtime.
2. Provide safe trait abstractions (`trait PlatformLogger`).
3. If successful, safe Rust trait implementations will invoke it dynamically. Else, they return an error (`Result::Err`) which the daemon catches to activate fallback implementations (`stdout`).

## Step 3: Functional Structuring
- **Logging Layer**: `tizenclaw` will deploy a singular `tracing` core subscriber. If `libdlog` loaded successfully, it routes events to `dlog_print`. Otherwise, `tracing-subscriber::fmt` is used.
- **Cargo.toml Unified Workspace**: `build.rs` of `tizen-sys` will be simplified to remove `-l` linker directives, allowing its inclusion inside the `[workspace.members]` alongside `tizenclaw-core`. This enables 100% native verification on WSL via single `cargo test` command.
- **Error Handling Strategy**: Use `Result` chaining via `thiserror` matching specific load failures for graceful degradation.
