---
description: Tizen gbs build workflow
---

# TizenClaw GBS Build Workflow

프로젝트 빌드 관련 변경 사항(코드 수정, CMakeLists.txt 수정, 패키징 스펙 변경)이 발생하면 아래 순서대로 자동 빌드를 수행하고 검증하세요.

1. **빌드를 실행합니다**: Tizen `gbs build`는 git repository의 commit된 소스를 기준으로 tarball을 만들지만, `--include-all` 옵션을 주면 커밋하지 않은 사항들도 포함하여 빌드합니다.
   명령어: `gbs build -A x86_64 --include-all`

2. **빌드 로그 확인**:
   - 성공 시: `~/GBS-ROOT/local/repos/tizen/x86_64/logs/success/`
   - 실패 시: `~/GBS-ROOT/local/repos/tizen/x86_64/logs/fail/`

   위 디렉토리 아래에 생성되는 로그 파일을 통해 빌드 결과를 검증할 수 있습니다.

// turbo-all
