---
description: SDB Device and Emulator Development Workflow
---

# SDB (Smart Development Bridge) Development Skill

Tizen 에뮬레이터 또는 실-디바이스 연결 환경에서 개발 및 디버깅을 진행할 때 사용하는 워크플로(Skill)입니다.
터미널 환경에서 Tizen 타겟 디바이스에 앱을 설치하거나 루트 권한으로 파일시스템을 통제해야 할 때 이 문서의 절차를 따릅니다.

## 1. 디바이스 연결 확인
현재 연결된 Tizen Emulator나 디바이스 리스트를 확인합니다.
```bash
sdb devices
```
* **결과 확인**: `emulator-26101` 또는 특정 USB 디바이스 ID가 `device` 상태로 표시되는지 확인해야 합니다. 만약 `offline` 이라면 Tizen 에뮬레이터 관리자에서 에뮬레이터를 재시작해야 합니다.

## 2. 루트 권한 획득 (Root On)
기본적으로 Tizen은 보안상의 이유로 타겟 쉘 진입 시 제한적 권한(`owner` 유저)을 부여합니다. 
루트 파일시스템 마운트나 시스템 파일 조작, 컨테이너 엔진(`crun`/`lxc`) 세팅을 위해서는 루트 권한을 획득해야 합니다.
```bash
sdb root on
```
*성공 시 `Switched to 'root' account mode` 와 같은 메시지가 출력됩니다.*

## 3. 루트 파일시스템 Read-Write 마운트
Tizen의 루트 파일시스템(`/`)이나 코어 시스템 파티션은 기본적으로 Read-Only(읽기 전용)로 마운트되어 있습니다. Daemon 설치나 런타임 종속성을 수정하려면 쓰기 가능(`rw`) 모드로 다시 마운트해야 합니다.
```bash
sdb shell mount -o remount,rw /
```
이후 `sdb shell` 로 진입하여 필요한 파일 수정, gdb 디버깅, 컨테이너 모듈 설치 작업을 `root` 권한으로 수행할 수 있습니다.

## 요약 스크립트 실행
TizenClaw Agent가 개발 편의를 위해 환경을 자동 셋업할 필요가 있을 때 다음 순차적 명령(sh)을 실행합니다.

```bash
sdb devices
sdb root on
sdb shell mount -o remount,rw /
```

// turbo-all
