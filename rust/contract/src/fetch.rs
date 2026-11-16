//! The HTTP port.

use std::fmt;

use crate::cancel::CancelToken;
use crate::future::BoxFuture;
use crate::shapes::step::HeaderList;
use crate::text::ascii_lower;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// A `Get` carries no body.
#[derive(Clone, PartialEq, Eq)]
pub struct FetchRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderList,
    pub body: Option<String>,
}

impl fmt::Debug for FetchRequest {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The key never prints.
        let headers: Vec<(&str, &str)> = self
            .headers
            .iter()
            .map(|(name, value)| {
                let secret = ascii_lower(name) == "authorization";
                (name.as_str(), if secret { "[key]" } else { value.as_str() })
            })
            .collect();
        out.debug_struct("FetchRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &headers)
            .field("body", &self.body)
            .finish()
    }
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

/// No redirect is followed: a 3xx is answered as the status it is.
/// The cancel covers the whole exchange, body reads included.
pub trait Fetch: Send + Sync {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_debug_print_hides_the_key() {
        let request = FetchRequest {
            method: Method::Post,
            url: "http://host/".to_owned(),
            headers: vec![
                ("Accept".to_owned(), "x".to_owned()),
                ("authorization".to_owned(), "Bearer secret-value".to_owned()),
            ],
            body: None,
        };
        let shown = format!("{request:?}");
        assert!(!shown.contains("secret-value"));
        assert!(shown.contains("[key]"));
        assert!(shown.contains("Accept"));
    }
}
