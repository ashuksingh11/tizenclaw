# 15 — Extending TizenClaw

## Overview
Four ways to extend the system, from easiest to hardest:

| Extension | Complexity | Rebuild daemon? | Pattern |
|---|---|---|---|
| Textual Skill (SKILL.md) | ⭐ Easy | No (hot-reload) | Prompt-level guidance |
| Executable Tool (tool.md + binary) | ⭐⭐ Medium | No | New function for LLM to call |
| Agent Role | ⭐⭐ Medium | Restart only | New persona with constrained tools |
| LLM Backend plugin (.so) | ⭐⭐⭐ Hard | Rebuild plugin, not daemon | C-ABI dynamic provider |

## Scenario 1: Adding a Textual Skill ✅

Textual skills are markdown documents that teach the LLM how to compose existing tools. They're the lowest-cost extension point.

### Step 1: Create the directory structure
```
/opt/usr/share/tizen-tools/skills/clear_caches/SKILL.md
```

Must be inside a subdirectory; loose .md files in the root `skills/` are ignored (security measure in `textual_skill_scanner.rs:96-104`).

### Step 2: Write SKILL.md
```markdown
---
description: "Clear system memory caches when memory pressure is high"
---

# Clear Caches Skill

When the user asks to free up memory, or when memory monitoring shows >90% usage:

1. Confirm with the user that dropping caches is safe right now.
2. Call `execute_cli` with arg: `echo 3 > /proc/sys/vm/drop_caches` (requires root).
3. Wait 2 seconds, then call `execute_code` to re-check memory via `free -m`.
4. Report the before/after memory values.

Don't run this more than once per minute — caches rebuild naturally.
```

The `description:` in YAML frontmatter is what appears in the system prompt's `<available_skills>` section.

### Step 3: Push to the device
```bash
sdb push clear_caches /opt/usr/share/tizen-tools/skills/
```

No daemon restart needed. `ToolWatcher` (main.rs:65-72) polls the filesystem and triggers `agent.reload_tools()` within seconds.

### Step 4: Verify
```bash
tizenclaw-cli 'list available skills'
# The agent should mention clear_caches in its response.
```

### Step 5: Test it
```bash
tizenclaw-cli 'free up memory'
# The agent should discover the skill, read the SKILL.md file, and follow its instructions.
```

### Gotchas
- SKILL.md must be at exactly `<skill_name>/SKILL.md` — case-sensitive
- The YAML frontmatter must be delimited by `---` on its own lines
- Only the `description:` field is parsed — everything else in the frontmatter is ignored
- Markdown content isn't auto-injected into the system prompt. The LLM sees only the skill name + description in `<available_skills>`, then uses `file_manager` or `execute_code` to read the full .md body when relevant

## Scenario 2: Adding an Executable Tool ✅

An executable tool is a binary (or script) the LLM can invoke directly with structured arguments.

### Step 1: Write the binary

Example in C (could be Python, Bash, Rust — anything):
```c
// get_battery_level.c
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    FILE* f = fopen("/sys/class/power_supply/battery/capacity", "r");
    if (!f) { printf("{\"error\":\"no battery\"}"); return 1; }
    int level;
    fscanf(f, "%d", &level);
    fclose(f);
    printf("{\"level\":%d}", level);
    return 0;
}
```

Output JSON to stdout on success, non-zero exit code on failure.

### Step 2: Install the binary
```bash
sdb push get_battery_level /usr/bin/
sdb shell chmod +x /usr/bin/get_battery_level
```

### Step 3: Create the tool manifest
```
/opt/usr/share/tizen-tools/get_battery_level/tool.md
```

Content:
```
name: get_battery_level
description: "Read current battery percentage. Returns {level: N} on success."
binary: /usr/bin/get_battery_level
timeout: 5
```

Supported frontmatter fields (from `tool_dispatcher.rs:85-125`):
- `name:` — tool identifier (defaults to directory name)
- `description:` — what the LLM sees in function schemas
- `binary:` — absolute path to executable (defaults to `/usr/bin/<name>`)
- `timeout:` — seconds (default 30)

### Step 4: Parameter schema (optional)
Today `tool_dispatcher.rs:122` hardcodes parameters to `{"type":"object","properties":{"args":{"type":"string"}}}`. To provide richer schemas, you'd need to extend `parse_tool_md` to read a `parameters:` YAML block. Currently:
- LLM passes `{"args": "..."}` → split by whitespace → positional CLI args
- OR LLM passes `{"key":"value"}` object → converted to `--key value`

