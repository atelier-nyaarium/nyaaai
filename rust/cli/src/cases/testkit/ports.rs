//! Cases for the scripted ports and the deadline.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::json::Record;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::model::{
    Binding, ModelRow, ModelRowCache, ModelRowVerifiedPrice, RowIdentity, RowIdentityCurated,
    TestRecord, TestRecordCache, TestRecordChecks, TestRecordChecksReasoning,
    TestRecordChecksRecovery, TestRecordChecksSchema, TestRecordChecksToolPlus,
    TestRecordVerifiedPrice,
};
use nyaaai_contract::shapes::provider::{ProviderEntry, ProviderKindName};
use nyaaai_contract::shapes::usage::{CallTag, Cost, GenerationStats, MeteredCall};
use nyaaai_contract::time::{
    Clock, Deadline, Ended, OffsetFrom, Scheduler, SteppedClock, day_key, deadline as deadline_of,
};
use nyaaai_contract::usage_sink::UsageSink;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_ports::{
    ScriptedEntryKeys, ScriptedModelRegistry, ScriptedProviderList, ScriptedUsageSink,
};

use super::super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

type Stops = Arc<Mutex<HashMap<String, Unsubscribe>>>;

/// Stops the named listener, when there is one.
fn stop_named(stops: &Stops, name: &Option<String>) {
    let Some(name) = name else { return };
    let taken = locked(stops).remove(name);
    if let Some(stop) = taken {
        stop();
    }
}

fn drained<T>(held: &Mutex<Vec<T>>) -> Vec<T> {
    std::mem::take(&mut *locked(held))
}

/// A row of fixed values.
fn row_of(key: &str) -> ModelRow {
    let mut bindings = Record::new();
    bindings.insert(
        "openrouter",
        Binding {
            id: format!("vendor/{key}"),
        },
    );
    ModelRow {
        key: key.to_owned(),
        bindings,
        family: key.to_owned(),
        label: key.to_owned(),
        reasoning: false,
        schema: false,
        cache: ModelRowCache::Implicit,
        verified: "2026-01-01".to_owned(),
        verified_price: ModelRowVerifiedPrice {
            prompt: 1.0,
            completion: 2.0,
        },
        identity: RowIdentity::Curated(RowIdentityCurated {
            key: key.to_owned(),
        }),
    }
}

/// A record of fixed values.
fn record_of(label: &str) -> TestRecord {
    TestRecord {
        label: label.to_owned(),
        date: "2026-01-01".to_owned(),
        cost: 0.01,
        estimated: None,
        checks: TestRecordChecks {
            tool_plus: TestRecordChecksToolPlus::Pass,
            recovery: TestRecordChecksRecovery::Pass,
            reasoning: TestRecordChecksReasoning::NA,
            schema: TestRecordChecksSchema::NA,
        },
        reasoning: false,
        cache: TestRecordCache::Implicit,
        verified_price: TestRecordVerifiedPrice {
            prompt: 1.0,
            completion: 2.0,
        },
    }
}

fn entry_of(id: &str) -> ProviderEntry {
    ProviderEntry {
        id: id.to_owned(),
        kind: ProviderKindName::Openrouter,
        label: id.to_owned(),
        plan: None,
        url: None,
        extra_usage: None,
    }
}

