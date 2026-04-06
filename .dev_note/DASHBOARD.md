# TizenClaw Dashboard

## Current Task
- Restore the generated web app flow
- Reconnect `tools/embedded/generate_web_app.md` to live Rust execution
- Match legacy `tizenclaw-cpp` behavior and `tizenclaw-webview` launch path
- Verify end-to-end launch with deployed `tizenclaw-webview`
- Keep host builds unaffected by Tizen-only launch support
- Route semantic dashboard app requests to `generate_web_app` even when
  the user does not explicitly say "web app"

## Stage 1: Planning
- Status: Complete
- Goal:
  - Recover the legacy generated web app lifecycle in Rust
  - Support app file generation under `/web/apps/<app_id>`
  - Restore dashboard-side app listing/detail/delete and Bridge API access
  - Reuse `tizenclaw-webview` launch behavior when available
- Notes:
  - `tools/embedded/generate_web_app.md` exists but runtime handling is missing
  - Rust `tizenclaw-web-dashboard` currently exposes only app list/detail
  - Legacy C++ implementation writes `manifest.json`, downloads assets,
    exposes bridge endpoints, and auto-launches bridge/webview apps

## Stage 2: Design
- Status: Complete
- Planned changes:
  - Add `generate_web_app` builtin declaration to workflow tools
  - Implement web app generation in `AgentCore` with manifest/assets support
  - Add IPC methods so `tizenclaw-web-dashboard` can execute bridge tools
    and enumerate allowed tools through the daemon
  - Extend `tizenclaw-web-dashboard` with `/api/apps` delete and
    `/api/bridge/{tool,tools,data,chat}` endpoints
  - Keep SSE bridge events out of scope unless required by build/test
- Risk notes:
  - Preserve current standalone dashboard process model
  - Keep path validation strict to avoid traversal via app ids or filenames
  - Use best-effort Tizen app launch so non-Tizen host paths do not panic

## Stage 3: Development
- Status: Complete
- Implemented:
  - Added `generate_web_app` builtin declaration back into workflow tools
  - Restored Rust-side web app generation in `AgentCore`
  - Added manifest writing, optional asset download, and app listing metadata
  - Added bridge IPC methods for tool execution and tool enumeration
  - Extended web dashboard with `/api/apps/:id` delete and
    `/api/bridge/{tool,tools,data,chat}` endpoints
  - Added bridge compatibility alias for `execute_cli` so legacy
    generated apps keep working without host-only build fallout
  - Kept Tizen-only app launch as best-effort runtime behavior guarded
    by Tizen environment detection

## Stage 4: Build & Deploy
- Status: Complete
- Command:
  - `./deploy.sh -a x86_64 -d emulator-26101`
  - `~/samba/github/tizenclaw-webview/deploy.sh -d emulator-26101`
- Results:
  - GBS build succeeded
  - Device deployment succeeded
  - Service restart succeeded
  - Companion webview app build/deploy succeeded after aligning
    pkg-config dependency with `chromium-efl`
- Service proof:
  - `tizenclaw.service`: active (running)
  - `tizenclaw-tool-executor.socket`: active (listening)

## Stage 5: Test & Review
- Status: Complete
- Build-time test proof:
  - `cargo test --release --offline -- --test-threads=1` ran inside
    the deploy pipeline
  - Main test suite passed: `182 passed; 0 failed`
  - `git diff --check` passed after edits
