# Uncommitted Formatting Cleanup Planning

## Goal
Review the Rust source diffs left outside commit `a18b06bb`, confirm
whether they are intentional formatting cleanups, and deliver them in a
separate validated commit.

## Scope
- `src/tizenclaw/src/core/task_scheduler.rs`
- `src/tizenclaw/src/generic/infra/container_engine.rs`
- `src/tizenclaw/src/llm/gemini.rs`

## Capability Classification
- Formatting cleanup review: One-shot Worker
- Build and deployment validation via `./deploy.sh -a x86_64`:
  One-shot Worker
- Post-deploy runtime smoke verification: One-shot Worker

## Inputs
- The current uncommitted Git diff
- Existing Rust formatting conventions already present in the codebase
- QEMU/device validation results produced by `./deploy.sh`

## Expected Output
- A yes/no decision on whether the remaining diffs should be kept
- A validated commit containing only the intended formatting changes
- Dashboard evidence showing deploy and runtime verification

## Constraints
- No local `cargo build`, `cargo test`, `cargo check`, or `cargo clippy`
- No runtime behavior expansion or FFI boundary change
- Use the normal x86_64 deployment path only
