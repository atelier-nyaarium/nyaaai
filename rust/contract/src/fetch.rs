//! The HTTP port.

use crate::cancel::CancelToken;
use crate::future::BoxFuture;
use crate::shapes::step::HeaderList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// A `Get` carries no body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderList,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    Cancelled,
    Failed(String),
}

/// A pending or later read ends `Cancelled` at the cancel given to `fetch`.
/// The connection is released at the end, on drop, or at the cancel.
pub trait Chunks: Send {
    /// `None` ends the body.
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>>;
}

pub struct FetchResponse {
    pub status: u16,
    /// In the `response_headers` form.
    pub headers: HeaderList,
    pub chunks: Box<dyn Chunks>,
}

/// The cancel covers the whole exchange, body reads included.
pub trait Fetch: Send + Sync {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>>;
}
