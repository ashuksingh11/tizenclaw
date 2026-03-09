# Task: Implement Tool Routing Guide for LLM

## Objectives
- Enhance `tools/routing_guide.md` with comprehensive tool selection logic.
- Integrate the routing guide into the LLM's system prompt dynamically.
- Ensure the guide is correctly packaged and deployed to the device.
- Verify LLM's adherence to the guide via the emulator.

## Detailed Plan

### 1. Analysis & Refinement
- [ ] Review all available tool categories: `embedded`, `skills`, `custom_skills`, `actions`.
- [ ] Refine `tools/routing_guide.md` with explicit priority rules and usage patterns.

### 2. Implementation (Develop)
- [ ] Update `tools/routing_guide.md` with detailed instructions.
- [ ] Modify `src/tizenclaw/core/agent_core.cc`:
    - Add `LoadRoutingGuide()` helper method.
    - Update `BuildSystemPrompt()` to append the routing guide content.
- [ ] Update `src/tizenclaw/CMakeLists.txt` to install `tools/routing_guide.md` to `/opt/usr/share/tizenclaw/tools/`.
- [ ] Update `packaging/tizenclaw.spec` to include the routing guide in the RPM.

### 3. Local Verification
- [ ] Run `gbs build` to ensure the project compiles and unit tests pass.
- [ ] Verify that `routing_guide.md` is correctly included in the build root.

### 4. Device Verification
- [ ] Deploy the RPM to the emulator.
- [ ] Verify that the system prompt includes the routing guide content (check logs).
- [ ] Test LLM tool selection logic using `tizenclaw-cli chat` with scenarios like:
    - "Set display brightness to 50" (should prefer `action_` if available).
    - "List all running apps" (should use `list_apps`).
    - "Search for Tizen documentation" (should use `web_search`).

### 5. Completion
- [ ] Commit changes following `commit_guidelines.md`.
- [ ] Push to main.
