//! The codec interface.

use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Encoded, Failure, HeaderList, SseItem, StepEvent, StepResult, TurnStep,
};

use crate::failure::ProviderError;

pub type StepTurn = TurnStep;

/// Whether the reply streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Stream,
    Complete,
}

/// Pushed one SSE item at a time. Events an item completes are appended
/// to `events`, kept when the item then fails.
pub trait StreamDecoder: Send {
    fn push(&mut self, item: &SseItem, events: &mut Vec<StepEvent>) -> Result<(), ProviderError>;

    /// Whether the decoder reads no further item: after an end marker or a failure.
    fn over(&self) -> bool;

    /// The input ended: an early end throws `ProviderError`.
    fn end(&mut self, events: &mut Vec<StepEvent>) -> Result<(), ProviderError>;
}

pub trait Codec: Send + Sync {
    /// `Err`: the request was refused before sending.
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String>;

    /// Ends at the API's end marker; in-stream errors and early ends fail.
    fn stream(&self) -> Box<dyn StreamDecoder + '_>;

    /// A non-streaming reply, as the stream would have assembled it.
    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError>;

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure;
}
