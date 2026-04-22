//! Context Engine — Size-based context window pressure management.
//!
//! Controls when and how conversation history is compacted when a positive
//! token budget is configured. Uses a three-phase compaction strategy:
//!
//! ## Compaction Trigger
//! Compaction is triggered when estimated token usage reaches or exceeds
//! `compact_threshold` × `budget`.
//!
//! ## Compaction Phases
//! 1. **Pin**: Always keep the system prompt (role="system") and the original
//!    user request (first role="user" message). These are never removed.
//! 2. **Prune**: Drop `tool` result messages that are not referenced in any
//!    later `assistant` message. These are safe to discard.
//! 3. **Truncate**: If still over budget, drop the oldest non-pinned messages
//!    (excluding the most recent 30%) until under threshold.
//!
//! ## Token Estimation
//! - Primary: `WordPieceTokenizer` when vocabulary is loaded (accurate).
//! - Fallback: `chars / 3.5` heuristic when tokenizer is unavailable.

use crate::llm::backend::LlmMessage;
use crate::core::wordpiece_tokenizer::WordPieceTokenizer;
use std::collections::HashSet;
use std::sync::Arc;

const HEURISTIC_CHARS_PER_TOKEN: f32 = 3.5;
const COMPACT_THRESHOLD: f32 = 0.90;
pub const DEFAULT_TOOL_RESULT_BUDGET_CHARS: usize = 4_000;

fn char_boundary_prefix(text: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }

    match text.char_indices().nth(max_chars) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

pub fn truncate_tool_results(messages: &mut Vec<LlmMessage>, budget_chars: usize) {
    if budget_chars == 0 {
        return;
    }

    for msg in messages.iter_mut() {
        if msg.role != "tool" {
            continue;
        }

        let result_str = msg.tool_result.to_string();
        if result_str.chars().count() <= budget_chars {
            continue;
        }

        let truncated = char_boundary_prefix(&result_str, budget_chars).to_string();
        msg.tool_result = serde_json::json!({
            "output": truncated,
            "_truncated": true
        });
    }
}

// ─── Trait ──────────────────────────────────────────────────────────────────

pub trait ContextEngine: Send + Sync {
    /// Returns true if compaction is recommended, i.e. token utilization
    /// is at or above the configured `compact_threshold`.
    fn should_compact(&self, messages: &[LlmMessage], budget: usize) -> bool;

    /// Perform phased compaction on `messages` to fit within `budget` tokens.
    /// Returns the compacted message list.
    fn compact(&self, messages: Vec<LlmMessage>, budget: usize) -> Vec<LlmMessage>;

    /// Estimate the total token count for a slice of messages.
    fn estimate_tokens(&self, messages: &[LlmMessage]) -> usize;
}

// ─── Size-Based Implementation ───────────────────────────────────────────────

/// Size-based context engine.
///
/// Triggers compaction based on token utilization (≥90% of budget by default)
/// when a positive budget is supplied.
pub struct SizedContextEngine {
    compact_threshold: f32,
    tokenizer: Option<Arc<WordPieceTokenizer>>,
}

impl SizedContextEngine {
    /// Default token budget: 0 disables compaction until configured.
    pub const DEFAULT_BUDGET: usize = 0;
    /// Compact when utilization reaches 90% of budget.
    pub const DEFAULT_THRESHOLD: f32 = 0.90;

