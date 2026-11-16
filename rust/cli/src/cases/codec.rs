//! Codec cases.

use nyaaai_agentic::codec::{Codec, Mode};
use nyaaai_agentic::dialects::local::{LocalDialect, LocalReplay, local_chat};
use nyaaai_agentic::dialects::opencode::opencode_chat;
use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::gemini::codec::gemini_codec;
use nyaaai_agentic::hosts::opencode::{OpenCodeHost, opencode_host};
use nyaaai_agentic::messages::codec::messages_codec;
use nyaaai_agentic::responses::codec::responses_codec;
use nyaaai_agentic::step::StepAssembler;
use nyaaai_contract::shapes::step::Failure;
use nyaaai_contract::time::SteppedClock;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

type Built = Box<dyn Codec>;

fn local_of(name: &CodecName) -> Outcome<Built> {
    let Some(local) = &name.local else {
        return Err("a local codec needs its `local` part".to_owned());
    };
    if let Some(path) = &local.chat_path
        && !path.starts_with('/')
    {
        return Err(format!("a chat path starts with a slash: {path}"));
    }
    let dialect = LocalDialect {
        label: local.label.clone(),
        replay: local.replay.map(|replay| match replay {
            LocalNameReplay::Reasoning => LocalReplay::Reasoning,
            LocalNameReplay::ReasoningContent => LocalReplay::ReasoningContent,
        }),
        chat_path: local.chat_path.clone(),
        optional_key: local.optional_key == Some(true),
    };
    Ok(Box::new(local_chat(
        name.facts.as_ref(),
        &local.host,
        dialect,
    )))
}

fn host_of(name: &CodecName) -> Outcome<OpenCodeHost<SteppedClock>> {
    let Some(prefix) = &name.prefix else {
        return Err("a native codec needs its `prefix`".to_owned());
    };
    Ok(opencode_host(
        prefix,
        SteppedClock::new(name.now_ms.unwrap_or(0), 0),
    ))
}

/// A later family adds one arm here, and one row in `codecOf.ts`.
fn codec_of(name: &CodecName) -> Outcome<Built> {
    let key = format!("{}/{}", name.family.as_str(), name.dialect);
    match key.as_str() {
        "chat-completions/openrouter" => Ok(Box::new(OPENROUTER_CHAT)),
        "chat-completions/opencode" => Ok(Box::new(opencode_chat(
            name.facts.as_ref(),
            SteppedClock::new(name.now_ms.unwrap_or(0), 0),
        ))),
        "chat-completions/local" => local_of(name),
        "responses/opencode" => Ok(Box::new(responses_codec(
            host_of(name)?,
            name.facts.as_ref(),
        ))),
        "messages/opencode" => Ok(Box::new(messages_codec(
            host_of(name)?,
            name.facts.as_ref(),
        ))),
        "generate-content/opencode" => {
            Ok(Box::new(gemini_codec(host_of(name)?, name.facts.as_ref())))
        }
        _ => Err(format!("no codec named {key}")),
    }
}

pub fn encode(input: CodecEncodeInput) -> Outcome<CodecEncodeResult> {
    let mode = match input.mode {
        CodecEncodeInputMode::Stream => Mode::Stream,
        CodecEncodeInputMode::Complete => Mode::Complete,
    };
    Ok(match codec_of(&input.codec)?.encode(&input.request, mode) {
        Ok(encoded) => CodecEncodeResult {
            encoded: Some(encoded),
            refusal: None,
        },
        Err(refusal) => CodecEncodeResult {
            encoded: None,
            refusal: Some(refusal),
        },
    })
}

pub fn stream(input: CodecStreamInput) -> Outcome<CodecStreamResult> {
    let codec = codec_of(&input.codec)?;
    let mut decoder = codec.stream();
    let mut assembler = StepAssembler::new();
    let mut events = Vec::new();
    let mut failure: Option<Failure> = None;
    let mut fed = |pushed: Vec<_>| {
        for event in pushed {
            events.push(event);
            let held = events.last().map(|event| assembler.add(event));
            if let Some(Err(error)) = held {
                return Some(*error.failure);
            }
        }
        None
    };
    for item in &input.items {
        let mut pushed = Vec::new();
        let outcome = decoder.push(item, &mut pushed);
        failure = fed(pushed).or(outcome.err().map(|error| *error.failure));
        if failure.is_some() {
            break;
        }
    }
    if failure.is_none() {
        let mut pushed = Vec::new();
        let outcome = decoder.end(&mut pushed);
        failure = fed(pushed).or(outcome.err().map(|error| *error.failure));
    }
    let result = failure.is_none().then(|| assembler.result());
    Ok(CodecStreamResult {
        events,
        result,
        failure,
    })
}

pub fn complete(input: CodecCompleteInput) -> Outcome<CodecCompleteResult> {
    Ok(match codec_of(&input.codec)?.complete(&input.body) {
        Ok(result) => CodecCompleteResult {
            result: Some(result),
            failure: None,
        },
        Err(error) => CodecCompleteResult {
            result: None,
            failure: Some(*error.failure),
        },
    })
}

pub fn failure(input: CodecFailureInput) -> Outcome<CodecFailureResult> {
    Ok(CodecFailureResult {
        failure: codec_of(&input.codec)?.failure(input.status, &input.body, &input.headers),
    })
}
