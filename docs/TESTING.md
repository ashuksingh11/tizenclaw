# Testing and Test Automation

With the successful migration of TizenClaw from a legacy C++ daemon to a full **Rust 1.83+ Workspace** architecture, testing has been completely redefined. 
Moving away from slow Google Test cross-compilations, the framework now enforces rapid local feedback cycles directly on the developer's workstation via the active WSL Mock-Sys Test Strategy.

## 1. Local Host Tests (`test-mock`)

Because Tizen-native APIs (such as `tizen-sys` wrappers for `dlog`, `app_control`, and `bundle`) cannot link on a standard Ubuntu or Windows machine, the crate employs conditional compilation toggles via Cargo features (`mock-sys`).

This feature conditionally skips native library bindings to test pure application logic (like the **PromptBuilder** string templates, **TextualSkillScanner** file IO patterns, and **ToolPolicy** loops).

**Prerequisite Pipeline Configuration (.cargo/config.toml)**

A standard alias ensures that every developer reliably invokes the exact Mock-Sys compilation parameters:

```toml
[alias]
test-mock = "test --workspace --features mock-sys"
```

### Running Test Units
To evaluate all logic, you execute:
```bash
cargo test-mock
```
Every isolated `#[cfg(test)]` submodule is thoroughly verified locally in `< 2.0s`. No `gbs` or remote emulation delays.

## 2. Remote Test Runtimes (`deploy.sh`)

When logic is proven locally, developers use `deploy.sh` to package RPMs via Tizen Git Build System (`gbs`) and deploy the active build over the Smart Development Bridge (`sdb`).

A standard workflow for complete remote emulation integration runs like this:

### The E2E Live Interaction Testing Loop

```bash
# 1. Package the Rust Binaries for the active X86_64 Emulator
./deploy.sh -d emulator-26101

# 2. Invoke the deployed rust-based TizenClaw Daemon 
# to ensure it can successfully reply from LLM via UNIX Socket.
sdb -s emulator-26101 shell tizenclaw-cli 'ping'

# 3. Pull telemetry back from the device token tracker logs
sdb -s emulator-26101 shell tizenclaw-cli --usage
```

This procedure guarantees a stable, fully orchestrated release loop.
