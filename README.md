<p align="center">
  <img src="docs/img/tizenclaw.jpg" alt="TizenClaw Logo" width="280">
</p>

<h1 align="center">TizenClaw (Rust Edition)</h1>

<p align="center">
  <strong>The Agentic OS Layer for Tizen Devices</strong><br>
  Control, monitor, and automate your Tizen TV and embedded devices through natural language.<br>
  Powered by a next-gen Rust asynchronous daemon, Textual Skills, and real-time screen perception.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/Rust-1.83%2B-orange.svg" alt="Language">
  <img src="https://img.shields.io/badge/Tizen_10.0%2B-Supported-brightgreen.svg" alt="Platform">
  <img src="https://img.shields.io/badge/LLM_Backends-Multi--Provider-purple.svg" alt="LLM Backends">
</p>

<p align="center">
  <a href="#-key-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-documentation">Documentation</a>
</p>

---

## 🔍 Overview

**TizenClaw** is a native system daemon built entirely in **Rust** that brings advanced Agentic AI capabilities to [Tizen OS](https://www.tizen.org/). It replaces traditional UI navigation with intent-driven natural language control, executing device-level commands, orchestrating workflows, and perceiving screen state in real-time.

Originally written in C++, the project has been fully migrated to Rust to provide **zero-overhead concurrency (`tokio`)**, absolute memory safety, and seamless `cargo`-based cross-platform testing (WSL Mock-Sys).

---

## ✨ Key Features

### 🚀 Rust-Native Asynchronous Engine
- **Memory Safe & Zero-Cost Abstractions**: Built with Rust & `tokio` for handling multiple concurrent LLM streams, WebDashboard SSE events, and IPC sockets without mutex contention.
- **Micro-Footprint**: Minimal RSS memory usage, highly optimized for embedded TVs and smart appliances.

### 🧠 Dual-Track Intelligence (Skills & Tools)
- **Executable Tools**: Python/Shell scripts running securely via a separate `tizenclaw-tool-executor` daemon validating connections via `SO_PEERCRED`.
- **Textual Skills (Standardized Spec)**: Dynamic Markdown-based `.md` workflows that instruct the LLM *how* to use the tools, enabling instant agent upgrades without recompilation.

### 📱 Tizen Native FFI Access
- **`tizen-sys`** and **`libtizenclaw`**: Direct `bindgen` Rust ABI wrappers wrapping C libraries (`bundle`, `dlog`, `app_control`, `ecore_wl2`, `wayland`).
- Supports deploying to `x86_64` (emulators), `armv7l`, and `aarch64` Tizen targets.

### 📡 Multi-Channel Connectivity
- Built-in IPC Abstract Sockets (`tizenclaw.sock`).
- Feature-rich Interactive CLI (`tizenclaw-cli`) offering interactive prompts, streaming outputs, and token/metric usage viewing.

---

## 🚀 Quick Start

### 1. Prerequisites (For Tizen Deployment)
- **sdb** (Smart Development Bridge) correctly installed and in `PATH`.
- Tizen GBS (Git Build System).
- Tizen Emulator or physical device connected (`sdb devices`).

### 2. Build & Deploy
We provide an automated cross-compilation script that handles GBS builds and RPM packaging.

```bash
# Full build + automatic deployment to attached emulator
./deploy.sh -d emulator-26101
```

### 3. Usage & CLI Testing
Once deployed on the device, jump into the `sdb shell`:

```bash
# Connect to the deployed proxy shell
sdb -s emulator-26101 shell

# Run TizenClaw CLI in single-shot mode
tizenclaw-cli 'tell me the battery level'

# Run in interactive streaming mode
tizenclaw-cli --stream
```

### 4. Local Host Testing (Mock-Sys)
Develop and test core logic entirely on your Host OS (WSL Ubuntu / Linux) without booting an emulator!

```bash
# Uses the `.cargo/config.toml` alias to bypass Tizen library linking
cargo test-mock
```

---

## 🏗 Architecture Overview

The workspace is organized into robust, decoupled crates:

1. **`tizenclaw`:** The central daemon containing `AgentCore`, `PromptBuilder`, `ToolWatcher`, and IPC listeners.
2. **`tizenclaw-tool-executor`:** A constrained runtime for shelling out native OS tools and Python sub-scripts, interacting via a Unix socket.
3. **`libtizenclaw`:** A C-ABI bridge so that legacy C/C++ Tizen applications can embed and query the AI agent natively.
4. **`tizenclaw-cli`:** The user-facing Rust utility bridging stdio to the daemon's abstract IPC server.
5. **`tizen-sys`:** Unsafe FFI wrappers dynamically linking to or mocking the Tizen Platform API.

> 📖 **Read the full [Architecture Spec](docs/ARCHITECTURE.md)**.

---

## 📚 Documentation

The documentation has been completely rewritten to reflect the new Rust Agent architecture.

| Guide | Description |
|---|---|
| **[Architecture Spec](docs/ARCHITECTURE.md)** | Core engine components, Worker models, and IPC flows. |
| **[Skills & Tools Guide](docs/SKILLS_AND_TOOLS.md)** | How to build standardized textual `.md` workflows and native OS execute tools. |
| **[Testing & Deployment](docs/TESTING.md)** | Guide on using Mock-sys for local testing and `deploy.sh`. |
| **[API Reference](docs/API_REFERENCE.md)** | Integrating with `libtizenclaw` (C-ABI) and `tizenclaw-cli`. |

---

## 📄 License
This project is licensed under the [Apache License 2.0](LICENSE).
Copyright 2024-2026 Samsung Electronics Co., Ltd.
