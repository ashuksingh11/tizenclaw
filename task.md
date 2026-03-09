# Task: Move skills to tools directory

## Objectives
- [x] Move the `skills/` directory to `tools/skills/`.
- [x] Update all source code, scripts, build configuration, and documentation that reference the `skills/` directory.
- [x] Verify the changes via local build and device deployment.

## Detailed Plan

### 1. Analysis & Preparation
- [x] List all files in the current `skills/` directory.
- [x] Confirm the target structure: `tools/skills/`.
- [x] Create `task.md` (this file).

### 2. Implementation (Develop)
- [x] Move `skills/` to `tools/skills/`.
- [x] Update `CMakeLists.txt` and `src/tizenclaw/CMakeLists.txt`.
- [x] Update C++ source files:
  - `src/tizenclaw/core/agent_core.cc`
  - `src/tizenclaw/core/tizenclaw.cc`
  - `src/tizenclaw/channel/web_dashboard.cc`
  - `src/tizenclaw/channel/mcp_server.cc`
  - `src/tizenclaw/infra/container_engine.cc`
  - `src/tizenclaw/core/skill_watcher.hh`
- [x] Update Python scripts:
  - `skills/skill_executor.py` (now `tools/skills/skill_executor.py`)
- [x] Update shell scripts:
  - `scripts/skills_secure_container.sh`
  - `scripts/ci_build.sh`
- [x] Update packaging files:
  - `packaging/tizenclaw.spec`
- [x] Update documentation:
  - `README.md`
  - `docs/ROADMAP.md`
  - `docs/ROADMAP_KOR.md`
  - `tools/embedded/file_manager.md`
- [x] Update Dockerfile:
  - `scripts/Dockerfile`

### 3. Local Verification
- [x] Run `gbs build` for the emulator architecture (`x86_64`).
- [x] Check if unit tests pass during the build.

### 4. Device Verification
- [x] Deploy the RPM to the emulator.
- [x] Restart the `tizenclaw` service.
- [x] Verify using `tizenclaw-cli` that tools (formerly skills) are still correctly discovered and executable.

### 5. Completion
- [ ] Commit changes following `commit_guidelines.md`.
