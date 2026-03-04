---
description: Tizen gbs build workflow
---

# TizenClaw GBS Build Workflow

프로젝트 빌드 관련 변경 사항(코드 수정, CMakeLists.txt 수정, 패키징 스펙 변경)이 발생하면 아래 순서대로 자동 빌드를 수행하고 검증하세요.

1. **변경 사항 커밋 대기열 올리기**: Tizen `gbs build`는 git repository의 commit된 소스를 기준으로 tarball을 만들기 때문에 작업 내용이 있으면 반드시 사전에 `git commit`을 하거나 `--include-all` 옵션을 고려해야 합니다.
2. 커밋을 진행합니다: `git add . && git commit -m "Auto-commit before build"`
3. GBS 빌드를 실행합니다: `gbs build`

// turbo-all
