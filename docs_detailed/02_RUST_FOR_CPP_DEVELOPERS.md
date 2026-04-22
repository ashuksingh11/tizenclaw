# Rust for C++ Developers

This is not a generic Rust tutorial. It covers the specific Rust patterns used in TizenClaw, explained through C/C++ analogies. Every example references actual source code in this repository.


## 1. Ownership and Borrowing

### The mental model

If you know C++ RAII, you already understand Rust ownership at a conceptual level:

| C++ | Rust | What it means |
|---|---|---|
| `std::unique_ptr<T>` | `T` (owned value) | Exactly one owner. Destroyed when owner goes out of scope. |
| `const T*` or `const T&` | `&T` | Shared, read-only borrow. Multiple allowed simultaneously. |
| `T*` or `T&` | `&mut T` | Exclusive, mutable borrow. Only one at a time. |
| `std::move(x)` | `x` (moved by default) | Ownership transfers. Original variable is invalidated. |

The key difference: C++ trusts you to get this right. Rust **enforces** it at compile time. You cannot have a `&mut T` and a `&T` to the same data simultaneously. You cannot use a value after it has been moved.

### How TizenClaw uses ownership

Look at `AgentCore::new()` in `src/tizenclaw/src/core/agent_core.rs:111`:

```rust
pub fn new(platform: Arc<libtizenclaw::PlatformContext>) -> Self {
    AgentCore {
        platform,                                          // owned Arc (shared pointer)
        backend: tokio::sync::RwLock::new(None),           // owned RwLock
        fallback_backends: tokio::sync::RwLock::new(Vec::new()),
        session_store: Mutex::new(None),                   // owned Mutex
        tool_dispatcher: tokio::sync::RwLock::new(ToolDispatcher::new()),
        key_store: Mutex::new(KeyStore::new()),
        system_prompt: RwLock::new(String::new()),
        // ...
    }
}
```

Each field is **owned** by `AgentCore`. When `AgentCore` is dropped, every field is dropped in declaration order -- no explicit destructor needed, no `delete`, no `release()`. This is exactly like a C++ struct where every member is a `unique_ptr` or value type, except the compiler guarantees you did not accidentally alias any of them.

### The borrow checker in practice

In C++, this compiles and crashes at runtime:

```cpp
std::vector<int> v = {1, 2, 3};
int& ref = v[0];
v.push_back(4);   // may reallocate -- ref is now dangling
std::cout << ref;  // undefined behavior
```

In Rust, the equivalent is a compile error:

```rust
let mut v = vec![1, 2, 3];
let r = &v[0];     // immutable borrow of v
v.push(4);          // ERROR: cannot borrow v as mutable while r exists
println!("{}", r);
```

This is not theoretical. TizenClaw processes untrusted LLM responses, constructs message vectors, and passes references into async tasks. The borrow checker prevents an entire class of bugs that would be subtle data races or use-after-free in C++.


## 2. Arc and Mutex vs shared_ptr and mutex

### Arc = atomic reference-counted pointer

In `src/tizenclaw/src/main.rs:61`:

```rust
let agent = Arc::new(agent);  // like make_shared<AgentCore>(...)
let agent_clone = agent.clone();  // increments refcount, not a deep copy
```

C++ equivalent:

```cpp
auto agent = std::make_shared<AgentCore>(std::move(agent_raw));
auto agent_clone = agent;  // refcount++
```

`Arc` stands for **A**tomic **R**eference **C**ount. It is `std::shared_ptr` but with a twist: Rust requires the inner type to be `Send + Sync` to share across threads.

### Send and Sync -- what C++ does not have

These are **marker traits** (zero-cost, no methods) that tell the compiler:

| Trait | Meaning | C++ equivalent |
|---|---|---|
| `Send` | This type can be **moved** to another thread | (no equivalent -- you just hope for the best) |
| `Sync` | This type can be **referenced** from multiple threads | (no equivalent -- you wrap everything in a mutex and pray) |

`Arc<T>` is `Send + Sync` only if `T` is `Send + Sync`. This is why `AgentCore` fields use `tokio::sync::RwLock` and `std::sync::Mutex` -- they make their contents `Sync`. Without the lock wrappers, the compiler would refuse to put `AgentCore` inside an `Arc`.

### Fine-grained locking

In C++, a common pattern is one big mutex for the whole object:

```cpp
class AgentCore {
    std::mutex mtx_;
    // every method does: std::lock_guard lock(mtx_);
};
```

TizenClaw uses **per-field locking** instead (`src/tizenclaw/src/core/agent_core.rs:96-108`):

