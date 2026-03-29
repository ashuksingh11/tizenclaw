# TizenClaw Refactoring & Modification Plan (Planner & Architect)

## 1. Project Analysis (Planner Perspective)
The `tizenclaw` currently depends heavily on Tizen OS `.so` files (like `libdlog.so` or `libsoup-2.4.so`) through static linking inside `tizen-sys/build.rs`. Because of this, it forces the exclusion of `tizen-sys` and `tizenclaw-core` from the main Cargo workspace (`Cargo.toml`) and breaks standard Linux/WSL compatibility unless complex `mock-sys` features are used.

**Goal:** Transform the Tizen integration layer into a fully dynamic loading architecture (using graceful degradation) so that:
1. The single binary can boot on any Linux kernel or OSX.
2. If `libdlog.so` and Tizen frameworks exist (probed at runtime), it binds to them via FFI.
3. If they do not exist, the daemon falls back to standard `stdout`/`tracing` equivalents seamlessly.

## 2. Architecture & Design Modification Plan (Architect Perspective)

### A. Removing Static Bindings
- Delete all `cargo:rustc-link-lib=...` directives from `src/tizen-sys/build.rs`.
- Consolidate the `tizen-sys` codebase to remove the `#![cfg(feature = "mock-sys")]` complexity since dynamic resolution handles this gracefully.

### B. Dynamic Library Loading (`libloading`)
- For Tizen C-APIs, we will implement `lazy_static` or `LazyLock` structures that invoke `libloading::Library::new("libdlog.so.0")` internally.
- Functions like `dlog_print` will be retrieved via `.get::<Symbol<unsafe extern "C" fn(...)>>`.
- Expose safe Rust `Result<T, E>` wrappers in `tizen-sys` to the rest of the daemon.

### C. Workspace Integration
- Reintroduce `tizen-sys` and `tizenclaw-core` into the `Cargo.toml` `workspace.members`.
- Eliminate the need for isolated `cargo test-mock` offline builds; standard `cargo build` and `cargo test` across the entire workspace will now pass cleanly anywhere.

## 3. Execution Sequencing (Development)
Upon approval, the `developing-code` persona will begin executing Phase 4 (Development) following this sequential modification logic under TDD rules.
