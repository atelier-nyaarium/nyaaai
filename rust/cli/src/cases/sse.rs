//! Server-sent event cases.

use nyaaai_agentic::sse::SseDecoder;
use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::json::{Value, stringify_json};
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::step::SseItem;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

fn bytes_of(chunk: &str) -> Outcome<Vec<u8>> {
    base64_decode(chunk).ok_or_else(|| "a chunk is not base64".to_owned())
}

/// Items of each push, then of `end`.
fn decode_chunks(chunks: &[&[u8]]) -> (Vec<Vec<SseItem>>, Vec<SseItem>) {
    let mut decoder = SseDecoder::new();
    let pushed = chunks.iter().map(|chunk| decoder.push(chunk)).collect();
    (pushed, decoder.end())
}

fn flat_items(chunks: &[&[u8]]) -> Vec<SseItem> {
    let (pushed, ended) = decode_chunks(chunks);
    pushed.into_iter().flatten().chain(ended).collect()
}

fn text_of(items: &[SseItem]) -> String {
    stringify_json(&Value::Array(items.iter().map(Shape::encode).collect()))
}

pub fn items(input: SseItemsInput) -> Outcome<SseItemsResult> {
    let chunks = input
        .chunks
        .iter()
        .map(|chunk| bytes_of(chunk))
        .collect::<Outcome<Vec<_>>>()?;
    let slices: Vec<&[u8]> = chunks.iter().map(Vec::as_slice).collect();
    let (pushed, ended) = decode_chunks(&slices);
    Ok(SseItemsResult { pushed, ended })
}

pub fn splits(input: SseSplitsInput) -> Outcome<SseSplitsResult> {
    let bytes = bytes_of(&input.bytes)?;
    let items = flat_items(&[&bytes]);
    let whole = text_of(&items);
    let different = (1..bytes.len())
        .find(|cut| text_of(&flat_items(&[&bytes[..*cut], &bytes[*cut..]])) != whole);
    let singles: Vec<&[u8]> = bytes.chunks(1).collect();
    Ok(SseSplitsResult {
        items,
        agree: different.is_none(),
        first_different: different.map(|cut| cut as i64),
        bytewise: text_of(&flat_items(&singles)) == whole,
    })
}
