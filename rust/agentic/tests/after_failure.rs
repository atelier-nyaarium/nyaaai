//! A stream decoder that failed stays failed.

use nyaaai_agentic::codec::Codec;
use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::gemini::codec::gemini_codec;
use nyaaai_agentic::hosts::opencode::opencode_host;
use nyaaai_agentic::messages::codec::messages_codec;
use nyaaai_agentic::responses::codec::responses_codec;
use nyaaai_contract::shapes::step::{SseItem, SseItemComment, SseItemData, SseItemDone, StepEvent};
use nyaaai_contract::time::SteppedClock;

fn bad() -> SseItem {
    SseItem::Data(SseItemData {
        data: "{not json".to_owned(),
        event: None,
    })
}

fn good() -> SseItem {
    SseItem::Data(SseItemData {
        data: "{\"choices\":[{\"delta\":{\"content\":\"a\"}}]}".to_owned(),
        event: None,
    })
}

/// A failed decoder answers the same failure to every later call and appends nothing.
fn holds(codec: &dyn Codec) {
    let mut decoder = codec.stream();
    let mut events: Vec<StepEvent> = Vec::new();
    let first = decoder
        .push(&bad(), &mut events)
        .expect_err("a bad chunk fails");
    let events_after_failure = events.len();
    for item in [
        good(),
        SseItem::Comment(SseItemComment {
            text: "x".to_owned(),
        }),
        SseItem::Done(SseItemDone {}),
    ] {
        let again = decoder
            .push(&item, &mut events)
            .expect_err("a later push repeats the failure");
        assert_eq!(again, first);
    }
    assert_eq!(decoder.end(&mut events).expect_err("end repeats it"), first);
    assert_eq!(decoder.end(&mut events).expect_err("and again"), first);
    assert_eq!(events.len(), events_after_failure);
}

/// An early end is a failure that stays.
fn end_holds(codec: &dyn Codec) {
    let mut decoder = codec.stream();
    let mut events: Vec<StepEvent> = Vec::new();
    let first = decoder.end(&mut events).expect_err("an early end fails");
    assert_eq!(decoder.end(&mut events).expect_err("and again"), first);
    assert_eq!(
        decoder
            .push(&good(), &mut events)
            .expect_err("a later push repeats it"),
        first
    );
    assert!(events.is_empty());
}

fn host() -> nyaaai_agentic::hosts::opencode::OpenCodeHost<SteppedClock> {
    opencode_host("/v1", SteppedClock::new(0, 0))
}

#[test]
fn chat_stays_failed() {
    holds(&OPENROUTER_CHAT);
    end_holds(&OPENROUTER_CHAT);
}

#[test]
fn responses_stays_failed() {
    holds(&responses_codec(host(), None));
    end_holds(&responses_codec(host(), None));
}

#[test]
fn messages_stays_failed() {
    holds(&messages_codec(host(), None));
    end_holds(&messages_codec(host(), None));
}

#[test]
fn gemini_stays_failed() {
    holds(&gemini_codec(host(), None));
    end_holds(&gemini_codec(host(), None));
}

#[test]
fn a_clean_end_marker_still_ignores_later_items() {
    let mut decoder = OPENROUTER_CHAT.stream();
    let mut events: Vec<StepEvent> = Vec::new();
    let finish = SseItem::Data(SseItemData {
        data: "{\"choices\":[{\"finish_reason\":\"stop\"}]}".to_owned(),
        event: None,
    });
    decoder.push(&finish, &mut events).expect("a finish");
    decoder
        .push(&SseItem::Done(SseItemDone {}), &mut events)
        .expect("the end marker");
    let held = events.len();
    decoder
        .push(&bad(), &mut events)
        .expect("later items are ignored");
    decoder.end(&mut events).expect("a clean end");
    assert_eq!(events.len(), held);
}
