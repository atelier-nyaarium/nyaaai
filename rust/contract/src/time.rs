//! Millisecond and ISO times.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cancel::CancelToken;
use crate::future::BoxFuture;
use crate::json::Value;
use crate::number::{number_of_text, round_half_up};
use crate::text::trim_text;

const DAY_MS: i64 = 86_400_000;

/// Clock values use milliseconds.
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

/// How a sleep ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slept {
    Done,
    Cancelled,
}

/// Cancels one scheduled call.
pub struct Scheduled {
    cancel: Box<dyn Fn() + Send + Sync>,
}

impl Scheduled {
    pub fn new(cancel: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            cancel: Box::new(cancel),
        }
    }

    pub fn cancel(&self) {
        (self.cancel)();
    }
}

/// A clock that also waits.
pub trait Scheduler: Clock + Send + Sync {
    /// A cancelled token ends the sleep early.
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept>;

    /// Runs once after `ms`.
    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled;
}

/// Runs on `tokio`; call inside a runtime with time enabled.
pub struct SystemScheduler;

fn wait(ms: i64) -> tokio::time::Sleep {
    tokio::time::sleep(Duration::from_millis(ms.max(0).unsigned_abs()))
}

impl Clock for SystemScheduler {
    fn now_ms(&self) -> i64 {
        SystemClock.now_ms()
    }
}

