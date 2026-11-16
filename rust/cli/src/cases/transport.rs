//! Transport cases.

use std::path::Path;
use std::sync::{Arc, Mutex};

use nyaaai_agentic::codec::{Codec, Mode};
use nyaaai_agentic::transport::{
    Auth, KeyAuth, KeySource, Transport, TransportDeps, TransportError, is_key_shape, normalize_key,
};
use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::Method;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::step::{CanonicalRequest, Encoded, Failure, HeaderList, StepResult};
use nyaaai_testkit::cases::{CaseContext, Outcome, block_on, failed};
use nyaaai_testkit::keys::FakeKeys;
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_fetch::{ScriptedChunk, ScriptedEnd, ScriptedExchange, ScriptedFetch};
use nyaaai_testkit::trace::{read_trace, replay_fetch};

use super::codec::codec_of;
use super::shapes::*;

const CALLER_REASON: &str = "caller aborted";

/// The codec's headers, with the case's added to every request.
struct HeadedCodec {
    inner: Arc<dyn Codec>,
    added: Vec<(String, String)>,
}

impl Codec for HeadedCodec {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        let encoded = self.inner.encode(request, mode)?;
        let mut headers = encoded.headers.clone().unwrap_or_default();
        for (name, value) in &self.added {
            headers.insert(name.as_str(), value.clone());
        }
        Ok(Encoded {
            headers: Some(headers),
            ..encoded
        })
    }

    fn stream(&self) -> Box<dyn nyaaai_agentic::codec::StreamDecoder + '_> {
        self.inner.stream()
    }

    fn complete(&self, body: &Value) -> Result<StepResult, nyaaai_agentic::failure::ProviderError> {
        self.inner.complete(body)
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.inner.failure(status, body, headers)
    }
}

struct Keys(Arc<FakeKeys>);

impl KeySource for Keys {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        self.0.get()
    }
}

fn bytes_of(chunk: &TransportChunk) -> Outcome<Vec<u8>> {
    match chunk {
        TransportChunk::Text(held) => Ok(held.text.clone().into_bytes()),
        TransportChunk::Repeat(held) => Ok(held.text.repeat(held.times as usize).into_bytes()),
        TransportChunk::Bytes(held) => {
            base64_decode(&held.base64).ok_or_else(|| "a chunk is not base64".to_owned())
        }
    }
}

fn delay_of(chunk: &TransportChunk) -> i64 {
    match chunk {
        TransportChunk::Text(held) => held.delay_ms,
        TransportChunk::Repeat(held) => held.delay_ms,
        TransportChunk::Bytes(held) => held.delay_ms,
    }
    .unwrap_or(0)
}

