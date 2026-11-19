//! Pool cooldown cases.

use nyaaai_agentic::health::pool::{
    ALLOWANCE_SOURCES, BALANCE_COOL_MS, PROBE_FLOOR_MS, probe_at as probe_time, step as step_of,
    verdict as verdict_of,
};
use nyaaai_contract::shapes::health::{PoolEvent, PoolEventAdmitted};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn step(input: PoolStepInput) -> Outcome<PoolStepResult> {
    Ok(PoolStepResult {
        runs: input
            .runs
            .iter()
            .map(|run| {
                let mut state = run.start.clone();
                PoolStepResultRunsItem {
                    states: run
                        .steps
                        .iter()
                        .map(|held| {
                            state = step_of(&state, &held.event, held.now_ms, input.pause_ms);
                            state.clone()
                        })
                        .collect(),
                }
            })
            .collect(),
    })
}

pub fn verdict(input: PoolVerdictInput) -> Outcome<PoolVerdictResult> {
    Ok(PoolVerdictResult {
        verdicts: input
            .rows
            .iter()
            .map(|row| verdict_of(&row.state, row.exempt, row.now_ms))
            .collect(),
    })
}

pub fn probe_at(input: PoolProbeAtInput) -> Outcome<PoolProbeAtResult> {
    Ok(PoolProbeAtResult {
        times: input
            .rows
            .iter()
            .map(|row| probe_time(row.now_ms, row.eta))
            .collect(),
        balance_cool_ms: BALANCE_COOL_MS,
        probe_floor_ms: PROBE_FLOOR_MS,
        allowance_sources: ALLOWANCE_SOURCES
            .iter()
            .map(|each| (*each).to_owned())
            .collect(),
    })
}

pub fn instant(input: PoolInstantInput) -> Outcome<PoolInstantResult> {
    Ok(PoolInstantResult {
        rows: input
            .rows
            .iter()
            .map(|row| PoolInstantResultRowsItem {
                verdict: verdict_of(&row.state, row.exempt, row.now_ms),
                admitted: step_of(
                    &row.state,
                    &PoolEvent::Admitted(PoolEventAdmitted {
                        ticket: row.ticket.clone(),
                    }),
                    row.now_ms,
                    0.0,
                ),
            })
            .collect(),
    })
}