impl Scheduler for SystemScheduler {
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept> {
        let cancel = cancel.clone();
        Box::pin(async move {
            match cancel.run_until(wait(ms)).await {
                Some(()) => Slept::Done,
                None => Slept::Cancelled,
            }
        })
    }

    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled {
        let cancel = CancelToken::new();
        let held = cancel.clone();
        tokio::spawn(async move {
            if held.run_until(wait(ms)).await.is_some() {
                run();
            }
        });
        Scheduled::new(move || cancel.cancel())
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

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            at: 0,
        }
    }

    fn is_digit_at(&self, at: usize) -> bool {
        self.bytes.get(at).is_some_and(u8::is_ascii_digit)
    }

    /// Exactly `count` ASCII digits.
    fn digits(&mut self, count: usize) -> Option<i64> {
        let held = self.bytes.get(self.at..self.at + count)?;
        let mut number = 0;
        for digit in held {
            if !digit.is_ascii_digit() {
                return None;
            }
            number = number * 10 + i64::from(digit - b'0');
        }
        self.at += count;
        Some(number)
    }

    fn literal(&mut self, wanted: &str) -> bool {
        let found = self.bytes[self.at..].starts_with(wanted.as_bytes());
        if found {
            self.at += wanted.len();
        }
        found
    }

    /// One of `names`, case-sensitive.
    fn name(&mut self, names: &[&'static str]) -> Option<&'static str> {
        let found = names
            .iter()
            .find(|held| self.bytes[self.at..].starts_with(held.as_bytes()))?;
        self.at += found.len();
        Some(found)
    }

    /// `hh:mm:ss`.
    fn clock(&mut self) -> Option<(i64, i64, i64)> {
        let hour = self.digits(2)?;
        self.literal(":").then_some(())?;
        let minute = self.digits(2)?;
        self.literal(":").then_some(())?;
        Some((hour, minute, self.digits(2)?))
    }

    fn at_end(&self) -> bool {
        self.at == self.bytes.len()
    }
}

/// Millisecond-precision ISO parsing.
pub fn parse_iso_time(text: &str) -> Option<i64> {
    let mut cursor = Cursor::new(text);
    let year = cursor.digits(4)?;
    cursor.literal("-").then_some(())?;
    let month = cursor.digits(2)?;
    cursor.literal("-").then_some(())?;
    let day = cursor.digits(2)?;
    cursor.literal("T").then_some(())?;
    let (hour, minute, second) = cursor.clock()?;

    let mut millis = 0;
    if cursor.literal(".") {
        let from = cursor.at;
        while cursor.is_digit_at(cursor.at) {
            cursor.at += 1;
        }
        let count = cursor.at - from;
        if !(1..=9).contains(&count) {
            return None;
        }
        for place in 0..3 {
            let digit = cursor
                .bytes
                .get(from + place)
                .filter(|_| place < count)
                .map_or(0, |digit| digit - b'0');
            millis = millis * 10 + i64::from(digit);
        }
    }

    let mut offset = 0;
    if !cursor.literal("Z") {
        let sign = if cursor.literal("-") {
            -1
        } else if cursor.literal("+") {
            1
        } else {
            return None;
        };
        let offset_hour = cursor.digits(2)?;
        cursor.literal(":").then_some(())?;
        let offset_minute = cursor.digits(2)?;
        if offset_hour > 23 || offset_minute > 59 {
            return None;
        }
        offset = sign * (offset_hour * 60 + offset_minute) * 60_000;
    }
    if !cursor.at_end() {
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

fn month_of(name: &str) -> i64 {
    MONTHS
        .iter()
        .position(|held| *held == name)
        .map_or(0, |at| at as i64 + 1)
}

fn civil_ms(year: i64, month: i64, day: i64, time: (i64, i64, i64)) -> Option<i64> {
    let (hour, minute, second) = time;
    if !(1..=12).contains(&month) || day < 1 || day > days_in(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(days_from_civil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000)
}

fn imf_date(text: &str) -> Option<i64> {
    let mut cursor = Cursor::new(text);
    cursor.name(&SHORT_DAYS)?;
    cursor.literal(", ").then_some(())?;
    let day = cursor.digits(2)?;
    cursor.literal(" ").then_some(())?;
    let month = cursor.name(&MONTHS)?;
    cursor.literal(" ").then_some(())?;
    let year = cursor.digits(4)?;
    cursor.literal(" ").then_some(())?;
    let time = cursor.clock()?;
    if !cursor.literal(" GMT") || !cursor.at_end() {
        return None;
    }
    civil_ms(year, month_of(month), day, time)
}

fn rfc850_date(text: &str, now_ms: i64) -> Option<i64> {
    let mut cursor = Cursor::new(text);
    cursor.name(&LONG_DAYS)?;
    cursor.literal(", ").then_some(())?;
    let day = cursor.digits(2)?;
    cursor.literal("-").then_some(())?;
    let month = cursor.name(&MONTHS)?;
    cursor.literal("-").then_some(())?;
    let short = cursor.digits(2)?;
    cursor.literal(" ").then_some(())?;
    let time = cursor.clock()?;
    if !cursor.literal(" GMT") || !cursor.at_end() {
        return None;
    }
    let now_year = day_key(now_ms, 0)
        .and_then(|key| key[..4].parse::<i64>().ok())
        .unwrap_or(1970);
    let near = now_year.div_euclid(100) * 100 + short;
    let year = if near - now_year > 50 {
        near - 100
    } else if now_year - near >= 50 {
        near + 100
    } else {
        near
    };
    civil_ms(year, month_of(month), day, time)
}

fn asctime_date(text: &str) -> Option<i64> {
    let mut cursor = Cursor::new(text);
    cursor.name(&SHORT_DAYS)?;
    cursor.literal(" ").then_some(())?;
    let month = cursor.name(&MONTHS)?;
    cursor.literal(" ").then_some(())?;
    let day = if cursor.literal(" ") {
        cursor.digits(1)?
    } else {
        cursor.digits(2)?
    };
    cursor.literal(" ").then_some(())?;
    let time = cursor.clock()?;
    cursor.literal(" ").then_some(())?;
    let year = cursor.digits(4)?;
    if !cursor.at_end() {
        return None;
    }
    civil_ms(year, month_of(month), day, time)
}

/// Case-sensitive; RFC 850 uses a 50-year pivot.
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
    let text = trim_text(header.unwrap_or(""));
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
        Value::String(text) => parse_stamp(trim_text(text), now_ms),
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
    fn the_system_scheduler_sleeps_and_stops() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            let scheduler = SystemScheduler;
            let token = CancelToken::new();
            assert_eq!(scheduler.sleep(1, &token).await, Slept::Done);
            let slept = scheduler.sleep(60_000, &token);
            token.cancel();
            assert_eq!(slept.await, Slept::Cancelled);

            let (sender, receiver) = tokio::sync::oneshot::channel();
            scheduler.schedule(
                1,
                Box::new(move || {
                    let _ = sender.send(7);
                }),
            );
            assert_eq!(receiver.await, Ok(7));
            let (sender, receiver) = tokio::sync::oneshot::channel::<i32>();
            scheduler
                .schedule(
                    1,
                    Box::new(move || {
                        let _ = sender.send(7);
                    }),
                )
                .cancel();
            assert!(receiver.await.is_err());
        });
    }

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