pub fn entry_keys(input: TestkitEntryKeysInput) -> Outcome<TestkitEntryKeysResult> {
    block_on(async move {
        let keys = ScriptedEntryKeys::new();
        let stops: Stops = Arc::new(Mutex::new(HashMap::new()));
        let heard = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let mut steps = Vec::new();
        for step in &input.steps {
            let name = step.name.clone().unwrap_or_default();
            let id = step.id.clone().unwrap_or_default();
            let mut got = None;
            match step.op {
                TestkitEntryKeysInputStepsItemOp::Listen => {
                    let heard = Arc::clone(&heard);
                    let held = name.clone();
                    let (others, unlistens) = (Arc::clone(&stops), step.unlistens.clone());
                    let stop = keys.on_change(Box::new(move |changed| {
                        locked(&heard).push((held.clone(), changed.to_owned()));
                        stop_named(&others, &unlistens);
                    }));
                    locked(&stops).insert(name, stop);
                }
                TestkitEntryKeysInputStepsItemOp::Unlisten => {
                    stop_named(&stops, &Some(name.clone()));
                }
                TestkitEntryKeysInputStepsItemOp::Set => {
                    let answer = match step.answer {
                        Some(TestkitEntryKeysInputStepsItemAnswer::Key) => {
                            KeyAnswer::Key(step.key.clone().unwrap_or_default())
                        }
                        Some(TestkitEntryKeysInputStepsItemAnswer::Refused) => KeyAnswer::Refused,
                        Some(TestkitEntryKeysInputStepsItemAnswer::Unready) => KeyAnswer::Unready,
                        Some(TestkitEntryKeysInputStepsItemAnswer::None) | None => KeyAnswer::None,
                    };
                    keys.set(&id, answer);
                }
                TestkitEntryKeysInputStepsItemOp::Get => {
                    let (kind, key) = match keys.get(&id).await {
                        KeyAnswer::Key(key) => ("key", Some(key)),
                        KeyAnswer::None => ("none", None),
                        KeyAnswer::Refused => ("refused", None),
                        KeyAnswer::Unready => ("unready", None),
                    };
                    got = Some(TestkitEntryKeysResultStepsItemGot {
                        kind: kind.to_owned(),
                        key,
                    });
                }
            }
            steps.push(TestkitEntryKeysResultStepsItem {
                heard: drained(&heard)
                    .into_iter()
                    .map(|(name, id)| TestkitEntryKeysResultStepsItemHeardItem { name, id })
                    .collect(),
                got,
                reads: keys.reads() as i64,
            });
        }
        TestkitEntryKeysResult { steps }
    })
}

pub fn provider_list(input: TestkitProviderListInput) -> Outcome<TestkitProviderListResult> {
    let list = ScriptedProviderList::new(Vec::new());
    let stops: Stops = Arc::new(Mutex::new(HashMap::new()));
    let heard = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut steps = Vec::new();
    for step in &input.steps {
        let name = step.name.clone().unwrap_or_default();
        let mut ids = None;
        match step.op {
            TestkitProviderListInputStepsItemOp::Listen => {
                let heard = Arc::clone(&heard);
                let held = name.clone();
                let (others, unlistens) = (Arc::clone(&stops), step.unlistens.clone());
                let stop = list.on_change(Box::new(move || {
                    locked(&heard).push(held.clone());
                    stop_named(&others, &unlistens);
                }));
                locked(&stops).insert(name, stop);
            }
            TestkitProviderListInputStepsItemOp::Unlisten => {
                stop_named(&stops, &Some(name.clone()));
            }
            TestkitProviderListInputStepsItemOp::Set => {
                let held = step.ids.clone().unwrap_or_default();
                list.set(held.iter().map(|id| entry_of(id)).collect());
            }
            TestkitProviderListInputStepsItemOp::Read => {
                ids = Some(list.entries().into_iter().map(|entry| entry.id).collect());
            }
        }
        steps.push(TestkitProviderListResultStepsItem {
            heard: drained(&heard),
            ids,
            reads: list.reads() as i64,
        });
    }
    Ok(TestkitProviderListResult { steps })
}

