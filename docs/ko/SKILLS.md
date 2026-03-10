# TizenClaw 스킬 레퍼런스

TizenClaw는 **35개 컨테이너 스킬** (Python, OCI 샌드박스)과 **10개 이상의 내장 도구** (네이티브 C++)를 제공합니다.

> 컨테이너 스킬은 `ctypes` FFI를 통해 Tizen C-API를 직접 호출합니다. 비동기 스킬은 **tizen-core** 이벤트 루프를 사용합니다.

---

## 컨테이너 스킬 (Python)

### 앱 관리

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `list_apps` | — | `app_manager` | 설치된 앱 목록 조회 |
| `send_app_control` | `app_id`, `operation`, `uri`, `mime`, `extra_data` | `app_control` | 명시적 app_id 또는 암시적 인텐트(operation/URI/MIME)로 앱 실행 |
| `terminate_app` | `app_id` | `app_manager` | 실행 중인 앱 종료 |
| `get_package_info` | `package_id` | `package_manager` | 패키지 상세 정보 (버전, 타입, 크기) |

### 디바이스 정보 & 센서

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `get_device_info` | — | `system_info` | 모델, OS 버전, 플랫폼 정보 |
| `get_system_info` | — | `system_info` | 하드웨어 상세 (CPU, 화면, 기능) |
| `get_runtime_info` | — | `runtime_info` | CPU/메모리 사용량 |
| `get_storage_info` | — | `storage` | 내부/외부 저장소 공간 |
| `get_system_settings` | — | `system_settings` | 로케일, 시간대, 글꼴, 배경화면 |
| `get_sensor_data` | `sensor_type` | `sensor` | 가속도계, 자이로, 조도, 근접 센서 등 |
| `get_thermal_info` | — | `device` (thermal) | 디바이스 온도 (AP, CP, 배터리) |

### 네트워크 & 연결

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `get_wifi_info` | — | `wifi-manager` | 현재 WiFi 연결 상세 |
| `get_bluetooth_info` | — | `bluetooth` | 블루투스 어댑터 상태 |
| `get_network_info` | — | `connection` | 네트워크 타입, IP 주소, 상태 |
| `get_data_usage` | — | `connection` (통계) | WiFi/셀룰러 데이터 사용량 |
| `scan_wifi_networks` | — | `wifi-manager` + **tizen-core** ⚡ | 주변 WiFi 액세스 포인트 스캔 (비동기) |
| `scan_bluetooth_devices` | `action` | `bluetooth` + **tizen-core** ⚡ | 주변 BT 장치 탐색 또는 페어링 목록 (비동기) |

### 디스플레이 & 하드웨어 제어

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `get_display_info` | — | `device` (display) | 밝기, 상태, 최대 밝기 |
| `control_display` | `brightness` | `device` (display) | 디스플레이 밝기 설정 |
| `control_haptic` | `duration_ms` | `device` (haptic) | 디바이스 진동 |
| `control_led` | `action`, `brightness` | `device` (flash) | 카메라 플래시 LED on/off |
| `control_volume` | `action`, `sound_type`, `volume` | `sound_manager` | 볼륨 레벨 조회/설정 |
| `control_power` | `action`, `resource` | `device` (power) | CPU/디스플레이 잠금 요청/해제 |

### 미디어 & 콘텐츠

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `get_battery_info` | — | `device` (battery) | 배터리 잔량 및 충전 상태 |
| `get_sound_devices` | — | `sound_manager` (device) | 오디오 디바이스 목록 (스피커, 마이크) |
| `get_media_content` | `media_type`, `max_count` | `media-content` | 디바이스 미디어 파일 검색 |
| `get_metadata` | `file_path` | `metadata-extractor` | 미디어 파일 메타데이터 추출 (제목, 아티스트, 앨범, 길이 등) |
| `get_mime_type` | `file_extension`, `file_path`, `mime_type` | `mime-type` | MIME 타입 ↔ 확장자 조회 |

### 시스템 액션

