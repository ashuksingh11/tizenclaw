---
description: Tizen Crash Dump 디버깅 워크플로우 (sdb shell + gdb)
---

# Crash Dump 디버깅 워크플로우

tizenclaw 프로세스에서 crash가 발생했을 때, 디바이스에 저장된 crash dump 파일을 이용하여 GDB로 디버깅하는 절차입니다.

## 사전 조건
- `sdb devices`로 디바이스가 연결되어 있어야 합니다.
- `sdb root on`으로 root 권한을 확보합니다.

## 디버깅 절차

### 1. 디바이스 셸 진입
```bash
sdb shell
```

### 2. crash dump 디렉터리로 이동
```bash
cd /opt/usr/share/crash/dump/
ls
```
- `tizenclaw_<PID>_<TIMESTAMP>.zip` 형태의 파일을 확인합니다.

### 3. dump 파일 압축 해제
```bash
unzip tizenclaw_<PID>_<TIMESTAMP>.zip
cd tizenclaw_<PID>_<TIMESTAMP>
```

### 4. coredump tarball 해제
```bash
tar -xvf tizenclaw_<PID>_<TIMESTAMP>.coredump.tar
ls
```
- `tizenclaw_<PID>_<TIMESTAMP>.coredump` 파일이 생성됩니다.

### 5. GDB로 디버깅 시작
```bash
gdb /usr/bin/tizenclaw tizenclaw_<PID>_<TIMESTAMP>.coredump
```

### 6. GDB 주요 명령어
| 명령어 | 설명 |
|---|---|
| `bt` | 전체 백트레이스 출력 |
| `bt full` | 지역 변수 포함 백트레이스 |
| `info threads` | 전체 스레드 목록 |
| `thread apply all bt` | 모든 스레드 백트레이스 |
| `frame <N>` | 특정 프레임으로 이동 |
| `info registers` | 레지스터 값 확인 |
| `quit` | GDB 종료 |

## 예시 (전체 흐름)
```bash
sdb shell
cd /opt/usr/share/crash/dump/
unzip tizenclaw_74434_20260305224907.zip
cd tizenclaw_74434_20260305224907
tar -xvf tizenclaw_74434_20260305224907.coredump.tar
gdb /usr/bin/tizenclaw tizenclaw_74434_20260305224907.coredump
```

GDB 진입 후:
```
(gdb) bt
(gdb) bt full
(gdb) quit
```
