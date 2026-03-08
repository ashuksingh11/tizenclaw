---
description: Main Development Workflow (Plan -> Develop -> Verify)
---

# TizenClaw Main Development Workflow

이 워크플로우는 TizenClaw 프로젝트의 핵심 개발 프로세스(계획 -> 개발 -> 검증)를 정의합니다. AGENT는 항상 이 프로세스를 따라 작업을 수행해야 합니다.

## 1. Plan (계획)
- 목적과 요구사항을 정확히 파악합니다.
- 기존 코드 분석 및 적용 가능한 워크플로우(`/coding_rules`, `/commit_guidelines` 등)를 확인합니다.
- 구현 전에 작업 단위(`task.md`)를 작성하고 세부 계획을 세웁니다.

## 2. Develop (개발 & 로컬 검증)
- 소스 코드를 수정하고 단위 테스트를 추가/수정합니다.
- `gbs_build.md` 워크플로우를 참조하여 로컬에서 코드를 빌드하고 검증합니다.
  - 명령어: `gbs build -A x86_64 --include-all`
- `gtest_integration.md` 워크플로우를 참조하여 컴포넌트 단위 검증이 통과하는지 확인합니다.

## 3. Verify (기기 배포 및 검증)
작성 및 로컬 검증이 완료된 코드는 실제 타겟(Tizen Emulator 또는 실기기)에 배포하여 동작을 확인해야 합니다.
해당 작업은 `deploy_to_emulator.md` 워크플로우를 따릅니다.

> [!TIP]
> 배포 후 crash가 발생한 경우 `crash_debug.md` 워크플로우를 참조하여 crash dump를 분석합니다.

1. **디바이스 연결 확인**
   - 명령어: `sdb devices`
   - 타겟 디바이스가 `device` 상태인지 확인합니다.

2. **권한 및 쓰기 권한 확보**
   - 명령어: `sdb root on`
   - 명령어: `sdb shell mount -o remount,rw /`

3. **RPM 패키지 배포 및 설치**
   - 명령어: `sdb push ~/GBS-ROOT/local/repos/tizen/x86_64/RPMS/tizenclaw-1.0.0-1.x86_64.rpm /tmp/`
   - 명령어: `sdb shell rpm -Uvh --force /tmp/tizenclaw-1.0.0-1.x86_64.rpm`

4. **데몬 재시작 및 상태 확인**
   - 명령어: `sdb shell systemctl daemon-reload`
   - 명령어: `sdb shell systemctl restart tizenclaw`
   - 명령어: `sdb shell systemctl status tizenclaw -l`

## 4. Commit (작업 완료)
모든 검증이 끝났다면 `commit_guidelines.md` 워크플로우에 맞추어 `git commit`을 수행하여 작업을 마무리합니다.
상세한 규칙은 해당 워크플로우를 참조하되, 핵심 사항은 아래와 같습니다.

### 커밋 메시지 기본 구조
Conventional Commits 스타일로 작성하며, **커밋 메시지는 반드시 영어(English)로** 작성합니다.

```text
Title (Under 50 chars, clear and concise English)

Provide a detailed explanation of the implemented features, bug fixes,
or structural changes. Describe 'Why' and 'What' was done extensively
but clearly. (Wrap text at 72 characters)
```

### 작성 예시 (Good)
```text
Switch from LXC to lightweight runc for ContainerEngine

Refactored the ContainerEngine implementation to use the lightweight
`runc` CLI via `std::system` instead of relying on `liblxc` APIs.
This change was necessary because the Tizen 10 GBS build environment
does not provide the `pkgconfig(lxc)` dependency.
```

### 금지 사항
- Verification/Testing Results 블록 등 기계적 텍스트는 커밋 메시지에 **절대 포함하지 않습니다.**
- 봇이 생성하는 불필요하고 장황한 문구를 넣지 않습니다.

### 커밋 타이밍
1. 문서에 명시된 단위 기능 1개가 구현됨
2. `gbs build`(내부 `%check`의 gtest 포함)가 에러 없이 통과됨
3. `git add .` 후 상기 포맷에 맞춰 `git commit` 수행

---

## ⚠️ Shell 명령 실행 시 주의사항 (AGENT 필독)

이 프로젝트의 개발 환경(WSL)에서 `run_command` 도구로 실행한 명령의 stdout/stderr가 `command_status`에 캡처되지 않는 경우가 빈번합니다. 특히 `sudo`, `chroot`, `fakeroot` 등을 내부적으로 사용하는 `gbs build`는 **항상 "No output"**으로 표시됩니다.

### ❌ 절대 하지 말 것
1. **파이프(`|`) 사용 금지** — `tail`, `grep` 등으로 파이프하면 버퍼링으로 인해 출력이 전혀 보이지 않습니다.
   - ❌ `gbs build -A x86_64 --include-all 2>&1 | tail -50`
   - ❌ `git log | head -20`
2. **"No output"을 "멈춤"으로 판단하지 말 것** — 명령은 정상 실행 중일 수 있습니다.
3. **출력이 없다고 반복적으로 `command_status`를 호출하지 말 것** — 불필요한 대기를 유발합니다.

### ✅ 올바른 패턴

#### `gbs build` 빌드 완료 감지
```bash
# 1. 빌드 실행 (WaitMsBeforeAsync=3000)
gbs build -A x86_64 --include-all 2>&1

# 2. RPM 수정시간 폴링으로 완료 감지 (command_status WaitDurationSeconds=60, 최대 5회)
stat -c '%Y' ~/GBS-ROOT/local/repos/tizen/x86_64/RPMS/tizenclaw-*.x86_64.rpm 2>/dev/null

# 3. 빌드 결과 확인
ls -lt ~/GBS-ROOT/local/repos/tizen/x86_64/logs/success/ 2>/dev/null | head -3
ls -lt ~/GBS-ROOT/local/repos/tizen/x86_64/logs/fail/ 2>/dev/null | head -3

# 4. 테스트 결과 확인
grep -E "PASSED|FAILED|tests from" ~/GBS-ROOT/local/repos/tizen/x86_64/logs/success/tizenclaw-*/log.txt | tail -5
```

#### `git` 명령 실행
```bash
# git add, commit, push 등은 WaitMsBeforeAsync=10000 으로 설정하여
# 동기적으로 완료될 수 있도록 합니다.
# 출력이 없어도 Exit code: 0 이면 성공입니다.
git add <files>
git commit -m "message"
git push origin main
```

#### 일반 shell 명령
```bash
# sdb, curl 등 짧은 명령은 WaitMsBeforeAsync=5000~10000 설정
# 출력이 캡처되지 않으면 Exit code로 성공 여부 판단
```

> [!CAUTION]
> `command_status`에서 "No output"이 반복되면 **명령이 정상 실행 중**일 가능성이 높습니다. 절대로 "멈춤"으로 판단하여 terminate하거나 동일 명령을 재실행하지 마세요. 위의 폴링 패턴을 사용하세요.

---

## 워크플로우 참조 목록
본 AGENTS 워크플로우에서 참조하는 세부 워크플로우 파일 목록입니다.

| 워크플로우 | 파일 | 참조 단계 |
|---|---|---|
| 코딩 규칙 | `coding_rules.md` | Plan |
| 커밋 가이드라인 | `commit_guidelines.md` | Commit |
| GBS 빌드 | `gbs_build.md` | Develop |
| GTest 단위 테스트 | `gtest_integration.md` | Develop |
| 에뮬레이터 배포 | `deploy_to_emulator.md` | Verify |
| Crash Dump 디버깅 | `crash_debug.md` | Verify |