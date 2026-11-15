//! Time cases.

use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn parse_iso(input: TimeParseIsoInput) -> Outcome<TimeParseIsoResult> {
    let results = input
        .texts
        .iter()
        .map(|text| nyaaai_contract::time::parse_iso_time(text))
        .collect();
    Ok(TimeParseIsoResult { results })
}

pub fn http_date(input: TimeHttpDateInput) -> Outcome<TimeHttpDateResult> {
    let results = input
        .texts
        .iter()
        .map(|text| nyaaai_contract::time::parse_http_date(text, input.now_ms))
        .collect();
    Ok(TimeHttpDateResult { results })
}

pub fn retry_after(input: TimeRetryAfterInput) -> Outcome<TimeRetryAfterResult> {
    let results = input
        .headers
        .iter()
        .map(|header| nyaaai_contract::time::parse_retry_after(header.as_deref(), input.now_ms))
        .collect();
    Ok(TimeRetryAfterResult { results })
}

pub fn reset_stamp(input: TimeResetStampInput) -> Outcome<TimeResetStampResult> {
    let results = input
        .values
        .iter()
        .map(|value| nyaaai_contract::time::reset_stamp_ms(value, input.now_ms))
        .collect();
    Ok(TimeResetStampResult { results })
}

pub fn day_key(input: TimeDayKeyInput) -> Outcome<TimeDayKeyResult> {
    let rows = input
        .times
        .iter()
        .map(|ms| {
            input
                .offsets
                .iter()
                .map(|offset| nyaaai_contract::time::day_key(*ms, *offset))
                .collect()
        })
        .collect();
    Ok(TimeDayKeyResult { rows })
}

pub fn iso(input: TimeIsoInput) -> Outcome<TimeIsoResult> {
    let texts = input
        .times
        .iter()
        .map(|ms| nyaaai_contract::time::iso_time(*ms))
        .collect();
    Ok(TimeIsoResult { texts })
}
