---
description: Deploy RPM to Tizen Emulator over sdb
---

# Deploy TizenClaw to Emulator

이 워크플로우는 `gbs build`를 통해 생성된 RPM 패키지를 로컬에 연결된 Tizen Emulator에 설치(업데이트)하고 데몬을 재시작하는 과정을 자동화합니다. 

기본 전제:
- Tizen Emulator 또는 실기기가 켜져 있어야 합니다. (`sdb devices`로 확인 가능)
- 디바이스 아키텍처는 `sdb capability`의 `cpu_arch` 필드로 자동 감지합니다. (예: `x86_64`, `aarch64`)
- `gbs build -A <arch>` (또는 `--include-all` 옵션)가 성공적으로 완료되어 `~/GBS-ROOT/local/repos/tizen/<arch>/RPMS/`에 대상 RPM이 존재해야 합니다.

아래 단계들을 순서대로 실행하세요.

1. **루트 권한 획득**
   ```bash
   sdb root on
   ```

2. **루트 파일시스템 읽기/쓰기 모드 재마운트**
   에뮬레이터의 `/` (루트 파티션)는 기본적으로 읽기 전용입니다. RPM 설치를 위해 쓰기 모드로 변경합니다.
   ```bash
   sdb shell mount -o remount,rw /
   ```

3. **RPM 파일 푸시 및 설치**
   최신 빌드된 TizenClaw 패키지를 디바이스의 `/tmp/` 경로로 전송한 후 강제 설치합니다.
   ```bash
   # 아키텍처 자동 감지
   ARCH=$(sdb capability 2>/dev/null | grep '^cpu_arch:' | cut -d':' -f2)
   [ -z "${ARCH}" ] && ARCH=x86_64

   # 빌드 결과물 전송 및 설치
   sdb push ~/GBS-ROOT/local/repos/tizen/${ARCH}/RPMS/tizenclaw-1.0.0-1.${ARCH}.rpm /tmp/
   sdb shell rpm -Uvh --force /tmp/tizenclaw-1.0.0-1.${ARCH}.rpm
   ```

4. **데몬 재시작 및 상태 확인**
   새로 설치된 systemd 데몬을 로드하고 재가동합니다.
   ```bash
   sdb shell systemctl daemon-reload
   sdb shell systemctl restart tizenclaw
   sdb shell systemctl status tizenclaw -l
   ```

// turbo-all