fn exchange_of(held: &TransportExchange) -> Outcome<ScriptedExchange> {
    let chunks = held
        .chunks
        .iter()
        .map(|chunk| {
            Ok(ScriptedChunk {
                bytes: bytes_of(chunk)?,
                delay_ms: delay_of(chunk),
            })
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ScriptedExchange {
        status: held.status as u16,
        headers: held.headers.clone(),
        chunks,
        head_ms: held.head_ms.unwrap_or(0),
        end: match held.end {
            None | Some(TransportExchangeEnd::Closed) => ScriptedEnd::Closed,
            Some(TransportExchangeEnd::Dropped) => ScriptedEnd::Dropped,
            Some(TransportExchangeEnd::Refused) => ScriptedEnd::Refused,
        },
        error: held.error.clone(),
    })
}

fn key_auth_of(held: Option<&TransportKeyAuth>) -> KeyAuth {
    match held {
        None | Some(TransportKeyAuth::Stored(_)) => KeyAuth::Stored,
        Some(TransportKeyAuth::None(_)) => KeyAuth::None,
        Some(TransportKeyAuth::Key(held)) => KeyAuth::Key(held.key.clone()),
    }
}

fn slashed(path: &str) -> Outcome<&str> {
    if path.starts_with('/') {
        Ok(path)
    } else {
        Err(format!("a JSON path starts with a slash: {path}"))
    }
}

fn aborts(call: &TransportCall) -> (Option<i64>, bool) {
    let (after, before) = match call {
        TransportCall::Stream(held) => (held.abort_after_ms, held.aborted_before),
        TransportCall::Complete(held) => (held.abort_after_ms, held.aborted_before),
        TransportCall::GetJson(held) => (held.abort_after_ms, held.aborted_before),
        TransportCall::PostJson(held) => (held.abort_after_ms, held.aborted_before),
    };
    (after, before == Some(true))
}

fn outcome() -> TransportOutcome {
    TransportOutcome {
        events: Vec::new(),
        result: None,
        answer: Value::Null,
        failure: None,
        thrown: None,
        aborted: false,
        now_ms: 0,
    }
}

fn recorded(outcome: &mut TransportOutcome, error: TransportError) {
    match error {
        TransportError::Cancelled => outcome.thrown = Some(CALLER_REASON.to_owned()),
        TransportError::Provider(error) => outcome.failure = Some(*error.failure),
    }
}

async fn run_call(
    transport: &Transport,
    codec: &dyn Codec,
    call: &TransportCall,
    caller: &CancelToken,
) -> Outcome<TransportOutcome> {
    let mut outcome = outcome();
    match call {
        TransportCall::Stream(held) => {
            let mut stream = transport.stream(codec, &held.request, caller);
            loop {
                match stream.next().await {
                    Ok(Some(event)) => outcome.events.push(event),
                    Ok(None) => break,
                    Err(error) => {
                        recorded(&mut outcome, error);
                        break;
                    }
                }
            }
        }
        TransportCall::Complete(held) => {
            match transport.complete(codec, &held.request, caller).await {
                Ok(result) => outcome.result = Some(result),
                Err(error) => recorded(&mut outcome, error),
            }
        }
        TransportCall::GetJson(held) => {
            let auth = key_auth_of(held.key_auth.as_ref());
            match transport
                .get_json(slashed(&held.path)?, caller, &auth)
                .await
            {
                Ok(answer) => outcome.answer = answer,
                Err(error) => recorded(&mut outcome, error),
            }
        }
        TransportCall::PostJson(held) => {
            let auth = key_auth_of(held.key_auth.as_ref());
            let sent = transport
                .post_json(slashed(&held.path)?, held.body.as_ref(), caller, &auth)
                .await;
            match sent {
                Ok(answer) => outcome.answer = answer,
                Err(error) => recorded(&mut outcome, error),
            }
        }
    }
    Ok(outcome)
}

fn played_of(
    script: &TransportScript,
    wheel: &Arc<TimerWheel>,
    root: &Path,
) -> Outcome<ScriptedFetch> {
    match script {
        TransportScript::Inline(held) => {
            let exchanges = held
                .exchanges
                .iter()
                .map(exchange_of)
                .collect::<Outcome<Vec<_>>>()?;
            Ok(ScriptedFetch::new(exchanges, Arc::clone(wheel)))
        }
        TransportScript::Trace(held) => {
            let trace = read_trace(&root.join(&held.file)).map_err(failed)?;
            replay_fetch(&trace).map_err(failed)
        }
    }
}

pub fn exchange(
    input: TransportExchangeInput,
    context: &CaseContext,
) -> Outcome<TransportExchangeResult> {
    let wheel = Arc::new(TimerWheel::new(input.start_ms));
    let played = Arc::new(played_of(&input.script, &wheel, context.root())?);
    let keys = Arc::new(
        FakeKeys::new(input.keys.clone()).ok_or_else(|| "fakeKeys needs a value".to_owned())?,
    );
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let settled = Arc::new(Mutex::new(Vec::<String>::new()));
    let base: Arc<dyn Codec> = Arc::from(codec_of(&input.codec)?);
    let codec: Arc<dyn Codec> = match &input.codec_headers {
        None => Arc::clone(&base),
        Some(added) => Arc::new(HeadedCodec {
            inner: Arc::clone(&base),
            added: added
                .iter()
                .map(|(name, value)| (name.to_owned(), value.clone()))
                .collect(),
        }),
    };
    let sink = Arc::clone(&log);
    let transport = Transport::new(TransportDeps {
        fetch: played.clone(),
        scheduler: Arc::new(FakeScheduler::new(Arc::clone(&wheel))),
        key: Arc::new(Keys(Arc::clone(&keys))),
        log: Box::new(move |line| {
            sink.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(line.to_owned());
        }),
        base_url: input.base_url.clone(),
        label: input.label.clone(),
        user_agent: input.user_agent.clone(),
        errors: base,
        auth: match input.auth {
            None | Some(TransportExchangeInputAuth::Bearer) => Auth::Bearer,
            Some(TransportExchangeInputAuth::None) => Auth::None,
            Some(TransportExchangeInputAuth::Optional) => Auth::Optional,
        },
        idle_ms: input.idle_ms,
    });
    let told = Arc::clone(&settled);
    transport.on_settled(move |model| {
        told.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(model.to_owned());
    });
    let calls = block_on(async {
        let mut calls = Vec::new();
        for call in &input.calls {
            let caller = CancelToken::new();
            let (after, before) = aborts(call);
            if before {
                caller.cancel();
            } else if let Some(ms) = after {
                let held = caller.clone();
                wheel.after_run(ms, "caller", Box::new(move || held.cancel()));
            }
            let mut outcome = run_call(&transport, &*codec, call, &caller).await?;
            wheel.cancel("caller");
            outcome.aborted = caller.is_cancelled();
            outcome.now_ms = nyaaai_contract::time::Clock::now_ms(&*wheel);
            calls.push(outcome);
        }
        Ok::<_, String>(calls)
    })??;
    let requests = played
        .requests()
        .into_iter()
        .map(|sent| TransportSent {
            method: match sent.method {
                Method::Get => "GET",
                Method::Post => "POST",
            }
            .to_owned(),
            url: sent.url,
            headers: sent.headers,
            body: sent.body,
        })
        .collect();
    let taken = |held: &Mutex<Vec<String>>| {
        held.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    };
    Ok(TransportExchangeResult {
        calls,
        requests,
        log: taken(&log),
        settled: taken(&settled),
        key_reads: keys.reads() as i64,
        read: played.read().into_iter().map(|read| read as i64).collect(),
        waiting: wheel.waiting(),
    })
}

pub fn key(input: TransportKeyInput) -> Outcome<TransportKeyResult> {
    Ok(TransportKeyResult {
        shape: is_key_shape(&input.text),
        normalized: normalize_key(&input.text),
    })
}
