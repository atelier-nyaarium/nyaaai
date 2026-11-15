//! Millisecond and ISO times.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::js::trim;
use crate::json::Value;
use crate::number::{number_of_text, round_half_up};

const DAY_MS: i64 = 86_400_000;

/// Time in milliseconds.
pub trait Clock {
    fn now_ms(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as i64)
    }
}

/// Advances on each read.
pub struct SteppedClock {
    start_ms: i64,
    step_ms: i64,
    reads: AtomicI64,
}

impl SteppedClock {
    pub fn new(start_ms: i64, step_ms: i64) -> Self {
        Self {
            start_ms,
            step_ms,
            reads: AtomicI64::new(0),
        }
    }
}

impl Clock for SteppedClock {
    fn now_ms(&self) -> i64 {
        let reads = self.reads.fetch_add(1, Ordering::Relaxed);
        self.start_ms
            .saturating_add(self.step_ms.saturating_mul(reads))
    }
}

fn leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in(year: i64, month: i64) -> i64 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted = if month <= 2 { year - 1 } else { year };
    let era = shifted.div_euclid(400);
    let year_of_era = shifted - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn digits(bytes: &[u8], at: &mut usize, count: usize) -> Option<i64> {
    let held = bytes.get(*at..*at + count)?;
    let mut number = 0;
    for digit in held {
        if !digit.is_ascii_digit() {
            return None;
        }
        number = number * 10 + i64::from(digit - b'0');
    }
    *at += count;
    Some(number)
}

fn literal(bytes: &[u8], at: &mut usize, wanted: u8) -> Option<()> {
    (bytes.get(*at) == Some(&wanted)).then(|| *at += 1)
}

