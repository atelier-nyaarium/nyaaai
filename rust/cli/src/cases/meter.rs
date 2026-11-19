//! Meter cases.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_agentic::health::health::Billing;
use nyaaai_agentic::usage::meter::{Bills, MeterDeps, provider_meter};
use nyaaai_contract::shapes::usage::MeteredCall;
use nyaaai_testkit::cases::Outcome;
use nyaaai_testkit::scripted_ports::ScriptedUsageSink;

use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A health that notes what it was told.
struct Noting(Arc<Mutex<Option<MeterBilled>>>);

impl Bills for Noting {
    fn billed(&self, id: &str, reply: &Billing) {
        *locked(&self.0) = Some(MeterBilled {
            entry: id.to_owned(),
            source: reply.source.clone(),
            allowance: reply.allowance,
            cost: reply.cost,
        });
    }
}

type Stats = (String, String, MeteredCall);

pub fn called(input: MeterCalledInput) -> Outcome<MeterCalledResult> {
    let sink = Arc::new(ScriptedUsageSink::new());
    let billed: Arc<Mutex<Option<MeterBilled>>> = Arc::default();
    let stats: Arc<Mutex<Option<Stats>>> = Arc::default();
    let meter = provider_meter(MeterDeps {
        usage: sink.clone(),
        health: Arc::new(Noting(Arc::clone(&billed))),
        stats: {
            let stats = Arc::clone(&stats);
            Arc::new(move |id, generation_id, call| {
                *locked(&stats) = Some((id.to_owned(), generation_id.to_owned(), call.clone()));
            })
        },
    });
    let mut calls = Vec::new();
    for each in &input.calls {
        sink.set_ids(each.id.iter().cloned().collect());
        *locked(&billed) = None;
        *locked(&stats) = None;
        meter.called(&each.call);
        let recorded = sink
            .recorded()
            .last()
            .cloned()
            .ok_or_else(|| "The sink was given nothing.".to_owned())?;
        calls.push(MeterCalledResultCallsItem {
            recorded,
            billed: locked(&billed).take(),
            stats: locked(&stats).take().map(|(id, generation_id, call)| {
                MeterCalledResultCallsItemStats {
                    id,
                    generation_id,
                    call,
                }
            }),
        });
    }
    Ok(MeterCalledResult { calls })
}
