//! Chat dialect for URL servers.

use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CodecFacts, Encoded, Failure, FailureCategory, HeaderList, NormalizedUsage,
    Serving, StepResult, ThoughtDelta, TurnPart, TurnStep,
};

use super::opencode::{effort_fields, reasoning_thought};
use crate::chat::codec::{
    ChatCodec, ChatDialect, ToolChoiceRule, chat_codec, code_of, failure_status, usable,
};
use crate::chat::text::thought_text;
use crate::chat::wire::{Delta, Usage};
use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{ProviderError, WireError};
use crate::native::HostStatus;

/// The field a server reads thoughts back from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalReplay {
    Reasoning,
    ReasoningContent,
}

impl LocalReplay {
    fn key(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::ReasoningContent => "reasoning_content",
        }
    }
}

/// One server's chat differences.
#[derive(Debug, Clone)]
pub struct LocalDialect {
    /// Failure text name.
    pub label: String,
    /// `None`: thoughts stay out.
    pub replay: Option<LocalReplay>,
    /// Defaults to `/v1/chat/completions`.
    pub chat_path: Option<String>,
    /// Requests carry a key when set.
    pub optional_key: bool,
}

const PATH: &str = "/v1/chat/completions";

const REASONING: &str = "reasoning";

fn category_of(code: Option<f64>, keyed: bool) -> FailureCategory {
    let Some(code) = code else {
        return FailureCategory::Stream;
    };
    if keyed && code == 401.0 {
        FailureCategory::KeyRejected
    } else if keyed && code == 402.0 {
        FailureCategory::OutOfCredit
    } else if code == 404.0 {
        FailureCategory::NotFound
    } else if code == 401.0 || code == 403.0 {
        FailureCategory::Refused
    } else if code == 429.0 {
        FailureCategory::RateLimited
    } else if code == 408.0 || code >= 500.0 {
        FailureCategory::Unavailable
    } else {
        FailureCategory::BadRequest
    }
}

/// Schema for forced tools.
fn forced_as_schema(request: &CanonicalRequest) -> CanonicalRequest {
    let forced = request.tool_choice.as_ref().and_then(|choice| {
        request
            .tools
            .iter()
            .flatten()
            .find(|tool| tool.name == choice.name)
    });
    let Some(forced) = forced else {
        return request.clone();
    };
    if request.schema.is_some() {
        return request.clone();
    }
    let mut made = request.clone();
    made.tool_choice = None;
    made.tools = None;
    made.schema = Some(nyaaai_contract::shapes::step::CanonicalRequestSchema {
        name: forced.name.clone(),
        json: forced.parameters.clone(),
    });
    made
}

/// Serving entry label.
pub struct LocalChatDialect {
    facts: Option<CodecFacts>,
    host: String,
    local: LocalDialect,
}

impl ChatDialect for LocalChatDialect {
    fn path(&self) -> &str {
        self.local.chat_path.as_deref().unwrap_or(PATH)
    }

    fn fields(&self, request: &CanonicalRequest, mode: Mode) -> Object {
        let efforts = self.facts.as_ref().map_or(&[][..], |facts| &facts.efforts);
        effort_fields(efforts, request, mode)
    }

    // Thoughts stay model-local.
    fn replay(&self, step: &TurnStep, request: &CanonicalRequest) -> Object {
        let mut held = Object::new();
        let own = step
            .producer
            .as_ref()
            .is_some_and(|producer| producer.model == request.model);
        let Some(replay) = self.local.replay.filter(|_| own) else {
            return held;
        };
        let text: String = step
            .parts
            .iter()
            .filter_map(|part| match part {
                TurnPart::Thought(thought) if thought.type_ == REASONING => {
                    Some(thought_text(thought))
                }
                _ => None,
            })
            .collect();
        if !text.is_empty() {
            held.insert(replay.key(), Value::String(text));
        }
        held
    }

    fn thoughts(&self, delta: &Delta) -> Vec<ThoughtDelta> {
        let text = delta
            .reasoning_content
            .as_deref()
            .or(delta.reasoning.as_deref());
        match text {
            Some(text) if !text.is_empty() => vec![reasoning_thought(REASONING, text)],
            _ => Vec::new(),
        }
    }

    // Grammar ignores `strict` and descriptions.
    fn schema(&self, json: &Object) -> Result<Option<Object>, String> {
        Ok(Some(json.clone()))
    }

    fn prompt_schema(&self) -> bool {
        true
    }

    fn tool_choice(&self, _name: &str) -> ToolChoiceRule {
        ToolChoiceRule::Omit
    }

    fn usage(&self, usage: &Usage) -> NormalizedUsage {
        let free = self
            .facts
            .as_ref()
            .is_some_and(|facts| facts.free == Some(true));
        NormalizedUsage {
            input: usage.prompt_tokens,
            output: usage.completion_tokens,
            cached: None,
            cache_write: None,
            reasoning: None,
            cost: free.then_some(0.0),
            upstream_cost: None,
            byok: None,
            billing_source: None,
            list_cost: None,
            allowance: None,
        }
    }

    fn serving(&self, id: Option<&str>, _provider: Option<&str>) -> Serving {
        Serving {
            host: Some(self.host.clone()),
            generation_id: id.map(str::to_owned),
        }
    }

    // 400: unsupported features possible.
    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        _headers: Option<&HeaderList>,
    ) -> Failure {
        let message = usable(error.message.as_deref()).map_or_else(
            || format!("{} reported an error.", self.local.label),
            str::to_owned,
        );
        Failure {
            category: category_of(code_of(status, error), self.local.optional_key),
            status: failure_status(status),
            retry_after_ms: None,
            resets_at: None,
            message,
            reasons: None,
            billed: None,
        }
    }
}

/// A URL server's codec.
pub struct LocalChat {
    inner: ChatCodec<LocalChatDialect>,
}

impl Codec for LocalChat {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        self.inner.encode(&forced_as_schema(request), mode)
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        self.inner.stream()
    }

    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError> {
        self.inner.complete(body)
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.inner.failure(status, body, headers)
    }
}

pub fn local_chat(facts: Option<&CodecFacts>, host: &str, local: LocalDialect) -> LocalChat {
    LocalChat {
        inner: chat_codec(LocalChatDialect {
            facts: facts.cloned(),
            host: host.to_owned(),
            local,
        }),
    }
}
