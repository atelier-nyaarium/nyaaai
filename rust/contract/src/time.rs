//! Millisecond and ISO times.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::runtime::Handle;

use crate::cancel::CancelToken;
use crate::future::BoxFuture;
use crate::json::Value;
use crate::number::{number_of_text, round_half_up};
use crate::text::trim_text;
use crate::time_tokens::{LONG_DAYS, MONTHS, SHORT_DAYS};
use crate::zone::system_zone;

const DAY_MS: i64 = 86_400_000;

/// Clock values use milliseconds.
pub trait Clock {
    fn now_ms(&self) -> i64;

    /// Minutes east of UTC at that instant; 0 when unknown. System clocks read the zone once.
    fn utc_offset_minutes(&self, at_ms: i64) -> i64;
}

/// An offset in force from an instant on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffsetFrom {
    pub from_ms: i64,
    pub minutes: i64,
}

/// The latest entry's offset at or before `at_ms`; a later entry wins a tie; 0 before the first.
pub fn scripted_offset(offsets: &[OffsetFrom], at_ms: i64) -> i64 {
    let mut held: Option<&OffsetFrom> = None;
    for offset in offsets {
        if offset.from_ms <= at_ms && held.is_none_or(|last| offset.from_ms >= last.from_ms) {
            held = Some(offset);
        }
    }
    held.map_or(0, |offset| offset.minutes)
}

/// The system's time; offsets use `TZ`, else `/etc/localtime`, read once; unreadable is UTC.
pub struct SystemClock;

fn system_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as i64)
}

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        system_now_ms()
    }

    fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
        system_zone().offset_minutes(at_ms)
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

/// A clock that also waits. Delays are whole milliseconds, at most 2^53-1.
pub trait Scheduler: Clock + Send + Sync {
    /// A cancelled token ends the sleep early.
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept>;

    /// Runs once after `ms`.
    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled;
}

/// Waits on the given runtime, which needs its time driver; on an ended runtime a sleep ends cancelled.
pub struct SystemScheduler {
    runtime: Handle,
}

impl SystemScheduler {
    pub fn new(runtime: Handle) -> Self {
        Self { runtime }
    }
}

/// The instant `ms` from the call; a span too far to count is thirty years.
fn due(ms: i64) -> tokio::time::Instant {
    let now = tokio::time::Instant::now();
    now.checked_add(Duration::from_millis(ms.max(0).unsigned_abs()))
        .unwrap_or_else(|| now + Duration::from_secs(30 * 365 * 86_400))
}

impl Clock for SystemScheduler {
    fn now_ms(&self) -> i64 {
        system_now_ms()
    }

    fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
        SystemClock.utc_offset_minutes(at_ms)
    }
}

impl Scheduler for SystemScheduler {
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept> {
        let cancel = cancel.clone();
        let at = due(ms);
        let (done, woken) = tokio::sync::oneshot::channel();
        self.runtime.spawn(async move {
            let ended = cancel
                .run_until(tokio::time::sleep_until(at))
                .await
                .is_some();
            let _ = done.send(ended);
        });
        Box::pin(async move {
            match woken.await {
                Ok(true) => Slept::Done,
                _ => Slept::Cancelled,
            }
        })
    }

    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled {
        let cancel = CancelToken::new();
        let held = cancel.clone();
        let at = due(ms);
        self.runtime.spawn(async move {
            if held.run_until(tokio::time::sleep_until(at)).await.is_some() {
                run();
            }
        });
        Scheduled::new(move || cancel.cancel())
    }
}

/// Which ended a deadline first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    Deadline,
    Caller,
}

/// Ends at a deadline or the caller's cancel; clear or drop it when done.
pub struct Deadline {
    token: CancelToken,
    /// A child of the caller's token; only the caller cancels it.
    link: Mutex<Option<CancelToken>>,
    timed_out: Arc<AtomicBool>,
    /// What `ended` answered at the clear: 0 none, 1 deadline, 2 caller.
    frozen: AtomicU8,
    timer: Option<Scheduled>,
}

impl Deadline {
    /// Cancelled by the deadline or the caller, until cleared.
    pub fn token(&self) -> &CancelToken {
        &self.token
    }

    /// Which came first; `None` while neither has; a clear keeps it.
    pub fn ended(&self) -> Option<Ended> {
        if self.timed_out.load(Ordering::SeqCst) {
            return Some(Ended::Deadline);
        }
        let link = self.link.lock().unwrap_or_else(|held| held.into_inner());
        match &*link {
            Some(link) => link.is_cancelled().then_some(Ended::Caller),
            None => match self.frozen.load(Ordering::SeqCst) {
                2 => Some(Ended::Caller),
                _ => None,
            },
        }
    }

    /// Drops the timer and the caller; a later caller cancel does not reach the token.
    pub fn clear(&self) {
        if let Some(timer) = &self.timer {
            timer.cancel();
        }
        let ended = self.ended();
        let mut link = self.link.lock().unwrap_or_else(|held| held.into_inner());
        if link.take().is_some() && ended == Some(Ended::Caller) {
            self.frozen.store(2, Ordering::SeqCst);
        }
    }
}

impl Drop for Deadline {
    fn drop(&mut self) {
        self.clear();
    }
}

/// `ms` or the caller's cancel, first wins; ties go by schedule order.
pub fn deadline(scheduler: &dyn Scheduler, ms: i64, caller: &CancelToken) -> Deadline {
    let link = caller.child();
    let token = link.child();
    let timed_out = Arc::new(AtomicBool::new(false));
    let timer = if token.is_cancelled() {
        None
    } else {
        let (held, flag) = (token.clone(), Arc::clone(&timed_out));
        Some(scheduler.schedule(
            ms,
            Box::new(move || {
                if !held.is_cancelled() {
                    flag.store(true, Ordering::SeqCst);
                    held.cancel();
                }
            }),
        ))
    };
    Deadline {
        token,
        link: Mutex::new(Some(link)),
        timed_out,
        frozen: AtomicU8::new(0),
        timer,
    }
}

/// Advances on each read; the offsets are the test's.
pub struct SteppedClock {
    start_ms: i64,
    step_ms: i64,
    reads: AtomicI64,
    offsets: Vec<OffsetFrom>,
}

impl SteppedClock {
    pub fn new(start_ms: i64, step_ms: i64) -> Self {
        Self::with_offsets(start_ms, step_ms, Vec::new())
    }

    pub fn with_offsets(start_ms: i64, step_ms: i64, offsets: Vec<OffsetFrom>) -> Self {
        Self {
            start_ms,
            step_ms,
            reads: AtomicI64::new(0),
            offsets,
        }
    }
}

impl Clock for SteppedClock {
    fn now_ms(&self) -> i64 {
        let reads = self.reads.fetch_add(1, Ordering::Relaxed);
        self.start_ms
            .saturating_add(self.step_ms.saturating_mul(reads))
    }

    fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
        scripted_offset(&self.offsets, at_ms)
    }
}

pub(crate) fn leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub(crate) fn days_in(year: i64, month: i64) -> i64 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub(crate) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted = if month <= 2 { year - 1 } else { year };
    let era = shifted.div_euclid(400);
    let year_of_era = shifted - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

pub(crate) fn civil_from_days(days: i64) -> (i64, i64, i64) {
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
            let scheduler = SystemScheduler::new(Handle::current());
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
    }
}