### Step 5: Verify
```bash
sdb push tool_manifest /opt/usr/share/tizen-tools/get_battery_level/
# ToolWatcher auto-reloads within ~5s
tizenclaw-cli 'how much battery is left?'
# Agent should call get_battery_level and report the result
```

### Step 6: Tag with safety metadata ⚠️
Currently `ToolDecl.side_effect` defaults to `"reversible"` (tool_dispatcher.rs:123). To mark a tool as irreversible, the parse_tool_md would need extending to read a `side_effect:` field. When SafetyGuard is wired up (⚠️ today), this tag will matter.

### Testing
Direct test: `/usr/bin/get_battery_level` → should emit `{"level":N}` on stdout.
End-to-end: invoke via `tizenclaw-cli` and check daemon logs for "Executing tool 'get_battery_level'".

## Scenario 3: Adding an Agent Role ⚠️

Agent roles give the LLM a specialized persona with restricted tools.

### ⚠️ Integration caveat
As of this doc, `AgentRoleRegistry::load_roles()` reads `config["roles"]` but `data/config/agent_roles.json` uses `"agents"` as its top-level key. Also, `AgentCore::process_prompt` doesn't actually swap system prompts per role today. So adding a role doesn't automatically produce runtime behavior change — you'd need to also wire the registry and factory. See 12_MULTI_AGENT_ORCHESTRATION.md for the full gap analysis.

