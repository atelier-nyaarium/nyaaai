//! Model request metering: where a call's record, bill and stats go.

use std::sync::Arc;

use nyaaai_contract::shapes::usage::{CallTag, MeteredCall};
use nyaaai_contract::usage_sink::UsageSink;

use crate::health::health::{Billing, ProviderHealth};

/// The features the page labels; any other string is kept and shown by its id.
pub type Feature = String;

/// The features the page labels.
pub const KNOWN_FEATURES: [&str; 4] = ["chat", "analysis", "wand", "tester"];

/// Where the health is told a reply's billing.
pub trait Bills: Send + Sync {
    fn billed(&self, id: &str, reply: &Billing);
}

impl Bills for ProviderHealth {
    fn billed(&self, id: &str, reply: &Billing) {
        ProviderHealth::billed(self, id, reply);
    }
}

/// Told each call a request made.
pub trait Meter: Send + Sync {
    fn called(&self, call: &MeteredCall);
}

/// Queues delayed stats for a record that names a generation: the record's id, the generation id
/// and the call.
pub type StatsHook = Arc<dyn Fn(&str, &str, &MeteredCall) + Send + Sync>;

pub struct MeterDeps {
    /// Where the call is written; the id it answers names the record.
    pub usage: Arc<dyn UsageSink>,
    pub health: Arc<dyn Bills>,
    pub stats: StatsHook,
}

/// A call made for an analysis lane.
pub fn on_behalf_of(tag: &CallTag, lane: &str) -> CallTag {
    CallTag {
        feature: "analysis".to_owned(),
        lane: Some(lane.to_owned()),
        folder: tag.folder.clone(),
    }
}

struct ProviderMeter {
    d: MeterDeps,
}

impl Meter for ProviderMeter {
    fn called(&self, call: &MeteredCall) {
        let id = self.d.usage.record(call);
        if let (Some(id), Some(generation_id)) = (&id, &call.generation_id) {
            (self.d.stats)(id, generation_id, call);
        }
        let reply = call.usage.as_ref();
        let source = reply.and_then(|held| held.billing_source.as_ref());
        if let (Some(entry), Some(source)) = (&call.entry, source) {
            self.d.health.billed(
                entry,
                &Billing {
                    source: source.clone(),
                    allowance: reply.and_then(|held| held.allowance),
                    cost: reply.and_then(|held| held.cost),
                },
            );
        }
    }
}

/// Records calls and billing.
pub fn provider_meter(d: MeterDeps) -> Arc<dyn Meter> {
    Arc::new(ProviderMeter { d })
}
