# Planning Progress: Runtime Resilience & Feature Toggle Architecture

Based on the directive: "tizenclaw에서는 tizen관련 so가 설치될 때만 tizen 기능을 이용합니다."

## Step 1: Project Requirements Analysis
The `tizenclaw` daemon needs conditional capabilities based on dynamic host libraries rather than hard dependency linkages. Currently, the `tizen-sys/build.rs` statically links flags (`-ldlog`, `-lsoup-2.4`, etc.), making the binary crash instantly inside WSL/Linux if those `.so` files are unavailable, necessitating a cumbersome `mock-sys` cargo feature and workspace exclusion behavior.

## Step 2: Agent Feature Listing
| Capability | Core Daemon Need | Tizen Dependent | Fallback Behavior |
| --- | --- | --- | --- |
| Logging | Yes | `libdlog.so` | Standard `stdout` / `tracing` |
| Web Dashboard | Yes | `libsoup-2.4.so` | Graceful disable / pure TCP listener |
| Task Spawning | Yes | No (`tokio`) | N/A |
| IPC Sockets | Yes | No (Abstract UDS) | N/A |

## Step 3: Module Integration Planning
- **Execution Mode**: `tizenclaw` is a Daemon running on multiple environments.
- We must abolish the isolated `tizen-sys` compilation requirement and integrate it back into the main `Cargo.toml` members.
- **Cross-platform guarantee**: Single binary runs on both Desktop Linux and Embedded Tizen safely by probing for `libxxx.so` at boot.
