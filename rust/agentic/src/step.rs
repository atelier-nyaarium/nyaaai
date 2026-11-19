//! Joining streamed step events.

use std::cmp::Ordering;
use std::collections::HashMap;

use nyaaai_contract::json::number_to_string;
use nyaaai_contract::shapes::step::{
    FailureBilled, FailureCategory, FinishReason, NormalizedUsage, ReplyPart, ReplyPartCall,
    Serving, StepEvent, StepEventCall, StepResult, TextPart, ThoughtPart,
};
use nyaaai_contract::text::trim_text;

use crate::failure::{FailureExtra, ProviderError, failed};

pub fn reply_text(result: &StepResult) -> String {
    result
        .parts
        .iter()
        .filter_map(|part| match part {
            ReplyPart::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

/// By index.
pub fn reply_calls(result: &StepResult) -> Vec<ReplyPartCall> {
    let mut calls: Vec<ReplyPartCall> = result
        .parts
        .iter()
        .filter_map(|part| match part {
            ReplyPart::Call(call) => Some(call.clone()),
            _ => None,
        })
        .collect();
    calls.sort_by(|left, right| {
        left.index
            .partial_cmp(&right.index)
            .unwrap_or(Ordering::Equal)
    });
    calls
}

pub fn reply_thoughts(result: &StepResult) -> Vec<ThoughtPart> {
    result
        .parts
        .iter()
        .filter_map(|part| match part {
            ReplyPart::Thought(thought) => Some(thought.clone()),
            _ => None,
        })
        .collect()
}

/// Adds the delta's segments to the thought, joining at the seam unless split; fields replace.
fn merge_thought(into: &mut ThoughtPart, part: &ThoughtPart, split: bool) {
    if let Some((first, rest)) = part
        .segments
        .as_deref()
        .and_then(|segments| segments.split_first())
    {
        let held = into.segments.get_or_insert_with(Vec::new);
        match held.last_mut() {
            Some(last) if !split && last.kind == first.kind => last.text.push_str(&first.text),
            _ => held.push(first.clone()),
        }
        held.extend_from_slice(rest);
    }
    if part.opaque.is_some() {
        into.opaque.clone_from(&part.opaque);
    }
    if part.id.is_some() {
        into.id.clone_from(&part.id);
    }
    if part.format.is_some() {
        into.format.clone_from(&part.format);
    }
    if part.index.is_some() {
        into.index = part.index;
    }
    if part.anchor.is_some() {
        into.anchor.clone_from(&part.anchor);
    }
}

/// Map key: zero has one spelling.
fn index_key(index: f64) -> u64 {
    (index + 0.0).to_bits()
}

/// Calls join by index; thoughts by block.
pub struct StepAssembler {
    parts: Vec<ReplyPart>,
    calls: HashMap<u64, usize>,
    blocks: HashMap<String, usize>,
    /// Last blockless position.
    loose: Option<usize>,
    serving: Serving,
    finish: Option<FinishReason>,
    usage: Option<NormalizedUsage>,
    seen: bool,
    started: bool,
}

impl Default for StepAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl StepAssembler {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            calls: HashMap::new(),
            blocks: HashMap::new(),
            loose: None,
            serving: Serving {
                host: None,
                generation_id: None,
            },
            finish: None,
            usage: None,
            seen: false,
            started: false,
        }
    }

    pub fn output_seen(&self) -> bool {
        self.seen
    }

    /// Failed-attempt billing, if reported.
    pub fn billed(&self) -> Option<FailureBilled> {
        self.started.then(|| FailureBilled {
            usage: self.usage.clone(),
            host: self.serving.host.clone(),
            generation_id: self.serving.generation_id.clone(),
        })
    }

    /// Current assembled parts.
    pub fn snapshot(&self) -> Vec<ReplyPart> {
        self.parts.clone()
    }

    /// Whether parts changed.
    pub fn add(&mut self, event: &StepEvent) -> Result<bool, ProviderError> {
        self.started = true;
        match event {
            StepEvent::Text(text) => {
                if text.text.is_empty() {
                    return Ok(false);
                }
                self.add_text(&text.text, text.phase.as_deref());
                Ok(true)
            }
            StepEvent::Thought(thought) => {
                self.add_thought(&thought.part, thought.block.as_deref(), thought.split);
                Ok(true)
            }
            StepEvent::Call(call) => {
                self.add_call(call)?;
                Ok(true)
            }
            StepEvent::Serving(held) => {
                if self.serving.host.is_none() && held.serving.host.is_some() {
                    self.serving.host.clone_from(&held.serving.host);
                }
                if self.serving.generation_id.is_none() && held.serving.generation_id.is_some() {
                    self.serving
                        .generation_id
                        .clone_from(&held.serving.generation_id);
                }
                Ok(false)
            }
            StepEvent::Usage(held) => {
                self.usage = Some(held.usage.clone());
                Ok(false)
            }
            StepEvent::Finish(held) => {
                // Usage repeats finish.
                self.finish.get_or_insert(held.reason);
                Ok(false)
            }
        }
    }

    /// Missing call ids and empty arguments are filled.
    pub fn result(&self) -> StepResult {
        let parts = self
            .parts
            .iter()
            .map(|part| match part {
                ReplyPart::Call(call) => {
                    let mut call = call.clone();
                    if call.wire_id.is_empty() {
                        call.wire_id = format!(
                            "{}_{}",
                            self.serving.generation_id.as_deref().unwrap_or("call"),
                            number_to_string(call.index)
                        );
                    }
                    if trim_text(&call.arguments_text).is_empty() {
                        call.arguments_text = "{}".to_owned();
                    }
                    ReplyPart::Call(call)
                }
                other => other.clone(),
            })
            .collect();
        StepResult {
            parts,
            finish: self.finish.unwrap_or(FinishReason::Empty),
            usage: self.usage.clone(),
            serving: self.serving.clone(),
        }
    }

    /// Joins the last text of the same phase.
    fn add_text(&mut self, text: &str, phase: Option<&str>) {
        self.seen = true;
        if let Some(ReplyPart::Text(last)) = self.parts.last_mut()
            && last.phase.as_deref() == phase
        {
            last.text.push_str(text);
            return;
        }
        self.parts.push(ReplyPart::Text(TextPart {
            text: text.to_owned(),
            phase: phase.map(str::to_owned),
        }));
    }

    /// Blocks join by ID; loose deltas merge.
    fn add_thought(&mut self, part: &ThoughtPart, block: Option<&str>, split: Option<bool>) {
        self.seen = true;
        let at = match block {
            None => self.continued(part),
            Some(block) => self.keyed(block, part),
        };
        if let Some(at) = at
            && let Some(ReplyPart::Thought(known)) = self.parts.get_mut(at)
        {
            merge_thought(known, part, split == Some(true));
            return;
        }
        match block {
            None => self.loose = Some(self.parts.len()),
            Some(block) => {
                self.blocks.insert(block.to_owned(), self.parts.len());
            }
        }
        self.parts.push(ReplyPart::Thought(part.clone()));
    }

    /// Unsealed loose tail of this type.
    fn loose_tail(&self, type_: &str) -> Option<usize> {
        let at = self.parts.len().checked_sub(1)?;
        match self.parts.get(at) {
            Some(ReplyPart::Thought(last))
                if self.loose == Some(at) && last.type_ == type_ && last.opaque.is_none() =>
            {
                Some(at)
            }
            _ => None,
        }
    }

    fn continued(&self, part: &ThoughtPart) -> Option<usize> {
        if part.opaque.is_none() {
            self.loose_tail(&part.type_)
        } else {
            None
        }
    }

    /// A block's first delta adopts the loose tail.
    fn keyed(&mut self, block: &str, part: &ThoughtPart) -> Option<usize> {
        if let Some(known) = self.blocks.get(block) {
            return Some(*known);
        }
        let at = self.loose_tail(&part.type_)?;
        self.blocks.insert(block.to_owned(), at);
        self.loose = None;
        Some(at)
    }

    fn add_call(&mut self, event: &StepEventCall) -> Result<(), ProviderError> {
        if !event.index.is_finite() || event.index.fract() != 0.0 || event.index < 0.0 {
            return Err(failed(
                FailureCategory::Stream,
                "A tool call had no valid index.",
                FailureExtra::default(),
            ));
        }
        self.seen = true;
        let key = index_key(event.index);
        let at = self.calls.get(&key).copied();
        if let Some(at) = at
            && let Some(ReplyPart::Call(known)) = self.parts.get_mut(at)
        {
            if !known.wire_id.is_empty()
                && !event.wire_id.is_empty()
                && known.wire_id != event.wire_id
            {
                return Err(failed(
                    FailureCategory::Stream,
                    "Two tool calls shared an index.",
                    FailureExtra::default(),
                ));
            }
            if known.wire_id.is_empty() {
                known.wire_id.clone_from(&event.wire_id);
            }
            if known.name.is_empty() {
                known.name.clone_from(&event.name);
            }
            known.arguments_text.push_str(&event.arguments_text);
            return Ok(());
        }
        self.calls.insert(key, self.parts.len());
        self.parts.push(ReplyPart::Call(ReplyPartCall {
            index: event.index,
            wire_id: event.wire_id.clone(),
            name: event.name.clone(),
            arguments_text: event.arguments_text.clone(),
        }));
        Ok(())
    }
}
