//! Helpers shared by the native API codecs.

use nyaaai_contract::json::{Object, Value, parse_json, stringify_json};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Failure, FailureBilled, FailureCategory, FinishReason, HeaderList,
    NormalizedUsage, Serving, StepEvent, StepEventServing, StepResult, Turn,
};

use crate::failure::{FailureExtra, NumberOrText, ProviderError, WireError, failed, message_of};
use crate::step::StepAssembler;

/// Loose provider JSON.
pub type Json = Object;

/// Where a failure came from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HostStatus {
    Http(i64),
    Stream,
}

/// Gateway-specific API behavior.
pub trait Host {
    /// API path prefix.
    fn prefix(&self) -> &str;

    fn headers(&self, request: &CanonicalRequest) -> Vec<(String, String)>;

    /// Billing and usage adjustments.
    fn usage(&self, own: NormalizedUsage, raw: &Object) -> NormalizedUsage;

    /// HTTP 200 stream errors.
    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        headers: Option<&HeaderList>,
    ) -> Failure;
}

pub fn is_json(value: &Value) -> bool {
    matches!(value, Value::Object(_))
}

pub fn string_of(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}

pub fn number_of(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

pub fn schema_prompt(json: &Object) -> String {
    format!(
        "Answer with one JSON object matching this JSON Schema, and nothing else:\n{}",
        stringify_json(&Value::Object(json.clone()))
    )
}

/// System, summary, prompt schema.
pub fn system_texts(request: &CanonicalRequest, prompted: Option<&str>) -> Vec<String> {
    let mut texts = request.system.clone();
    texts.extend(request.summary.clone());
    texts.extend(prompted.map(str::to_owned));
    texts
}

/// Non-object arguments become `{}`.
pub fn arguments_of(arguments_text: &str) -> Json {
    match parse_json(arguments_text) {
        Some(Value::Object(parsed)) => parsed,
        _ => Json::new(),
    }
}

/// Index after latest user.
pub fn current_turn_from(history: &[Turn]) -> usize {
    history
        .iter()
        .rposition(|turn| matches!(turn, Turn::User(_)))
        .map_or(0, |at| at + 1)
}

pub fn parsed_chunk(data: &str) -> Result<Json, ProviderError> {
    match parse_json(data) {
        Some(Value::Object(chunk)) => Ok(chunk),
        _ => Err(failed(
            FailureCategory::Stream,
            "A stream chunk was not JSON.",
            FailureExtra::default(),
        )),
    }
}

/// Native errors wrap `error`; a text is the message.
pub fn wire_error(status: i64, body: &Value) -> WireError {
    let held = body.get("error");
    let error = held.and_then(Value::as_object);
    let field = |key: &str| error.and_then(|error| error.get(key));
    let type_ = field("type")
        .and_then(Value::as_str)
        .or_else(|| field("status").and_then(Value::as_str))
        .map(str::to_owned);
    let message = match held {
        Some(text @ Value::String(_)) => message_of(text),
        _ => field("message").and_then(message_of),
    }
    .map_or_else(|| format!("HTTP {status}"), str::to_owned);
    let resets_at = match field("resetsAt") {
        Some(Value::String(text)) => Some(NumberOrText::Text(text.clone())),
        Some(Value::Number(number)) => Some(NumberOrText::Number(*number)),
        _ => None,
    };
    WireError {
        code: None,
        type_,
        message: Some(message),
        metadata: None,
        resets_at,
    }
}

/// Stream failure with billing.
pub fn stream_failure(
    host: &dyn Host,
    error: &WireError,
    usage: Option<NormalizedUsage>,
    serving: &Serving,
) -> ProviderError {
    let failure = host.failure(HostStatus::Stream, error, None);
    if usage.is_none() && serving.generation_id.is_none() && serving.host.is_none() {
        return ProviderError::new(failure);
    }
    ProviderError::new(Failure {
        billed: Some(FailureBilled {
            usage,
            host: serving.host.clone(),
            generation_id: serving.generation_id.clone(),
        }),
        ..failure
    })
}

/// Assemble whole-reply events.
pub fn assembled<'a>(
    events: impl IntoIterator<Item = &'a StepEvent>,
) -> Result<StepResult, ProviderError> {
    let mut assembler = StepAssembler::new();
    for event in events {
        assembler.add(event)?;
    }
    let result = assembler.result();
    if result.finish == FinishReason::Empty {
        return Err(failed(
            FailureCategory::Stream,
            "The provider ended the step.",
            FailureExtra::default(),
        ));
    }
    Ok(result)
}

pub fn serving_events(generation_id: &Value) -> Vec<StepEvent> {
    match generation_id.as_str() {
        Some(id) if !id.is_empty() => vec![StepEvent::Serving(StepEventServing {
            serving: Serving {
                host: None,
                generation_id: Some(id.to_owned()),
            },
        })],
        _ => Vec::new(),
    }
}
