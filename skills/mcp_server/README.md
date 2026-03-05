# TizenClaw MCP Server

TizenClaw의 스킬들을 표준 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 도구로 노출합니다. 이를 통해 Claude Desktop 등의 외부 LLM 클라이언트가 Tizen 단말을 직접 제어할 수 있습니다.

## 사용 방법

1. **Tizen 단말에 배포**: `gbs build` 혹은 `sdb push`를 통해 `skills/mcp_server` 폴더를 단말의 `/opt/usr/share/tizenclaw/skills/mcp_server`에 위치시킵니다.

2. **Claude Desktop 설정**:
   PC의 `claude_desktop_config.json` (보통 `%APPDATA%\Claude\claude_desktop_config.json` 또는 `~/Library/Application Support/Claude/claude_desktop_config.json`)에 다음 설정을 추가합니다:

   ```json
   {
     "mcpServers": {
       "tizenclaw": {
         "command": "sdb",
         "args": [
           "shell",
           "python3",
           "/opt/usr/share/tizenclaw/skills/mcp_server/server.py"
         ]
       }
     }
   }
   ```

3. **Claude 실행**: Claude Desktop을 재시작하면 TizenClaw의 스킬들이 'Tools'로 등록됩니다. 이제 "Tizen에서 앱 목록 보여줘" 등의 자연어 명령으로 단말을 제어할 수 있습니다.

## 제공되는 도구 (Tools)
- `list_apps`: 설치된 앱 목록 조회
- `get_wifi_info`: Wi-Fi 상태 조회
- `get_battery_info`: 배터리 상태 조회
- `get_bluetooth_info`: 블루투스 상태 조회
- 기타 `skills/` 디렉터리에 `manifest.json`이 있는 모든 스킬

## 기술 상세
- **Transport**: `stdio` 기반 트랜스포트 지원 (JSON-RPC 2.0)
- **Tool Discovery**: `skills/` 하위 디렉터리의 `manifest.json`을 자동 스캔하여 MCP Tool Definition으로 변환합니다.
