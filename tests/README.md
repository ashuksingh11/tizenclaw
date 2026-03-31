# TizenClaw Automated Test Suite

End-to-end test automation framework for TizenClaw. Runs against real devices
(emulators, TVs, refrigerators, etc.) via `sdb` connection.

## Quick Start

```bash
# Run all verification suites
./tests/verification/run_all.sh

# Target a specific device
./tests/verification/run_all.sh -d <device-serial>

# Run a specific suite
./tests/verification/run_all.sh -s cli_tools

# Run multiple suites
./tests/verification/run_all.sh -s service,mcp,regression

# List available suites
./tests/verification/run_all.sh --list

# Run a single test file
./tests/verification/cli_tools/test_device_info.sh -d <device-serial>
```

## Prerequisites

1. **Device connected** — Verify with `sdb devices`
2. **TizenClaw deployed** — Run `./deploy.sh` first
3. **Service running** — `sdb shell systemctl is-active tizenclaw` → `active`
4. **jq installed** (host) — Required for MCP and JSON assertion tests
   ```bash
   sudo apt-get install jq
   ```

## Directory Structure

```
tests/
├── e2e/                             # E2E smoke tests (deploy.sh -t)
│   ├── test_smoke.sh
│   └── test_mcp.sh
└── verification/                    # Full verification suites (deploy.sh -T)
    ├── run_all.sh                   # Master runner
    ├── lib/
    │   └── test_framework.sh        # Shared assertion & utility library
    ├── service/                     # Daemon health & infrastructure
    ├── cli_tools/                   # CLI tool validation (13 tools)
    ├── embedded_tools/              # Session, workflow, pipeline, code exec
    ├── llm_integration/             # LLM agent prompt/response/streaming
    ├── mcp/                         # MCP JSON-RPC compliance
    └── regression/                  # Crash resilience & edge cases
```

## Test Suites

### `service` — Daemon Health
Checks service status, binary installation, IPC socket, tool loading, work
directories, restart resilience, and web dashboard access.

### `cli_tools` — CLI Tool Validation
Tests each CLI tool binary directly on the device. Validates JSON output
structure, correct data fields, and CRUD operations (for file manager).
Gracefully skips tests when hardware is unavailable (e.g., sensors on emulator).

### `embedded_tools` — Embedded Tool Operations
Tests session management, workflow CRUD, pipeline CRUD, task management,
and code execution through `tizenclaw-cli` prompts.

### `llm_integration` — LLM Agent Tests
Validates the full agentic loop: natural language prompt → LLM reasoning →
tool invocation → response. Tests Korean/English prompts, multi-tool calls,
streaming mode, and error handling.

### `mcp` — MCP Protocol Compliance
Validates MCP JSON-RPC 2.0 protocol: `initialize`, `tools/list`, error codes,
malformed input, notifications, and edge cases.

### `regression` — Regression & Stability
Tests for crash resilience under rapid calls, concurrent sessions, empty
prompts, Unicode, special characters, and memory usage monitoring.

## Options

| Flag | Description |
|------|-------------|
| `-d, --device <serial>` | Target a specific device (from `sdb devices`) |
| `-s, --suite <names>` | Comma-separated suite names to run |
| `-v, --verbose` | Enable verbose log output |
| `-t, --timeout <seconds>` | Per-command timeout (default: 30) |
| `--list` | List available test suites |

## Writing New Tests

1. Create a new `test_<feature>.sh` file in the appropriate suite directory
2. Source the framework:
   ```bash
   SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
   source "${SCRIPT_DIR}/../lib/test_framework.sh"
   tc_parse_args "$@"
   tc_preflight
   ```
3. Use `suite_begin` / `section` / assertions / `suite_end`
4. Available assertions:
   - `assert_contains`, `assert_not_contains`
   - `assert_not_empty`, `assert_empty`
   - `assert_eq`, `assert_ne`, `assert_ge`, `assert_le`
   - `assert_file_exists`, `assert_dir_exists`
   - `assert_json_valid`, `assert_json`, `assert_json_eq`
   - `assert_json_array_ge`
5. Device helpers:
   - `sdb_shell` — remote shell command
   - `cli_exec <tool> <args>` — execute a CLI tool
   - `tc_cli <prompt>` — send prompt to tizenclaw-cli
   - `tc_cli_session <id> <prompt>` — with session
   - `tc_device_profile` — detect TV/mobile/wearable
   - `tc_tool_exists <path>` — check binary on device

## Device Profiles

Tests automatically detect the device profile and skip hardware-specific
tests on unsupported devices:

| Profile | Example Devices |
|---------|----------------|
| `tv` | Samsung Smart TV |
| `mobile` | Tizen Mobile Emulator |
| `wearable` | Galaxy Watch |
| `iot` | Smart Refrigerator, etc. |

## CI Integration

The master runner returns exit code `0` only when all suites pass,
making it suitable for CI pipelines:

```yaml
# Example CI step
- name: E2E Tests
  run: |
    sdb connect $DEVICE_IP
    ./tests/verification/run_all.sh -d $DEVICE_SERIAL
```

## Test Directory Layout

| Location | Type | Purpose |
|----------|------|---------|
| `tests/e2e/` | Shell | E2E smoke tests (used by `deploy.sh -t`) |
| `tests/verification/` | Shell | Full verification suites (used by `deploy.sh -T`) |
