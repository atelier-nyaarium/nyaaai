//! The HTTP port.

use crate::cancel::CancelToken;
use crate::future::BoxFuture;
use crate::shapes::step::HeaderList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

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

/// The body, one chunk per read.
pub trait Chunks: Send {
    /// `None` ends the body.
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>>;
}

pub struct FetchResponse {
    pub status: u16,
    pub headers: HeaderList,
    pub chunks: Box<dyn Chunks>,
}

pub trait Fetch: Send + Sync {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>>;
}