### Step 1: Design the role
Decide:
- Name (snake_case, no spaces)
- Description (one sentence)
- System prompt (full persona instructions)
- Allowed tools (subset of registered tool names)
- Who can delegate to this role (edit orchestrator's `can_delegate_to`)

### Step 2: Add to agent_roles.json
Append to the `agents` array:
```json
{
  "name": "media_curator",
  "type": "worker",
  "description": "Finds and recommends media content based on user viewing history.",
  "system_prompt": "You are the TizenClaw Media Curator...",
  "tools": ["search_knowledge", "execute_code"],
  "auto_start": false
}
```

### Step 3: Add to orchestrator's delegation whitelist
In the orchestrator entry:
```json
"can_delegate_to": [
  "device_monitor",
  ...,
  "media_curator"
]
```

### Step 4: Deploy
```bash
sdb push agent_roles.json /opt/usr/share/tizenclaw/config/
sdb shell systemctl restart tizenclaw
```

Daemon restart is required — `agent_roles.json` isn't hot-reloaded.

### Step 5: Verify by asking the orchestrator
```bash
tizenclaw-cli 'recommend something to watch'
# Orchestrator should respond by calling `run_supervisor` with agent="media_curator"
# (Assuming run_supervisor tool is registered — another ⚠️ gap)
```

## Scenario 4: Adding an LLM Backend Plugin ⚠️

Custom LLM providers can be shipped as `.so` plugins loaded at runtime via `dlopen`.

Files:
- `src/tizenclaw/src/llm/plugin_llm_backend.rs` — Rust-side loader
- `src/tizenclaw/src/llm/plugin_manager.rs` — discovery & lifecycle
- `src/libtizenclaw-sdk/include/tizenclaw_llm_backend.h` — C ABI contract

### Step 1: Implement the C ABI
Required exported functions (in your `.so`):
```c
// Return provider info as JSON: {"name":"my_llm","version":"1.0.0"}
const char* llm_plugin_info(void);

// Initialize with config JSON. Return 0 on success, non-zero on error.
int llm_plugin_init(const char* config_json);

// Send a chat request; return response JSON. Caller will free via llm_plugin_free_string.
const char* llm_plugin_chat(const char* request_json);

// Return the backend name (used for selection in llm_config.json).
const char* llm_plugin_name(void);

// Free a string allocated by the plugin.
void llm_plugin_free_string(const char* s);

// Optional: cleanup on unload.
void llm_plugin_shutdown(void);
```

### Step 2: Build the plugin
```bash
# Compile as a shared object
gcc -shared -fPIC -o libmyllm.so my_llm.c -I/path/to/tizenclaw-sdk/include
# Or in Rust: cdylib crate-type
```

### Step 3: Deploy
```bash
sdb push libmyllm.so /opt/usr/share/tizenclaw/plugins/llm/
```

### Step 4: Register in llm_config.json
```json
{
  "active_backend": "my_llm",
  "fallback_backends": ["gemini"],
  "backends": {
    "my_llm": {
      "plugin_path": "/opt/usr/share/tizenclaw/plugins/llm/libmyllm.so",
      "api_key": "...",
      "custom_option": "value"
    }
  }
}
```

### Step 5: Restart the daemon
```bash
sdb shell systemctl restart tizenclaw
```

On boot:
- `AgentCore::initialize` → `create_backend("my_llm")` — falls through to the plugin_llm_backend loader
- Plugin manager does `dlopen` → resolves `llm_plugin_*` symbols → wraps in a `Box<dyn LlmBackend>` adapter
- Initialize is called with the config block → your plugin reads it

### Step 6: Verify
```bash
sdb shell journalctl -u tizenclaw -f
# Look for: "Primary LLM backend 'my_llm' initialized"
tizenclaw-cli 'hello'
# Agent should respond via your plugin
```

### Plugin debugging tips
- Any crash in the plugin will take down the daemon — use Rust's cdylib with catch_unwind wrappers if possible
- Plugin errors should go to stderr — systemd-journald will capture them
- For `dlopen` path issues: `sdb shell ldd /opt/usr/share/tizenclaw/plugins/llm/libmyllm.so` to check dep resolution

## Verification Matrix

| Extension | How to test locally (mock-sys) | How to test on device |
|---|---|---|
| Skill | `cargo test -p tizenclaw --features mock-sys` (unit tests cover TextualSkillScanner) | `sdb push` then `tizenclaw-cli` |
| Tool | Direct binary invocation + unit test of ToolDecl parsing | Same as skill + check agent can call it |
| Agent role | Config validation test only (parser doesn't run end-to-end) | Restart daemon + test delegation |
| LLM plugin | Compile + standalone test of C ABI via a minimal harness | Watch journalctl for init message + send a prompt |

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Skill not appearing in prompt | Not in `<skill_name>/SKILL.md` subdir, or no `description:` field | Check file structure; review daemon log after ToolWatcher reloads |
| Tool not callable by LLM | Missing from `/opt/usr/share/tizen-tools/<tool>/tool.md`, or `binary:` path wrong | `sdb shell ls /usr/bin/<tool_name>` and verify tool.md |
| Tool times out | Default timeout is 30s | Add `timeout: 60` to tool.md frontmatter |
| New agent role not delegated to | orchestrator's `can_delegate_to` not updated, OR run_supervisor tool not registered | Check JSON + verify the run_supervisor tool exists in `/opt/usr/share/tizen-tools/` |
| Plugin doesn't load | dlopen fails (missing deps) or symbol missing | `ldd <plugin>.so` + `nm -D <plugin>.so | grep llm_plugin_` |
| Plugin loads but daemon crashes | Panic in plugin code | Wrap init/chat in catch_unwind; log stack |

## FAQ

Q: Can I reload an agent role without restarting the daemon?
A: Not today. `AgentRoleRegistry::load_roles` is called only at startup. A reload IPC method would need to be added.

Q: What's the difference between a skill and a tool?
A: A tool is an actual callable function (binary) the LLM can invoke. A skill is a markdown guide teaching the LLM how and when to use existing tools. Skills don't add capabilities; they add judgment.

Q: Can a tool be a bash script instead of a compiled binary?
A: Yes. Point `binary:` at a script (e.g., `/usr/local/bin/my_tool.sh`) with a shebang. Make it executable.

Q: Do my extensions survive an OTA update?
A: Depends on paths. Files under `/opt/usr/share/tizen-tools/` (user data) generally persist. Binaries in `/usr/bin/` may be overwritten. Best practice: keep custom binaries in `/opt/usr/bin/custom/`.

Q: How do I tell the LLM to prefer my new tool over a similar one?
A: Rename the similar old tool, or write its description to say "deprecated — use X instead". The LLM chooses tools based on description matching.

Q: Is there a way to share a skill across multiple devices?
A: Not built-in. You'd push the same `<skill>/SKILL.md` via sdb to each device. A fleet distribution system is 🚧 planned (SwarmManager).

Q: Can I write an LLM plugin in Rust instead of C?
A: Yes — compile as `cdylib` and export the C functions via `#[no_mangle] extern "C"`. The ABI is C regardless of source language.

Q: How do I test a plugin during development without rebuilding the whole daemon?
A: Build a minimal harness binary in Rust that calls `dlopen` on your `.so` with the same ABI. Iterate there, then deploy to the daemon once stable.
