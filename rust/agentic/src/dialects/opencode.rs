//! OpenCode Chat dialect.

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CodecFacts, Effort, Failure, HeaderList, NormalizedUsage, Serving,
    ThoughtDelta, ThoughtPart, ThoughtPartSegmentsItem, ThoughtPartSegmentsItemKind, TurnPart,
    TurnStep,
};
use nyaaai_contract::time::Clock;

use crate::chat::codec::{ChatCodec, ChatDialect, ToolChoiceRule, chat_codec};
use crate::chat::text::thought_text;
use crate::chat::wire::{Delta, Usage};
use crate::codec::Mode;
use crate::failure::WireError;
use crate::hosts::opencode::{SESSION_HEADER, opencode_chat_usage, opencode_failure};
use crate::native::HostStatus;
use crate::schema_projection::strict_schema;

const REASONING: &str = "reasoning_content";

/// Post-translation token floor.
const MIN_TOKENS: i64 = 16;

/// DeepSeek V4 aliases.
fn family_of(model: &str) -> &str {
    let name = model.rfind('/').map_or(model, |at| &model[at + 1..]);
    name.find('-').map_or(name, |dash| &name[..dash])
}

fn own_thoughts(step: &TurnStep, type_: &str) -> String {
    step.parts
        .iter()
        .filter_map(|part| match part {
            TurnPart::Thought(thought) if thought.type_ == type_ => Some(thought_text(thought)),
            _ => None,
        })
        .collect()
}

/// Preserve tool-turn reasoning.
fn replay_of(step: &TurnStep, request: &CanonicalRequest) -> Object {
    let own = match &step.producer {
        Some(producer) if family_of(&producer.model) == family_of(&request.model) => {
            own_thoughts(step, REASONING)
        }
        _ => String::new(),
    };
    let mut held = Object::new();
    if !own.is_empty() {
        held.insert(REASONING, Value::String(own));
    } else if step
        .parts
        .iter()
        .any(|part| matches!(part, TurnPart::Call(_)))
    {
        held.insert(REASONING, Value::from(""));
    }
    held
}

/// A reasoning thought from plain text.
pub fn reasoning_thought(type_: &str, text: &str) -> ThoughtDelta {
    ThoughtDelta {
        part: ThoughtPart {
            type_: type_.to_owned(),
            segments: Some(vec![ThoughtPartSegmentsItem {
                kind: ThoughtPartSegmentsItemKind::Content,
                text: text.to_owned(),
            }]),
            opaque: None,
            id: None,
            format: None,
            index: None,
            anchor: None,
        },
        block: None,
        split: None,
    }
}

/// Map named and any-tool.
fn choice_of(choice: &[String], name: &str) -> ToolChoiceRule {
    if choice.iter().any(|held| held == "tool") {
        let mut function = Object::new();
        function.insert("name", Value::from(name));
        let mut held = Object::new();
        held.insert("type", Value::from("function"));
        held.insert("function", Value::Object(function));
        return ToolChoiceRule::Send(Value::Object(held));
    }
    if choice.iter().any(|held| held == "required") {
        ToolChoiceRule::Send(Value::from("required"))
    } else {
        ToolChoiceRule::Omit
    }
}

/// Request fields of URL-server and OpenCode dialects.
pub fn effort_fields(efforts: &[Effort], request: &CanonicalRequest, mode: Mode) -> Object {
    let mut fields = Object::new();
    if mode == Mode::Stream {
        let mut options = Object::new();
        options.insert("include_usage", Value::Bool(true));
        fields.insert("stream_options", Value::Object(options));
    }
    if let Some(effort) = request.effort
        && efforts.contains(&effort)
    {
        fields.insert("reasoning_effort", Value::from(effort.as_str()));
    }
    fields
}

fn normalized(usage: &Usage) -> NormalizedUsage {
    let prompt = usage.prompt_tokens_details.as_ref();
    opencode_chat_usage(
        NormalizedUsage {
            input: usage.prompt_tokens,
            output: usage.completion_tokens,
            cached: prompt.and_then(|held| held.cached_tokens),
            cache_write: prompt.and_then(|held| held.cache_write_tokens),
            reasoning: usage.reasoning_tokens.flatten(),
            cost: None,
            upstream_cost: None,
            byok: None,
            billing_source: None,
            list_cost: None,
            allowance: None,
        },
        usage,
    )
}

/// Catalog gates optional fields.
pub struct OpenCodeDialect<C> {
    facts: Option<CodecFacts>,
    clock: C,
}

impl<C: Clock + Send + Sync> ChatDialect for OpenCodeDialect<C> {
    fn path(&self) -> &str {
        "/openai/v1/chat/completions"
    }

    fn headers(&self, request: &CanonicalRequest) -> Option<Record<String>> {
        let mut held = Record::new();
        held.insert(SESSION_HEADER, request.session.clone());
        Some(held)
    }

    fn fields(&self, request: &CanonicalRequest, mode: Mode) -> Object {
        let efforts = self.facts.as_ref().map_or(&[][..], |facts| &facts.efforts);
        effort_fields(efforts, request, mode)
    }

    fn replay(&self, step: &TurnStep, request: &CanonicalRequest) -> Object {
        let api = self.facts.as_ref().map(|facts| &facts.api);
        if api.and_then(|api| api.reasoning.as_deref()) == Some(REASONING) {
            replay_of(step, request)
        } else {
            Object::new()
        }
    }

    fn thoughts(&self, delta: &Delta) -> Vec<ThoughtDelta> {
        match delta.reasoning_content.as_deref() {
            Some(text) if !text.is_empty() => vec![reasoning_thought(REASONING, text)],
            _ => Vec::new(),
        }
    }

    fn schema(&self, json: &Object) -> Result<Option<Object>, String> {
        let structured = self
            .facts
            .as_ref()
            .is_some_and(|facts| facts.api.structured);
        if structured {
            strict_schema(json).map(Some)
        } else {
            Ok(None)
        }
    }

    fn tool_choice(&self, name: &str) -> ToolChoiceRule {
        let choice = self
            .facts
            .as_ref()
            .map_or(&[][..], |facts| &facts.api.choice);
        choice_of(choice, name)
    }

    fn min_tokens(&self) -> i64 {
        MIN_TOKENS
    }

    fn usage(&self, usage: &Usage) -> NormalizedUsage {
        normalized(usage)
    }

    fn serving(&self, id: Option<&str>, _provider: Option<&str>) -> Serving {
        Serving {
            host: None,
            generation_id: id.map(str::to_owned),
        }
    }

    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        headers: Option<&HeaderList>,
    ) -> Failure {
        opencode_failure(&self.clock, status, error, headers)
    }
}

/// Reset stamps count from the clock.
pub fn opencode_chat<C: Clock + Send + Sync>(
    facts: Option<&CodecFacts>,
    clock: C,
) -> ChatCodec<OpenCodeDialect<C>> {
    chat_codec(OpenCodeDialect {
        facts: facts.cloned(),
        clock,
    })
}

/// Failures before model lookup.
pub fn opencode_errors<C: Clock + Send + Sync>(clock: C) -> ChatCodec<OpenCodeDialect<C>> {
    opencode_chat(None, clock)
}
