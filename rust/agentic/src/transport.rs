//! A codec's calls over the Fetch port.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use nyaaai_contract::base64::base64_encode;
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Chunks, Fetch, FetchError, FetchRequest, FetchResponse, Method};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::headers::header_of;
use nyaaai_contract::json::{Value, parse_json, stringify_json};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Encoded, Failure, FailureBilled, FailureCategory, HeaderList, Serving,
    StepEvent, StepEventServing, StepResult,
};
use nyaaai_contract::text::{Utf8Decoder, ascii_lower, encode_path_segment, trim_text};
use nyaaai_contract::time::{Scheduled, Scheduler, parse_retry_after};

use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{FailureExtra, ProviderError, failed};
use crate::sse::SseDecoder;

/// Where a key comes from.
pub trait KeySource: Send + Sync {
    fn get(&self) -> BoxFuture<'_, Option<String>>;
}

struct StaticKey(String);

impl KeySource for StaticKey {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async move { Some(self.0.clone()) })
    }
}

pub fn static_key(key: &str) -> impl KeySource {
    StaticKey(key.to_owned())
}

/// What the deps' key means for a request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Auth {
    #[default]
    Bearer,
    None,
    /// A bearer when a key exists.
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAuth {
    None,
    Stored,
    Key(String),
}

pub struct TransportDeps {
    pub fetch: Arc<dyn Fetch>,
    /// Idle timer; dates `Retry-After`.
    pub scheduler: Arc<dyn Scheduler>,
    pub key: Arc<dyn KeySource>,
    /// Status, generation, host.
    pub log: Box<dyn Fn(&str) + Send + Sync>,
    pub base_url: String,
    /// Provider error/log label.
    pub label: String,
    /// The consumer's `name/version`.
    pub user_agent: String,
    /// Decodes failed reads.
    pub errors: Arc<dyn Codec>,
    pub auth: Auth,
    /// Default two minutes.
    pub idle_ms: Option<i64>,
}

/// How a call ended, short of an answer.
#[derive(Debug, Clone, PartialEq)]
pub enum TransportError {
    /// The caller's token was cancelled.
    Cancelled,
    Provider(ProviderError),
}

impl fmt::Display for TransportError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => out.write_str("the call was cancelled"),
            Self::Provider(error) => error.fmt(out),
        }
    }
}

impl Error for TransportError {}

impl From<ProviderError> for TransportError {
    fn from(error: ProviderError) -> Self {
        Self::Provider(error)
    }
}

/// Names a listener of `on_settled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListenerId(u64);

/// What went wrong inside a call, before it is mapped.
enum Fault {
    Provider(ProviderError),
    Fetch(FetchError),
}

impl From<ProviderError> for Fault {
    fn from(error: ProviderError) -> Self {
        Self::Provider(error)
    }
}

impl From<FetchError> for Fault {
    fn from(error: FetchError) -> Self {
        Self::Fetch(error)
    }
}

#[derive(Default)]
struct Seen {
    status: Option<u16>,
    generation: Option<String>,
    host: Option<String>,
}

const IDLE_MS: i64 = 120_000;
const ERROR_BODY_BYTES: usize = 64 * 1024;
/// The names a codec cannot set.
const OWNED_HEADERS: [&str; 4] = ["content-type", "accept", "authorization", "user-agent"];
const REDACTED: &str = "[key]";

type Listener = Arc<dyn Fn(&str) + Send + Sync>;

/// Scrubbed codec transport.
pub struct Transport {
    d: TransportDeps,
    idle_ms: i64,
    settled: Mutex<Vec<(u64, Listener)>>,
    listeners: AtomicU64,
}

/// The caller's token and the idle abort, as one.
struct Cut {
    /// Cancelled by the caller or by silence.
    token: CancelToken,
    /// Whether the idle abort fired.
    idle: Arc<AtomicBool>,
    scheduler: Arc<dyn Scheduler>,
    ms: i64,
    pending: Mutex<Option<Scheduled>>,
}

