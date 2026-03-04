---
description: Tizen gtest & ctest Unit Testing Workflow
---

# TizenClaw Unit Testing Workflow

이 모듈은 Tizen `rpc-port` 의 gtest/gmock 테스트 구조를 참고하여 만들어졌으며, GBS 빌드 시 RPM 패키징의 `%check` 섹션에서 `ctest`를 통해 자동으로 구동되도록 설계되었습니다.

## 1. 디렉터리 구조 및 파일 네이밍
프로젝트 최상위의 `test/unit_tests/` 디렉터리에 단위 테스트 코드들을 모아둡니다.
- `main.cc`: gtest 환경 초기화(`testing::InitGoogleTest`) 및 전체 테스트 실행(`RUN_ALL_TESTS()`)
- `CMakeLists.txt`: 테스트 실행 파일 생성 및 `gtest`, `gmock` 링크 설정
- `mock/`: 시스템 API나 타 모듈의 gmock 클래스와 가짜(Fake) 구현부를 모아두는 곳
- `*_test.cc`: 각 컴포넌트나 클래스별 단위 테스트 파일 (예: `agent_core_test.cc`)

## 2. CMakeLists.txt 구성 규칙
- `test/unit_tests/CMakeLists.txt` 에서는 `tizenclaw` 타겟 구현 파일들을 다시 엮거나(shared/static library 연동), `add_executable()` 에 `*_test.cc` 파일과 함께 컴파일하여 하나의 통합 바이너리(예: `tizenclaw-unittests`)를 생성해야 합니다.
- `pkg_check_modules`를 통해 `gtest`와 `gmock` 의존성을 가져와 링킹(`target_link_libraries`)합니다.
- Tizen 플랫폼에서는 `add_test(NAME TizenClawTests COMMAND tizenclaw-unittests)`를 추가하여 CTest가 이를 인식하도록 만듭니다.

## 3. RPM Spec 파일 (`%check` 섹션)
`packaging/tizenclaw.spec` 파일은 다음을 포함해야 합니다.
```spec
%check
cd build
ctest -V
```
이 섹션은 `gbs build` 과정에서 컴파일( `%build` ) 직후에 실행되며, `ctest` 가 실패하면 전체 패키지 빌드도 실패하게 됩니다.

## 4. Mock 작성 및 로그 DLOG 후킹
- Tizen CAPI (`dlog_print` 등)가 외부 환경(예: gbs build chroot)에서 실패하거나 방해되지 않도록 `main.cc` 혹은 `mock/` 헤더에서 빈 껍데기 매크로 처리하거나, `printf`로 우회(Redirect)하도록 리디파인(Redefine)합니다.
- 복잡한 내부 컴포넌트(LXC Container 엔진 등)의 경우 `gmock`을 사용하여 의존성을 제어하고 행위(Behavior) 위주로 테스트합니다.

## 개발 가이드
새로운 코드를 작성하면:
1. `test/unit_tests/` 에 해당 컴포넌트의 `_test.cc` 파일을 생성합니다.
2. `TEST_F` 나 `TEST` 매크로를 이용해 시나리오를 작성합니다.
3. CMakeLists에 소스를 추가하고, 로컬 환경 또는 `gbs build`를 돌려 `%check` 가 정상 통과하는지 확인합니다.
