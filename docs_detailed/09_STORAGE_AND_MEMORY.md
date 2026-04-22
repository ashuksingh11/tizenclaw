# 09 — Storage & Memory (Schema Reference)

This document is the reference for TizenClaw's storage subsystems.

> **Major change in April 2026**: Session transcripts moved from a SQLite `messages` table to a file-based layout (Markdown + JSONL per session directory). This enables atomic writes, crash safety on flash storage, and simpler compaction. See [11_MEMORY_SESSION_DEEPDIVE.md](11_MEMORY_SESSION_DEEPDIVE.md) for the flow.

## Storage Subsystems

| Store | Backing | Integration | File / Module |
|---|---|---|---|
| `SessionStore` | Files + SQLite index | ✅ Integrated | `storage/session_store.rs` |
| `MemoryStore` | SQLite + ONNX models | ✅ Integrated (wired April 2026) | `storage/memory_store.rs` |
| `EmbeddingStore` | (absorbed into MemoryStore) | Not a standalone field | — |
| `AuditLogger` | SQLite | ⚠️ Partial usage | `storage/audit_logger.rs` |

All SQLite databases use WAL mode + `PRAGMA synchronous=NORMAL` for write throughput on embedded flash.

## 1. SessionStore — File-Based Transcripts ✅

File: `src/tizenclaw/src/storage/session_store.rs` (~1,979 lines)

### 1.1 Struct (lines 143-147)
```rust
pub struct SessionStore {
    base_dir: PathBuf,
    db: Arc<Mutex<rusqlite::Connection>>,
    lock: Arc<RwLock<()>>,  // path-level locking for atomic writes
}
```

### 1.2 Directory layout per session
```
{base_dir}/{session_id}/
├── {YYYY-MM-DD}.md       # Daily Markdown transcript (append-only)
├── transcript.jsonl      # Structured event log (append-only)
├── compacted.md          # Compaction snapshot (Markdown, replaceable)
└── compacted.jsonl       # Compaction snapshot (structured)
```

### 1.3 SQLite tables (session_store.rs:298-312)
Only two tables — messages are NOT in SQL anymore:

```sql
CREATE TABLE session_index (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL
);

CREATE TABLE token_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    date TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    cache_creation_input_tokens INTEGER DEFAULT 0,   -- Anthropic prompt cache
    cache_read_input_tokens INTEGER DEFAULT 0        -- Anthropic prompt cache
);
```

New cache token columns track Anthropic's prompt-caching feature for cost analytics. Migration via `ensure_token_usage_columns()` uses `PRAGMA table_info` for backward-compat ALTER TABLE.