impl Cut {
    fn new(caller: &CancelToken, scheduler: Arc<dyn Scheduler>, ms: i64) -> Self {
        let cut = Self {
            token: caller.child(),
            idle: Arc::new(AtomicBool::new(false)),
            scheduler,
            ms,
            pending: Mutex::new(None),
        };
        cut.arm();
        cut
    }

    /// One timer, replaced at every call.
    fn arm(&self) {
        self.clear();
        let token = self.token.clone();
        let idle = Arc::clone(&self.idle);
        let scheduled = self.scheduler.schedule(
            self.ms,
            Box::new(move || {
                idle.store(true, Ordering::SeqCst);
                token.cancel();
            }),
        );
        *self.slot() = Some(scheduled);
    }

    fn clear(&self) {
        if let Some(scheduled) = self.slot().take() {
            scheduled.cancel();
        }
    }

    fn idled(&self) -> bool {
        self.idle.load(Ordering::SeqCst)
    }

    fn slot(&self) -> std::sync::MutexGuard<'_, Option<Scheduled>> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for Cut {
    fn drop(&mut self) {
        self.clear();
    }
}

/// An open stream's state.
struct Open<'a> {
    key: Option<String>,
    cut: Cut,
    seen: Seen,
    body: Option<Box<dyn Chunks>>,
    body_done: bool,
    sse: SseDecoder,
    decoder: Box<dyn StreamDecoder + 'a>,
    events: VecDeque<StepEvent>,
    failed: Option<Fault>,
    ended: bool,
}

impl Open<'_> {
    fn scrubbed(&mut self, event: StepEvent) -> StepEvent {
        let StepEvent::Serving(held) = event else {
            return event;
        };
        let serving = scrubbed_serving(&held.serving, self.key.as_deref());
        if self.seen.generation.is_none() {
            self.seen.generation.clone_from(&serving.generation_id);
        }
        if self.seen.host.is_none() {
            self.seen.host.clone_from(&serving.host);
        }
        StepEvent::Serving(StepEventServing { serving })
    }

    fn push_items(&mut self, items: Vec<nyaaai_contract::shapes::step::SseItem>) {
        for item in items {
            let mut made = Vec::new();
            let result = self.decoder.push(&item, &mut made);
            self.events.extend(made);
            if let Err(error) = result {
                self.failed = Some(Fault::Provider(error));
                return;
            }
        }
    }

    fn finish(&mut self) {
        let mut made = Vec::new();
        let result = self.decoder.end(&mut made);
        self.events.extend(made);
        match result {
            Err(error) => self.failed = Some(Fault::Provider(error)),
            Ok(()) => self.ended = true,
        }
    }
}

enum StreamState<'a> {
    Fresh,
    Open(Box<Open<'a>>),
    Closed,
}

/// A stream's events, read one at a time.
pub struct EventStream<'a> {
    transport: &'a Transport,
    codec: &'a dyn Codec,
    request: &'a CanonicalRequest,
    cancel: CancelToken,
    state: StreamState<'a>,
}

impl EventStream<'_> {
    /// `None` ends the stream.
    pub async fn next(&mut self) -> Result<Option<StepEvent>, TransportError> {
        if matches!(self.state, StreamState::Fresh) {
            match self
                .transport
                .open(self.codec, self.request, &self.cancel)
                .await
            {
                Ok(open) => self.state = StreamState::Open(Box::new(open)),
                Err(error) => {
                    self.state = StreamState::Closed;
                    return Err(error);
                }
            }
        }
        loop {
            let StreamState::Open(open) = &mut self.state else {
                return Ok(None);
            };
            if let Some(event) = open.events.pop_front() {
                return Ok(Some(open.scrubbed(event)));
            }
            if let Some(fault) = open.failed.take() {
                if !self.cancel.is_cancelled() {
                    self.transport.notify(&self.request.model);
                }
                let error = self.transport.mapped(
                    fault,
                    &self.cancel,
                    Some(&open.cut),
                    open.key.as_deref(),
                );
                self.close();
                return Err(error);
            }
            if open.ended {
                self.close();
                self.transport.notify(&self.request.model);
                return Ok(None);
            }
            self.transport.advance(open).await;
        }
    }

    fn close(&mut self) {
        if let StreamState::Open(open) = std::mem::replace(&mut self.state, StreamState::Closed) {
            open.cut.clear();
            self.transport.log(&open.seen);
        }
    }
}