- Runtime verification:
  - Created `webtest_demo` through direct daemon `bridge_tool` IPC
    using `generate_web_app`
  - Confirmed app files under `/opt/usr/share/tizenclaw/web/apps/webtest_demo`
  - Confirmed manifest persisted `allowed_tools: ["execute_cli"]`
  - Confirmed `GET /api/apps` and `GET /api/apps/webtest_demo` return
    the generated app metadata
  - Confirmed `GET /api/bridge/tools?app_id=webtest_demo` returns
    legacy-compatible `execute_cli`
  - Confirmed `POST /api/bridge/tool` successfully ran
    `tizen-device-info-cli battery`
  - Confirmed `POST/GET /api/bridge/data` round-trip works for
    `app_id=webtest_demo`, `key=mode`
  - Deployed and registered `org.tizen.tizenclaw-webview` on
    `emulator-26101`
  - Verified manual launch with
    `app_launcher -s org.tizen.tizenclaw-webview __APP_SVC_URI__ ...`
    succeeds on device
  - Updated Rust launch flow to prefer legacy-compatible AUL bundle launch
    for `QvaPeQ7RDA.tizenclawbridge` and `org.tizen.tizenclaw-webview`
  - Created `webtest_autorun2` via daemon IPC after redeploy and verified
    the webview process auto-starts with
    `__APP_SVC_URI__=http://localhost:9090/apps/webtest_autorun2/`
  - Verified `dlogutil` from `TIZENCLAW_WEBVIEW` reports the generated app
    URI, proving auto-launch is wired through at runtime
  - Confirmed non-Tizen host path now falls back to a user-facing host URL
    message instead of depending on Tizen launch support
  - Identified a generated app regression where separate `css` and `js`
    files were written but not auto-linked into `index.html` when the
    model omitted explicit `<link>` or `<script>` tags
  - Fixed `generate_web_app` so generated apps under
    `/opt/usr/share/tizenclaw/web/apps/<app_id>/` auto-inject
    `style.css` and `app.js` references when needed
  - Hardened tool guidance so browser UI requests prefer
    `generate_web_app` instead of `run_generated_code`
  - Added semantic dashboard request detection so creation and update
    prompts like games, screens, dashboards, and UI edits are routed to
    `generate_web_app` without requiring explicit "web app" wording
  - Added a fallback that parses assistant JSON app specs and executes
    `generate_web_app` server-side when the model returns app payloads as
    plain text instead of calling the tool directly
  - Verified `/api/apps/<id>` delete removes the app directory from
    `/opt/usr/share/tizenclaw/web/apps/<app_id>` using `webtest_delete`
  - Re-verified on `emulator-26101` that a prompt asking for a
    Tetris game without saying "web app" created
    `/opt/usr/share/tizenclaw/web/apps/webtest_semantic_a`
  - Re-verified that a follow-up "modify/improve" prompt for the same
    `app_id` updated `index.html`, relaunched webview, and reflected the
    requested `Score Board` text
  - Re-verified that `DELETE /api/apps/webtest_semantic_a` removed the
    generated app directory and removed it from `/api/apps`
- Runtime log proof:
  - `tizenclaw.service`: active (running)
  - `tizenclaw-tool-executor.socket`: active (listening)

## Supervisor Gate Log
- Stage 1 PASS: Web app restoration scope recorded in dashboard
- Stage 2 PASS: Rust/CPP parity design captured before edits
- Stage 3 PASS: Rust web app generation and bridge routes restored
- Stage 4 PASS: x86_64 deploy completed on emulator-26101
- Stage 5 PASS: Generated app, bridge API, and webview auto-launch
  verified on device
  Additional PASS: app delete API removes generated app directories under
  the legacy C++-compatible web apps path
  Additional PASS: semantic dashboard prompts now route to web app
  generation and update flows without explicit "web app" wording
  Additional PASS: semantic update prompts modified the generated app and
  app delete removed the device files and dashboard listing

## Stage 6: Commit & Push
- Status: Complete
- Notes:
  - Previous restoration commit: `e57fb0da`
    `Restore generated web app bridge flow`
  - Workspace cleaned with `.agent/scripts/cleanup_workspace.sh`
  - Removed untracked local `.pc` artifacts before staging
  - Removed tracked local CLI ELF binaries from `tools/cli/*` and added
    ignore rules so regenerated executables do not get committed again
  - Selected only semantic routing and IPC cleanup files for commit to
    avoid unrelated local `tools/cli/*` changes
  - Commit created with `.tmp/commit_msg.txt` and `git commit -F`
    following repository message rules
  - Commit stage prepared for branch-local completion without push
    because the user requested commit creation only
  - Stage 6 PASS: workspace cleaned, extraneous files removed, and
    commit prepared with `.tmp/commit_msg.txt`