pub fn model_registry(input: TestkitModelRegistryInput) -> Outcome<TestkitModelRegistryResult> {
    block_on(async move {
        let registry = ScriptedModelRegistry::empty();
        let stops: Stops = Arc::new(Mutex::new(HashMap::new()));
        let heard = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut steps = Vec::new();
        for step in &input.steps {
            let name = step.name.clone().unwrap_or_default();
            let keys = step.keys.clone().unwrap_or_default();
            let mut seen = None;
            let mut outcome = None;
            match step.op {
                TestkitModelRegistryInputStepsItemOp::Listen => {
                    let heard = Arc::clone(&heard);
                    let held = name.clone();
                    let (others, unlistens) = (Arc::clone(&stops), step.unlistens.clone());
                    let stop = registry.on_change(Box::new(move || {
                        locked(&heard).push(held.clone());
                        stop_named(&others, &unlistens);
                    }));
                    locked(&stops).insert(name, stop);
                }
                TestkitModelRegistryInputStepsItemOp::Unlisten => {
                    stop_named(&stops, &Some(name.clone()));
                }
                TestkitModelRegistryInputStepsItemOp::SetRows => {
                    registry.set_rows(keys.iter().map(|key| row_of(key)).collect());
                }
                TestkitModelRegistryInputStepsItemOp::SetHidden => {
                    let mut stored = registry.stored();
                    stored.hidden = keys;
                    registry.set_stored(stored);
                }
                TestkitModelRegistryInputStepsItemOp::Refuse => {
                    registry.refuse(Some(step.why.clone().unwrap_or_default()));
                }
                TestkitModelRegistryInputStepsItemOp::Allow => registry.refuse(None),
                TestkitModelRegistryInputStepsItemOp::Record => {
                    let why = registry
                        .record(
                            step.entry.as_deref().unwrap_or(""),
                            step.id.as_deref().unwrap_or(""),
                            record_of(step.label.as_deref().unwrap_or("")),
                        )
                        .await;
                    outcome = Some(why);
                }
                TestkitModelRegistryInputStepsItemOp::Read => {
                    let stored = registry.stored();
                    let mut tested = Vec::new();
                    for (entry, records) in stored.tested.iter() {
                        for (id, record) in records.iter() {
                            tested.push(format!("{entry}/{id}={}", record.label));
                        }
                    }
                    seen = Some((
                        registry.rows().into_iter().map(|row| row.key).collect(),
                        stored.hidden.clone(),
                        tested,
                    ));
                }
            }
            let (rows, hidden, tested) = match seen {
                Some((rows, hidden, tested)) => (Some(rows), Some(hidden), Some(tested)),
                None => (None, None, None),
            };
            steps.push(TestkitModelRegistryResultStepsItem {
                heard: drained(&heard),
                rows,
                hidden,
                tested,
                outcome,
                reads: registry.reads() as i64,
                written: registry.written().len() as i64,
            });
        }
        TestkitModelRegistryResult { steps }
    })
}

pub fn deadline(input: TestkitDeadlineInput) -> Outcome<TestkitDeadlineResult> {
    let scheduler = FakeScheduler::new(Arc::new(TimerWheel::new(input.start_ms)));
    let mut callers: HashMap<String, CancelToken> = HashMap::new();
    let mut held: Vec<(String, Deadline)> = Vec::new();
    let mut steps = Vec::new();
    for step in &input.steps {
        let id = step.id.clone().unwrap_or_default();
        let ms = step.ms.unwrap_or(0);
        match step.op {
            TestkitDeadlineInputStepsItemOp::Caller => {
                callers.insert(id, CancelToken::new());
            }
            TestkitDeadlineInputStepsItemOp::AbortCaller => {
                if let Some(caller) = callers.get(&id) {
                    caller.cancel();
                }
            }
            TestkitDeadlineInputStepsItemOp::AbortAt => {
                if let Some(caller) = callers.get(&id) {
                    let caller = caller.clone();
                    scheduler.schedule(ms, Box::new(move || caller.cancel()));
                } else {
                    scheduler.schedule(ms, Box::new(|| {}));
                }
            }
            TestkitDeadlineInputStepsItemOp::Deadline => {
                let caller = step
                    .caller
                    .as_ref()
                    .and_then(|name| callers.get(name))
                    .cloned()
                    .unwrap_or_default();
                held.push((id, deadline_of(&scheduler, ms, &caller)));
            }
            TestkitDeadlineInputStepsItemOp::Clear => {
                for (each, made) in &held {
                    if *each == id {
                        made.clear();
                    }
                }
            }
            TestkitDeadlineInputStepsItemOp::Advance => {
                scheduler.wheel().advance(ms);
            }
        }
        steps.push(TestkitDeadlineResultStepsItem {
            now_ms: scheduler.now_ms(),
            waiting: scheduler.wheel().waiting(),
            deadlines: held
                .iter()
                .map(|(id, made)| TestkitDeadlineResultStepsItemDeadlinesItem {
                    id: id.clone(),
                    aborted: made.token().is_cancelled(),
                    ended: made.ended().map(|which| {
                        match which {
                            Ended::Deadline => "deadline",
                            Ended::Caller => "caller",
                        }
                        .to_owned()
                    }),
                })
                .collect(),
        });
    }
    Ok(TestkitDeadlineResult { steps })
}