/// Millisecond-precision ISO parsing.
pub fn parse_iso_time(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let mut at = 0;
    let year = digits(bytes, &mut at, 4)?;
    literal(bytes, &mut at, b'-')?;
    let month = digits(bytes, &mut at, 2)?;
    literal(bytes, &mut at, b'-')?;
    let day = digits(bytes, &mut at, 2)?;
    literal(bytes, &mut at, b'T')?;
    let hour = digits(bytes, &mut at, 2)?;
    literal(bytes, &mut at, b':')?;
    let minute = digits(bytes, &mut at, 2)?;
    literal(bytes, &mut at, b':')?;
    let second = digits(bytes, &mut at, 2)?;

    let mut millis = 0;
    if literal(bytes, &mut at, b'.').is_some() {
        let from = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        let count = at - from;
        if !(1..=9).contains(&count) {
            return None;
        }
        for place in 0..3 {
            let digit = bytes
                .get(from + place)
                .filter(|_| place < count)
                .map_or(0, |digit| digit - b'0');
            millis = millis * 10 + i64::from(digit);
        }
    }

    let mut offset = 0;
    if literal(bytes, &mut at, b'Z').is_none() {
        let sign = match bytes.get(at)? {
            b'+' => 1,
            b'-' => -1,
            _ => return None,
        };
        at += 1;
        let offset_hour = digits(bytes, &mut at, 2)?;
        literal(bytes, &mut at, b':')?;
        let offset_minute = digits(bytes, &mut at, 2)?;
        if offset_hour > 23 || offset_minute > 59 {
            return None;
        }
        offset = sign * (offset_hour * 60 + offset_minute) * 60_000;
    }
    if at != bytes.len() {
        return None;
    }
    if !(1..=12).contains(&month) || day < 1 || day > days_in(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(
        days_from_civil(year, month, day) * DAY_MS
            + ((hour * 60 + minute) * 60 + second) * 1000
            + millis
            - offset,
    )
}

/// ECMAScript Date limit.
const LAST_MS: i64 = 8_640_000_000_000_000;

/// Formats ECMAScript Date times.
pub fn iso_time(ms: i64) -> Option<String> {
    if !(-LAST_MS..=LAST_MS).contains(&ms) {
        return None;
    }
    let days = ms.div_euclid(DAY_MS);
    let within = ms.rem_euclid(DAY_MS);
    let (year, month, day) = civil_from_days(days);
    let spelled_year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else {
        format!("{}{:06}", if year < 0 { '-' } else { '+' }, year.abs())
    };
    Some(format!(
        "{spelled_year}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        within / 3_600_000,
        within / 60_000 % 60,
        within / 1000 % 60,
        within % 1000
    ))
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const SHORT_DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const LONG_DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
/// Below this: seconds.
const SECONDS_BELOW: f64 = 1e11;
/// Largest exact double integer.
const SAFE: f64 = 9_007_199_254_740_991.0;

/// Exactly `count` ASCII digits.
fn number_of(text: &str, count: usize) -> Option<i64> {
    let mut at = 0;
    digits(text.as_bytes(), &mut at, count).filter(|_| text.len() == count)
}

fn month_of(name: &str) -> Option<i64> {
    MONTHS
        .iter()
        .position(|held| *held == name)
        .map(|at| at as i64 + 1)
}

fn civil_ms(year: i64, month: i64, day: i64, clock: &str) -> Option<i64> {
    let mut parts = clock.split(':');
    let hour = number_of(parts.next()?, 2)?;
    let minute = number_of(parts.next()?, 2)?;
    let second = number_of(parts.next()?, 2)?;
    if parts.next().is_some() || day < 1 || day > days_in(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(days_from_civil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000)
}

/// IMF-fixdate format.
fn imf_date(text: &str) -> Option<i64> {
    let (day_name, rest) = text.split_once(", ")?;
    let parts: Vec<&str> = rest.split(' ').collect();
    let [day, month, year, clock, "GMT"] = parts.as_slice() else {
        return None;
    };
    if !SHORT_DAYS.contains(&day_name) {
        return None;
    }
    civil_ms(
        number_of(year, 4)?,
        month_of(month)?,
        number_of(day, 2)?,
        clock,
    )
}

/// RFC 850 format.
fn rfc850_date(text: &str, now_ms: i64) -> Option<i64> {
    let (day_name, rest) = text.split_once(", ")?;
    let parts: Vec<&str> = rest.split(' ').collect();
    let [date, clock, "GMT"] = parts.as_slice() else {
        return None;
    };
    let date: Vec<&str> = date.split('-').collect();
    let [day, month, short] = date.as_slice() else {
        return None;
    };
    if !LONG_DAYS.contains(&day_name) {
        return None;
    }
    let now_year = day_key(now_ms, 0)
        .and_then(|key| key[..4].parse::<i64>().ok())
        .unwrap_or(1970);
    let near = now_year.div_euclid(100) * 100 + number_of(short, 2)?;
    let year = if near - now_year > 50 {
        near - 100
    } else if now_year - near >= 50 {
        near + 100
    } else {
        near
    };
    civil_ms(year, month_of(month)?, number_of(day, 2)?, clock)
}

/// Asctime format.
fn asctime_date(text: &str) -> Option<i64> {
    if !text.is_ascii() || text.len() != 24 {
        return None;
    }
    let spaced = [3, 7, 10, 19].iter().all(|at| text.as_bytes()[*at] == b' ');
    if !spaced || !SHORT_DAYS.contains(&&text[..3]) {
        return None;
    }
    let day = &text[8..10];
    let day = number_of(day.strip_prefix(' ').unwrap_or(day), day.trim_start().len())?;
    civil_ms(
        number_of(&text[20..], 4)?,
        month_of(&text[4..7])?,
        day,
        &text[11..19],
    )
}

/// Exact-case HTTP dates; RFC 850 uses a 50-year pivot.
pub fn parse_http_date(text: &str, now_ms: i64) -> Option<i64> {
    imf_date(text)
        .or_else(|| rfc850_date(text, now_ms))
        .or_else(|| asctime_date(text))
}

fn parse_stamp(text: &str, now_ms: i64) -> Option<i64> {
    parse_iso_time(text).or_else(|| parse_http_date(text, now_ms))
}

fn whole_ms(ms: f64) -> Option<i64> {
    (ms.is_finite() && ms.abs() <= SAFE).then(|| round_half_up(ms) as i64)
}

/// Nonnegative wait in milliseconds.
pub fn parse_retry_after(header: Option<&str>, now_ms: i64) -> Option<i64> {
    let text = trim(header.unwrap_or(""));
    if text.is_empty() {
        return None;
    }
    let wait = match number_of_text(text) {
        Some(seconds) => whole_ms(seconds * 1000.0)?,
        None => whole_ms(parse_stamp(text, now_ms)?.saturating_sub(now_ms) as f64)?,
    };
    Some(wait.max(0))
}

/// Seconds, milliseconds, or dates.
pub fn reset_stamp_ms(value: &Value, now_ms: i64) -> Option<i64> {
    match value {
        Value::Number(held) if held.abs() < SECONDS_BELOW => whole_ms(held * 1000.0),
        Value::Number(held) => whole_ms(*held),
        Value::String(text) => parse_stamp(trim(text), now_ms),
        _ => None,
    }
}

/// UTC-offset day; years 0000-9999.
pub fn day_key(ms: i64, offset_minutes: i64) -> Option<String> {
    let local = ms.checked_add(offset_minutes.checked_mul(60_000)?)?;
    iso_time(local)
        .filter(|text| text.len() == 24)
        .map(|text| text[..10].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_back_what_it_writes() {
        for ms in [
            0,
            1,
            -1,
            1_794_528_000_123,
            951_782_400_000,
            -62_167_219_200_000,
            253_402_300_799_999,
        ] {
            let spelled = iso_time(ms).unwrap();
            assert_eq!(parse_iso_time(&spelled), Some(ms), "{spelled}");
        }
        assert_eq!(
            iso_time(1_794_528_000_123).as_deref(),
            Some("2026-11-13T00:00:00.123Z")
        );
        assert_eq!(iso_time(LAST_MS + 1), None);
    }

    #[test]
    fn refuses_other_text() {
        for text in [
            "",
            "2026-11-13",
            "2026-11-13T00:00:00",
            "2026-02-30T00:00:00Z",
            "2026-11-13 00:00:00Z",
        ] {
            assert_eq!(parse_iso_time(text), None, "{text}");
        }
    }
}
