//! Kind cases.

use nyaaai_agentic::kinds::openrouter::key::{KeyCheck, KeyFailure, check_key};
use nyaaai_agentic::transport::{KeyAuth, TransportError};
use nyaaai_testkit::cases::{Outcome, block_on};

use super::catalog::{BASE_URL, CALLER_REASON, caller_token, network_of};
use super::shapes::*;

pub fn openrouter_key(input: KindOpenrouterKeyInput) -> Outcome<KindOpenrouterKeyResult> {
    let net = network_of(input.start_ms, &input.exchanges, input.stored.as_deref())?;
    let mut checks = Vec::new();
    let mut tags = 0;
    for held in &input.checks {
        tags += 1;
        let (auth, after, before) = match held {
            KindOpenrouterKeyInputChecksItem::Stored(held) => {
                (KeyAuth::Stored, held.abort_after_ms, held.aborted_before)
            }
            KindOpenrouterKeyInputChecksItem::Key(held) => (
                KeyAuth::Key(held.key.clone()),
                held.abort_after_ms,
                held.aborted_before,
            ),
        };
        let token = caller_token(&net.wheel, &format!("caller-{tags}"), after, before);
        let result = block_on(check_key(&net.transport, &auth, &token))?;
        checks.push(match result {
            Ok(KeyCheck::Ok {
                usage,
                limit,
                reset,
            }) => KindOpenrouterKeyResultChecksItem {
                ok: Some(true),
                reason: None,
                usage,
                limit,
                reset,
                thrown: None,
            },
            Ok(KeyCheck::Failed(reason)) => KindOpenrouterKeyResultChecksItem {
                ok: Some(false),
                reason: Some(match reason {
                    KeyFailure::Rejected => KindOpenrouterKeyResultChecksItemReason::Rejected,
                    KeyFailure::Unreachable => KindOpenrouterKeyResultChecksItemReason::Unreachable,
                }),
                usage: None,
                limit: None,
                reset: None,
                thrown: None,
            },
            Err(error) => KindOpenrouterKeyResultChecksItem {
                ok: None,
                reason: None,
                usage: None,
                limit: None,
                reset: None,
                thrown: Some(match error {
                    TransportError::Cancelled => CALLER_REASON.to_owned(),
                    TransportError::Provider(held) => held.failure.message.clone(),
                }),
            },
        });
        net.wheel.cancel(&format!("caller-{tags}"));
    }
    Ok(KindOpenrouterKeyResult {
        checks,
        requests: net
            .played
            .requests()
            .iter()
            .map(|sent| KindOpenrouterKeyResultRequestsItem {
                path: sent.url[BASE_URL.len()..].to_owned(),
                authorization: sent
                    .headers
                    .iter()
                    .find(|(name, _)| name == "Authorization")
                    .map(|(_, value)| value.clone()),
            })
            .collect(),
        waiting: net.wheel.waiting(),
    })
}
