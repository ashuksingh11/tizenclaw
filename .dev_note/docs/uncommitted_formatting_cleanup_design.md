# Uncommitted Formatting Cleanup Design

## Design Summary
This cycle preserves the currently uncommitted Rust diffs only if they
remain formatting-only refactors. The implementation must not alter task
scheduler logic, container execution behavior, or Gemini request
construction.

## Structural Rules
- Keep all existing function signatures, return types, and control flow
  unchanged.
- Restrict the diff to line wrapping, indentation, and expression layout.
- Do not introduce new FFI boundaries or modify existing ones.
- Preserve the current `Send + Sync` ownership behavior because no lock
  type, async boundary, or shared-state contract is being changed.

## Runtime Boundary Notes
- `task_scheduler.rs` keeps the existing filesystem and scheduling flow
  intact while only normalizing expression layout.
- `container_engine.rs` keeps the same Tokio command execution path and
  JSON result contract.
- `gemini.rs` keeps the same request filtering logic for empty parts.

## Dynamic Loading Strategy
The current `libloading` strategy remains unchanged because this cycle
does not touch dynamic symbol acquisition or Tizen `.so` integration.

## Validation Plan
- Confirm the diff is formatting-only through Git review.
- Run `./deploy.sh -a x86_64` to validate compile and deployment health.
- Inspect runtime service state and recent journal logs before commit.