    pub fn new() -> Self {
        SizedContextEngine {
            compact_threshold: COMPACT_THRESHOLD,
            tokenizer: None,
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.compact_threshold = threshold.clamp(0.5, 0.99);
        self
    }

    pub fn with_tokenizer(mut self, tokenizer: Arc<WordPieceTokenizer>) -> Self {
        self.tokenizer = Some(tokenizer);
        self
    }

    pub fn budget_tool_result_message(&self, message: LlmMessage, max_chars: usize) -> (LlmMessage, bool) {
        if message.role != "tool" || max_chars == 0 {
            return (message, false);
        }

        let serialized = message.tool_result.to_string();
        if serialized.chars().count() <= max_chars {
            return (message, false);
        }

        let mut batch = vec![message];
        truncate_tool_results(&mut batch, max_chars);
        let message = batch.into_iter().next().unwrap_or_default();

        (message, true)
    }

    pub fn budget_tool_result_messages(
        &self,
        messages: Vec<LlmMessage>,
        max_chars: usize,
    ) -> (Vec<LlmMessage>, usize) {
        let mut budgeted = 0;
        let result = messages
            .into_iter()
            .map(|message| {
                let (message, changed) = self.budget_tool_result_message(message, max_chars);
                if changed {
                    budgeted += 1;
                }
                message
            })
            .collect();

        (result, budgeted)
    }
}

impl Default for SizedContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEngine for SizedContextEngine {
    fn estimate_tokens(&self, messages: &[LlmMessage]) -> usize {
        if let Some(tokenizer) = self.tokenizer.as_ref() {
            let total = messages
                .iter()
                .map(|message| {
                    let mut count = 0usize;
                    if !message.text.is_empty() {
                        count += tokenizer.count_tokens(&message.text);
                    }
                    if !message.reasoning_text.is_empty() {
                        count += tokenizer.count_tokens(&message.reasoning_text);
                    }
                    if !message.tool_result.is_null() {
                        count += tokenizer.count_tokens(&message.tool_result.to_string());
                    }
                    count
                        + message
                            .tool_calls
                            .iter()
                            .map(|tc| {
                                tokenizer.count_tokens(&tc.name)
                                    + tokenizer.count_tokens(&tc.args.to_string())
                            })
                            .sum::<usize>()
                })
                .sum::<usize>();
            return total;
        }

        let total_chars: usize = messages
            .iter()
            .map(|m| {
                m.text.chars().count()
                    + m.reasoning_text.chars().count()
                    + m.tool_result.to_string().chars().count()
                    + m.tool_calls
                        .iter()
                        .map(|tc| tc.args.to_string().chars().count() + tc.name.chars().count())
                        .sum::<usize>()
            })
            .sum();
        if total_chars == 0 {
            0
        } else {
            ((total_chars as f32) / HEURISTIC_CHARS_PER_TOKEN).ceil() as usize
        }
    }

    fn should_compact(&self, messages: &[LlmMessage], budget: usize) -> bool {
        if budget == 0 {
            return false;
        }
        let estimated = self.estimate_tokens(messages);
        let threshold_tokens = ((budget as f32) * self.compact_threshold) as usize;
        estimated >= threshold_tokens
    }

    fn compact(&self, messages: Vec<LlmMessage>, budget: usize) -> Vec<LlmMessage> {
        let mut messages = messages;
        truncate_tool_results(&mut messages, DEFAULT_TOOL_RESULT_BUDGET_CHARS);

        let before = self.estimate_tokens(&messages);
        log::debug!(
            "[ContextEngine] Compacting: ~{} tokens / {} budget ({:.1}%)",
            before,
            budget,
            if budget > 0 {
                before as f32 / budget as f32 * 100.0
            } else {
                0.0
            }
        );

        // ── Phase 1: Identify pinned messages ──────────────────────────────
        // Pin: system prompt + first user message (never removed)
        let mut pinned_indices = HashSet::new();
        let mut first_system_found = false;
        let mut first_user_found = false;
        for (i, msg) in messages.iter().enumerate() {
            if msg.role == "system" && !first_system_found {
                pinned_indices.insert(i);
                first_system_found = true;
            } else if msg.role == "user" && !first_user_found {
                pinned_indices.insert(i);
                first_user_found = true;
            }
        }

        // ── Phase 2: Identify tool results safe to prune ──────────────────
        let mut prunable_indices = HashSet::new();
        let mut later_assistant_tool_ids = HashSet::new();
        for (i, msg) in messages.iter().enumerate().rev() {
            if msg.role == "assistant" {
                later_assistant_tool_ids.extend(msg.tool_calls.iter().map(|tc| tc.id.clone()));
                continue;
            }

            if msg.role == "tool"
                && !pinned_indices.contains(&i)
                && (msg.tool_call_id.is_empty()
                    || !later_assistant_tool_ids.contains(&msg.tool_call_id))
            {
                prunable_indices.insert(i);
            }
        }

        // ── Phase 3: Build compacted list without pruned messages ──────────
        let compacted_items: Vec<(usize, LlmMessage)> = messages
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !prunable_indices.contains(i))
            .enumerate()
            .map(|(stable_idx, (_, m))| (stable_idx, m))
            .collect();

        // ── Phase 4: If still over budget, drop oldest non-pinned messages ─
        let target = ((budget as f32) * self.compact_threshold) as usize;
        let mut compacted_items = compacted_items;
        let mut rebuilt_pinned = HashSet::new();
        let mut first_system_found = false;
        let mut first_user_found = false;
        for (stable_idx, msg) in compacted_items.iter() {
            if msg.role == "system" && !first_system_found {
                rebuilt_pinned.insert(*stable_idx);
                first_system_found = true;
            } else if msg.role == "user" && !first_user_found {
                rebuilt_pinned.insert(*stable_idx);
                first_user_found = true;
            }
        }
        let non_pinned_stable_indices: Vec<usize> = compacted_items
            .iter()
            .filter_map(|(stable_idx, _)| (!rebuilt_pinned.contains(stable_idx)).then_some(*stable_idx))
            .collect();
        let preserve_recent = ((non_pinned_stable_indices.len() as f32) * 0.30).ceil() as usize;
        let preserved_recent_indices: HashSet<usize> = non_pinned_stable_indices
            .iter()
            .rev()
            .take(preserve_recent)
            .copied()
            .collect();