```rust
pub struct AgentCore {
    platform: Arc<libtizenclaw::PlatformContext>,               // immutable after init -- no lock
    backend: tokio::sync::RwLock<Option<Box<dyn LlmBackend>>>,  // async RwLock
    fallback_backends: tokio::sync::RwLock<Vec<Box<dyn LlmBackend>>>,
    session_store: Mutex<Option<SessionStore>>,                  // std Mutex (SQLite is !Sync)
    tool_dispatcher: tokio::sync::RwLock<ToolDispatcher>,        // reads frequent, writes rare
    key_store: Mutex<KeyStore>,
    system_prompt: RwLock<String>,                               // std RwLock (sync reads)
    // ...
}
```

Why two kinds of locks?

| Lock type | When to use | C++ analogy |
|---|---|---|
| `tokio::sync::RwLock` | In async code (`await`-compatible). Yields the task while waiting. | `std::shared_mutex` + coroutine integration |
| `std::sync::Mutex` | For non-async operations or types that are not `Sync` (like SQLite). Blocks the OS thread. | `std::mutex` |
| `std::sync::RwLock` | For sync code where reads vastly outnumber writes. | `std::shared_mutex` |

The `session_store` uses `std::sync::Mutex` specifically because `rusqlite::Connection` is not `Sync` (SQLite's threading model does not permit shared references), so it cannot go inside a `tokio::sync::RwLock`.


## 3. Traits vs Virtual Classes

### Basic traits

A Rust `trait` is like a C++ pure virtual base class (interface). The `LlmBackend` trait in `src/tizenclaw/src/llm/backend.rs:94-103`:

```rust
#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    fn initialize(&mut self, config: &Value) -> bool;
    async fn chat(
        &self, messages: &[LlmMessage], tools: &[LlmToolDecl],
        on_chunk: Option<&(dyn Fn(&str) + Send + Sync)>, system_prompt: &str,
    ) -> LlmResponse;
    fn get_name(&self) -> &str;
    fn shutdown(&mut self) {}  // default implementation -- like a non-pure virtual
}
```

C++ equivalent:

```cpp
class ILlmBackend {
public:
    virtual bool initialize(const json& config) = 0;
    virtual LlmResponse chat(const vector<LlmMessage>& messages,
                             const vector<LlmToolDecl>& tools,
                             function<void(string_view)> on_chunk,
                             string_view system_prompt) = 0;
    virtual string_view get_name() const = 0;
    virtual void shutdown() {}  // non-pure virtual with default
    virtual ~ILlmBackend() = default;
};
```

Notable differences:

- `: Send + Sync` -- the trait requires implementors to be thread-safe. No C++ equivalent.
- `&self` / `&mut self` -- explicit receiver, like a const/non-const method.
- `shutdown()` has a default body `{}` -- exactly like a non-pure virtual in C++.
- `#[async_trait::async_trait]` -- a macro that transforms `async fn` into a trait-compatible form (Rust traits do not natively support async methods yet).

### The Channel trait

Another real example from `src/tizenclaw/src/channel/mod.rs:23-29`:

```rust
pub trait Channel: Send {
    fn name(&self) -> &str;
    fn start(&mut self) -> bool;
    fn stop(&mut self);
    fn is_running(&self) -> bool;
    fn send_message(&self, text: &str) -> Result<(), String>;
}
```

This is the interface for all communication channels (web dashboard, Telegram, Discord, etc.). Note `: Send` -- channels can be moved to another thread but do not need to be shared (`Sync` is not required because `ChannelRegistry` owns them behind `Box<dyn Channel>`).

### Dynamic dispatch: Box<dyn Trait>

| Rust | C++ | Semantics |
|---|---|---|
| `Box<dyn LlmBackend>` | `std::unique_ptr<ILlmBackend>` | Heap-allocated, owned, vtable dispatch |
| `&dyn LlmBackend` | `const ILlmBackend&` | Borrowed reference, vtable dispatch |
| `Arc<dyn LlmBackend>` | `std::shared_ptr<ILlmBackend>` | Shared ownership, vtable dispatch |

In TizenClaw, backends are stored as `Option<Box<dyn LlmBackend>>` -- an optional, uniquely-owned, dynamically-dispatched backend. Channels are stored as `Vec<Box<dyn Channel>>` in the `ChannelRegistry`.


## 4. Enums and Pattern Matching vs Tagged Unions

### Rust enums are discriminated unions

Rust enums are far more powerful than C/C++ enums. They are closer to `std::variant` but with exhaustive pattern matching enforced by the compiler.

C++ approach:

```cpp
std::unique_ptr<ILlmBackend> create_backend(std::string_view name) {
    if (name == "gemini") return std::make_unique<GeminiBackend>();
    if (name == "openai" || name == "xai") return std::make_unique<OpenAiBackend>(name);
    if (name == "anthropic") return std::make_unique<AnthropicBackend>();
    if (name == "ollama") return std::make_unique<OllamaBackend>();
    return nullptr;
}
```

Rust version from `src/tizenclaw/src/llm/backend.rs:106-114`:

```rust
pub fn create_backend(name: &str) -> Option<Box<dyn LlmBackend>> {
    match name {
        "gemini" => Some(Box::new(super::gemini::GeminiBackend::new())),
        "openai" | "xai" => Some(Box::new(super::openai::OpenAiBackend::new(name))),
        "anthropic" => Some(Box::new(super::anthropic::AnthropicBackend::new())),
        "ollama" => Some(Box::new(super::ollama::OllamaBackend::new())),
        _ => None,
    }
}
```

The `match` expression is exhaustive -- the `_ => None` arm is the catch-all (like `default:` in a `switch`). If you remove it, the code will not compile. This prevents the classic C++ bug of adding a new enum variant and forgetting to handle it in a switch.

### Option and Some/None

`Option<T>` is Rust's replacement for nullable pointers:

| Rust | C++ | Meaning |
|---|---|---|
| `Some(value)` | `value` (non-null pointer) | A value is present |
| `None` | `nullptr` | No value |
| `Option<Box<dyn LlmBackend>>` | `std::unique_ptr<ILlmBackend>` (nullable) | May or may not hold a backend |

The compiler forces you to check for `None` before using the value. No null pointer dereferences.


## 5. Error Handling: Result vs errno/exceptions

Rust has no exceptions. Error handling uses the `Result<T, E>` type:

```rust
enum Result<T, E> {
    Ok(T),    // success
    Err(E),   // failure
}
```

### The ? operator

The `?` operator is syntactic sugar for "return early on error." Compare:

C++ (exceptions):
```cpp
SessionStore store = SessionStore::create(path);  // throws on failure
store.add_message(id, role, text);                 // throws on failure
```

C++ (error codes):
```cpp
auto result = SessionStore::create(path);
if (!result.ok()) return result.error();
auto& store = result.value();
```

Rust:
```rust
let store = SessionStore::new(path)?;  // returns Err(...) if it fails
store.add_message(id, role, text);
```

### How TizenClaw handles errors

TizenClaw uses `Result<T, String>` in many places -- the error type is a human-readable string. Look at the `Channel` trait:

```rust
fn send_message(&self, text: &str) -> Result<(), String>;
```

And at `SessionStore::new()` in `src/tizenclaw/src/storage/session_store.rs`:

```rust
match SessionStore::new(&db_path.to_string_lossy()) {
    Ok(store) => {
        log::info!("Session store initialized");
        if let Ok(mut ss) = self.session_store.lock() {
            *ss = Some(store);
        }
    }
    Err(e) => log::error!("Session store failed: {}", e),
}
```

This is equivalent to a try/catch in C++, but the compiler forces you to handle the error. You cannot accidentally ignore a `Result` (the compiler emits a warning for unused `Result` values).

### unwrap() and expect()

You will see `.unwrap()` and `.expect("msg")` in some places. These are like `assert()` -- they panic (crash) if the `Result` is `Err` or `Option` is `None`. In production code, they indicate "this should never fail; if it does, something is catastrophically wrong."

```rust
// Safe: this config file should always parse
let json: Value = serde_json::from_str(&content).unwrap();

// Better: with context on what went wrong
let json: Value = serde_json::from_str(&content)
    .expect("Failed to parse llm_config.json");
```


## 6. Async/Await vs Callbacks and Thread Pools

### The Tokio runtime

TizenClaw uses `#[tokio::main]` to launch an async runtime. From `src/tizenclaw/src/main.rs:30-31`:

```rust
#[tokio::main]
async fn main() {
```

This is equivalent to:

```cpp
int main() {
    // Create a thread pool with worker threads
    auto runtime = tokio::Runtime::new_multi_thread();
    runtime.block_on(async_main());
}
```

The `#[tokio::main]` macro expands to creating a multi-threaded Tokio runtime and blocking on the `async main()` function. Under the hood, Tokio manages a pool of OS threads and schedules thousands of lightweight tasks (green threads) onto them.

### async fn and .await

An `async fn` returns a `Future` -- similar to `std::future` in C++ but with zero heap allocation (the state machine is stack-allocated):

```rust
// This does NOT execute chat() -- it creates a Future
let future = backend.chat(messages, tools, None, &system_prompt);

// This drives the Future to completion, yielding the thread while waiting
let response = future.await;
```

C++ analogy (approximately):

```cpp
// C++20 coroutine
co_await backend.chat(messages, tools, nullptr, system_prompt);
```

### tokio::sync::RwLock vs std::shared_mutex

Standard `std::sync::RwLock` blocks the OS thread when contended. In async code, this wastes a Tokio worker thread. `tokio::sync::RwLock` instead **yields** the task back to the scheduler:

```rust
// This yields the async task if the lock is held, not the OS thread
let be_guard = self.backend.read().await;
```

C++ equivalent (if C++ had coroutine-aware mutexes):

```cpp
auto be_guard = co_await self.backend.read();  // hypothetical
```

### Parallel tool execution

TizenClaw executes multiple tool calls in parallel using `futures_util::future::join_all`. From `src/tizenclaw/src/core/agent_core.rs:451-461`:

```rust
let mut futures_list = Vec::new();
for tc in response.tool_calls.iter() {
    futures_list.push(async {
        let result = td_guard.execute(&tc.name, &tc.args).await;
        LlmMessage::tool_result(&tc.id, &tc.name, result)
    });
}
let results = futures_util::future::join_all(futures_list).await;
```

C++ equivalent:

```cpp
std::vector<std::future<LlmMessage>> futures;
for (auto& tc : response.tool_calls) {
    futures.push_back(std::async(std::launch::async, [&] {
        auto result = td.execute(tc.name, tc.args);
        return LlmMessage::tool_result(tc.id, tc.name, result);
    }));
}
for (auto& f : futures) results.push_back(f.get());
```

The Rust version is more efficient: `join_all` runs all futures concurrently on the same Tokio task, not on separate OS threads. If any tool call does I/O (HTTP request, subprocess), the runtime interleaves their execution.


## 7. FFI: Crossing the Rust-C Boundary

### extern "C" functions

Rust can export and import C functions. The signal handler in `src/tizenclaw/src/main.rs:26-28`:

```rust
extern "C" fn signal_handler(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}
```

This is exactly like writing:

```c
void signal_handler(int sig) {
    running = false;
}
```

The `extern "C"` tells Rust to use the C calling convention (no name mangling, standard ABI).

### Raw libc calls

The IPC server uses raw `libc` calls for socket operations (`src/tizenclaw/src/core/ipc_server.rs:52-95`):

```rust
unsafe {
    let fd = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0);
    let mut addr: libc::sockaddr_un = std::mem::zeroed();
    addr.sun_family = libc::AF_UNIX as u16;
    // ...
    libc::bind(fd, &addr as *const _ as *const libc::sockaddr, addr_len);
    libc::listen(fd, 5);
}
```

The `unsafe` block is required because raw pointer operations and FFI calls bypass Rust's safety guarantees. Every `unsafe` block in TizenClaw is a deliberate decision and should be reviewed carefully. The `libc` crate provides type-safe wrappers around POSIX constants and function signatures.

### The C-ABI client library

`libtizenclaw-client` (`src/libtizenclaw-client/src/lib.rs`) exposes Rust functionality to C/C++ callers:

```rust
#[no_mangle]
pub extern "C" fn tizenclaw_create() -> *mut TizenClawHandle {
    // ...
}
```

Key patterns:

| Rust pattern | C equivalent | Purpose |
|---|---|---|
| `#[no_mangle]` | (default in C) | Prevent Rust name mangling so C can find the symbol |
| `extern "C"` | (default in C) | Use C calling convention |
| `*mut T` / `*const T` | `T*` / `const T*` | Raw pointers for cross-language data passing |
| `CStr::from_ptr(ptr)` | (just use the pointer) | Safe conversion from C string to Rust string |
| `CString::new(s)` | `strdup(s)` | Create a C-compatible null-terminated string |

Thread safety in the C client library is handled by wrapping the internal state in `Arc<Mutex<...>>`, so multiple C threads can call into the same handle safely.

### Error codes

The C client library defines error codes matching a C header (`tizenclaw.h`):

```rust
const TIZENCLAW_ERROR_NONE: i32 = 0;
const TIZENCLAW_ERROR_INVALID_PARAMETER: i32 = -1;
const TIZENCLAW_ERROR_OUT_OF_MEMORY: i32 = -2;
const TIZENCLAW_ERROR_NOT_INITIALIZED: i32 = -3;
const TIZENCLAW_ERROR_LLM_FAILED: i32 = -6;
```

This is the standard Tizen error code pattern, familiar to any Tizen C developer.


## 8. Cargo vs CMake

### Workspace structure

TizenClaw uses a Cargo **workspace** -- analogous to a CMake project with multiple subdirectories. The root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "src/libtizenclaw",
    "src/tizenclaw",
    "src/tizenclaw-cli",
    "src/libtizenclaw-client",
    "src/libtizenclaw-sdk",
    "src/tizenclaw-tool-executor",
]
exclude = [
    "src/tizen-sys",
    "src/tizenclaw-core",
]
```

CMake equivalent:

```cmake
project(tizenclaw)
add_subdirectory(src/libtizenclaw)
add_subdirectory(src/tizenclaw)
add_subdirectory(src/tizenclaw-cli)
# ...
```

### Dependencies

In CMake, you manage dependencies with `find_package()`, `pkg-config`, or vendored submodules. In Cargo, dependencies are declared in `Cargo.toml` and automatically downloaded/compiled:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "time", "macros"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
```