impl Drop for EventStream<'_> {
    fn drop(&mut self) {
        self.close();
    }
}

impl Transport {
    pub fn new(d: TransportDeps) -> Self {
        Self {
            idle_ms: d.idle_ms.unwrap_or(IDLE_MS),
            d,
            settled: Mutex::new(Vec::new()),
            listeners: AtomicU64::new(0),
        }
    }

    /// After each unaborted reply.
    pub fn on_settled(&self, listener: impl Fn(&str) + Send + Sync + 'static) -> ListenerId {
        let id = self.listeners.fetch_add(1, Ordering::Relaxed);
        self.held().push((id, Arc::new(listener)));
        ListenerId(id)
    }

    /// Whether the listener was held.
    pub fn off_settled(&self, id: ListenerId) -> bool {
        let mut held = self.held();
        let before = held.len();
        held.retain(|(each, _)| *each != id.0);
        held.len() < before
    }

    fn held(&self) -> std::sync::MutexGuard<'_, Vec<(u64, Listener)>> {
        self.settled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Nothing runs until the first `next`.
    pub fn stream<'a>(
        &'a self,
        codec: &'a dyn Codec,
        request: &'a CanonicalRequest,
        cancel: &CancelToken,
    ) -> EventStream<'a> {
        EventStream {
            transport: self,
            codec,
            request,
            cancel: cancel.clone(),
            state: StreamState::Fresh,
        }
    }

    pub async fn complete(
        &self,
        codec: &dyn Codec,
        request: &CanonicalRequest,
        cancel: &CancelToken,
    ) -> Result<StepResult, TransportError> {
        if cancel.is_cancelled() {
            return Err(TransportError::Cancelled);
        }
        let encoded = encoded_of(codec, request, Mode::Complete)?;
        let key = self.request_key().await?;
        let cut = Cut::new(cancel, Arc::clone(&self.d.scheduler), self.idle_ms);
        let mut seen = Seen::default();
        let run = async {
            let mut response = self
                .post(
                    codec,
                    &encoded,
                    key.as_deref(),
                    &cut.token,
                    "application/json",
                    &mut seen,
                )
                .await?;
            cut.arm();
            let text = read_text(&mut *response.chunks, None).await?;
            let Some(body) = parse_json(&text) else {
                return Err(Fault::Provider(failed(
                    FailureCategory::Stream,
                    "The reply was not JSON.",
                    FailureExtra::default(),
                )));
            };
            let decoded = codec.complete(&body)?;
            let serving = scrubbed_serving(&decoded.serving, key.as_deref());
            seen.generation.clone_from(&serving.generation_id);
            seen.host.clone_from(&serving.host);
            Ok::<_, Fault>(StepResult { serving, ..decoded })
        };
        let outcome = run.await;
        match outcome {
            Err(fault) => {
                if !cancel.is_cancelled() {
                    self.notify(&request.model);
                }
                let error = self.mapped(fault, cancel, Some(&cut), key.as_deref());
                cut.clear();
                self.log(&seen);
                Err(error)
            }
            Ok(result) => {
                cut.clear();
                self.log(&seen);
                self.notify(&request.model);
                Ok(result)
            }
        }
    }

    pub async fn get_json(
        &self,
        path: &str,
        cancel: &CancelToken,
        auth: &KeyAuth,
    ) -> Result<Value, TransportError> {
        self.json(Method::Get, path, None, cancel, auth).await
    }

    pub async fn post_json(
        &self,
        path: &str,
        body: Option<&Value>,
        cancel: &CancelToken,
        auth: &KeyAuth,
    ) -> Result<Value, TransportError> {
        self.json(Method::Post, path, body, cancel, auth).await
    }

    async fn json(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        cancel: &CancelToken,
        auth: &KeyAuth,
    ) -> Result<Value, TransportError> {
        if cancel.is_cancelled() {
            return Err(TransportError::Cancelled);
        }
        let key = self.key_for(auth).await?;
        let mut headers: HeaderList = vec![
            ("Accept".to_owned(), "application/json".to_owned()),
            ("User-Agent".to_owned(), self.d.user_agent.clone()),
        ];
        if let Some(key) = &key {
            headers.push(("Authorization".to_owned(), format!("Bearer {key}")));
        }
        if body.is_some() {
            headers.push(("Content-Type".to_owned(), "application/json".to_owned()));
        }
        let request = FetchRequest {
            method,
            url: format!("{}{path}", self.d.base_url),
            headers,
            body: body.map(stringify_json),
        };
        let run = async {
            let mut response = self.d.fetch.fetch(request, cancel.clone()).await?;
            if !ok(&response) {
                return Err(Fault::Provider(
                    self.failure(&*self.d.errors, response).await?,
                ));
            }
            let answer = parse_json(&read_text(&mut *response.chunks, None).await?);
            answer.ok_or_else(|| {
                Fault::Provider(failed(
                    FailureCategory::Network,
                    &format!("{path} did not answer JSON."),
                    FailureExtra::default(),
                ))
            })
        };
        run.await
            .map_err(|fault| self.mapped(fault, cancel, None, key.as_deref()))
    }

    fn notify(&self, model: &str) {
        let listeners: Vec<Listener> = self
            .held()
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in listeners {
            listener(model);
        }
    }

    async fn request_key(&self) -> Result<Option<String>, ProviderError> {
        self.key_for(&KeyAuth::Stored).await
    }

    async fn key_for(&self, auth: &KeyAuth) -> Result<Option<String>, ProviderError> {
        if *auth == KeyAuth::None || self.d.auth == Auth::None {
            return Ok(None);
        }
        let key = match auth {
            KeyAuth::Stored => self.d.key.get().await,
            KeyAuth::Key(key) => Some(key.clone()),
            KeyAuth::None => None,
        };
        let empty = key.as_deref().is_none_or(str::is_empty);
        if self.d.auth == Auth::Optional && *auth == KeyAuth::Stored && empty {
            return Ok(None);
        }
        self.checked(key).map(Some)
    }

    fn checked(&self, key: Option<String>) -> Result<String, ProviderError> {
        let Some(key) = key.filter(|key| !key.is_empty()) else {
            return Err(failed(
                FailureCategory::KeyMissing,
                &format!("No {} key is set.", self.d.label),
                FailureExtra::default(),
            ));
        };
        if !is_key_shape(&key) {
            return Err(ProviderError::new(Failure {
                category: FailureCategory::KeyRejected,
                status: nyaaai_contract::shapes::step::FailureStatus::Number(401.0),
                retry_after_ms: None,
                resets_at: None,
                message: format!("The {} key is malformed.", self.d.label),
                reasons: None,
                billed: None,
            }));
        }
        Ok(key)
    }

    async fn post(
        &self,
        codec: &dyn Codec,
        encoded: &Encoded,
        key: Option<&str>,
        cancel: &CancelToken,
        accept: &str,
        seen: &mut Seen,
    ) -> Result<FetchResponse, Fault> {
        let mut headers: HeaderList = encoded
            .headers
            .iter()
            .flat_map(|held| held.iter())
            .filter(|(name, _)| !OWNED_HEADERS.contains(&ascii_lower(name).as_str()))
            .map(|(name, value)| (name.to_owned(), value.clone()))
            .collect();
        headers.push(("Content-Type".to_owned(), "application/json".to_owned()));
        headers.push(("Accept".to_owned(), accept.to_owned()));
        if let Some(key) = key {
            headers.push(("Authorization".to_owned(), format!("Bearer {key}")));
        }
        headers.push(("User-Agent".to_owned(), self.d.user_agent.clone()));
        let request = FetchRequest {
            method: Method::Post,
            url: format!("{}{}", self.d.base_url, encoded.path),
            headers,
            body: Some(stringify_json(&encoded.body)),
        };
        let response = self.d.fetch.fetch(request, cancel.clone()).await?;
        seen.status = Some(response.status);
        if !ok(&response) {
            return Err(Fault::Provider(self.failure(codec, response).await?));
        }
        Ok(response)
    }

    async fn failure(
        &self,
        codec: &dyn Codec,
        mut response: FetchResponse,
    ) -> Result<ProviderError, Fault> {
        let text = read_text(&mut *response.chunks, Some(ERROR_BODY_BYTES)).await?;
        let body = parse_json(&text).unwrap_or(Value::Null);
        let decoded = codec.failure(i64::from(response.status), &body, &response.headers);
        let retry_after_ms = decoded.retry_after_ms.or_else(|| {
            parse_retry_after(
                header_of(&response.headers, "retry-after"),
                self.d.scheduler.now_ms(),
            )
        });
        Ok(ProviderError::new(Failure {
            retry_after_ms,
            ..decoded
        }))
    }

    fn log(&self, seen: &Seen) {
        let status = seen
            .status
            .map_or_else(|| "-".to_owned(), |status| status.to_string());
        (self.d.log)(&format!(
            "{} {} {} {}",
            self.d.label,
            status,
            seen.generation.as_deref().unwrap_or("-"),
            seen.host.as_deref().unwrap_or("-")
        ));
    }

    /// Opens a stream; a failure after the key is read is logged.
    async fn open<'a>(
        &self,
        codec: &'a dyn Codec,
        request: &CanonicalRequest,
        cancel: &CancelToken,
    ) -> Result<Open<'a>, TransportError> {
        if cancel.is_cancelled() {
            return Err(TransportError::Cancelled);
        }
        let encoded = encoded_of(codec, request, Mode::Stream)?;
        let key = self.request_key().await?;
        let cut = Cut::new(cancel, Arc::clone(&self.d.scheduler), self.idle_ms);
        let mut seen = Seen::default();
        let posted = self
            .post(
                codec,
                &encoded,
                key.as_deref(),
                &cut.token,
                "text/event-stream",
                &mut seen,
            )
            .await;
        match posted {
            Err(fault) => {
                if !cancel.is_cancelled() {
                    self.notify(&request.model);
                }
                let error = self.mapped(fault, cancel, Some(&cut), key.as_deref());
                cut.clear();
                self.log(&seen);
                Err(error)
            }
            Ok(response) => {
                cut.arm();
                Ok(Open {
                    key,
                    cut,
                    seen,
                    body: Some(response.chunks),
                    body_done: false,
                    sse: SseDecoder::new(),
                    decoder: codec.stream(),
                    events: VecDeque::new(),
                    failed: None,
                    ended: false,
                })
            }
        }
    }

    /// Reads on, re-arming the idle abort.
    async fn advance(&self, open: &mut Open<'_>) {
        if open.decoder.over() {
            open.body = None;
            open.finish();
            return;
        }
        if open.body_done {
            open.finish();
            return;
        }
        let Some(body) = open.body.as_mut() else {
            open.finish();
            return;
        };
        match body.next().await {
            Err(error) => open.failed = Some(Fault::Fetch(error)),
            Ok(Some(bytes)) => {
                open.cut.arm();
                let items = open.sse.push(&bytes);
                open.push_items(items);
            }
            Ok(None) => {
                open.cut.arm();
                let items = open.sse.end();
                open.push_items(items);
                open.body_done = true;
                open.body = None;
            }
        }
    }

    fn mapped(
        &self,
        fault: Fault,
        cancel: &CancelToken,
        cut: Option<&Cut>,
        key: Option<&str>,
    ) -> TransportError {
        if cancel.is_cancelled() {
            return TransportError::Cancelled;
        }
        match fault {
            Fault::Provider(error) => {
                TransportError::Provider(ProviderError::new(scrubbed_failure(*error.failure, key)))
            }
            Fault::Fetch(error) => {
                if cut.is_some_and(Cut::idled) {
                    return TransportError::Provider(failed(
                        FailureCategory::Idle,
                        &format!("{} sent nothing for too long.", self.d.label),
                        FailureExtra::default(),
                    ));
                }
                let message = match error {
                    FetchError::Failed(message) => message,
                    FetchError::Cancelled => "the exchange was cancelled".to_owned(),
                };
                TransportError::Provider(failed(
                    FailureCategory::Network,
                    &scrubbed(&message, key),
                    FailureExtra::default(),
                ))
            }
        }
    }
}

