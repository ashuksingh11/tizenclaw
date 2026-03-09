# Task: LLM Tool Selection Optimization (Routing Guide)

## 1. Goal
LLM이 자연어 대화 맥락에서 다양한 도구(Python 스킬, 내장 도구, Tizen Action)를 스스로 올바르게 선택하고 분류할 수 있도록 전략적 가이드(`routing_guide.md`)를 제공하고 시스템 프롬프트에 통합한다.

## 2. Requirements
- [x] `tools/routing_guide.md` 파일 생성: 도구 선택 원칙, 우선순위, 예시 포함.
- [x] `CMakeLists.txt` 수정: `routing_guide.md` 설치 설정 추가.
- [x] `tizenclaw.spec` 수정: RPM 패키지에 `routing_guide.md` 포함.
- [x] `agent_core.cc` 수정: `BuildSystemPrompt`에서 가이드 파일을 로드하여 프롬프트 상단에 배치.
- [ ] 빌드 및 에뮬레이터 배포: 수정사항 반영 확인.
- [ ] 동작 검증: 모호한 자연어 요청 시 LLM의 도구 선택 능력이 향상되었는지 확인.

## 3. Implementation Details
- 파일 경로: `/opt/usr/share/tizenclaw/tools/routing_guide.md`
- 우선순위 정책: Tizen Action (`action_`) > Python Skill (`control_`)
- 안전 정책: 모호한 상태에서는 `get_` 도구로 확인 먼저 수행

## 4. Verification Plan
- `sdb shell journalctl -u tizenclaw -f`를 통해 LLM이 생성한 프롬프트에 가이드 내용이 포함되었는지 확인.
- "배터리 알려줘" 또는 "화면 어둡게 해줘"와 같은 요청 시 도구 선택 로그 확인.
