# The Tool-Skill Separation (Two-Track Architecture)

TizenClaw's major superpower is its completely decoupled **Tool implementation** (Executable binaries) and **Skill definition** (Textual cognitive guardrails for the LLM). Before the Rust migration, C++ manifest YAML mappings loosely merged the two. The current specification enforces strict separation, strongly inspired by modern workflow specifications.

---

## 🚀 1. Executable Tools (The Actions)

Tools are hard-coded commands or scripts configured under `/opt/usr/share/tizen-tools/`.

TizenClaw uses a **ToolWatch** loop to reload standard scripts gracefully without disrupting main thread operations or `tokio` asynchronous IO loops. When an application developer desires a new binary execution path:

1. They deploy a binary to `/usr/bin/foo-tool`.
2. They allow `tizenclaw-tool-executor` daemon (or via native bindings in `AgentCore`) to call it.
3. The LLM is granted abstract function declarations via JSON schemas to utilize the tool via JSON-RPC.

**Built-in Preloaded Tools**:
- `execute_code`: Running arbitrary Python blobs in a sub-process.
- `execute_cli`: Abstract interaction gateway for system binary shelling.
- Memory handling (`remember`/`forget`).

---

## 🧠 2. Textual Skills (The Workflows)

Executing tools randomly is error-prone. To codify robust system automation and intelligence, TizenClaw deploys purely textual `SKILL.md` documents.

A `SKILL.md` acts as a complex procedural instruction that is transparently loaded into the LLM system prompt via `PromptBuilder`.

### The Textual Workflow Standard
A valid skill explicitly demands:
1. Being stored hierarchically within `/opt/usr/share/tizen-tools/skills/<skill_name>/SKILL.md`.
2. Ignoring loose top-level markdown nodes (like `cli tool.md`) to avoid false prompt payload injections.

### Writing a Skill

**Example: `skills/cleanup_cache/SKILL.md`**
```yaml
---
description: Clears all system memory caches to perform GC logic on the Tizen board.
---

When the user asks you to clear the memory, immediately run the `execute_cli` tool with the command `echo 3 > /proc/sys/vm/drop_caches`. 

Ensure you check the total memory footprint before and after returning the results back to the user to prove your workflow's success.
```

The background `TextualSkillScanner` continuously parses these sub-directory assets and loads the YAML Frontmatter description.
When a User prompt is evaluated, `AgentCore` renders all available workflows as explicit references (e.g., `<available_skills>`) to give the LLM cognitive pointers without executing physical bounds checks.

### Why This Design?

- **Hot Swapping API Intents**: You can literally alter the way TizenClaw behaves and responds by rewriting a `.md` skill via SSH—on the fly, in production—with zero daemon restarts (`sdb push`).
- **No Compilation Cycles**: Updating tool logic or business requirements no longer necessitates GBS C++ toolchain recompiles.