### 1.4 Key methods
- **Session lifecycle**: `new(base_dir, db_path)`, `session_workdir(session_id)`, `ensure_session(session_id)`, `list_sessions()`, `clear_session(session_id)`, `clear_all()`
- **Message append** (writes to today's `.md` file):
  - `append_message(session_id, role, text)` — legacy append
  - `add_structured_user_message(session_id, content)` — modern structured event
  - `add_structured_assistant_text_message(session_id, text)`
  - `add_structured_tool_call_message(session_id, tool_calls)`
  - `add_structured_tool_result_message(session_id, tool_id, tool_name, result)`
- **History load**: `load_session_context(session_id, limit: usize) -> (Vec<SessionMessage>, bool)` — returns tail-limited messages + `from_compacted` flag
- **Compaction**: `load_compacted`, `save_compacted`, `load_compacted_structured`, `save_compacted_structured` — atomic replace of `compacted.md` via `.tmp` → `rename()` + `sync_all`
- **Token tracking**: `record_usage`, `load_token_usage`, `load_daily_usage`
- **Legacy compat**: `add_message(session_id, role, content)`, `get_messages(session_id, limit)`

### 1.5 History merge algorithm (session_store.rs:520-569)
1. Load `compacted.md` (or `.jsonl`) if present → base message list
2. Load today's `{date}.md` (or `transcript.jsonl`) → incremental messages
3. Deduplicate via `deduplicate_after_compacted()` (hash on role + preview)
4. Tail-limit to `limit` messages
5. Return `(messages, from_compacted)`

### 1.6 Concurrency
File locking via `Arc<RwLock<()>>`. Read lock for loads, write lock for appends. SQLite connection is serialized via `Arc<Mutex<Connection>>`. No in-memory message caching — disk is source of truth.

## 2. MemoryStore — Long-Term Facts ✅

File: `src/tizenclaw/src/storage/memory_store.rs`

### 2.1 Instantiation (runtime_core_impl.rs:1151-1167)
```rust
let mem_dir = paths.data_dir.join("memory");
let mem_db = mem_dir.join("memories.db");
let model_dir = paths.data_dir.join("models");
MemoryStore::new(mem_dir, mem_db, model_dir)
```

### 2.2 Integration in process_prompt (process_prompt.rs:443-463)
```rust
if literal_json_output || should_skip_memory_for_prompt(prompt) {
    loop_state.record_prefetch_memory(None);
} else if let Ok(ms) = self.memory_store.lock() {
    if let Some(store) = ms.as_ref() {
        let mem_str = store.load_relevant_for_prompt(prompt, 5, 0.1);
        if !mem_str.is_empty() {
            let memory_context = format!(
                "## Context from Long-Term Memory\n<long_term_memory>\n{}\n</long_term_memory>",
                mem_str
            );
            // injected into messages
        }
    }
}
```

### 2.3 Schema
```sql
CREATE TABLE memories (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT DEFAULT 'general',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
CREATE INDEX idx_mem_category ON memories(category);
```

Embeddings are handled internally by MemoryStore using ONNX models from `data_dir/models/` (absorbing the previous separate `EmbeddingStore` concept).

### 2.4 Key API
- `load_relevant_for_prompt(prompt, top_k: usize, threshold: f32) -> String` — returns formatted string for prompt injection
- `set(key, value, category)`
- `get(key)`
- `get_by_category(category, limit)`
- `delete(key)`

## 3. AuditLogger ⚠️

File: `src/tizenclaw/src/storage/audit_logger.rs`

Partial usage — not wired to all code paths. Records `ipc_auth`, `tool_exec`, `llm_call` events.

## 4. Filesystem Layout

```
{data_dir}/
├── sessions/                       # SessionStore base_dir
│   ├── session_index.db            # SQLite: metadata + token_usage
│   ├── default/                    # per-session dirs
│   │   ├── 2026-04-22.md
│   │   ├── transcript.jsonl
│   │   ├── compacted.md
│   │   └── compacted.jsonl
│   └── user-telegram-12345/
│       └── ...
├── memory/
│   ├── memories.db                 # MemoryStore SQL
│   └── ...
├── models/                         # ONNX embedding models
└── audit.db                        # AuditLogger (if enabled)
```

On Tizen: `{data_dir} = /opt/usr/share/tizenclaw/data/`
On Linux dev: `{data_dir} = $XDG_DATA_HOME/tizenclaw/`

## 5. Backup & Migration

### Backup
- Stop daemon → copy `{data_dir}` directory → restart. Atomic.
- For live backup: SQLite files + transcript.jsonl can be copied while daemon runs (reads use shared lock).
- Markdown files are append-only; copy is always safe.

### Schema migration
- `token_usage` table has `ensure_token_usage_columns()` helper that does `PRAGMA table_info` + conditional ALTER TABLE. Models the pattern for future additive migrations.
- File-based transcripts have no schema — free-form append.

## 6. How Message Flow Differs From Pre-April-2026

| Aspect | Pre-merge (SQL) | Post-merge (files) |
|---|---|---|
| Message storage | `messages` table rows | `.md` + `.jsonl` files per session dir |
| History load | `SELECT ... ORDER BY id DESC LIMIT N` | Parse today's file + compacted, dedup, tail-limit |
| Windowing constant | `MAX_CONTEXT_MESSAGES = 20` | `MAX_CONTEXT_MESSAGES = 120`, plus `SizedContextEngine` token-aware compaction |
| Tool call in history | flat content string | structured `LlmToolCall` field on `SessionMessage` |
| Crash safety | SQLite atomicity | `.tmp` → rename + `sync_all` for compaction snapshots |
| Compaction | none | `compacted.md` + `compacted.jsonl` snapshots |

## See Also

- **[11_MEMORY_SESSION_DEEPDIVE.md](11_MEMORY_SESSION_DEEPDIVE.md)** — full flow: session creation → history load → memory retrieval → prompt build
- **[13_SAFETY_AND_POLICY.md](13_SAFETY_AND_POLICY.md)** — audit logger's role

## FAQ

**Q: Why move messages out of SQLite?**
A: Append-only markdown + JSONL is crash-safe on flash (no torn writes), human-readable for debugging, and simplifies compaction — just rewrite the compacted file, don't DELETE/INSERT rows. SQL retains what benefits from indexed queries: session index and token analytics.

**Q: Can I still query the full conversation in SQL?**
A: No — you'd need to parse the `.md` or `.jsonl` files. For simple queries, `grep` works on markdown transcripts. For structured queries, parse `transcript.jsonl` line-by-line.

**Q: What if the daemon crashes mid-write?**
A: Daily `.md` append is a single write call, so either the line is there or it isn't — no partial state. Compaction uses `.tmp` + rename, so the old `compacted.md` remains valid until the new one is fully fsync'd to disk.

**Q: How does load_session_context deduplicate overlapping compacted + recent messages?**
A: `deduplicate_after_compacted()` at session_store.rs:1299-1315 hashes `role + preview_text` of each message and drops duplicates. Preview text is the first N characters.

**Q: Does MemoryStore actually do semantic search now?**
A: Yes — `load_relevant_for_prompt(prompt, top_k, threshold)` computes an embedding via ONNX (models from `data_dir/models/`), searches with cosine similarity, filters by threshold, and returns top_k results formatted for prompt injection.

**Q: What's the `from_compacted` boolean for?**
A: Tells the caller that the message list starts from a compaction snapshot, not from the original conversation start. Useful for UI distinction ("you're seeing a compacted history") and to avoid confusing the LLM about provenance.

**Q: Where do token_usage records go?**
A: SQLite `token_usage` table in `session_index.db` (same file as `session_index` table). Queryable via `load_token_usage(session_id)` and `load_daily_usage(date)`.
