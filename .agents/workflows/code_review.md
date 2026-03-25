---
description: Code Review Checklist and Review-Fix Loop (max 5 iterations)
---

# Code Review Workflow

This workflow defines a structured code review process for all code changes.
It must be performed after the Verify stage and before the Commit stage.

## Review Checklist

Review all changed source files against the following 10 categories in order.

### 1. Coding Style
- Verify compliance with rules defined in `coding_rules.md`
- Google C++ Style Guide adherence (2-space indent, PascalCase functions, snake_case variables, etc.)
- Member variable trailing underscore `_` usage
- 80-character column limit compliance
- `#include` group ordering and header guard format

### 2. Correctness
- Verify logic behaves as intended
- **Zero compiler warnings compliance**: All build warnings must be treated as errors and resolved before proceeding.
- Boundary condition handling
- Missing error/exception handling
- Return value validation (including `[[nodiscard]]` usage)
- Edge case handling (nullptr, empty containers, etc.)

### 3. Memory Issues
- Potential memory leaks
- Dangling pointer / Use-after-free
- Double-free
- Correct usage of `std::make_unique`/`std::shared_ptr`
- Avoid raw `new`/`delete`
- RAII pattern application

### 4. Performance
- Unnecessary object copies (pass-by-value vs. pass-by-const-reference)
- Inefficient loops (O(n²) or worse)
- Unnecessary repeated memory allocation/deallocation
- Lock contention and synchronization issues
- `constexpr` applicability

### 5. Logic Issues
- Dead code presence
- Unreachable branches
- Incorrect conditional expressions or incomplete branch handling
- Variable shadowing (`-Wshadow` violations)
- Deep nesting (3+ levels) — Guard Clause refactoring needed

### 6. Security
- Missing external input validation
- Potential buffer overflows
- Privilege escalation vulnerabilities
- Injection attack vectors (command injection, path traversal, etc.)
- Hardcoded credentials presence
- `tizen-manifest.xml` privilege configuration correctness

### 7. Thread Safety
- Race condition potential (concurrent access to shared resources)
- Deadlock risk (multi-lock acquisition order)
- GLib event loop callback thread safety
- Correct usage of `std::mutex`/`std::lock_guard`
- Plain variable usage where atomic operations are needed
- Object destruction before async operation completion

### 8. Resource Management
- File descriptor (fd) leaks
- Socket, D-Bus connection, and system resource release verification
- GLib resource management (`g_free`, `g_object_unref`, `g_variant_unref`, etc.)
- Container lifecycle (mount/unmount, cleanup tasks) omissions
- Exception safety when RAII pattern is not applied

### 9. Test Coverage
- gtest additions/modifications for changed code
- Unit test existence for new public functions
- Test coverage for boundary conditions and error paths
- Verify existing tests are not broken by changes

### 10. Error Propagation & Logging
- Appropriate `dlog_print` usage (DLOG_ERROR, DLOG_WARN, DLOG_INFO levels)
- Correct error propagation to callers
- Sufficient context information in log messages for debugging
- Silent failure detection — at minimum, errors must be logged
- Verify sensitive information is not exposed in logs

## Review-Fix Loop

When issues are found, follow the loop below. **Repeats up to 5 times maximum.**

```
┌─────────────────────────────────────────────────┐
│                  Review-Fix Loop                │
│                  (max 5 iterations)             │
│                                                 │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│   │ Develop  │───▶│  Verify  │───▶│  Review  │  │
│   │ (fix)    │    │ (build/  │    │ (check-  │  │
│   │          │    │  deploy/ │    │  list)   │  │
│   │          │    │  test)   │    │          │  │
│   └──────────┘    └──────────┘    └─────┬────┘  │
│        ▲                                │       │
│        │          FAIL                  │       │
│        └────────────────────────────────┘       │
│                                 │               │
│                            PASS │               │
│                                 ▼               │
│                          ┌──────────┐           │
│                          │  Commit  │           │
│                          └──────────┘           │
└─────────────────────────────────────────────────┘
```

### Procedure

1. Review all changed files against the 10-category checklist in order.
2. **PASS**: If no issues are found in any category, proceed to the Commit stage.
3. **FAIL**: If one or more issues are found:
   - Record the discovered issues clearly (category, filename, line number, description).
   - Return to the **Develop** stage to fix the issues.
   - Build/deploy via `deploy.sh`, then re-run the **Verify** stage.
   - Perform a **Review** again on the corrected code.
4. This loop repeats up to **5 times maximum**.
5. If issues remain after 5 iterations, **escalate** to the user for a decision.

> [!CAUTION]
> To prevent infinite loops, you must report to the user and request
> a decision on whether to proceed after exceeding 5 iterations.

### Review Result Recording Format

Record results for each iteration in the following format:

```
## Review Iteration N/5

| Category | File | Line | Severity | Description |
|----------|------|------|----------|-------------|
| Memory   | foo.cc | 42 | HIGH | potential use-after-free |
| Style    | bar.hh | 15 | LOW  | missing trailing underscore |

**Result**: FAIL → Returning to Develop step
```

Severity levels:
- **HIGH**: Must fix (memory issues, security vulnerabilities, crash-causing)
- **MEDIUM**: Should fix (performance, logic errors)
- **LOW**: Nice to fix (style, code cleanup)