        while self.estimate_tokens(
            &compacted_items
                .iter()
                .map(|(_, message)| message.clone())
                .collect::<Vec<_>>(),
        ) > target
        {
            let drop_idx = compacted_items.iter().position(|(stable_idx, _)| {
                !rebuilt_pinned.contains(stable_idx) && !preserved_recent_indices.contains(stable_idx)
            });

            match drop_idx {
                Some(idx) => {
                    compacted_items.remove(idx);
                }
                None => break,
            }
        }

        let compacted: Vec<LlmMessage> = compacted_items.into_iter().map(|(_, message)| message).collect();

        let after = self.estimate_tokens(&compacted);
        log::debug!(
            "[ContextEngine] Compacted: {} → {} msgs | ~{} → ~{} tokens ({:.1}% of budget)",
            compacted.len() + prunable_indices.len(),
            compacted.len(),
            before,
            after,
            if budget > 0 {
                after as f32 / budget as f32 * 100.0
            } else {
                0.0
            }
        );

        compacted
    }
}

// ─── Legacy Alias (backward compat) ─────────────────────────────────────────

/// Backward-compatible alias. Use `SizedContextEngine` for new code.
pub type SimpleContextEngine = SizedContextEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::backend::{LlmMessage, LlmToolCall};
    use serde_json::json;

    fn msg(role: &str, text: &str) -> LlmMessage {
        LlmMessage {
            role: role.into(),
            text: text.into(),
            ..Default::default()
        }
    }

    fn tool_msg(call_id: &str, text: &str) -> LlmMessage {
        LlmMessage {
            role: "tool".into(),
            text: text.into(),
            tool_call_id: call_id.into(),
            ..Default::default()
        }
    }

    fn assistant_with_tool_call(text: &str, call_id: &str, name: &str) -> LlmMessage {
        LlmMessage {
            role: "assistant".into(),
            text: text.into(),
            tool_calls: vec![LlmToolCall {
                id: call_id.into(),
                name: name.into(),
                args: json!({}),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_estimate_tokens_basic() {
        let engine = SizedContextEngine::new();
        // 35 chars / 3.5 = 10 tokens
        let msgs = vec![msg("user", "hello world foo bar baz qux qui")];
        let est = engine.estimate_tokens(&msgs);
        assert!(est > 0);
    }

    #[test]
    fn test_should_compact_below_threshold() {
        let engine = SizedContextEngine::new();
        // ~1 token of messages vs 1_000_000 budget → should NOT compact
        let msgs = vec![msg("user", "hi")];
        assert!(!engine.should_compact(&msgs, 1_000_000));
    }

    #[test]
    fn test_should_compact_above_threshold() {
        let engine = SizedContextEngine::new();
        // Large messages with tiny budget
        let big_text = "a".repeat(10_000);
        let msgs = vec![msg("user", &big_text)];
        assert!(engine.should_compact(&msgs, 100)); // way over 90% of 100
    }

    #[test]
    fn test_should_compact_zero_budget_never() {
        let engine = SizedContextEngine::new();
        let msgs = vec![msg("user", "huge message ".repeat(1000).as_str())];
        assert!(!engine.should_compact(&msgs, 0));
    }

    #[test]
    fn test_compact_pins_system_and_first_user() {
        let engine = SizedContextEngine::new();
        let messages = vec![
            msg("system", "You are TizenClaw."),
            msg("system", "Secondary system note"),
            msg("user", "Original goal"),
            msg("assistant", "Thinking..."),
            msg("user", "Follow-up"),
            msg("assistant", "Done."),
        ];
        // Force compaction by using tiny budget
        let budget = 10;
        let compact = engine.compact(messages, budget);
        // System and first user must be present
        assert!(compact.iter().any(|m| m.role == "system"));
        assert!(compact
            .iter()
            .any(|m| m.role == "user" && m.text == "Original goal"));
        assert!(!compact
            .iter()
            .any(|m| m.role == "system" && m.text == "Secondary system note"));
    }

    #[test]
    fn test_compact_prunes_unreferenced_tool_results() {
        let engine = SizedContextEngine::new();
        // Tool result with call_id "orphan" is not referenced by any assistant
        let messages = vec![
            msg("system", "prompt"),
            msg("user", "goal"),
            tool_msg("orphan", "result data that can be dropped"),
            msg("assistant", "Final answer"),
        ];
        let budget = 1_000;
        // With large budget, should_compact would be false;
        // force compact anyway to test pruning logic
        let compact = engine.compact(messages, budget);
        // Orphaned tool result should be removed
        assert!(!compact
            .iter()
            .any(|m| m.role == "tool" && m.tool_call_id == "orphan"));
    }

    #[test]
    fn test_compact_keeps_referenced_tool_results() {
        let engine = SizedContextEngine::new();
        // Tool result with call_id "ref1" IS referenced by assistant
        let messages = vec![
            msg("system", "prompt"),
            msg("user", "goal"),
            tool_msg("ref1", "important result"),
            assistant_with_tool_call("Using ref1", "ref1", "get_data"),
            msg("assistant", "Done"),
        ];
        let budget = 50;
        let compact = engine.compact(messages, budget);
        // Referenced tool result should be kept
        assert!(compact
            .iter()
            .any(|m| m.role == "tool" && m.tool_call_id == "ref1" && m.text == "important result"));
    }

    #[test]
    fn test_compact_prunes_tool_only_referenced_by_earlier_assistant() {
        let engine = SizedContextEngine::new();
        let messages = vec![
            msg("system", "prompt"),
            msg("user", "goal"),
            assistant_with_tool_call("Plan to use ref1", "ref1", "get_data"),
            tool_msg("ref1", "late result"),
            msg("assistant", "Done"),
        ];

        let compact = engine.compact(messages, 50);
        assert!(!compact
            .iter()
            .any(|m| m.role == "tool" && m.tool_call_id == "ref1"));
    }

    #[test]
    fn test_compact_returns_at_least_system_and_user() {
        let engine = SizedContextEngine::new();
        let messages = vec![
            msg("system", "S"),
            msg("user", "U"),
            msg("assistant", "A1"),
            msg("assistant", "A2"),
            msg("assistant", "A3"),
            msg("assistant", "A4"),
            msg("assistant", "A5"),
        ];
        // Extremely small budget forces maximum pruning
        let compact = engine.compact(messages, 1);
        assert!(compact.iter().any(|m| m.role == "system"));
        assert!(compact.iter().any(|m| m.role == "user"));
    }

    #[test]
    fn test_with_threshold_clamps() {
        let engine = SizedContextEngine::new().with_threshold(0.3);
        // Clamped to 0.5
        assert!(engine.compact_threshold >= 0.5);
        let engine2 = SizedContextEngine::new().with_threshold(1.5);
        // Clamped to 0.99
        assert!(engine2.compact_threshold <= 0.99);
    }

    #[test]
    fn test_budget_tool_result_message_truncates_large_payload() {
        let engine = SizedContextEngine::new();
        let large = "x".repeat(DEFAULT_TOOL_RESULT_BUDGET_CHARS + 25);
        let message = LlmMessage::tool_result("call1", "read_file", json!({ "data": large }));

        let (budgeted, changed) =
            engine.budget_tool_result_message(message, DEFAULT_TOOL_RESULT_BUDGET_CHARS);

        assert!(changed);
        assert_eq!(budgeted.tool_call_id, "call1");
        assert_eq!(budgeted.tool_name, "read_file");
        assert_eq!(budgeted.tool_result["_truncated"], json!(true));
        assert!(budgeted.tool_result["output"]
            .as_str()
            .unwrap_or_default()
            .starts_with("{\"data\":"));
    }

    #[test]
    fn test_budget_tool_result_message_keeps_small_payload() {
        let engine = SizedContextEngine::new();
        let message = LlmMessage::tool_result("call1", "battery", json!({ "percent": 50 }));

        let (budgeted, changed) =
            engine.budget_tool_result_message(message.clone(), DEFAULT_TOOL_RESULT_BUDGET_CHARS);

        assert!(!changed);
        assert_eq!(budgeted.tool_result, message.tool_result);
    }

    #[test]
    fn test_truncate_tool_results_preserves_utf8_boundaries() {
        let mut messages = vec![LlmMessage::tool_result(
            "call1",
            "unicode",
            json!({ "output": "한글🙂emoji" }),
        )];

        truncate_tool_results(&mut messages, 7);

        assert_eq!(messages[0].tool_result["_truncated"], json!(true));
        let output = messages[0].tool_result["output"]
            .as_str()
            .unwrap_or_default();
        assert!(std::str::from_utf8(output.as_bytes()).is_ok());
    }
}