/// Printable ASCII, excluding spaces.
pub fn is_key_shape(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|char| ('\u{21}'..='\u{7e}').contains(&char))
}

/// Trimmed key, or `None`.
pub fn normalize_key(input: &str) -> Option<String> {
    let key = trim_text(input);
    is_key_shape(key).then(|| key.to_owned())
}

fn ok(response: &FetchResponse) -> bool {
    (200..=299).contains(&response.status)
}

/// Codec refusals stay local.
fn encoded_of(
    codec: &dyn Codec,
    request: &CanonicalRequest,
    mode: Mode,
) -> Result<Encoded, ProviderError> {
    codec.encode(request, mode).map_err(|message| {
        failed(
            FailureCategory::BadRequest,
            &message,
            FailureExtra::default(),
        )
    })
}

/// Decodes to the limit, then stops reading.
async fn read_text(chunks: &mut dyn Chunks, limit: Option<usize>) -> Result<String, FetchError> {
    let mut decoder = Utf8Decoder::new();
    let mut text = String::new();
    let mut size = 0usize;
    while let Some(chunk) = chunks.next().await? {
        let take = limit.map_or(chunk.len(), |limit| chunk.len().min(limit - size));
        text.push_str(&decoder.push(&chunk[..take]));
        size += chunk.len();
        if limit.is_some_and(|limit| size >= limit) {
            break;
        }
    }
    text.push_str(&decoder.end());
    Ok(text)
}

