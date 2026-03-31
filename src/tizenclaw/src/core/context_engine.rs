//! Context Engine — Controls context window pressure and compaction.
//!
//! Provides strategies for summarizing and truncating session history
//! when approaching LLM token limits on restrained hardware.

use crate::llm::backend::LlmMessage;

pub trait ContextEngine: Send + Sync {
    /// Returns true if compaction is recommended based on the current usage.
    fn should_compact(&self, messages: &[LlmMessage], budget: usize) -> bool;

    /// Perform compaction on the messages to fit within the budget.
    fn compact(&self, messages: Vec<LlmMessage>, budget: usize) -> Vec<LlmMessage>;
}

pub struct SimpleContextEngine;

impl SimpleContextEngine {
    pub fn new() -> Self {
        SimpleContextEngine
    }
}

impl ContextEngine for SimpleContextEngine {
    fn should_compact(&self, messages: &[LlmMessage], budget: usize) -> bool {
        // Assume roughly 4 chars per token for heuristic estimate
        let total_chars: usize = messages.iter().map(|m| m.text.len()).sum();
        let estimated_tokens = total_chars / 4;
        
        // Compact if estimated usage is > 80% of budget
        estimated_tokens > (budget * 8) / 10
    }

    fn compact(&self, mut messages: Vec<LlmMessage>, budget: usize) -> Vec<LlmMessage> {
        log::info!("ContextEngine: Compacting history (budget: {})", budget);
        
        // Strategy: Keep System and last N messages.
        // For now, let's keep the last 5 messages + system message.
        if messages.len() > 6 {
            let mut compacted = Vec::new();
            // Keep system (if first)
            if let Some(first) = messages.get(0) {
                 if first.role == "system" {
                     compacted.push(first.clone());
                 }
            }
            
            // Keep last 5
            let start = messages.len() - 5;
            compacted.extend(messages[start..].to_vec());
            messages = compacted;
        }

        messages
    }
}
