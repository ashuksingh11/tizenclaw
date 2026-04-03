# Agent Loop V2: Cognitive Planning & Dynamic Fallback

## Objective
To augment the TizenClaw autonomous agent daemon with explicit task decomposition (Plan-and-Solve) and proactive error recovery/fallback mechanisms, moving beyond simple reactive ReAct strategies.

## Inputs & Outputs
- **Input:** Raw user prompt via `SessionStore`.
- **Output:** Structured `plan_steps` injected into `AgentLoopState`, and responsive alternative strategies when trapped in an idle loop (`EvalVerdict::Stuck`).
- **Resource Constraints consideration:** The explicit Planning phase requires an additional LLM call at the start of a session. To prevent CPU/Power spikes, this is only triggered if the prompt string length or intent implies a complex multi-step task.

## Daemon Integration Planning
- **Module Convention:** `tizenclaw/src/core/` (Specifically `agent_core.rs` and `agent_loop_state.rs`).
- **Execution Mode:** **Daemon Sub-task** (Part of the continuous async core processing loop).
- **Fallback Constraints:** If the explicitly generated plan fails repeatedly, the fallback module (Dynamic Fallback) inserts a strong negative prompt overriding previous memory actions to force a new behavioral pathway.

## State Models
1.  **Phase 3 (Planning)**:
    - Analyzes `prompt`.
    - If `complex`, calls LLM with system prompt instructing it to return a JSON array of steps.
    - Parses and saves to `AgentLoopState.plan_steps`.
2.  **Phase 7 (Evaluating) -> Phase 8 (RePlanning)**:
    - Instead of bailing on `Stuck`, it pushes a system-level observation into the context: *"System Error: You are stuck in a loop. Re-evaluate your plan and use a different tool."* and proceeds to next round.
