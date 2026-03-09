---
description: Tizen gbs build workflow
---

# TizenClaw GBS Build Workflow

프로젝트 빌드 관련 변경 사항(코드 수정, CMakeLists.txt 수정, 패키징 스펙 변경)이 발생하면 아래 순서대로 자동 빌드를 수행하고 검증하세요.

1. **빌드를 실행합니다**: Tizen `gbs build`는 git repository의 commit된 소스를 기준으로 tarball을 만들지만, `--include-all` 옵션을 주면 커밋하지 않은 사항들도 포함하여 빌드합니다.
   - **아키텍처 감지**: `sdb capability`의 `cpu_arch` 필드로 자동 감지 (감지 실패 시 `x86_64` 폴백)
     ```bash
     ARCH=$(sdb capability 2>/dev/null | grep '^cpu_arch:' | cut -d':' -f2)
     [ -z "${ARCH}" ] && ARCH=x86_64
     ```
   - **빌드 명령어**: `gbs build -A ${ARCH} --include-all`

   > [!CAUTION]
   > `--noinit` 옵션을 사용하지 마세요. 의존성 설치 누락 등 빌드 환경 불일치 문제가 발생할 수 있습니다. 항상 초기화를 포함한 전체 빌드를 수행하세요.

2. **빌드 완료 확인**: 빌드가 정상적으로 완료되면 마지막에 `info: Done` 메시지가 출력됩니다. 이 메시지가 나타나면 빌드 성공입니다.

3. **빌드 로그 확인**:
   - 성공 시: `~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/success/`
   - 실패 시: `~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/fail/`

   위 디렉토리 아래에 생성되는 로그 파일을 통해 빌드 결과를 검증할 수 있습니다.

4. **주의: 파이프(`|`) 사용 금지**
   `gbs build` 명령어의 출력을 `| tail`, `| grep` 등 파이프로 필터링하지 마세요. 파이프가 출력을 버퍼링하여 빌드가 멈춘 것처럼 보이는 현상이 발생합니다.
   - ❌ `gbs build -A ${ARCH} --include-all 2>&1 | tail -50`
   - ✅ `gbs build -A ${ARCH} --include-all 2>&1`

   빌드 결과 확인이 필요한 경우, 빌드 완료 후 로그 파일을 직접 확인하세요.

5. **AGENT 전용: 빌드 완료 감지 방법**
   `gbs build`는 내부적으로 `sudo`와 `chroot` 환경에서 실행되므로, `command_status` 도구가 stdout/stderr를 전혀 캡처하지 못합니다 (항상 "No output"으로 표시됨).

   따라서 다음 방법으로 빌드 완료를 감지하세요:
   ```bash
   # 빌드 실행 (WaitMsBeforeAsync=3000 으로 백그라운드 전환)
   gbs build -A ${ARCH} --include-all 2>&1

   # 빌드 완료 감지: RPM 파일의 수정시간 확인 (폴링)
   # command_status의 WaitDurationSeconds=60 으로 설정하고,
   # 최대 5회 반복하여 RPM 파일이 최신인지 확인
   stat -c '%Y' ~/GBS-ROOT/local/repos/tizen/${ARCH}/RPMS/tizenclaw-*.${ARCH}.rpm 2>/dev/null
   ```

   **빌드 성공/실패 판별:**
   - 성공: `~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/success/` 디렉토리에 최신 로그 확인
   - 실패: `~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/fail/` 디렉토리에 최신 로그 확인
   ```bash
   # 빌드 결과 확인 (성공 여부)
   ls -lt ~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/success/ 2>/dev/null | head -5
   ls -lt ~/GBS-ROOT/local/repos/tizen/${ARCH}/logs/fail/ 2>/dev/null | head -5
   ```

// turbo-all
