# TizenClaw 구현 상세 계획 (Implementation Phases)

본 문서는 Phase 1(기반 시스템 구축)을 완료한 TizenClaw 프로젝트의 향후 구현(Phase 2 ~ Phase 5) 과정에서 필요한 상세 기술 항목 및 개발진(사용자) 확인이 필요한 질문들을 정리한 문서입니다.

## Phase 2: 실행 환경(Container) 구축
**목표**: `crun` 또는 `runc` 기반의 OCI 경량 컨테이너를 Tizen Service App 내에서 구동하여 샌드박스(격리) 환경을 확보합니다.
- **주요 구현 항목**:
  1. LXC 또는 `crun` 라이브러리를 Agent Core에 연동 (C/C++ API 사용)
  2. 컨테이너 생성 및 스펙 파일(`config.json`) 동적 생성 로직 구현
  3. Namespace 분할 (User, PID, Mount, Network) 및 Tizen RootFS 마운트

**❓ 확인 필요 사항 (질문)**:
1. **LXC/crun 바이너리 확보**: Tizen 10.0 에뮬레이터 또는 디바이스에 `lxc` 나 `crun` 바이너리 및 라이브러리가 이미 내장되어 있나요? 아니면 GBS 빌드 시 외부 의존성(rpm)으로 미리 땡겨오거나, 소스를 직접 포함해서 스태틱으로 빌드해야 하나요?
2. **SMACK & kUEP 정책**: 컨테이너가 생성될 때 부여해야 할 특정 SMACK Label(`_` 이외에 System Service 전용 레이블)이 정해져 있나요? kUEP 우회를 위해 커널에 추가해야 하는 Capability나 파라미터가 있다면 무엇인지 확인이 필요합니다.

---

## Phase 3: 이미징 및 런타임 캡슐화
**목표**: Python과 Node.js가 포함된 초경량 RootFS(Alpine 타볼 등)를 Tizen에 배포/탑재합니다.
- **주요 구현 항목**:
  1. 호스트 머신(Ubuntu 등)에서 TizenClaw 전용 RootFS 빌드 스크립트(Dockerfile -> tar.gz 내보내기) 작성
  2. RPM 패키징 시 RootFS 타볼을 `/opt/usr/apps/org.tizen.tizenclaw/data/rootfs.tar.gz` 경로에 포함
  3. Service App 시작 시 해당 타볼을 특정 경로에 `setup` 및 `mount` 하는 로직 구현

**❓ 확인 필요 사항 (질문)**:
1. **용량 제한**: Tizen 디바이스의 `/opt/usr/apps/` 영역에 배포할 수 있는 최대 패키지(tpk/rpm) 용량 제한이 있나요? Python/Node.js 포함 시 최소 50~100MB 가량이 예상됩니다.
2. **RootFS 다운로드 방식**: 패키지 크기를 줄이기 위해 런타임(앱 최초 실행 시)에 네트워크를 통해 RootFS 이미지를 다운로드 받도록 설계하는 것이 나을까요? 아니면 rpm 배포본에 포함하는 것이 좋을까요?

---

## Phase 4: Skills 시스템 및 API 래퍼 구축
**목표**: 스킬 폴더를 마운트하고, Agent가 Tizen Device C-API를 Python/Node.js에서 호출할 수 있는 브리지를 제공합니다.
- **주요 구현 항목**:
  1. Agent Core에서 `/opt/usr/home/owner/share/tizenclaw/skills/` 폴더 감시 및 매니페스트 읽기 로직 (Prompt Builder 연동)
  2. Tizen Device API `pybind11` 통신 브리지 (Python 래퍼) 모듈 개발
  3. OpenClaw 호환 기본 스킬(System Info, Network 등) Python 래핑 적용

**❓ 확인 필요 사항 (질문)**:
1. Tizen Device API 중 가장 먼저 포팅(Wrapping)하여 테스트하길 원하시는 단일 API(예: Wi-Fi 상태 조회, 배터리 정보 등) 모듈은 무엇인가요?
2. OpenClaw 스킬 코드를 수정 없이 가져다 쓸 예정인지, 아니면 Tizen 환경에 맞춰 경량화/단순화 패치가 필요한 형태인지 방향성을 알려주세요.

---

## Phase 5: MCP 서버 및 실증 테스트
**목표**: 표준 MCP(Model Context Protocol) 커넥터를 연동하여 외부 LLM(Claude 등)이 Tizen 단말과 원격/로컬로 직접 Context를 교환하도록 구성합니다.
- **주요 구현 항목**:
  1. MCP Server Connector (WebSocket 또는 stdio/파이프라인 통신) 모듈 추가
  2. Tizen Service App 네트워크/로컬 포트 오픈 및 보안 처리
  3. LLM -> AgentCore -> Skill Container -> Tizen API 엔드투엔드 구동 검증

**❓ 확인 필요 사항 (질문)**:
1. MCP 연결을 단말 내부 App(또는 Daemon)에서 처리할 것인지, 아니면 단말 외부(PC, 클라우드 환경)에서 포트 포워딩(`sdb forward` 등)을 통해 Tizen 시스템에 접근하는 형태인지 결정이 필요합니다.
