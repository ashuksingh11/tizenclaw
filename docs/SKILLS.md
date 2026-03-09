# TizenClaw Skills Reference

TizenClaw provides **35 container skills** (Python, sandboxed via OCI) and **10+ built-in tools** (native C++).

> Container skills use `ctypes` FFI to call Tizen C-API directly. Async skills use the **tizen-core** event loop for callback-based APIs.

---

## Container Skills (Python)

### App Management

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `list_apps` | — | `app_manager` | List all installed applications |
| `send_app_control` | `app_id`, `operation`, `uri`, `mime`, `extra_data` | `app_control` | Launch app via explicit app_id or implicit intent (operation/URI/MIME) |
| `terminate_app` | `app_id` | `app_manager` | Terminate a running app |
| `get_package_info` | `package_id` | `package_manager` | Query package details (version, type, size) |

### Device Info & Sensors

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `get_device_info` | — | `system_info` | Model, OS version, platform info |
| `get_system_info` | — | `system_info` | Hardware details (CPU, screen, features) |
| `get_runtime_info` | — | `runtime_info` | CPU and memory usage statistics |
| `get_storage_info` | — | `storage` | Internal/external storage space |
| `get_system_settings` | — | `system_settings` | Locale, timezone, font, wallpaper |
| `get_sensor_data` | `sensor_type` | `sensor` | Accelerometer, gyroscope, light, proximity, etc. |
| `get_thermal_info` | — | `device` (thermal) | Device temperature (AP, CP, battery) |

### Network & Connectivity

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `get_wifi_info` | — | `wifi-manager` | Current WiFi connection details |
| `get_bluetooth_info` | — | `bluetooth` | Bluetooth adapter state |
| `get_network_info` | — | `connection` | Network type, IP address, status |
| `get_data_usage` | — | `connection` (statistics) | WiFi/cellular data usage stats |
| `scan_wifi_networks` | — | `wifi-manager` + **tizen-core** ⚡ | Scan nearby WiFi access points (async) |
| `scan_bluetooth_devices` | `action` | `bluetooth` + **tizen-core** ⚡ | Discover nearby BT devices or list bonded (async) |

### Display & Hardware Control

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `get_display_info` | — | `device` (display) | Brightness, state, max brightness |
| `control_display` | `brightness` | `device` (display) | Set display brightness level |
| `control_haptic` | `duration_ms` | `device` (haptic) | Vibrate the device |
| `control_led` | `action`, `brightness` | `device` (flash) | Camera flash LED on/off |
| `control_volume` | `action`, `sound_type`, `volume` | `sound_manager` | Get/set volume levels |
| `control_power` | `action`, `resource` | `device` (power) | Request/release CPU/display lock |

### Media & Content

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `get_battery_info` | — | `device` (battery) | Battery level and charging status |
| `get_sound_devices` | — | `sound_manager` (device) | List audio devices (speakers, mics) |
| `get_media_content` | `media_type`, `max_count` | `media-content` | Search media files on device |
| `get_metadata` | `file_path` | `metadata-extractor` | Extract media file metadata (title, artist, album, duration, etc.) |
| `get_mime_type` | `file_extension`, `file_path`, `mime_type` | `mime-type` | MIME type ↔ extension lookup |

### System Actions

| Skill | Parameters | C-API | Description |
|-------|-----------|-------|-------------|
| `play_tone` | `tone`, `duration_ms` | `tone_player` | Play DTMF or beep tones |
| `play_feedback` | `pattern` | `feedback` | Play sound/vibration patterns |
| `send_notification` | `title`, `body` | `notification` | Post notification to device |
| `schedule_alarm` | `app_id`, `datetime` | `alarm` | Schedule alarm at specific time |
| `download_file` | `url`, `destination`, `file_name` | `url-download` + **tizen-core** ⚡ | Download URL to device (async) |
| `web_search` | `query` | — (Wikipedia) | Web search via Wikipedia API |

> ⚡ = Async skill using **tizen-core** event loop (`tizen_core_task_create` → `add_idle_job` → `task_run` → callback → `task_quit`)

---

## Built-in Tools (AgentCore, Native C++)

| Tool | Description |
|------|-------------|
| `execute_code` | Execute Python code in sandbox |
| `file_manager` | Read/write/list files on device |
| `manage_custom_skill` | Create/update/delete/list custom skills at runtime |
| `create_task` | Create a scheduled task |
| `list_tasks` | List active scheduled tasks |
| `cancel_task` | Cancel a scheduled task |
| `create_session` | Create a new chat session |
| `list_sessions` | List active sessions |
| `send_to_session` | Send message to another session |
| `ingest_document` | Ingest document into RAG store |
| `search_knowledge` | Semantic search in RAG store |
| `execute_action` | Execute a Tizen Action Framework action |
| `action_<name>` | Per-action tools (auto-discovered from Action Framework) |

---

## Runtime Custom Skills

The LLM can create new skills at runtime using the `manage_custom_skill` tool. Custom skills are stored at `/opt/usr/share/tizenclaw/tools/custom_skills/` and are **immediately available** after creation (no restart needed).

| Operation | Description |
|-----------|-------------|
| `create` | Generate `manifest.json` + Python script from LLM-generated code |
| `update` | Modify existing skill code or description |
| `delete` | Remove a custom skill |
| `list` | List all custom skills |

Custom skills follow the same structure as built-in skills: `manifest.json` (tool schema) + `<name>.py` (Python script using `CLAW_ARGS` env and `ctypes` FFI).

---

## Multi-Agent System

TizenClaw supports a multi-agent architecture with specialized agents:

| Agent | Type | Role |
|-------|------|------|
| **Orchestrator** | supervisor | Analyzes requests, decomposes goals, delegates to specialized agents |
| **Skill Manager** | worker | Creates/updates/deletes custom skills at runtime via `manage_custom_skill` |
| **Device Monitor** | worker | Monitors battery, temperature, memory, storage, network health |

Agents are defined in `config/agent_roles.json` and communicate via `create_session` / `send_to_session` tools. The orchestrator is the default entry point when multi-agent mode is enabled.

---

## Async Pattern (tizen-core)

Skills marked with ⚡ use an async pattern for callback-based Tizen APIs:

```
tizen_core_init()
  → tizen_core_task_create("main", false)
    → tizen_core_add_idle_job(start_api_call)
    → tizen_core_add_timer(timeout_ms, safety_timeout)
    → tizen_core_task_run()          ← blocks until quit
      → API callback fires
        → collect results
        → tizen_core_task_quit()
  → return results
```

This enables Python FFI to use any callback-based Tizen C-API without threading.