/// A call of fixed values.
fn call_of(model: &str, at: i64, generation_id: Option<&String>) -> MeteredCall {
    MeteredCall {
        tag: CallTag {
            feature: "chat".to_owned(),
            lane: None,
            folder: None,
        },
        model: model.to_owned(),
        entry: None,
        label: None,
        at: at as f64,
        ms: 1.0,
        first_token_ms: None,
        usage: None,
        cost: Cost {
            usd: 0.0,
            estimated: false,
            unpriced: None,
        },
        provider: None,
        generation_id: generation_id.cloned(),
        finish_reason: None,
        failure: None,
        cancelled: None,
    }
}

pub fn usage_sink(input: TestkitUsageSinkInput) -> Outcome<TestkitUsageSinkResult> {
    block_on(async move {
        let sink = ScriptedUsageSink::new();
        // The call last handed over, which a step may edit.
        let mut last: Option<MeteredCall> = None;
        let mut steps = Vec::new();
        for step in &input.steps {
            let id = step.id.clone().unwrap_or_default();
            let at = step.at.unwrap_or(0);
            let mut answer = None;
            let mut filled = None;
            let mut unfilled = None;
            match step.op {
                TestkitUsageSinkInputStepsItemOp::Ids => {
                    sink.set_ids(step.ids.clone().unwrap_or_default());
                }
                TestkitUsageSinkInputStepsItemOp::None => {
                    sink.answer_none(step.on.unwrap_or(false));
                }
                TestkitUsageSinkInputStepsItemOp::Record => {
                    let call = call_of(
                        step.model.as_deref().unwrap_or(""),
                        at,
                        step.generation_id.as_ref(),
                    );
                    answer = Some(sink.record(&call));
                    last = Some(call);
                }
                TestkitUsageSinkInputStepsItemOp::Edit => {
                    if let Some(call) = last.as_mut() {
                        call.model = step.model.clone().unwrap_or_default();
                    }
                }
                TestkitUsageSinkInputStepsItemOp::Fill => {
                    sink.fill(
                        &id,
                        &GenerationStats {
                            provider: Some(step.provider.clone().unwrap_or_default()),
                            latency_ms: None,
                            generation_ms: None,
                            cost: None,
                            account_cost: None,
                            upstream_cost: None,
                            byok: None,
                            in_: None,
                            out: None,
                            finish: None,
                        },
                    );
                }
                TestkitUsageSinkInputStepsItemOp::MarkFilled => {
                    sink.set_filled(step.ids.clone().unwrap_or_default());
                }
                TestkitUsageSinkInputStepsItemOp::Rows => {
                    sink.set_unfilled(step.rows.clone().unwrap_or_default());
                }
                TestkitUsageSinkInputStepsItemOp::Filled => {
                    filled = Some(sink.filled(&id, at).await);
                }
                TestkitUsageSinkInputStepsItemOp::Unfilled => {
                    unfilled = Some(sink.unfilled(at).await);
                }
            }
            let calls = sink.calls();
            steps.push(TestkitUsageSinkResultStepsItem {
                answer,
                filled,
                unfilled,
                calls: TestkitUsageSinkResultStepsItemCalls {
                    record: calls.record as i64,
                    fill: calls.fill as i64,
                    filled: calls.filled as i64,
                    unfilled: calls.unfilled as i64,
                },
                recorded: sink
                    .recorded()
                    .iter()
                    .map(|call| format!("{}@{}", call.model, call.at))
                    .collect(),
                fills: sink
                    .fills()
                    .iter()
                    .map(|(id, stats)| format!("{id}/{}", stats.provider.as_deref().unwrap_or("")))
                    .collect(),
                filled_asks: sink
                    .filled_asks()
                    .iter()
                    .map(|(id, at)| format!("{id}@{at}"))
                    .collect(),
                unfilled_asks: sink.unfilled_asks(),
            });
        }
        TestkitUsageSinkResult { steps }
    })
}

