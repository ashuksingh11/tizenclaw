---
description: TizenClaw Coding Rules and Guidelines
---

# TizenClaw Agent Support Rules

본 저장소에서 TizenClaw를 구현할 때, Agent(AI)는 항상 다음 코딩 스타일과 규칙을 최우선으로 준수해야 합니다.

## 1. C++ 코딩 스타일
- **스타일 가이드**: [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html)를 엄격하게 따릅니다.
- **최대 글꼴 줄바꿈 (Line Wrap)**: 소스코드, 주석, 헤더 파일의 모든 텍스트는 **80자를 넘지 않도록 (Column limit: 80)** 적절하게 줄바꿈합니다. 
- **들여쓰기(Indentation)**: 2칸(Space 2)을 사용합니다 (탭 사용 금지).
- **명명 규칙**:
  - Class/Struct: PascalCase (예: `AgentCore`, `SandboxManager`)
  - 변수명: snake_case (예: `app_data`, `cmd_line`)
  - 멤버 변수: `m_` 접두사 또는 뒤에 `_` 접미사 통일 적용 (예: `m_initialized` 또는 `initialized_`)
  - 함수명: PascalCase 또는 Tizen C API 스타일 래핑 시 snake_case 허용.

## 2. CMake 및 빌드 지원
- Tizen GBS (Gerrit Build System) 환경을 타겟으로 작성하며, CMake를 통해 `gbs build`가 항상 성공해야 합니다.
- 새로운 C++ 소스 파일 추가 시 반드시 `CMakeLists.txt`의 `SOURCES` 리스트를 업데이트하세요.

## 3. Tizen 특화 룰
- 권한이 필요한 기능(Network, LXC 구동, AppManager 등)은 반드시 `tizen-manifest.xml`의 `<privileges>` 블록에 명시합니다.
- dlog 인터페이스(`dlog_print`)를 활용하여 시스템 로그를 충실히 남기고, C++ 예외(Exception)보다는 가급적 리턴 코드나 boolean 반환을 통해 에러 핸들링을 우선시합니다.
