# TizenClaw 요구사항 명세서 (Requirements Specification)

## 1. 개요 (Overview)
**TizenClaw**는 **openclaw** 및 **claude cowork** 프로젝트에서 영감을 받아, Tizen Embedded Linux 환경에 최적화된 형태의 Native Agent 구동 체계를 구축하는 것을 목표로 합니다. 
본 프로젝트는 시스템의 제한적인 리소스와 강력한 보안 환경(SMACK, DAC, kUEP) 속에서도 프롬프트 기반으로 동적이고 유연한 Agent 동작을 지원하는 런타임을 제공합니다.

## 2. 시스템 구동 환경 (System Environment)
- **OS**: Tizen Embedded Linux (Tizen 10.0 기준)
- **보안 환경**: SMACK (Simplified Mandatory Access Control Kernel) 및 DAC (Discretionary Access Control) 적용
- **커널 보호 옵션**: kUEP (Kernel Unprivileged Execution Protection) 활성화된 상태

## 3. 기능 요구사항 (Functional Requirements)

### 3.1. Agent Core Runtime (Agent 엔진)
- **Native C++ 기반**: 퍼포먼스와 시스템 저단 API 접근성 확보를 위해 코어는 C++ 기반으로 작성되어야 합니다.
- **프롬프트 기반 행동 결정**: 설정된 Prompt를 바탕으로 해야 할 일을 계획(Planning)하고 도구(Skills)를 탐색/실행할 수 있어야 합니다.
- **MCP (Model Context Protocol) 연동**: 모델과의 Context 교환을 원활하게 하기 위해 외부 또는 로컬의 MCP 서버와 연결되는 인터페이스를 제공해야 합니다.

### 3.2. Skills 구동 환경 및 호환성 (Skills Execution & Compatibility)
- **Skills 디렉터리 사용**: Tizen 내 `/usr/apps/org.tizen.tizenclaw/data/skills/` (또는 하위 경로)에 스킬 스크립트를 위치시키고 이를 동적으로 로드하여 사용할 수 있어야 합니다.
- **Tizen C API 연동**: Tizen Device C API를 스킬에서 접근할 수 있도록 Python 또는 Node.js 래퍼(Wrapper/Bindings) 제공이 필요합니다. 
  - *접근 전략*: 10.0의 Core API(System Info, Network 등)부터 하나씩 단계적으로 래핑(Wrapping) 작업을 진행합니다.
- **OpenClaw 배포 스킬 호환성**: OpenClaw의 스킬팩이 Tizen 내부에서도 약간의 수정 내지 원활히 동작할 수 있어야 합니다.

### 3.3. 경량 OCI 컨테이너 및 이미지 샌드박싱 (Lightweight Image-based Container)
- 단순한 `unshare()` 시스템 콜 호출에 머무르지 않고, 파일시스템 격리와 의존성 관리가 완벽한 **경량 OCI 컨테이너 런타임(예: `crun`, `runc` 또는 LXC)**을 활용하는 방식으로 구축합니다.
- **이미지 탑재(RootFS)**: Python/Node.js 런타임과 필수 라이브러리가 포함된 최소 용량의 컨테이너 이미지(예: Alpine Linux 기반 RootFS tarball)를 Tizen 앱(또는 지정된 스토리지 경로)에 탑재합니다. Agent는 실행 시 이 이미지를 압축 해제/마운트하여 컨테이너 환경의 베이스로 사용합니다.
- **kUEP 비활성화 지원**: 컨테이너 샌드박스로 구동되는 자식 프로세스 공간 내에서는 스크립트 실행 제약을 우회하기 위해 kUEP 정책을 비활성화 할 수 있는 방안이나 적절한 런타임 룰이 적용되어야 합니다.

## 4. 비기능 및 배포 요구사항 (Non-Functional & Deployment)

### 4.1. 배포 형태: System Service App
- **System Service App**: Agent는 Tizen Application Framework(AppFW)에서 백그라운드로 동작하는 **System Service App(C++ Native)** 형태로 구동됩니다.
- 이는 표준 Tizen 패키지(.tpk) 형태로 배포 및 관리가 용이하며, AppFW의 라이프사이클 관리(실행, 종료, 재시작 등)와 AppControl 기반의 IPC를 자연스럽게 활용할 수 있는 장점이 있습니다.
- 단, 샌드박싱(Namespace 분리)이나 kUEP 우회 등 시스템 레벨의 권한이 필요한 작업을 수행하기 위해, 해당 앱은 **Platform/System 권한**으로 서명되어야 하며 필요한 SMACK 예외 룰 및 Capability(`CAP_SYS_ADMIN` 등)를 부여받는 환경 구성이 동반되어야 합니다.

### 4.2. 필수 런타임 언어 (Python / Node.js) 확보
*현재 Tizen 환경에는 기본적으로 Python 및 Node.js가 미설치 상태입니다.* OpenClaw 스킬 활용 및 Python 래핑 API 구동을 위해서는 다음 중 하나의 전략이 필요합니다.
1. **정적 빌드 바이너리 내장**: TizenClaw 배포 시 빌드된 Python(또는 Node.js) 런타임 자체를 스태틱으로 묶어 앱 `data/` 하위에 배치 (Standalone 형태)
2. **시스템 패키징 매니저 활용**: Tizen GBS(Gerrit Build System)를 통해 `rpm` 형식의 Python / Node.js 패키지를 생성하고 OS 이미지에 사전 탑재하거나 후속 설치.
* 본 프로젝트는 TizenClaw 프레임워크 구축에 집중하므로, Tizen 전용 런타임 패키지 빌드 문서를 별도로 작성하거나 독립형 런타임 바이너리를 배포하는 방식으로 해결할 계획입니다.
