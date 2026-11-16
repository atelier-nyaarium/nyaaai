//! Native helper cases.

use nyaaai_agentic::failure::{NumberOrText, WireError as Wire};
use nyaaai_agentic::native::{
    Host, HostStatus, arguments_of, assembled as assemble_events, current_turn_from, is_json,
    number_of, parsed_chunk, schema_prompt as prompt_of, serving_events,
    stream_failure as failure_of, string_of, system_texts as system_of,
    wire_error as wire_error_of,
};
use nyaaai_contract::json::{Object, Value, parse_json};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CanonicalRequestCache, CanonicalRequestCacheKind, Failure, FailureStatus,
    HeaderList, NormalizedUsage,
};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

/// JSON text read as the codecs read provider JSON.
fn json_of(text: &str) -> Outcome<Value> {
    parse_json(text).ok_or_else(|| "a value text is not JSON".to_owned())
}

/// A host that only names where its failure came from.
struct CaseHost;

impl Host for CaseHost {
    fn prefix(&self) -> &str {
        "/v1"
    }

    fn headers(&self, _request: &CanonicalRequest) -> Vec<(String, String)> {
        Vec::new()
    }

    fn usage(&self, own: NormalizedUsage, _raw: &Object) -> NormalizedUsage {
        own
    }

    fn failure(&self, status: HostStatus, error: &Wire, _headers: Option<&HeaderList>) -> Failure {
        Failure {
            category: nyaaai_contract::shapes::step::FailureCategory::Stream,
            status: match status {
                HostStatus::Http(code) => FailureStatus::Number(code as f64),
                HostStatus::Stream => FailureStatus::Text("stream".to_owned()),
            },
            retry_after_ms: None,
            resets_at: None,
            message: error.message.clone().unwrap_or_default(),
            reasons: None,
            billed: None,
        }
    }
}

fn request_of(system: Vec<String>, summary: Option<String>) -> CanonicalRequest {
    CanonicalRequest {
        model: "m-1".to_owned(),
        system,
        summary,
        history: Vec::new(),
        state: None,
        tools: None,
        tool_choice: None,
        schema: None,
        effort: None,
        cache: CanonicalRequestCache {
            kind: CanonicalRequestCacheKind::None,
            ttl: None,
        },
        max_tokens: None,
        session: "s1".to_owned(),
        zdr_only: false,
    }
}

fn wire_of(held: &WireError) -> Wire {
    let stamp = |code: Option<&WireErrorCode>| {
        code.map(|code| match code {
            WireErrorCode::Number(number) => NumberOrText::Number(*number),
            WireErrorCode::Text(text) => NumberOrText::Text(text.clone()),
        })
    };
    Wire {
        code: stamp(held.code.as_ref()),
        type_: held.type_.clone(),
        message: held.message.clone(),
        metadata: held.metadata.clone(),
        resets_at: held.resets_at.as_ref().map(|at| match at {
            WireErrorResetsAt::Number(number) => NumberOrText::Number(*number),
            WireErrorResetsAt::Text(text) => NumberOrText::Text(text.clone()),
        }),
    }
}

fn wire_back(held: Wire) -> WireError {
    let stamp = |at: NumberOrText| match at {
        NumberOrText::Number(number) => WireErrorCode::Number(number),
        NumberOrText::Text(text) => WireErrorCode::Text(text),
    };
    WireError {
        code: held.code.map(stamp),
        type_: held.type_,
        message: held.message,
        metadata: held.metadata,
        resets_at: held.resets_at.map(|at| match at {
            NumberOrText::Number(number) => WireErrorResetsAt::Number(number),
            NumberOrText::Text(text) => WireErrorResetsAt::Text(text),
        }),
    }
}

pub fn readers(input: NativeReadersInput) -> Outcome<NativeReadersResult> {
    let rows = input
        .texts
        .iter()
        .map(|text| {
            let value = json_of(text)?;
            Ok(NativeReadersResultRowsItem {
                is_json: is_json(&value),
                string: string_of(&value).to_owned(),
                number: number_of(&value),
            })
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(NativeReadersResult { rows })
}

pub fn schema_prompt(input: NativeSchemaPromptInput) -> Outcome<NativeSchemaPromptResult> {
    Ok(NativeSchemaPromptResult {
        texts: input.schemas.iter().map(prompt_of).collect(),
    })
}

pub fn system_texts(input: NativeSystemTextsInput) -> Outcome<NativeSystemTextsResult> {
    Ok(NativeSystemTextsResult {
        texts: input
            .rows
            .into_iter()
            .map(|row| {
                system_of(
                    &request_of(row.system, row.summary),
                    row.prompted.as_deref(),
                )
            })
            .collect(),
    })
}

pub fn arguments(input: NativeArgumentsInput) -> Outcome<NativeArgumentsResult> {
    Ok(NativeArgumentsResult {
        values: input.texts.iter().map(|text| arguments_of(text)).collect(),
    })
}

pub fn current_turn(input: NativeCurrentTurnInput) -> Outcome<NativeCurrentTurnResult> {
    Ok(NativeCurrentTurnResult {
        indexes: input
            .histories
            .iter()
            .map(|history| current_turn_from(history) as i64)
            .collect(),
    })
}

pub fn chunk(input: NativeChunkInput) -> Outcome<NativeChunkResult> {
    Ok(NativeChunkResult {
        rows: input
            .texts
            .iter()
            .map(|text| match parsed_chunk(text) {
                Ok(chunk) => NativeChunkResultRowsItem {
                    chunk: Some(chunk),
                    failure: None,
                },
                Err(error) => NativeChunkResultRowsItem {
                    chunk: None,
                    failure: Some(*error.failure),
                },
            })
            .collect(),
    })
}

pub fn wire_error(input: NativeWireErrorInput) -> Outcome<NativeWireErrorResult> {
    let errors = input
        .rows
        .iter()
        .map(|row| Ok(wire_back(wire_error_of(row.status, &json_of(&row.body)?))))
        .collect::<Outcome<Vec<_>>>()?;
    Ok(NativeWireErrorResult { errors })
}

pub fn stream_failure(input: NativeStreamFailureInput) -> Outcome<NativeStreamFailureResult> {
    let error = failure_of(
        &CaseHost,
        &wire_of(&input.error),
        input.usage,
        &input.serving,
    );
    Ok(NativeStreamFailureResult {
        failure: *error.failure,
    })
}

pub fn assembled(input: NativeAssembledInput) -> Outcome<NativeAssembledResult> {
    Ok(match assemble_events(&input.events) {
        Ok(result) => NativeAssembledResult {
            result: Some(result),
            failure: None,
        },
        Err(error) => NativeAssembledResult {
            result: None,
            failure: Some(*error.failure),
        },
    })
}

pub fn serving(input: NativeServingInput) -> Outcome<NativeServingResult> {
    Ok(NativeServingResult {
        events: input.ids.iter().map(serving_events).collect(),
    })
}
