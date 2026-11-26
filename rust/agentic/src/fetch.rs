//! The `reqwest` adapter of the Fetch port.

use std::error::Error;
use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{
    Budgeted, Chunks, Fetch, FetchError, FetchRequest, FetchResponse, Method, RawFetch, budgeted,
};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::headers::response_headers;
use nyaaai_contract::time::Scheduler;
use reqwest::header::{HeaderName, HeaderValue};

/// No environment proxy, no redirects; bounded by each request's budget.
pub struct ReqwestFetch {
    inner: Budgeted<Raw>,
}

/// The client, unbudgeted.
struct Raw {
    client: reqwest::Client,
}

impl ReqwestFetch {
    pub fn new(scheduler: Arc<dyn Scheduler>) -> Result<Self, FetchError> {
        reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|client| Self::with_client(client, scheduler))
            .map_err(|error| FetchError::Failed(described(error)))
    }

    /// A host's own settings: its proxies and redirects are its own.
    pub fn with_client(client: reqwest::Client, scheduler: Arc<dyn Scheduler>) -> Self {
        Self {
            inner: budgeted(Raw { client }, scheduler),
        }
    }
}

impl Fetch for ReqwestFetch {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        self.inner.fetch(request, cancel)
    }
}

/// The error and its causes, with the URL cut.
fn described(error: reqwest::Error) -> String {
    let error = error.without_url();
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(held) = cause {
        text.push_str(": ");
        text.push_str(&held.to_string());
        cause = held.source();
    }
    text
}

/// The key is marked sensitive; a refused header keeps no text.
fn header_of(name: &str, value: &str) -> Result<(HeaderName, HeaderValue), FetchError> {
    let refused = || FetchError::Failed("a request header is not valid".to_owned());
    let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| refused())?;
    let mut value = HeaderValue::from_str(value).map_err(|_| refused())?;
    if name == reqwest::header::AUTHORIZATION {
        value.set_sensitive(true);
    }
    Ok((name, value))
}

struct Body {
    response: Option<reqwest::Response>,
    cancel: CancelToken,
    /// The body ended at the cancel.
    cut: bool,
}

impl Chunks for Body {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        Box::pin(async move {
            let Some(response) = self.response.as_mut() else {
                return if self.cut {
                    Err(FetchError::Cancelled)
                } else {
                    Ok(None)
                };
            };
            let read = self.cancel.run_until(response.chunk()).await;
            let outcome = match read {
                None => Err(FetchError::Cancelled),
                Some(Err(error)) => Err(FetchError::Failed(described(error))),
                Some(Ok(Some(bytes))) => return Ok(Some(bytes.to_vec())),
                Some(Ok(None)) => Ok(None),
            };
            // A finished or failed body releases the connection.
            self.cut = matches!(outcome, Err(FetchError::Cancelled));
            self.response = None;
            outcome
        })
    }
}

impl RawFetch for Raw {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        Box::pin(async move {
            if cancel.is_cancelled() {
                return Err(FetchError::Cancelled);
            }
            let method = match request.method {
                Method::Get => reqwest::Method::GET,
                Method::Post => reqwest::Method::POST,
            };
            let mut builder = self.client.request(method, &request.url);
            for (name, value) in &request.headers {
                let (name, value) = header_of(name, value)?;
                builder = builder.header(name, value);
            }
            if request.method == Method::Post
                && let Some(body) = request.body
            {
                builder = builder.body(body);
            }
            let sent = cancel.run_until(builder.send()).await;
            let response = match sent {
                None => return Err(FetchError::Cancelled),
                Some(Err(error)) => return Err(FetchError::Failed(described(error))),
                Some(Ok(response)) => response,
            };
            let raw: Vec<(String, String)> = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.as_str().to_owned(),
                        String::from_utf8_lossy(value.as_bytes()).into_owned(),
                    )
                })
                .collect();
            Ok(FetchResponse {
                status: response.status().as_u16(),
                headers: response_headers(&raw),
                chunks: Box::new(Body {
                    response: Some(response),
                    cancel,
                    cut: false,
                }),
            })
        })
    }
}
