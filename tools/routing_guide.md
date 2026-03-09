# Tool Selection & Routing Guide

This guide defines the principles and strategies for selecting the most appropriate tool among 35+ Python skills, 13+ built-in tools, and device-specific Tizen Actions.

## 1. Classification & Hierarchy

Tools are grouped into functional categories. If multiple tools seem applicable, prioritize based on the category.

### A. Device Control (Highest Priority: Tizen Actions)
For hardware-level controls (display, volume, notification), prioritize **Tizen Action Framework** tools (`action_`).
- **Priority**: `action_<name>` > `control_<name>` (Python skill)
- **Reason**: Actions are native, device-specific, and handle permissions/lifecycle via the platform.

### B. Information Query
Use `get_` prefix skills to fetch device status before attempting any state-changing operations.
- **Pattern**: If the user's goal is ambiguous (e.g., "Screen is dark"), always call `get_display_info` first to confirm the state.

### C. System Automation & Multi-Agent
Use built-in tools for managing the agent's own lifecycle and background tasks.
- **Task Management**: `create_task`, `list_tasks`, `cancel_task`
- **Agent Coordination**: `create_session`, `send_to_session`, `run_supervisor`
- **Skill Extension**: `manage_custom_skill`

## 2. Selection Principles

1. **Native over Scripted**: Prefer `action_` tools over Python `skills/` if both provide the same functionality.
2. **Context-Aware Chaining**: If an app ID is required but unknown, call `list_apps` first. Never guess an app ID.
3. **Safety First**: For high-risk operations (e.g., `terminate_app`, `control_power`), always confirm the target with the user or via a query skill if the intent is not 100% explicit.
4. **RAG vs. Memory**: Use `search_knowledge` for facts or technical documentation. Use conversation history for personal preferences or previous context.

## 3. Natural Language Mapping Examples

- "Turn off the lights" -> `action_flashlight` (if available) or `control_led(action="off")`
- "I'm busy now" -> `control_volume(action="set", sound_type="system", volume=0)`
- "Remind me later" -> `create_task(...)`
- "What's wrong with the phone?" -> `run_supervisor(goal="Check overall device health and status")`
