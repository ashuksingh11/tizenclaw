# TizenClaw — Detailed Documentation

This directory contains the in-depth architecture and developer documentation for TizenClaw. Start here to pick a reading path that matches your goal.

## File Index

| # | File | Focus |
|---|---|---|
| 01 | [OVERVIEW](01_OVERVIEW.md) | Project overview, workspace map, module integration inventory, glossary |
| 02 | [RUST_FOR_CPP_DEVELOPERS](02_RUST_FOR_CPP_DEVELOPERS.md) | Rust primer via C/C++ analogies (ownership, traits, async, FFI) |
| 03 | [ARCHITECTURE_DEEP_DIVE](03_ARCHITECTURE_DEEP_DIVE.md) | Three-tier topology, boot sequence, concurrency model, IPC protocol |
| 04 | [AGENTIC_AI_CONCEPTS](04_AGENTIC_AI_CONCEPTS.md) | ReAct loop, tool calling, session management, safety basics |
| 05 | [LLM_BACKEND_SYSTEM](05_LLM_BACKEND_SYSTEM.md) | LlmBackend trait, providers (Gemini, OpenAI, Anthropic, Ollama), plugins |
| 06 | [TOOLS_AND_SKILLS_GUIDE](06_TOOLS_AND_SKILLS_GUIDE.md) | ToolDispatcher, tool.md manifests, SKILL.md format, hot-reload |
| 07 | [CHANNELS_AND_IPC](07_CHANNELS_AND_IPC.md) | Channel trait, web dashboard, Telegram/Discord/Slack, MCP, IPC protocol |
| 08 | [PLATFORM_AND_FFI](08_PLATFORM_AND_FFI.md) | PlatformPlugin system, tizen-sys bindings, C-ABI client library |
| 09 | [STORAGE_AND_MEMORY](09_STORAGE_AND_MEMORY.md) | SQLite schemas reference (sessions, memory, embeddings, audit) |
| 10 | [BUILD_TEST_DEPLOY](10_BUILD_TEST_DEPLOY.md) | Cargo build, mock-sys testing, GBS cross-compile, deploy.sh, RPM packaging |
| **11** | [MEMORY_SESSION_DEEPDIVE](11_MEMORY_SESSION_DEEPDIVE.md) | **Deep dive**: session lifecycle, what's pulled into a prompt (today vs planned), storage layer map |
| **12** | [MULTI_AGENT_ORCHESTRATION](12_MULTI_AGENT_ORCHESTRATION.md) | **Deep dive**: orchestrator + 7 specialists, workflows, pipelines, agent roles |
| **13** | [SAFETY_AND_POLICY](13_SAFETY_AND_POLICY.md) | **Deep dive**: three-layer defense (SafetyGuard, ToolPolicy, safety_bounds.json), circuit breaker |
| **14** | [EVENT_BUS_TRIGGERS](14_EVENT_BUS_TRIGGERS.md) | **Deep dive**: pub/sub EventBus, AutonomousTrigger rules, direct vs evaluate modes |
| **15** | [EXTENDING_TIZENCLAW](15_EXTENDING_TIZENCLAW.md) | **Practical**: step-by-step recipes for adding skills, tools, agent roles, LLM plugins |

## Reading Paths

Pick the path that matches your goal. Each path is ordered — read top to bottom.

### "I'm new — give me the whole picture"
1. **01** — Overview
2. **03** — Architecture deep dive
3. **04** — Agentic AI concepts
4. **11** — Memory & session deep dive
5. **12** — Multi-agent orchestration

### "I want to extend TizenClaw (add skills/tools)"
1. **15** — Extension scenarios (read the "Scenario 1: Skill" or "Scenario 2: Tool" section first)
2. **06** — Tools & skills guide (reference)
3. **04** — Understand how the LLM uses what you've added
4. **11** — How your tool results flow into the conversation

### "I want to understand memory and sessions deeply"
1. **01** — Overview (see the module integration inventory in §3.5)
2. **11** — Memory & session deep dive ← the main document
3. **09** — Storage schemas for reference