/// The base64 spelling, without padding.
fn base64_form(key: &str) -> String {
    base64_encode(key.as_bytes())
        .trim_end_matches('=')
        .to_owned()
}

/// Scrub URL/base64 echoes.
fn scrubbed(text: &str, key: Option<&str>) -> String {
    let Some(key) = key.filter(|key| !key.is_empty()) else {
        return text.to_owned();
    };
    [key.to_owned(), encode_path_segment(key), base64_form(key)]
        .iter()
        .fold(text.to_owned(), |each, form| {
            each.replace(form.as_str(), REDACTED)
        })
}

/// Sanitize response-derived names.
fn scrubbed_serving(serving: &Serving, key: Option<&str>) -> Serving {
    Serving {
        host: serving.host.as_deref().map(|host| scrubbed(host, key)),
        generation_id: serving.generation_id.as_deref().map(|id| scrubbed(id, key)),
    }
}

fn scrubbed_failure(failure: Failure, key: Option<&str>) -> Failure {
    Failure {
        message: scrubbed(&failure.message, key),
        reasons: failure
            .reasons
            .map(|reasons| reasons.iter().map(|reason| scrubbed(reason, key)).collect()),
        billed: failure.billed.map(|billed| {
            let serving = scrubbed_serving(
                &Serving {
                    host: billed.host,
                    generation_id: billed.generation_id,
                },
                key,
            );
            FailureBilled {
                usage: billed.usage,
                host: serving.host,
                generation_id: serving.generation_id,
            }
        }),
        ..failure
    }
}
