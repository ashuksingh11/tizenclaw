# ONNX Dynamic Loading Architecture (RAG Bounds)

## 1. Overview
To mitigate extreme memory consumption on Tizen embedded targets, the ONNX Runtime will not be statically linked nor dynamically loaded at initialization.
Instead, the agent will load `libonnxruntime.so` on-demand via the `libloading` crate whenever a dense embedding vector needs to be computed for RAG (Retrieval-Augmented Generation) memory retrieval.

## 2. Dynamic Memory Lifecycles
```rust
pub struct DynamicOnnxEngine {
    // Isolated library handle. It automatically calls dlclose when dropped.
    library: libloading::Library,
    // Safely loaded C-FFI symbols...
}
```
1. **Load (dlopen)**: `DynamicOnnxEngine::new()` uses `Library::new("libonnxruntime.so")`. If this fails, the system safely falls back (returning an Option/Result) and simply ignores RAG context augmentation without crashing.
2. **Execute (Inference)**: Computes the 384-dimensional or 768-dimensional dense vector embeddings.
3. **Unload (dlclose)**: When `DynamicOnnxEngine` drops at the end of the `spawn_blocking` closure, memory is immediately released back to the OS.

## 3. Asynchronous Boundaries (Tokio)
Because ONNX inference is CPU-bound and locks up execution, invoking FFI functions directly within `AgentCore`'s asynchronous state machine will cause deadlocks or starve other tasks.
Therefore:
- Embedding computation MUST be wrapped in `tokio::task::spawn_blocking`.
- FFI pointers derived from `libloading` must be correctly marked or wrapped to satisfy `Send` boundaries if moved across threads, though optimally the `dlopen` and inference should be scoped entirely within the blocking thread payload.

## 4. Integration Point
Located at: `src/tizenclaw/src/infra/onnx_runtime.rs`
Dependencies: The `libloading` crate.