| 스킬 | 파라미터 | C-API | 설명 |
|------|---------|-------|------|
| `play_tone` | `tone`, `duration_ms` | `tone_player` | DTMF/비프 톤 재생 |
| `play_feedback` | `pattern` | `feedback` | 사운드/진동 피드백 패턴 재생 |
| `send_notification` | `title`, `body` | `notification` | 디바이스 알림 게시 |
| `schedule_alarm` | `app_id`, `datetime` | `alarm` | 특정 시간에 알람 예약 |
| `download_file` | `url`, `destination`, `file_name` | `url-download` + **tizen-core** ⚡ | URL에서 파일 다운로드 (비동기) |
| `web_search` | `query` | — (Wikipedia) | Wikipedia API 웹 검색 |

> ⚡ = **tizen-core** 이벤트 루프를 사용하는 비동기 스킬 (`tizen_core_task_create` → `add_idle_job` → `task_run` → 콜백 → `task_quit`)

---

## 내장 도구 (AgentCore, 네이티브 C++)

| 도구 | 설명 |
|------|------|
| `execute_code` | 샌드박스에서 Python 코드 실행 |
| `file_manager` | 디바이스 파일 읽기/쓰기/조회 |
| `manage_custom_skill` | 런타임 커스텀 스킬 생성/수정/삭제/조회 |
| `create_task` | 예약 작업 생성 |
| `list_tasks` | 활성 예약 작업 조회 |
| `cancel_task` | 예약 작업 취소 |
| `create_session` | 새 채팅 세션 생성 |
| `list_sessions` | 활성 세션 조회 |
| `send_to_session` | 다른 세션에 메시지 전송 |
| `ingest_document` | RAG 스토어에 문서 인덱싱 |
| `search_knowledge` | RAG 스토어 시맨틱 검색 |
| `execute_action` | Tizen Action Framework 액션 실행 |
| `action_<name>` | Per-action 도구 (Action Framework에서 자동 발견) |

---

## 런타임 커스텀 스킬

LLM이 `manage_custom_skill` 도구를 사용하여 런타임에 새로운 스킬을 생성할 수 있습니다. 커스텀 스킬은 `/opt/usr/share/tizenclaw/tools/custom_skills/`에 저장되며 생성 즉시 사용 가능합니다 (재시작 불필요).

| 작업 | 설명 |
|------|------|
| `create` | LLM이 생성한 코드로 `manifest.json` + Python 스크립트 자동 생성 |
| `update` | 기존 스킬 코드 또는 설명 수정 |
| `delete` | 커스텀 스킬 삭제 |
| `list` | 모든 커스텀 스킬 조회 |

커스텀 스킬은 내장 스킬과 동일한 구조: `manifest.json` (도구 스키마) + `<name>.py` (`CLAW_ARGS` 환경변수 + `ctypes` FFI 사용).

---

## 멀티 에이전트 시스템

TizenClaw는 전문화된 에이전트를 활용한 멀티 에이전트 아키텍처를 지원합니다:

| 에이전트 | 타입 | 역할 |
|---------|------|------|
| **Orchestrator** | supervisor | 요청 분석, 목표 분해, 전문 에이전트에 위임 |
| **Skill Manager** | worker | `manage_custom_skill`을 통한 런타임 스킬 CRUD |
| **Device Monitor** | worker | 배터리, 온도, 메모리, 저장소, 네트워크 상태 모니터링 |

에이전트는 `config/agent_roles.json`에 정의되며 `create_session` / `send_to_session` 도구를 통해 통신합니다.

---

## 비동기 패턴 (tizen-core)

⚡ 표시된 스킬은 콜백 기반 Tizen API를 위한 비동기 패턴을 사용합니다:

```
tizen_core_init()
  → tizen_core_task_create("main", false)
    → tizen_core_add_idle_job(API_호출_시작)
    → tizen_core_add_timer(타임아웃_ms, 안전_타임아웃)
    → tizen_core_task_run()          ← quit까지 블록
      → API 콜백 실행
        → 결과 수집
        → tizen_core_task_quit()
  → 결과 반환
```

이를 통해 Python FFI에서 스레딩 없이 모든 콜백 기반 Tizen C-API 사용 가능.