The `features` field is like CMake `option()` flags. `rusqlite`'s `bundled` feature compiles SQLite from source (like vendoring), so there is no system dependency on `libsqlite3-dev`.

### Features and conditional compilation

Cargo features replace `#ifdef` / `-DENABLE_FOO`:

```toml
[features]
mock-sys = []   # Enable mock system calls for testing
```

In code:

```rust
#[cfg(feature = "mock-sys")]
fn get_battery() -> u32 { 75 }  // fake value for tests

#[cfg(not(feature = "mock-sys"))]
fn get_battery() -> u32 { /* real implementation */ }
```

CMake equivalent:

```cmake
option(MOCK_SYS "Enable mock system calls" OFF)
```
```cpp
#ifdef MOCK_SYS
int get_battery() { return 75; }
#else
int get_battery() { /* real */ }
#endif
```

### Release profile

The workspace `Cargo.toml` configures release builds:

```toml
[profile.release]
opt-level = "s"    # Optimize for size (like -Os)
panic = "abort"    # No stack unwinding (like -fno-exceptions)
# lto = true       # Disabled: causes issues with ARM cross-compilation
```

This is equivalent to CMake:

```cmake
set(CMAKE_CXX_FLAGS_RELEASE "-Os -fno-exceptions")
# set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)  # LTO -- disabled
```

