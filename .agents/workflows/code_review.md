---
description: Code Review Checklist and Review-Fix Loop (max 5 iterations)
---

# Code Review Workflow

이 워크플로우는 코드 변경 사항에 대한 체계적인 리뷰 절차를 정의합니다.
Verify 단계 이후, Commit 이전에 반드시 수행해야 합니다.

## Review Checklist

변경된 모든 소스 파일에 대해 아래 10가지 카테고리를 순서대로 검토합니다.

### 1. 코딩 스타일 (Coding Style)
- `coding_rules.md`에 정의된 규칙 준수 여부 확인
- Google C++ Style Guide 준수 (2-space indent, PascalCase 함수명, snake_case 변수명 등)
- 멤버 변수 trailing underscore `_` 사용 여부
- 80자 컬럼 제한 준수
- `#include` 그룹 정렬 및 헤더 가드 형식

### 2. 무결성 (Correctness)
- 로직이 의도한 대로 동작하는지 확인
- 경계 조건(boundary conditions) 처리 여부
- 에러/예외 처리 누락 여부
- 반환값 검증 (`[[nodiscard]]` 활용 포함)
- nullptr/빈 컨테이너 등 엣지 케이스 처리

### 3. 메모리 이슈 (Memory Issues)
- 메모리 누수 (memory leak) 가능성
- Dangling pointer / Use-after-free
- Double-free
- `std::make_unique`/`std::shared_ptr` 올바른 사용 여부
- Raw `new`/`delete` 사용 지양 확인
- RAII 패턴 적용 여부

### 4. 성능 (Performance)
- 불필요한 객체 복사 (pass-by-value vs. pass-by-const-reference)
- O(n²) 이상의 비효율적 루프
- 불필요한 메모리 할당/해제 반복
- 락(lock) 경합 및 동기화 이슈
- `constexpr` 활용 가능 여부

### 5. 로직 문제 (Logic Issues)
- 데드코드 (dead code) 존재 여부
- 도달 불가능한 분기 (unreachable branch)
- 잘못된 조건식 또는 불완전한 분기 처리
- 변수 섀도잉 (`-Wshadow` 위반)
- 깊은 중첩(3+ level) — Guard Clause로 리팩터링 필요 여부

### 6. 보안 (Security)
- 외부 입력 검증 누락 (input validation)
- 버퍼 오버플로우 가능성
- 권한 에스컬레이션 취약점
- 인젝션 공격 벡터 (command injection, path traversal 등)
- 하드코딩된 비밀값(credentials) 존재 여부
- `tizen-manifest.xml` 권한 설정 적절성

### 7. 스레드 안전성 (Thread Safety)
- Race condition 가능성 (공유 자원 동시 접근)
- Deadlock 위험 (다중 락 획득 순서)
- GLib 이벤트 루프에서의 콜백 스레드 안전성
- `std::mutex`/`std::lock_guard` 적절한 사용 여부
- Atomic 연산이 필요한 곳에서의 일반 변수 사용
- 비동기 작업 완료 전 객체 소멸 가능성

### 8. 리소스 관리 (Resource Management)
- 파일 디스크립터(fd) 누수 여부
- 소켓, D-Bus 연결 등 시스템 리소스 해제 확인
- GLib 리소스 관리 (`g_free`, `g_object_unref`, `g_variant_unref` 등)
- 컨테이너 라이프사이클 (mount/unmount, 정리 작업) 누락
- RAII 패턴 미적용 시 예외 안전성 확보 여부

### 9. 테스트 커버리지 (Test Coverage)
- 변경된 코드에 대한 gtest 추가/수정 여부
- 새로운 public 함수의 단위 테스트 존재 여부
- 경계 조건 및 에러 경로에 대한 테스트 포함 여부
- 기존 테스트가 변경 사항에 의해 깨지지 않는지 확인

### 10. 에러 전파 및 로깅 (Error Propagation & Logging)
- `dlog_print` 적절한 활용 (DLOG_ERROR, DLOG_WARN, DLOG_INFO 레벨)
- 에러 발생 시 호출자에게 올바르게 전파되는지 확인
- 디버깅에 충분한 맥락 정보가 로그에 포함되는지
- 에러 무시(silent failure) 여부 — 최소한 로그는 남겨야 함
- 민감 정보가 로그에 노출되지 않는지 확인

## Review-Fix Loop

문제 발견 시 아래 루프를 따릅니다. **최대 5회까지 반복**합니다.

```
┌─────────────────────────────────────────────────┐
│                  Review-Fix Loop                │
│                  (max 5 iterations)             │
│                                                 │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│   │ Develop  │───▶│  Verify  │───▶│  Review  │  │
│   │ (수정)   │    │ (빌드/   │    │ (체크    │  │
│   │          │    │  배포/   │    │  리스트) │  │
│   │          │    │  테스트) │    │          │  │
│   └──────────┘    └──────────┘    └─────┬────┘  │
│        ▲                                │       │
│        │          FAIL                  │       │
│        └────────────────────────────────┘       │
│                                 │               │
│                            PASS │               │
│                                 ▼               │
│                          ┌──────────┐           │
│                          │  Commit  │           │
│                          └──────────┘           │
└─────────────────────────────────────────────────┘
```

### 절차

1. 변경된 모든 파일에 대해 위 10가지 체크리스트를 순서대로 검토한다.
2. **PASS**: 모든 항목에서 문제가 없으면 Commit 단계로 진행한다.
3. **FAIL**: 하나 이상의 문제가 발견되면:
   - 발견된 문제를 명확히 기록한다 (카테고리, 파일명, 라인번호, 설명).
   - **Develop** 단계로 돌아가 해당 문제를 수정한다.
   - `deploy.sh`로 빌드/배포 후 **Verify** 단계를 다시 수행한다.
   - 수정된 코드에 대해 **다시 Review**를 수행한다.
4. 이 루프는 **최대 5회**까지 반복한다.
5. 5회 반복 후에도 문제가 남아 있으면, 사용자에게 **에스컬레이션**하여 판단을 요청한다.

> [!CAUTION]
> 5회 반복 초과 시 무한 루프 방지를 위해 반드시 사용자에게 보고하고
> 추가 진행 여부에 대한 판단을 받아야 합니다.

### 리뷰 결과 기록 형식

각 반복(iteration)마다 결과를 아래 형식으로 기록합니다:

```
## Review Iteration N/5

| Category | File | Line | Severity | Description |
|----------|------|------|----------|-------------|
| Memory   | foo.cc | 42 | HIGH | potential use-after-free |
| Style    | bar.hh | 15 | LOW  | missing trailing underscore |

**Result**: FAIL → Returning to Develop step
```

Severity 등급:
- **HIGH**: 반드시 수정 (메모리 이슈, 보안 취약점, 크래시 유발)
- **MEDIUM**: 수정 권장 (성능, 로직 오류)
- **LOW**: 개선 권장 (스타일, 코드 정리)
