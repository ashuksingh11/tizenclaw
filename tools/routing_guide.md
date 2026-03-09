# TizenClaw Tool Selection & Routing Guide

You must follow this guide strictly when selecting tools to fulfill user requests. Tools are categorized by implementation type and priority.

## 1. Tool Categories

### A. Tizen Actions (`action_*`) - Highest Priority
Native Tizen Platform features. These are the fastest and most reliable for core device control.
- **Usage**: Use for display brightness, volume, flashlight, notifications, and core system settings.
- **Priority**: Always check if an `action_` tool exists for a task before using a Python skill.

### B. Embedded Tools (`embedded`) - High Priority
C++ built-in tools for system management and agent coordination.
- **Core Operations**: `file_manager` (file I/O), `task_scheduler` (automation).
- **Agent Coordination**: `supervisor_engine` (multi-agent delegation), `session_manager` (context handling).

### C. Standard Skills (`skills/`) - Medium Priority
Pre-defined Python scripts for specific functionalities (e.g., `web_search`, `get_battery_info`).
- **Usage**: Use when a native Tizen Action is not available or for specialized logic like web scraping or data parsing.

### D. Custom Skills (`custom_skills/`) - Dynamic Priority
User-defined or AI-generated scripts added at runtime.
- **Usage**: Use when standard tools/skills are insufficient for a specific, newly defined requirement.

## 2. Selection Strategy & Logic

1. **Prefer Native**: If `action_brightness` and `control_display` are both available, you MUST use `action_brightness`.
2. **Confirm State First**: Before changing a system state, use a `get_` skill (e.g., `get_display_info`, `get_battery_info`) to verify current values unless the user is explicit.
3. **Handle Failure Gracefully**:
   - If an `action_` tool fails, try the corresponding Python `skill` if it exists.
   - If a Python skill fails, explain the error and suggest an alternative if possible.
4. **App Interaction**:
   - Never guess an `app_id`. Use `list_apps` to find the correct identifier before calling `send_app_control` or `terminate_app`.
5. **Security & Safety**:
   - For irreversible operations (e.g., `delete_file`, `terminate_app`), always ask for confirmation unless the user's intent is absolutely clear and specific.
   - Paths for `file_manager` MUST start with `/tools/skills/` (for code) or `/data/` (for data).

## 3. Decision Tree Examples

- **"Make the screen brighter"**
  -> `get_display_info` (check current level) -> `action_brightness` (set new level).
- **"Search for the weather in Seoul"**
  -> `web_search(query="weather in Seoul")`.
- **"Kill the music player"**
  -> `list_apps(filter="music")` -> `terminate_app(app_id="...")`.
- **"Remind me to take medicine in 2 hours"**
  -> `create_task(command="send_notification(...)", trigger_type="interval", interval_seconds=7200)`.
