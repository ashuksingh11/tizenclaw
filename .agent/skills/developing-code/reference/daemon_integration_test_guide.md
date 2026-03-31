# TizenClaw Daemon Integration Test Guide

This guide defines the standard practice for validating the continuously running TizenClaw Autonomous AI Agent using `sdb shell` in target/emulator environments.

## 1. Test Architecture
All dynamic behaviors developed within the Rust AI Agent daemon MUST have corresponding shell script wrappers capturing the daemon's interactions natively at `tests/tizenclaw/test-*.sh`.

Since Rust `cargo test` runs hit fundamental system isolation boundaries from WSL, functional verification of background logic, event listening, and state-machine transitions MUST be executed against a running Target device or Emulator via `sdb shell`.

## 2. Standard Pattern
Integration test scripts must return exit code `0` on absolute success, and any non-zero code upon failure or systemic crash.
To test a long-running robust daemon, use `sdb shell` to trigger signals, interact via IPC/D-Bus/HTTP endpoints, or scrape output logs (e.g. `dlog` stream analysis), validating optimal runtime performance and accurate agent behaviors. 

### Template: `test-agent-capability.sh`
```bash
#!/bin/bash
set -e # Abort on any single sdb shell connection break

DAEMON_SERVICE="tizenclaw"
echo "Running autonomous runtime integration checks..."

# Test 1: Verify daemon systemctl status & steady state memory usage
STATUS_OUTPUT=$(sdb shell "systemctl status $DAEMON_SERVICE" || true)
if ! echo "$STATUS_OUTPUT" | grep -q "active (running)"; then
    echo "[Error] TizenClaw service crashed or failed to initialize!"
    exit 1
}

# Test 2: Stimulate autonomous module & observe asynchronous output
# For example, sending an IPC message or calling a dummy D-Bus method
TRIGGER_RES=$(sdb shell "your-trigger-command-here")
sleep 2 # Allow async tasks to dispatch and log events

# Test 3: Validate the target device logs
if ! sdb shell "dlogutil | grep TIZENCLAW | grep -q 'Expected Goal Output'"; then
    echo "[Error] Autonomous capability did not execute correctly based on log artifacts."
    exit 1
fi
```

## 3. Automation Pipeline
The overarching `tests/test_all.sh` script automates these background capability layers sequentially. Developer sub-agents must formulate compliant integration scripts whenever new perception or logic boundaries are added to the agent, guaranteeing ultimate stability against memory leaks or segmentation faults.
//turbo-all
