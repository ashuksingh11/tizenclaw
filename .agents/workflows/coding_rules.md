---
description: TizenClaw Coding Rules and Guidelines
---

# TizenClaw Agent Support Rules

When implementing TizenClaw in this repository, the Agent (AI) must **strictly** prioritize and adhere to the following coding styles and rules, which are derived directly from the project author's style guidelines.

## 1. C++ Coding Style
- **C++ Standard**: Use **C++20** (`-std=c++20`).
- **Style Guide**: Strictly follow the [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html).
- **Line Wrap & Formatting**:
  - **Column Limit**: Ensure all text in source code, comments, and header files is appropriately wrapped **not to exceed 80 characters (Column limit: 80)**.
  - **Indentation**: Use **2 spaces** (Space 2) strictly. Do not use tabs, and NEVER use 4 spaces.
  - **Brace Placement**: Use K&R/Stroustrup style. The opening brace `{` must be on the same line as the statement (e.g., `if (...) {`, `void Func() {`).
  - **Single-statement if/else**: If the block contains only a single statement, **omit the braces**. (e.g., `if (!exit_timer_) return;`)
  - **Access Modifiers**: `public:`, `private:`, `protected:` must be indented by **1 space** (` public:`).
  - **Namespaces**: Code inside a `namespace` block should **not** be indented.
  - **Pointers & References**: Attach `*` and `&` to the type, not the variable name. (e.g., `char** argv_`, `const std::string& package`)
- **Naming Conventions**:
  - **Class/Struct/Namespace**: `PascalCase` (e.g., `Watcherd`, `AgentCore`).
  - **Functions/Methods**: `PascalCase` (e.g., `HandlePkgmgrEventStart`, `SetExitTimer`).
  - **Local Variables/Parameters**: `snake_case` (e.g., `exit_timer`, `loader`).
  - **Member Variables**: `snake_case` with a single trailing underscore `_` (e.g., `argc_`, `exit_timer_`). **Never use the `m_` prefix.**
  - **Constants/Enums**: Prefix with a lowercase `k` followed by `PascalCase` (e.g., `kRegularUidMin`, `kMaxHistorySize`).
- **Includes & Headers**:
  - Use `#ifndef FILENAME_HH_` style header guards, matching the `.hh` extension.
  - Group includes logically: C system libs ➡️ C++ standard libs ➡️ Local project headers. Add a blank line between each group.
- **Modern C++ Features**:
  - Use `std::make_unique` and `std::shared_ptr` for resource management. Avoid raw `new`/`delete` where possible.
  - Use anonymous namespaces `namespace { ... }` in `.cc` files for internal linkage instead of `static`.
  - `[[nodiscard]]`: Apply to bool/state returning functions.
  - `std::filesystem`: Use instead of POSIX `opendir/readdir/stat`.
  - `map::contains()`: Use instead of `find() != end()`.
  - `std::ranges`: Prioritize range-based algorithms.
  - `using enum`: Apply for repeated enumeration use within scope.

## 2. CMake and Build Support
- Written targeting the Tizen GBS (Gerrit Build System) environment, `gbs build` must always succeed via CMake.
- When adding new C++ source files, you must update the `SOURCES` list in `CMakeLists.txt`.

## 3. Tizen-Specific Rules
- Features requiring privileges (Network, LXC execution, AppManager, etc.) must be explicitly stated in the `<privileges>` block of `tizen-manifest.xml`.
- Make full use of the dlog interface (`dlog_print`) to leave comprehensive system logs, and prioritize error handling via return codes or boolean returns over C++ exceptions whenever possible.
