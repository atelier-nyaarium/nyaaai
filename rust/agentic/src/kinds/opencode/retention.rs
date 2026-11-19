//! OpenCode's zero-retention table by plan.

use nyaaai_contract::shapes::provider::ProviderPlan;
use nyaaai_contract::time::parse_iso_time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    pub zdr: bool,
    /// Last zero-retention day, UTC.
    pub until: Option<&'static str>,
}

const ZERO: Retention = Retention {
    zdr: true,
    until: None,
};

/// 30-day or longer retention.
const KEPT: Retention = Retention {
    zdr: false,
    until: None,
};

const DEEPSEEK: Retention = Retention {
    zdr: true,
    until: Some("2026-11-30"),
};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Unlisted IDs: non-ZDR.
const GO: [(&str, Retention); 17] = [
    ("glm-*", ZERO),
    ("kimi-*", ZERO),
    ("minimax-*", ZERO),
    ("qwen*", ZERO),
    ("gemini-4-argon", ZERO),
    ("hy-*", ZERO),
    ("longcat-*", ZERO),
    ("mimo-*", ZERO),
    ("step-*", ZERO),
    ("deepseek-*", DEEPSEEK),
    ("grok-4.7", KEPT),
    ("gpt-6-luna", KEPT),
    ("gpt-5.6-luna", KEPT),
    ("claude-haiku-5-5", KEPT),
    ("beam-free", KEPT),
    ("muse-spark-*", KEPT),
    ("iceman*", KEPT),
];

const PAYG: [(&str, Retention); 15] = [
    ("glm-*", ZERO),
    ("kimi-*", ZERO),
    ("minimax-*", ZERO),
    ("qwen*", ZERO),
    ("gemini-*", ZERO),
    ("hy-*", ZERO),
    ("longcat-*", ZERO),
    ("mimo-*", ZERO),
    ("step-*", ZERO),
    ("deepseek-*", DEEPSEEK),
    ("gpt-*", KEPT),
    ("claude-*", KEPT),
    ("mistral-*", KEPT),
    ("grok-*", KEPT),
    ("beam*", KEPT),
];

/// Exact, then longest prefix.
pub fn retention_of(plan: ProviderPlan, id: &str) -> Option<Retention> {
    let table: &[(&str, Retention)] = match plan {
        ProviderPlan::Go => &GO,
        ProviderPlan::Payg => &PAYG,
    };
    longest(table, id)
}

/// Exact key first; of two prefixes of one length the first listed wins.
fn longest(table: &[(&str, Retention)], id: &str) -> Option<Retention> {
    if let Some((_, held)) = table.iter().find(|(key, _)| *key == id) {
        return Some(*held);
    }
    let mut best: Option<(usize, Retention)> = None;
    for (key, held) in table {
        let matches = key
            .strip_suffix('*')
            .is_some_and(|prefix| id.starts_with(prefix));
        if matches && best.is_none_or(|(length, _)| key.len() > length) {
            best = Some((key.len(), *held));
        }
    }
    best.map(|(_, held)| held)
}

pub fn is_zdr(plan: ProviderPlan, id: &str, now: i64) -> bool {
    let Some(retention) = retention_of(plan, id).filter(|held| held.zdr) else {
        return false;
    };
    match retention.until {
        None => true,
        Some(until) => {
            parse_iso_time(&format!("{until}T00:00:00Z")).is_some_and(|end| now < end + DAY_MS)
        }
    }
}