### "I want to understand safety and policy"
1. **13** — Safety & policy layered defense ← the main document
2. **12** — How agent roles interact with tool restrictions

### "I want to handle autonomous behavior (events, triggers)"
1. **14** — Event bus & autonomous triggers ← the main document
2. **12** — How triggers might delegate to specialist agents

### "I know C++ but not Rust"
1. **02** — Rust for C++ developers
2. **01** — Overview (project structure)
3. **03** — Architecture deep dive

### "I'm integrating from my C/C++ Tizen app"
1. **02** — Rust for C++ developers (FFI section)
2. **08** — Platform & FFI
3. **07** — Channels & IPC (if doing socket-level integration)

### "I need to build and deploy"
1. **10** — Build, test, deploy
2. **08** — Platform & FFI (for understanding cross-compile)

## Integration Status Badges

Throughout these docs, modules are marked with status badges to distinguish what's live from what's dormant:

- ✅ **Integrated** — actively used at runtime in `AgentCore::process_prompt` or the boot sequence
- ⚠️ **Built, not wired** — module exists and is tested, but no call site integrates it with the agent loop yet
- 🔧 **Stub** — skeleton only; functionality not yet implemented
- 🚧 **Planned** — documented intent only

As of April 2026 after the upstream merge, most subsystems that were previously ⚠️ dormant are now integrated: `MemoryStore` (line 447 of `process_prompt.rs`), `SafetyGuard` + `ToolPolicy` (lines 1252 / 1269), `EventBus`, `AgentRoleRegistry` (with `ensure_builtin_roles()` + schema-flexible `load_roles()`), and `WorkflowEngine` (`load_workflows_from` at init, match at loop start in `process_prompt_loop.rs`). `ActionBridge` is a new field routing `action_` tool calls. Remaining ⚠️ items: `PerceptionEngine`, full role-based tool filtering at dispatcher level (filtering today happens through `SessionPromptProfile`'s `allowed_tools`), and event publishers from most infra adapters. See **[01_OVERVIEW.md §3.5](01_OVERVIEW.md)** for the full module inventory.

## Diagrams

Every deep-dive doc includes Mermaid diagrams (flowcharts, sequence diagrams). GitHub renders these natively. If viewing locally, use a Markdown viewer that supports Mermaid (VS Code with the Mermaid extension, Obsidian, Typora, etc.).

Key sequence diagrams to know:
- **Session start → response**: [11, §3](11_MEMORY_SESSION_DEEPDIVE.md)
- **Memory retrieval (current + planned)**: [11, §6](11_MEMORY_SESSION_DEEPDIVE.md)
- **Tool call end-to-end with safety layers**: [13, §6](13_SAFETY_AND_POLICY.md)
- **Autonomous trigger firing**: [14, §3](14_EVENT_BUS_TRIGGERS.md)
- **ReAct agentic loop**: [04, §2](04_AGENTIC_AI_CONCEPTS.md)
- **IPC protocol flow**: [07](07_CHANNELS_AND_IPC.md)
- **Multi-agent delegation**: [12, §10](12_MULTI_AGENT_ORCHESTRATION.md)

## FAQ

Each doc has its own FAQ section at the bottom covering topic-specific questions. For cross-cutting questions (e.g., "why aren't these modules wired up?"), see **[01_OVERVIEW.md](01_OVERVIEW.md)**'s FAQ.

## Contributing to the Docs

When the code changes:
1. Update the affected doc's module references first (don't leave `⚠️` on a newly-wired module)
2. If a new subsystem is added, update **01_OVERVIEW.md §3.5** inventory + this README's file index
3. Sequence diagrams should match actual code flow — re-verify when refactoring `process_prompt`
4. Keep per-file FAQs current; consolidate into this README only if a question genuinely spans multiple docs

## Provenance

Documentation last regenerated: April 22, 2026
Verified against source code at: commit `918a872b` and following
Re-verified against commit after upstream merge (`63628b97`). Architecture refactored to multi-file `include!`-based `AgentCore` facade with 19 fields; memory, safety, event bus, workflows now integrated into `process_prompt`.
