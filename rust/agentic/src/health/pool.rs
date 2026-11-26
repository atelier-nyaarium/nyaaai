//! One entry's pool cooldown: the transition and the skip it asks for.

use nyaaai_contract::shapes::health::{
    Blocked, BlockedSpendLimit, PoolEvent, PoolEventRefusedReason, PoolState, PoolStateBalance,
    PoolStateBalanceProbe, PoolStateClear, PoolStatePaused, PoolStatePausedReason, PoolStateReset,
    PoolVerdict, Ticket,
};
use nyaaai_contract::shapes::provider::SkipReason;
use nyaaai_contract::time::MAX_SERVER_WAIT_MS;

pub fn clear() -> PoolState {
    PoolState::Clear(PoolStateClear {})
}

/// Go balance reset estimate.
pub const BALANCE_COOL_MS: f64 = 5.0 * 60.0 * 60_000.0;

/// ETA floor before halving.
pub const PROBE_FLOOR_MS: f64 = 10.0 * 60_000.0;

/// Pool billing sources.
pub const ALLOWANCE_SOURCES: [&str; 2] = ["go", "go-plus"];

/// Two tickets are the same when every field is equal.
fn same_ticket(left: &Ticket, right: &Ticket) -> bool {
    left == right
}

/// Halve remaining ETA.
pub fn probe_at(now_ms: f64, eta: f64) -> f64 {
    now_ms + (eta - now_ms).max(PROBE_FLOOR_MS) / 2.0
}

/// A known reset wins.
fn billed(state: &PoolState, source: &str, extra_usage: bool, now_ms: f64) -> PoolState {
    if source != "balance" {
        return if ALLOWANCE_SOURCES.contains(&source) && matches!(state, PoolState::Balance(_)) {
            clear()
        } else {
            state.clone()
        };
    }
    if extra_usage
        || matches!(state, PoolState::Reset(held) if now_ms < held.at)
        || matches!(state, PoolState::Paused(held) if now_ms < held.until)
    {
        return state.clone();
    }
    // A renewed window starts without a probe.
    let open = match state {
        PoolState::Balance(held) if now_ms < held.eta => Some(held),
        _ => None,
    };
    let eta = open.map_or(now_ms + BALANCE_COOL_MS, |held| held.eta);
    PoolState::Balance(PoolStateBalance {
        eta,
        next: probe_at(now_ms, eta),
        probe: open.and_then(|held| held.probe.clone()),
    })
}

fn paused_reason(reason: PoolEventRefusedReason) -> PoolStatePausedReason {
    match reason {
        PoolEventRefusedReason::Allowance => PoolStatePausedReason::Allowance,
        PoolEventRefusedReason::Credit => PoolStatePausedReason::Credit,
    }
}

/// The only transition.
pub fn step(state: &PoolState, event: &PoolEvent, now_ms: f64, pause_ms: f64) -> PoolState {
    match event {
        PoolEvent::Billed(held) => {
            if held.plan.as_deref() == Some("go") {
                billed(state, &held.source, held.extra_usage, now_ms)
            } else {
                state.clone()
            }
        }
        PoolEvent::Refused(held) => {
            let farthest = now_ms + MAX_SERVER_WAIT_MS as f64;
            let waited = held
                .retry_after_ms
                .map(|after| (now_ms + after).min(farthest));
            let known = held.resets_at.map(|at| at.min(farthest));
            let reset = if held.reason == PoolEventRefusedReason::Allowance {
                known.or(waited)
            } else {
                None
            };
            let until = reset.or(waited).unwrap_or(now_ms + pause_ms);
            // Pauses never shorten windows.
            if reset.is_none()
                && let PoolState::Balance(balance) = state
                && now_ms < balance.eta.max(balance.next)
            {
                return PoolState::Balance(PoolStateBalance {
                    next: balance.next.max(until),
                    ..balance.clone()
                });
            }
            let refused = match reset {
                None => PoolState::Paused(PoolStatePaused {
                    reason: paused_reason(held.reason),
                    until,
                }),
                Some(at) => PoolState::Reset(PoolStateReset { at }),
            };
            // A refusal never ends a running pause or reset earlier.
            let ends = match state {
                PoolState::Paused(running) => Some(running.until),
                PoolState::Reset(running) => Some(running.at),
                _ => None,
            };
            match ends {
                Some(ends) if now_ms < ends && until < ends => state.clone(),
                _ => refused,
            }
        }
        PoolEvent::ExtraUsage(held) => {
            if held.on {
                return if matches!(state, PoolState::Balance(_)) {
                    clear()
                } else {
                    state.clone()
                };
            }
            if held.last_bill.as_deref() == Some("balance") {
                billed(state, "balance", false, now_ms)
            } else {
                state.clone()
            }
        }
        // The first ticket admitted in the window is its probe, whatever its `probe` mark says.
        PoolEvent::Admitted(held) => match state {
            PoolState::Balance(balance)
                if balance.probe.is_none() && now_ms >= balance.next && now_ms < balance.eta =>
            {
                PoolState::Balance(PoolStateBalance {
                    probe: Some(PoolStateBalanceProbe {
                        ticket: held.ticket.clone(),
                        at: now_ms,
                    }),
                    ..balance.clone()
                })
            }
            _ => state.clone(),
        },
        PoolEvent::Settled(held) => match state {
            // Settlement preserves `next`.
            PoolState::Balance(balance)
                if balance
                    .probe
                    .as_ref()
                    .is_some_and(|probe| same_ticket(&probe.ticket, &held.ticket)) =>
            {
                PoolState::Balance(PoolStateBalance {
                    probe: None,
                    ..balance.clone()
                })
            }
            _ => state.clone(),
        },
    }
}

fn skip_of(reason: PoolStatePausedReason) -> SkipReason {
    match reason {
        PoolStatePausedReason::Allowance => SkipReason::Allowance,
        PoolStatePausedReason::Credit => SkipReason::Credit,
    }
}

/// Skip, and the last route's refusal.
pub fn verdict(state: &PoolState, exempt: bool, now_ms: f64) -> Option<PoolVerdict> {
    match state {
        PoolState::Clear(_) => None,
        PoolState::Paused(held) => (now_ms < held.until).then(|| PoolVerdict {
            skip: skip_of(held.reason),
            blocked: None,
        }),
        PoolState::Reset(held) => (now_ms < held.at).then_some(PoolVerdict {
            skip: SkipReason::Allowance,
            blocked: Some(Blocked::SpendLimit(BlockedSpendLimit {
                resets_at: Some(held.at),
                next_try_at: None,
            })),
        }),
        PoolState::Balance(held) => {
            if exempt {
                return None;
            }
            let next_try_at = match &held.probe {
                Some(probe) if now_ms < held.eta => {
                    Some(probe_at(probe.at, held.eta).max(held.next))
                }
                _ if now_ms < held.next => Some(held.next),
                _ => None,
            }?;
            Some(PoolVerdict {
                skip: SkipReason::Allowance,
                blocked: Some(Blocked::SpendLimit(BlockedSpendLimit {
                    resets_at: None,
                    next_try_at: Some(next_try_at),
                })),
            })
        }
    }
}
