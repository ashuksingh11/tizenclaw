use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    compact::CompactionPlan,
    config::RuntimeConfig,
    hooks::{HookPhase, HookSpec},
    permissions::{PermissionDecision, PermissionLevel, PermissionRequest, PermissionScope},
    prompt::PromptAssembly,
    session::{
        ConversationMessage, SessionCompactionMetadata, SessionContentBlock, SessionMessageRole,
        SessionRecord, SessionState,
    },
    usage::{TokenUsage, UsageSnapshot},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationTurn {
    pub role: MessageRole,
    pub content: String,
    pub metadata: BTreeMap<String, String>,
}

impl ConversationTurn {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ConversationLog {
    pub session_id: String,
    pub turns: Vec<ConversationTurn>,
    pub summary: Option<String>,
}

impl ConversationLog {
    pub fn push(&mut self, turn: ConversationTurn) {
        self.turns.push(turn);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ApiRequest {
    pub session_id: String,
    pub request_index: usize,
    pub prompt: PromptAssembly,
    pub prompt_text: String,
    #[serde(default)]
    pub messages: Vec<ConversationMessage>,
    #[serde(default)]
    pub available_tools: Vec<ToolDefinition>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub permission_scope: PermissionScope,
    #[serde(default)]
    pub minimum_permission_level: PermissionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolExecutionOutput {
    pub tool_call_id: String,
    pub output: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolFailure {
    pub tool_call_id: String,
    pub name: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelResponseEvent {
    TextDelta { text: String },
    ToolCall { call: ToolCallRequest },
    Usage { usage: UsageSnapshot },
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantEvent {
    Delta { text: String },
    ToolCall { call: ToolCallRequest },
    Usage { usage: UsageSnapshot },
    Completed { message: ConversationMessage },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookContext {
    pub phase: HookPhase,
    pub session_id: String,
    pub request_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<TurnSummary>,
    #[serde(default)]
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HookOutcome {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction: Option<CompactionPlan>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnUsageReport {
    #[serde(default)]
    pub snapshots: Vec<UsageSnapshot>,
    pub total_tokens: TokenUsage,
    pub total_cost_microunits: u64,
}

impl TurnUsageReport {
    fn record(&mut self, usage: UsageSnapshot) {
        self.total_tokens.input_tokens += usage.tokens.input_tokens;
        self.total_tokens.output_tokens += usage.tokens.output_tokens;
        self.total_cost_microunits += usage.cost_microunits;
        self.snapshots.push(usage);
    }

    fn latest(&self) -> Option<UsageSnapshot> {
        self.snapshots.last().cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnSummary {
    pub session_id: String,
    pub request_count: usize,
    pub tool_call_count: usize,
    #[serde(default)]
    pub tool_names: Vec<String>,
    pub assistant_text: String,
    pub final_message_count: usize,
    pub compacted: bool,
    pub summary: String,
    pub usage: TurnUsageReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationEvent {
    HookStarted {
        name: String,
        phase: HookPhase,
    },
    HookCompleted {
        name: String,
        phase: HookPhase,
        outcome: HookOutcome,
    },
    RequestPrepared {
        request: ApiRequest,
    },
    Assistant {
        event: AssistantEvent,
    },
    PermissionResolved {
        decision: PermissionDecision,
    },
    ToolExecutionStarted {
        call: ToolCallRequest,
    },
    ToolExecutionFinished {
        result: ToolExecutionOutput,
    },
    ToolExecutionFailed {
        failure: ToolFailure,
    },
    CompactionApplied {
        metadata: SessionCompactionMetadata,
    },
    SummaryUpdated {
        summary: TurnSummary,
    },
    TurnCompleted {
        summary: TurnSummary,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurnResult {
    pub summary: TurnSummary,
    #[serde(default)]
    pub events: Vec<ConversationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationEngineOptions {
    pub max_model_requests: usize,
}

impl Default for ConversationEngineOptions {
    fn default() -> Self {
        Self {
            max_model_requests: 8,
        }
    }
}

pub trait ModelTransport {
    fn stream(&mut self, request: &ApiRequest) -> Result<Vec<ModelResponseEvent>, ModelError>;
}

pub trait ToolExecutor {
    fn definitions(&self) -> Vec<ToolDefinition>;

    fn execute(&mut self, call: &ToolCallRequest) -> Result<ToolExecutionOutput, ToolRuntimeError>;
}

pub trait PermissionResolver {
    fn decide(
        &mut self,
        config: &RuntimeConfig,
        request: PermissionRequest,
    ) -> Result<PermissionDecision, ConversationRuntimeError>;
}

pub trait HookRunner {
    fn run(
        &mut self,
        hook: &HookSpec,
        context: &HookContext,
    ) -> Result<HookOutcome, HookRuntimeError>;
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelError {
    #[error("model transport failed: {message}")]
    Transport { message: String },
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolRuntimeError {
    #[error("tool {tool_name} was denied: {message}")]
    PermissionDenied { tool_name: String, message: String },
    #[error("tool {tool_name} failed: {message}")]
    Execution { tool_name: String, message: String },
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookRuntimeError {
    #[error("hook {name} in {phase:?} failed: {message}")]
    Execution {
        name: String,
        phase: HookPhase,
        message: String,
    },
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationRuntimeError {
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error(transparent)]
    Hook(#[from] HookRuntimeError),
    #[error("conversation exceeded the configured request limit of {max_model_requests}")]
    MaxModelRequestsExceeded { max_model_requests: usize },
    #[error("permission resolution failed: {message}")]
    Permission { message: String },
    #[error("conversation invariant violated: {message}")]
    Invariant { message: String },
}

pub struct ConversationEngine<M, T, P, H> {
    config: RuntimeConfig,
    hooks: Vec<HookSpec>,
    options: ConversationEngineOptions,
    model: M,
    tools: T,
    permissions: P,
    hook_runner: H,
}

impl<M, T, P, H> ConversationEngine<M, T, P, H>
where
    M: ModelTransport,
    T: ToolExecutor,
    P: PermissionResolver,
    H: HookRunner,
{
    pub fn new(
        config: RuntimeConfig,
        hooks: Vec<HookSpec>,
        model: M,
        tools: T,
        permissions: P,
        hook_runner: H,
    ) -> Self {
        Self {
            config,
            hooks,
            options: ConversationEngineOptions::default(),
            model,
            tools,
            permissions,
            hook_runner,
        }
    }

    pub fn with_options(mut self, options: ConversationEngineOptions) -> Self {
        self.options = options;
        self
    }

    pub fn prepare_request(
        &self,
        session: &SessionRecord,
        prompt: &PromptAssembly,
        request_index: usize,
    ) -> ApiRequest {
        ApiRequest {
            session_id: session.session_id.clone(),
            request_index,
            prompt: prompt.clone(),
            prompt_text: prompt.render(),
            messages: session.messages.clone(),
            available_tools: self.tools.definitions(),
            metadata: BTreeMap::from([
                (
                    "profile".to_string(),
                    format!("{:?}", self.config.profile).to_lowercase(),
                ),
                (
                    "permission_mode".to_string(),
                    format!("{:?}", self.config.permission_mode).to_lowercase(),
                ),
                (
                    "hooks_enabled".to_string(),
                    self.config.hooks_enabled.to_string(),
                ),
            ]),
        }
    }

    pub fn run_turn<F>(
        &mut self,
        session: &mut SessionRecord,
        prompt: &PromptAssembly,
        mut observer: F,
    ) -> Result<ConversationTurnResult, ConversationRuntimeError>
    where
        F: FnMut(&ConversationEvent),
    {
        let mut events = Vec::new();
        let mut usage = TurnUsageReport::default();
        let mut tool_names = Vec::new();
        let mut request_count = 0usize;
        let mut compacted = false;
        let final_assistant_text = loop {
            session.set_state(SessionState::Active);
            if request_count >= self.options.max_model_requests {
                return Err(ConversationRuntimeError::MaxModelRequestsExceeded {
                    max_model_requests: self.options.max_model_requests,
                });
            }

            let mut request = self.prepare_request(session, prompt, request_count);
            self.run_hooks(
                HookPhase::PrePrompt,
                request_count,
                session,
                None,
                None,
                Some(&mut request.metadata),
                &mut events,
                &mut observer,
            )?;

            emit(
                &mut events,
                &mut observer,
                ConversationEvent::RequestPrepared {
                    request: request.clone(),
                },
            );

            let model_events = self.model.stream(&request)?;
            let (assistant_text, tool_calls) =
                self.consume_model_events(&model_events, &mut usage, &mut events, &mut observer);

            if !assistant_text.is_empty() || !tool_calls.is_empty() {
                let assistant_message =
                    assistant_message(assistant_text.clone(), &tool_calls, usage.latest());
                session.push_message(assistant_message.clone());
                emit(
                    &mut events,
                    &mut observer,
                    ConversationEvent::Assistant {
                        event: AssistantEvent::Completed {
                            message: assistant_message,
                        },
                    },
                );
            }

            if tool_calls.is_empty() {
                break assistant_text;
            }

            for call in tool_calls {
                tool_names.push(call.name.clone());
                let permission = PermissionRequest {
                    scope: tool_permission_scope(&self.tools.definitions(), &call.name),
                    target: call.name.clone(),
                    reason: format!("execute tool call {}", call.id),
                    tool_name: Some(call.name.clone()),
                    minimum_level: tool_permission_level(&self.tools.definitions(), &call.name),
                    bash_plan: None,
                    metadata: BTreeMap::new(),
                };
                let decision =
                    self.permissions
                        .decide(&self.config, permission)
                        .map_err(|error| ConversationRuntimeError::Permission {
                            message: error.to_string(),
                        })?;
                session.permission_history.push(decision.clone());
                emit(
                    &mut events,
                    &mut observer,
                    ConversationEvent::PermissionResolved {
                        decision: decision.clone(),
                    },
                );

                if !decision.allowed {
                    let failure = ToolFailure {
                        tool_call_id: call.id.clone(),
                        name: call.name.clone(),
                        message: decision.rationale.clone(),
                        recoverable: true,
                    };
                    session.push_message(tool_error_message(&failure));
                    emit(
                        &mut events,
                        &mut observer,
                        ConversationEvent::ToolExecutionFailed { failure },
                    );
                    continue;
                }

                self.run_hooks(
                    HookPhase::PreTool,
                    request_count,
                    session,
                    Some(call.clone()),
                    None,
                    None,
                    &mut events,
                    &mut observer,
                )?;

                emit(
                    &mut events,
                    &mut observer,
                    ConversationEvent::ToolExecutionStarted { call: call.clone() },
                );

                match self.tools.execute(&call) {
                    Ok(result) => {
                        session.push_message(tool_success_message(&call, &result));
                        emit(
                            &mut events,
                            &mut observer,
                            ConversationEvent::ToolExecutionFinished {
                                result: result.clone(),
                            },
                        );
                        self.run_hooks(
                            HookPhase::PostTool,
                            request_count,
                            session,
                            Some(call),
                            None,
                            None,
                            &mut events,
                            &mut observer,
                        )?;
                    }
                    Err(error) => {
                        let failure = ToolFailure {
                            tool_call_id: call.id.clone(),
                            name: call.name.clone(),
                            message: error.to_string(),
                            recoverable: true,
                        };
                        session.push_message(tool_error_message(&failure));
                        emit(
                            &mut events,
                            &mut observer,
                            ConversationEvent::ToolExecutionFailed { failure },
                        );
                        self.run_hooks(
                            HookPhase::PostTool,
                            request_count,
                            session,
                            Some(call),
                            None,
                            None,
                            &mut events,
                            &mut observer,
                        )?;
                    }
                }
            }

            request_count += 1;
        };

        request_count += 1;

        let mut summary = TurnSummary {
            session_id: session.session_id.clone(),
            request_count,
            tool_call_count: tool_names.len(),
            tool_names,
            assistant_text: normalize_summary_text(&final_assistant_text),
            final_message_count: session.messages.len(),
            compacted: false,
            summary: String::new(),
            usage,
        };
        summary.summary = build_summary_text(&summary);

        let hook_outcomes = self.run_hooks_collect(
            HookPhase::PostSession,
            request_count.saturating_sub(1),
            session,
            None,
            Some(summary.clone()),
            &mut events,
            &mut observer,
        )?;
        for outcome in hook_outcomes {
            if let Some(summary_override) = outcome.summary_override {
                summary.summary = normalize_summary_text(&summary_override);
            }
            if let Some(plan) = outcome.compaction {
                if let Some(metadata) = apply_compaction(session, &summary.summary, &plan) {
                    compacted = true;
                    summary.compacted = true;
                    summary.final_message_count = session.messages.len();
                    emit(
                        &mut events,
                        &mut observer,
                        ConversationEvent::CompactionApplied { metadata },
                    );
                }
            }
        }

        if !compacted {
            summary.compacted = false;
        }
        session.set_summary(summary.summary.clone());
        session.set_state(SessionState::Completed);

        emit(
            &mut events,
            &mut observer,
            ConversationEvent::SummaryUpdated {
                summary: summary.clone(),
            },
        );
        emit(
            &mut events,
            &mut observer,
            ConversationEvent::TurnCompleted {
                summary: summary.clone(),
            },
        );

        Ok(ConversationTurnResult { summary, events })
    }

    fn consume_model_events<F>(
        &mut self,
        model_events: &[ModelResponseEvent],
        usage: &mut TurnUsageReport,
        events: &mut Vec<ConversationEvent>,
        observer: &mut F,
    ) -> (String, Vec<ToolCallRequest>)
    where
        F: FnMut(&ConversationEvent),
    {
        let mut assistant_text = String::new();
        let mut tool_calls = Vec::new();

        for event in model_events {
            match event {
                ModelResponseEvent::TextDelta { text } => {
                    assistant_text.push_str(text);
                    emit(
                        events,
                        observer,
                        ConversationEvent::Assistant {
                            event: AssistantEvent::Delta { text: text.clone() },
                        },
                    );
                }
                ModelResponseEvent::ToolCall { call } => {
                    tool_calls.push(call.clone());
                    emit(
                        events,
                        observer,
                        ConversationEvent::Assistant {
                            event: AssistantEvent::ToolCall { call: call.clone() },
                        },
                    );
                }
                ModelResponseEvent::Usage { usage: snapshot } => {
                    usage.record(snapshot.clone());
                    emit(
                        events,
                        observer,
                        ConversationEvent::Assistant {
                            event: AssistantEvent::Usage {
                                usage: snapshot.clone(),
                            },
                        },
                    );
                }
                ModelResponseEvent::Completed => {}
            }
        }

        (assistant_text, tool_calls)
    }

    fn run_hooks<F>(
        &mut self,
        phase: HookPhase,
        request_index: usize,
        session: &SessionRecord,
        tool_call: Option<ToolCallRequest>,
        summary: Option<TurnSummary>,
        request_metadata: Option<&mut BTreeMap<String, String>>,
        events: &mut Vec<ConversationEvent>,
        observer: &mut F,
    ) -> Result<(), ConversationRuntimeError>
    where
        F: FnMut(&ConversationEvent),
    {
        if !self.config.hooks_enabled {
            return Ok(());
        }

        let context = HookContext {
            phase: phase.clone(),
            session_id: session.session_id.clone(),
            request_index,
            tool_call,
            summary,
            message_count: session.messages.len(),
        };
        let mut metadata_target = request_metadata;

        for hook in self
            .hooks
            .iter()
            .filter(|hook| hook.enabled && hook.phase == phase)
        {
            emit(
                events,
                observer,
                ConversationEvent::HookStarted {
                    name: hook.name.clone(),
                    phase: phase.clone(),
                },
            );
            let outcome = self.hook_runner.run(hook, &context)?;
            if let Some(target) = metadata_target.as_deref_mut() {
                for (key, value) in &outcome.metadata {
                    target.insert(key.clone(), value.clone());
                }
            }
            emit(
                events,
                observer,
                ConversationEvent::HookCompleted {
                    name: hook.name.clone(),
                    phase: phase.clone(),
                    outcome,
                },
            );
        }

        Ok(())
    }

    fn run_hooks_collect<F>(
        &mut self,
        phase: HookPhase,
        request_index: usize,
        session: &SessionRecord,
        tool_call: Option<ToolCallRequest>,
        summary: Option<TurnSummary>,
        events: &mut Vec<ConversationEvent>,
        observer: &mut F,
    ) -> Result<Vec<HookOutcome>, ConversationRuntimeError>
    where
        F: FnMut(&ConversationEvent),
    {
        if !self.config.hooks_enabled {
            return Ok(Vec::new());
        }

        let context = HookContext {
            phase: phase.clone(),
            session_id: session.session_id.clone(),
            request_index,
            tool_call,
            summary,
            message_count: session.messages.len(),
        };
        let mut outcomes = Vec::new();

        for hook in self
            .hooks
            .iter()
            .filter(|hook| hook.enabled && hook.phase == phase)
        {
            emit(
                events,
                observer,
                ConversationEvent::HookStarted {
                    name: hook.name.clone(),
                    phase: phase.clone(),
                },
            );
            let outcome = self.hook_runner.run(hook, &context)?;
            emit(
                events,
                observer,
                ConversationEvent::HookCompleted {
                    name: hook.name.clone(),
                    phase: phase.clone(),
                    outcome: outcome.clone(),
                },
            );
            outcomes.push(outcome);
        }

        Ok(outcomes)
    }
}

fn emit<F>(events: &mut Vec<ConversationEvent>, observer: &mut F, event: ConversationEvent)
where
    F: FnMut(&ConversationEvent),
{
    observer(&event);
    events.push(event);
}

fn assistant_message(
    assistant_text: String,
    tool_calls: &[ToolCallRequest],
    usage: Option<UsageSnapshot>,
) -> ConversationMessage {
    let mut message = ConversationMessage::new(SessionMessageRole::Assistant);
    if !assistant_text.is_empty() {
        message.content.push(SessionContentBlock::Text {
            text: assistant_text,
        });
    }
    message
        .content
        .extend(tool_calls.iter().map(|call| SessionContentBlock::ToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            input: call.input.clone(),
        }));
    message.usage = usage;
    message
}

fn tool_success_message(
    call: &ToolCallRequest,
    result: &ToolExecutionOutput,
) -> ConversationMessage {
    let mut message = ConversationMessage::new(SessionMessageRole::Tool);
    message.name = Some(call.name.clone());
    if let Some(summary) = &result.summary {
        message.content.push(SessionContentBlock::Text {
            text: summary.clone(),
        });
    }
    message.content.push(SessionContentBlock::ToolResult {
        tool_call_id: result.tool_call_id.clone(),
        output: result.output.clone(),
    });
    message
}

fn tool_error_message(failure: &ToolFailure) -> ConversationMessage {
    let mut message = ConversationMessage::new(SessionMessageRole::Tool);
    message.name = Some(failure.name.clone());
    message.content.push(SessionContentBlock::Text {
        text: failure.message.clone(),
    });
    message.content.push(SessionContentBlock::ToolResult {
        tool_call_id: failure.tool_call_id.clone(),
        output: json!({
            "ok": false,
            "error": failure.message.clone(),
            "recoverable": failure.recoverable,
            "tool_name": failure.name.clone(),
        }),
    });
    message
}

fn build_summary_text(summary: &TurnSummary) -> String {
    if !summary.assistant_text.is_empty() {
        return summary.assistant_text.clone();
    }

    if !summary.tool_names.is_empty() {
        return format!("Executed tools: {}", summary.tool_names.join(", "));
    }

    format!(
        "Completed {} request(s) with no assistant text",
        summary.request_count
    )
}

fn normalize_summary_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut summary = collapsed.trim().to_string();
    if summary.len() > 160 {
        summary.truncate(160);
        summary.push_str("...");
    }
    summary
}

fn tool_permission_scope(definitions: &[ToolDefinition], tool_name: &str) -> PermissionScope {
    definitions
        .iter()
        .find(|definition| definition.name == tool_name)
        .map(|definition| definition.permission_scope.clone())
        .unwrap_or(PermissionScope::Execute)
}

fn tool_permission_level(definitions: &[ToolDefinition], tool_name: &str) -> PermissionLevel {
    definitions
        .iter()
        .find(|definition| definition.name == tool_name)
        .map(|definition| definition.minimum_permission_level)
        .unwrap_or(PermissionLevel::Standard)
}

fn apply_compaction(
    session: &mut SessionRecord,
    summary: &str,
    plan: &CompactionPlan,
) -> Option<SessionCompactionMetadata> {
    let source_count = session.messages.len();
    if source_count <= plan.max_items {
        return None;
    }

    let retain_count = plan
        .preserve_latest
        .max(1)
        .min(source_count)
        .min(plan.max_items.max(1));
    let start = source_count.saturating_sub(retain_count);
    session.messages = session.messages[start..].to_vec();

    let metadata = SessionCompactionMetadata {
        compacted_at: Some(plan.target.clone()),
        summary: Some(summary.to_string()),
        source_message_count: source_count,
        retained_message_count: session.messages.len(),
    };
    session.record_compaction(metadata.clone());
    Some(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{RuntimeConfig, RuntimeProfile},
        permissions::{PermissionDecisionSource, PermissionMode, PermissionOutcome},
        session::SessionRecord,
    };

    #[derive(Default)]
    struct FakeModel {
        responses: Vec<Vec<ModelResponseEvent>>,
        seen_requests: Vec<ApiRequest>,
    }

    impl FakeModel {
        fn new(responses: Vec<Vec<ModelResponseEvent>>) -> Self {
            Self {
                responses,
                seen_requests: Vec::new(),
            }
        }
    }

    impl ModelTransport for FakeModel {
        fn stream(&mut self, request: &ApiRequest) -> Result<Vec<ModelResponseEvent>, ModelError> {
            self.seen_requests.push(request.clone());
            if self.responses.is_empty() {
                return Err(ModelError::Transport {
                    message: "no scripted model response".to_string(),
                });
            }
            Ok(self.responses.remove(0))
        }
    }

    #[derive(Default)]
    struct FakeTools {
        definitions: Vec<ToolDefinition>,
        results: BTreeMap<String, Result<ToolExecutionOutput, ToolRuntimeError>>,
    }

    impl ToolExecutor for FakeTools {
        fn definitions(&self) -> Vec<ToolDefinition> {
            self.definitions.clone()
        }

        fn execute(
            &mut self,
            call: &ToolCallRequest,
        ) -> Result<ToolExecutionOutput, ToolRuntimeError> {
            self.results.remove(&call.name).unwrap_or_else(|| {
                Err(ToolRuntimeError::Execution {
                    tool_name: call.name.clone(),
                    message: "missing scripted tool result".to_string(),
                })
            })
        }
    }

    #[derive(Default)]
    struct FakePermissions {
        allowed: bool,
        seen: Vec<PermissionRequest>,
    }

    impl PermissionResolver for FakePermissions {
        fn decide(
            &mut self,
            _config: &RuntimeConfig,
            request: PermissionRequest,
        ) -> Result<PermissionDecision, ConversationRuntimeError> {
            self.seen.push(request.clone());
            Ok(PermissionDecision {
                request,
                allowed: self.allowed,
                outcome: if self.allowed {
                    PermissionOutcome::Allowed
                } else {
                    PermissionOutcome::Denied
                },
                rationale: if self.allowed {
                    "allowed by test policy".to_string()
                } else {
                    "blocked by test policy".to_string()
                },
                reasons: vec![if self.allowed {
                    "allowed by test policy".to_string()
                } else {
                    "blocked by test policy".to_string()
                }],
                source: PermissionDecisionSource::Mode,
                matched_rule: None,
                prompt: None,
            })
        }
    }

    #[derive(Default)]
    struct FakeHooks {
        outcomes: BTreeMap<String, HookOutcome>,
    }

    impl HookRunner for FakeHooks {
        fn run(
            &mut self,
            hook: &HookSpec,
            _context: &HookContext,
        ) -> Result<HookOutcome, HookRuntimeError> {
            Ok(self.outcomes.get(&hook.name).cloned().unwrap_or_default())
        }
    }

    fn host_config() -> RuntimeConfig {
        RuntimeConfig {
            profile: RuntimeProfile::Host,
            permission_mode: PermissionMode::AllowAll,
            ..RuntimeConfig::default()
        }
    }

    fn prompt() -> PromptAssembly {
        PromptAssembly::default()
    }

    fn session_with_user_message() -> SessionRecord {
        let mut session = SessionRecord::new("session-1", RuntimeProfile::Host);
        session.push_message(ConversationMessage::with_text(
            SessionMessageRole::User,
            "hello runtime",
        ));
        session
    }

    fn event_names(events: &[ConversationEvent]) -> Vec<String> {
        events
            .iter()
            .map(|event| match event {
                ConversationEvent::HookStarted { name, .. } => format!("hook_started:{name}"),
                ConversationEvent::HookCompleted { name, .. } => format!("hook_completed:{name}"),
                ConversationEvent::RequestPrepared { .. } => "request_prepared".to_string(),
                ConversationEvent::Assistant { event } => match event {
                    AssistantEvent::Delta { .. } => "assistant_delta".to_string(),
                    AssistantEvent::ToolCall { .. } => "assistant_tool_call".to_string(),
                    AssistantEvent::Usage { .. } => "assistant_usage".to_string(),
                    AssistantEvent::Completed { .. } => "assistant_completed".to_string(),
                },
                ConversationEvent::PermissionResolved { .. } => "permission_resolved".to_string(),
                ConversationEvent::ToolExecutionStarted { .. } => {
                    "tool_execution_started".to_string()
                }
                ConversationEvent::ToolExecutionFinished { .. } => {
                    "tool_execution_finished".to_string()
                }
                ConversationEvent::ToolExecutionFailed { .. } => {
                    "tool_execution_failed".to_string()
                }
                ConversationEvent::CompactionApplied { .. } => "compaction_applied".to_string(),
                ConversationEvent::SummaryUpdated { .. } => "summary_updated".to_string(),
                ConversationEvent::TurnCompleted { .. } => "turn_completed".to_string(),
            })
            .collect()
    }

    #[test]
    fn conversation_round_trip_serializes_cleanly() {
        let mut log = ConversationLog {
            session_id: "session-1".to_string(),
            ..ConversationLog::default()
        };
        log.push(ConversationTurn::new(MessageRole::User, "hello"));

        let json = serde_json::to_string(&log).expect("serialize conversation");
        let restored: ConversationLog =
            serde_json::from_str(&json).expect("deserialize conversation");

        assert_eq!(restored.turns.len(), 1);
        assert_eq!(restored.turns[0].role, MessageRole::User);
        assert_eq!(restored.turns[0].content, "hello");
    }

    #[test]
    fn runs_normal_assistant_only_turn() {
        let model = FakeModel::new(vec![vec![
            ModelResponseEvent::TextDelta {
                text: "Hello from the assistant.".to_string(),
            },
            ModelResponseEvent::Usage {
                usage: UsageSnapshot {
                    model: "gpt-test".to_string(),
                    tokens: TokenUsage {
                        input_tokens: 11,
                        output_tokens: 7,
                    },
                    cost_microunits: 19,
                },
            },
            ModelResponseEvent::Completed,
        ]]);
        let tools = FakeTools::default();
        let permissions = FakePermissions {
            allowed: true,
            ..FakePermissions::default()
        };
        let hooks = FakeHooks::default();
        let mut engine =
            ConversationEngine::new(host_config(), Vec::new(), model, tools, permissions, hooks);
        let mut session = session_with_user_message();

        let result = engine
            .run_turn(&mut session, &prompt(), |_| {})
            .expect("assistant-only turn succeeds");

        assert_eq!(session.state, SessionState::Completed);
        assert_eq!(session.messages.len(), 2);
        assert_eq!(result.summary.assistant_text, "Hello from the assistant.");
        assert_eq!(result.summary.usage.total_tokens.input_tokens, 11);
        assert_eq!(
            event_names(&result.events),
            vec![
                "request_prepared",
                "assistant_delta",
                "assistant_usage",
                "assistant_completed",
                "summary_updated",
                "turn_completed",
            ]
        );
    }

    #[test]
    fn runs_tool_execution_turn_and_reenters_model_loop() {
        let model = FakeModel::new(vec![
            vec![
                ModelResponseEvent::TextDelta {
                    text: "Checking the workspace.".to_string(),
                },
                ModelResponseEvent::ToolCall {
                    call: ToolCallRequest {
                        id: "tool-1".to_string(),
                        name: "list_files".to_string(),
                        input: json!({ "path": "." }),
                    },
                },
                ModelResponseEvent::Usage {
                    usage: UsageSnapshot {
                        model: "gpt-test".to_string(),
                        tokens: TokenUsage {
                            input_tokens: 9,
                            output_tokens: 4,
                        },
                        cost_microunits: 13,
                    },
                },
                ModelResponseEvent::Completed,
            ],
            vec![
                ModelResponseEvent::TextDelta {
                    text: "Found the expected files.".to_string(),
                },
                ModelResponseEvent::Completed,
            ],
        ]);
        let tools = FakeTools {
            definitions: vec![ToolDefinition {
                name: "list_files".to_string(),
                description: "List files in a directory".to_string(),
                permission_scope: PermissionScope::Read,
                minimum_permission_level: PermissionLevel::Low,
            }],
            results: BTreeMap::from([(
                "list_files".to_string(),
                Ok(ToolExecutionOutput {
                    tool_call_id: "tool-1".to_string(),
                    output: json!({ "entries": ["a.rs", "b.rs"] }),
                    summary: Some("Listed files successfully.".to_string()),
                }),
            )]),
        };
        let permissions = FakePermissions {
            allowed: true,
            ..FakePermissions::default()
        };
        let hooks = FakeHooks::default();
        let mut engine =
            ConversationEngine::new(host_config(), Vec::new(), model, tools, permissions, hooks);
        let mut session = session_with_user_message();

        let result = engine
            .run_turn(&mut session, &prompt(), |_| {})
            .expect("tool turn succeeds");

        assert_eq!(session.messages.len(), 4);
        assert_eq!(result.summary.request_count, 2);
        assert_eq!(result.summary.tool_names, vec!["list_files".to_string()]);
        assert_eq!(result.summary.assistant_text, "Found the expected files.");
        assert_eq!(
            event_names(&result.events),
            vec![
                "request_prepared",
                "assistant_delta",
                "assistant_tool_call",
                "assistant_usage",
                "assistant_completed",
                "permission_resolved",
                "tool_execution_started",
                "tool_execution_finished",
                "request_prepared",
                "assistant_delta",
                "assistant_completed",
                "summary_updated",
                "turn_completed",
            ]
        );
    }

    #[test]
    fn records_tool_failure_and_keeps_turn_stable() {
        let model = FakeModel::new(vec![
            vec![
                ModelResponseEvent::ToolCall {
                    call: ToolCallRequest {
                        id: "tool-1".to_string(),
                        name: "broken_tool".to_string(),
                        input: json!({}),
                    },
                },
                ModelResponseEvent::Completed,
            ],
            vec![
                ModelResponseEvent::TextDelta {
                    text: "The tool failed, but the turn completed.".to_string(),
                },
                ModelResponseEvent::Completed,
            ],
        ]);
        let tools = FakeTools {
            definitions: vec![ToolDefinition {
                name: "broken_tool".to_string(),
                description: "Always fails".to_string(),
                permission_scope: PermissionScope::Execute,
                minimum_permission_level: PermissionLevel::Standard,
            }],
            results: BTreeMap::from([(
                "broken_tool".to_string(),
                Err(ToolRuntimeError::Execution {
                    tool_name: "broken_tool".to_string(),
                    message: "simulated failure".to_string(),
                }),
            )]),
        };
        let permissions = FakePermissions {
            allowed: true,
            ..FakePermissions::default()
        };
        let hooks = FakeHooks::default();
        let mut engine =
            ConversationEngine::new(host_config(), Vec::new(), model, tools, permissions, hooks);
        let mut session = session_with_user_message();

        let result = engine
            .run_turn(&mut session, &prompt(), |_| {})
            .expect("tool failure is represented as a recoverable turn event");

        assert_eq!(session.messages.len(), 4);
        assert!(matches!(
            result
                .events
                .iter()
                .find(|event| matches!(event, ConversationEvent::ToolExecutionFailed { .. })),
            Some(_)
        ));
        assert_eq!(
            result.summary.assistant_text,
            "The tool failed, but the turn completed."
        );
    }

    #[test]
    fn applies_post_session_summary_and_compaction_hooks() {
        let model = FakeModel::new(vec![vec![
            ModelResponseEvent::TextDelta {
                text: "Verbose assistant output that will be summarized.".to_string(),
            },
            ModelResponseEvent::Completed,
        ]]);
        let tools = FakeTools::default();
        let permissions = FakePermissions {
            allowed: true,
            ..FakePermissions::default()
        };
        let hooks = FakeHooks {
            outcomes: BTreeMap::from([(
                "compact_turn".to_string(),
                HookOutcome {
                    summary_override: Some("Stable hook summary".to_string()),
                    compaction: Some(CompactionPlan {
                        target: "hook:compact_turn".to_string(),
                        max_items: 2,
                        preserve_latest: 2,
                    }),
                    metadata: BTreeMap::new(),
                },
            )]),
        };
        let mut engine = ConversationEngine::new(
            host_config(),
            vec![HookSpec {
                name: "compact_turn".to_string(),
                phase: HookPhase::PostSession,
                command: "compact".to_string(),
                enabled: true,
                env: BTreeMap::new(),
            }],
            model,
            tools,
            permissions,
            hooks,
        );
        let mut session = session_with_user_message();
        session.push_message(ConversationMessage::with_text(
            SessionMessageRole::Assistant,
            "older assistant message",
        ));
        session.push_message(ConversationMessage::with_text(
            SessionMessageRole::Tool,
            "older tool result",
        ));

        let result = engine
            .run_turn(&mut session, &prompt(), |_| {})
            .expect("hooked turn succeeds");

        assert_eq!(result.summary.summary, "Stable hook summary");
        assert!(result.summary.compacted);
        assert_eq!(session.summary.as_deref(), Some("Stable hook summary"));
        assert_eq!(session.messages.len(), 2);
        assert!(matches!(session.compaction, Some(_)));
        assert!(event_names(&result.events).contains(&"compaction_applied".to_string()));
    }

    #[test]
    fn preserves_explicit_event_order_for_cli_subscribers() {
        let model = FakeModel::new(vec![
            vec![
                ModelResponseEvent::TextDelta {
                    text: "Need a tool.".to_string(),
                },
                ModelResponseEvent::ToolCall {
                    call: ToolCallRequest {
                        id: "tool-1".to_string(),
                        name: "lookup".to_string(),
                        input: json!({ "q": "runtime" }),
                    },
                },
                ModelResponseEvent::Completed,
            ],
            vec![
                ModelResponseEvent::TextDelta {
                    text: "Finished.".to_string(),
                },
                ModelResponseEvent::Completed,
            ],
        ]);
        let tools = FakeTools {
            definitions: vec![ToolDefinition {
                name: "lookup".to_string(),
                description: "lookup".to_string(),
                permission_scope: PermissionScope::Read,
                minimum_permission_level: PermissionLevel::Low,
            }],
            results: BTreeMap::from([(
                "lookup".to_string(),
                Ok(ToolExecutionOutput {
                    tool_call_id: "tool-1".to_string(),
                    output: json!({ "result": "ok" }),
                    summary: None,
                }),
            )]),
        };
        let permissions = FakePermissions {
            allowed: true,
            ..FakePermissions::default()
        };
        let hooks = FakeHooks::default();
        let mut engine =
            ConversationEngine::new(host_config(), Vec::new(), model, tools, permissions, hooks);
        let mut session = session_with_user_message();
        let mut observed = Vec::new();

        let result = engine
            .run_turn(&mut session, &prompt(), |event| {
                observed.push(format!("{event:?}"))
            })
            .expect("ordered turn succeeds");

        assert_eq!(observed.len(), result.events.len());
        assert_eq!(
            event_names(&result.events),
            vec![
                "request_prepared",
                "assistant_delta",
                "assistant_tool_call",
                "assistant_completed",
                "permission_resolved",
                "tool_execution_started",
                "tool_execution_finished",
                "request_prepared",
                "assistant_delta",
                "assistant_completed",
                "summary_updated",
                "turn_completed",
            ]
        );
    }
}