### Building

| Task | Cargo | CMake |
|---|---|---|
| Debug build | `cargo build` | `cmake --build build` |
| Release build | `cargo build --release` | `cmake --build build --config Release` |
| Run tests | `cargo test` | `ctest` |
| Build one crate | `cargo build -p tizenclaw` | `cmake --build build --target tizenclaw` |
| Clean | `cargo clean` | `cmake --build build --target clean` |
| Check (no codegen) | `cargo check` | N/A |


## FAQ

**Q: Should I learn Rust before trying to extend TizenClaw?**
A: For tools/skills (Scenario 1 & 2 in **[15_EXTENDING_TIZENCLAW.md](15_EXTENDING_TIZENCLAW.md)**), no — they're language-agnostic (any executable + JSON manifest). For LLM plugins (Scenario 4), you need either C or Rust knowledge. For modifying the daemon internals (wiring SafetyGuard, adding MemoryStore integration), yes — you'll want familiarity with traits, async, and Arc/Mutex patterns.

**Q: What Rust features are most essential for understanding this codebase?**
A: In order: (1) ownership and borrowing, (2) `Result<T, E>` and `?`, (3) trait objects (`Box<dyn Trait>`), (4) `Arc<T>` for shared ownership, (5) `tokio::spawn` and async/await, (6) pattern matching on enums. Fancier features (lifetimes in generics, proc macros) appear but are rarer.

**Q: Does TizenClaw use `unsafe`?**
A: Yes, sparingly. Mostly for: raw `libc` socket/signal calls (ipc_server.rs, main.rs), FFI to C libraries (tizen-sys bindings), dlopen/dlsym plugin loading. All `unsafe` blocks are small and auditable.

**Q: How do I navigate the codebase quickly?**
A: Start with `src/tizenclaw/src/main.rs` (the 8-phase boot), then `core/agent_core.rs::process_prompt` (the heart). Use `rg <symbol>` to trace callers. Cargo.toml workspace shows module boundaries.

**Q: What's the hardest Rust concept in this codebase?**
A: Probably the mix of std and tokio sync primitives — why `std::Mutex<Option<SessionStore>>` sits alongside `tokio::sync::RwLock<Box<dyn LlmBackend>>`. See **[11_MEMORY_SESSION_DEEPDIVE.md](11_MEMORY_SESSION_DEEPDIVE.md)** section 9 for the rationale.