pub fn time_offset(input: TestkitTimeOffsetInput) -> Outcome<TestkitTimeOffsetResult> {
    let offsets: Vec<OffsetFrom> = input
        .offsets
        .iter()
        .map(|each| OffsetFrom {
            from_ms: each.from_ms,
            minutes: each.minutes,
        })
        .collect();
    let wheel = TimerWheel::new(input.start_ms);
    wheel.set_offsets(offsets.clone());
    let clock: Box<dyn Clock> = match input.clock {
        TestkitTimeOffsetInputClock::Stepped => {
            Box::new(SteppedClock::with_offsets(input.start_ms, 1, offsets))
        }
        TestkitTimeOffsetInputClock::Wheel => Box::new(wheel),
        TestkitTimeOffsetInputClock::Scheduler => Box::new(FakeScheduler::new(Arc::new(wheel))),
    };
    let rows = input
        .asks
        .iter()
        .map(|at_ms| {
            let minutes = clock.utc_offset_minutes(*at_ms);
            TestkitTimeOffsetResultRowsItem {
                at_ms: *at_ms,
                minutes,
                day: day_key(*at_ms, minutes),
            }
        })
        .collect();
    Ok(TestkitTimeOffsetResult { rows })
}

pub fn time_offset_run(input: TestkitTimeOffsetRunInput) -> Outcome<TestkitTimeOffsetRunResult> {
    let rows = |offsets: &[TestkitTimeOffsetRunInputOffsetsItem]| -> Vec<OffsetFrom> {
        offsets
            .iter()
            .map(|each| OffsetFrom {
                from_ms: each.from_ms,
                minutes: each.minutes,
            })
            .collect()
    };
    let wheel = Arc::new(TimerWheel::with_offsets(
        input.start_ms,
        rows(&input.offsets),
    ));
    let scheduler = FakeScheduler::new(Arc::clone(&wheel));
    let mut steps = Vec::new();
    for step in &input.steps {
        let mut minutes = None;
        match step.op {
            TestkitTimeOffsetRunInputStepsItemOp::Set => {
                let held = step.offsets.clone().unwrap_or_default();
                let converted = held
                    .iter()
                    .map(|each| OffsetFrom {
                        from_ms: each.from_ms,
                        minutes: each.minutes,
                    })
                    .collect();
                wheel.set_offsets(converted);
            }
            TestkitTimeOffsetRunInputStepsItemOp::Advance => {
                wheel.advance(step.ms.unwrap_or(0));
            }
            TestkitTimeOffsetRunInputStepsItemOp::Ask => {
                minutes = Some(scheduler.utc_offset_minutes(step.at_ms.unwrap_or(0)));
            }
        }
        steps.push(TestkitTimeOffsetRunResultStepsItem {
            now_ms: scheduler.now_ms(),
            minutes,
        });
    }
    Ok(TestkitTimeOffsetRunResult { steps })
}
