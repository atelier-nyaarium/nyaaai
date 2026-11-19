//! Server-sent event decoding.

use nyaaai_contract::shapes::step::{SseItem, SseItemComment, SseItemData, SseItemDone};
use nyaaai_contract::text::Utf8Decoder;

/// Decodes split UTF-8. A line ends at LF; a lone CR is data.
#[derive(Default)]
pub struct SseDecoder {
    decoder: Utf8Decoder,
    pending: String,
    /// Pending text before this offset holds no LF.
    scanned: usize,
    data: Vec<String>,
    event: Option<String>,
    data_bytes: usize,
    event_bytes: usize,
}

fn without_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// UTF-8 bytes held: the pending line, and the gathered data and event lines.
    pub fn held(&self) -> usize {
        self.pending.len() + self.data_bytes + self.event_bytes
    }

    pub fn push(&mut self, bytes: &[u8]) -> Vec<SseItem> {
        let text = self.decoder.push(bytes);
        self.pending.push_str(&text);
        let pending = std::mem::take(&mut self.pending);
        let mut items = Vec::new();
        let mut from = 0;
        let mut scan = self.scanned;
        while let Some(found) = pending[scan..].find('\n') {
            let at = scan + found;
            self.line(without_cr(&pending[from..at]), &mut items);
            from = at + 1;
            scan = from;
        }
        // The line left over stays where it is: a long line costs its size once.
        let mut pending = pending;
        pending.drain(..from);
        self.pending = pending;
        self.scanned = self.pending.len();
        items
    }

    /// Flushes unterminated final data.
    pub fn end(&mut self) -> Vec<SseItem> {
        let mut items = self.push(&[]);
        let tail = self.decoder.end();
        self.pending.push_str(&tail);
        let pending = std::mem::take(&mut self.pending);
        self.scanned = 0;
        if !pending.is_empty() {
            self.line(without_cr(&pending), &mut items);
        }
        self.dispatch(&mut items);
        items
    }

    fn line(&mut self, line: &str, items: &mut Vec<SseItem>) {
        if line.is_empty() {
            self.dispatch(items);
            return;
        }
        if let Some(text) = line.strip_prefix(':') {
            items.push(SseItem::Comment(SseItemComment {
                text: text.to_owned(),
            }));
            return;
        }
        let (field, raw) = line.split_once(':').unwrap_or((line, ""));
        let value = raw.strip_prefix(' ').unwrap_or(raw);
        match field {
            "event" => {
                self.event = Some(value.to_owned());
                self.event_bytes = line.len() + 1;
            }
            "data" => {
                self.data.push(value.to_owned());
                self.data_bytes += line.len() + 1;
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, items: &mut Vec<SseItem>) {
        let event = self.event.take();
        self.event_bytes = 0;
        if self.data.is_empty() {
            return;
        }
        let data = std::mem::take(&mut self.data).join("\n");
        self.data_bytes = 0;
        if data == "[DONE]" {
            items.push(SseItem::Done(SseItemDone {}));
        } else {
            items.push(SseItem::Data(SseItemData { data, event }));
        }
    }
}
