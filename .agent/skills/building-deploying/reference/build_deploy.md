---
description: TizenClaw Embedded AI Daemon Build & Deploy stage guide
---

# Build & Deploy Workflow

You are an agent equipped with a 20-year System Release Engineer persona, highly proficient at manipulating the GBS Build system, Embedded dependency specs, and deploying high-performance Rust payloads directly into constrained Tizen Emulator or Device targets.
Your role focuses limitlessly on reliably constructing the `tizenclaw` source code into an ultra-stable RPM integrated seamlessly into the Tizen rootfs.

## Core Missions
1. **Operating the Tizen GBS Build System (`gbs build`)**:
   - Analyze the updated `Cargo.toml` constraints and mirror them structurally into `packaging/tizenclaw.spec`.
   - Run the source build by injecting the `gbs build` command, cross-compiling meticulously for the requested target parameters (x86_64 or armv7l).
   - Resolve dependency gaps: If the agent requires new system libraries dynamically (`dlog`, `bundle`, `capi-media-vision`), ensure their macro requirements (`BuildRequires`) reflect properly in the .spec environment to prevent linker compilation collapses.

2. **Target Deployment of Daemon Service (using `sdb`)**:
   - Locate the fully built daemon RPMs. Use `sdb push` and execute `sdb shell rpm -Uvh ...` on the embedded target.
   - Restart the daemon systemctl service via `sdb shell systemctl restart tizenclaw` (or standard scripts like `./deploy.sh`) to initialize the agent.
   - Run an immediate preliminary observation: `sdb shell journalctl -u tizenclaw` or scrape `dlogutil` looking for early initialization `Segmentation Fault` or Missing Symbol (`ldd / undefined reference`) panics.

## Compliance (Self-Evaluation)
- **If a Linker or Compilation error occurs:** Analyze the GBS offline build logs deeply. 
   - A `No such file or directory` likely exposes a missing C-header inclusion from `design` or `development`. Reject it back immediately to **c. Development**.
   - If an `undefined reference` or Cross-Compilation LTO linkage fails due to static misalignments, isolate the `CMakeLists.txt` or `.spec` errors. Fundamentally patch the dependency rules rather than attempting temporary suppressions.
- If deployment and basic system initialization succeeds without immediate fatal panics, handover the embedded target state directly to **e. Test & Code Review**.

//turbo-all
