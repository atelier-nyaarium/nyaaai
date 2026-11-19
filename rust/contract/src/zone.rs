//! Host time zones, read by hand from TZif files and POSIX rules. The leap seconds of `right/`
//! zone files are not applied: their transitions read up to 27 seconds off.

use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use crate::time::{civil_from_days, days_from_civil, days_in};

/// A zone file over this many bytes is refused.
pub const MAX_ZONE_BYTES: u64 = 1 << 20;

const MAGIC: &[u8] = b"TZif";
const DAY_SECS: i64 = 86_400;
/// Rule dates when a POSIX string names daylight time and no dates.
const DEFAULT_RULE: &[u8] = b"M3.2.0,M11.1.0";

/// A zone's offsets from UTC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    /// Seconds since the epoch, ascending.
    transitions: Vec<i64>,
    /// The local time type each transition starts.
    kinds: Vec<usize>,
    /// Seconds east of UTC, by local time type.
    offsets: Vec<i32>,
    /// What governs the instants past the last transition.
    rule: Option<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    /// Seconds east of UTC.
    standard: i32,
    daylight: Option<Daylight>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Daylight {
    /// Seconds east of UTC.
    offset: i32,
    start: RuleDate,
    end: RuleDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuleDate {
    day: Day,
    /// Seconds into the day; below zero and past 24 hours are allowed.
    time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Day {
    /// The `week`th `weekday` (Sunday is 0) of a month; week 5 is the last.
    Month { month: u8, week: u8, weekday: u8 },
    /// `Jn`: day 1 to 365 of the year, February 29 never counted.
    Julian(u16),
    /// `n`: day 0 to 365, February 29 counted.
    Counted(u16),
}

impl Day {
    /// Days since the epoch of this date in `year`.
    fn number(self, year: i64) -> i64 {
        let january = days_from_civil(year, 1, 1);
        match self {
            Self::Month {
                month,
                week,
                weekday,
            } => {
                let first = days_from_civil(year, i64::from(month), 1);
                let first_weekday = (first + 4).rem_euclid(7);
                let ahead = (i64::from(weekday) - first_weekday).rem_euclid(7);
                let day = first + ahead + (i64::from(week) - 1) * 7;
                if day >= first + days_in(year, i64::from(month)) {
                    day - 7
                } else {
                    day
                }
            }
            Self::Julian(day) => {
                let leap_day = i64::from(days_in(year, 2) == 29 && day >= 60);
                january + i64::from(day) - 1 + leap_day
            }
            Self::Counted(day) => january + i64::from(day),
        }
    }
}

impl Rule {
    /// Seconds east of UTC at `secs` since the epoch.
    fn offset_at(&self, secs: i64) -> i64 {
        let standard = i64::from(self.standard);
        let Some(daylight) = &self.daylight else {
            return standard;
        };
        // The year of the instant alone, as the C library reads a rule.
        let (year, _, _) = civil_from_days(secs.div_euclid(DAY_SECS));
        let start = (daylight.start.day.number(year) * DAY_SECS + i64::from(daylight.start.time))
            .saturating_sub(standard);
        let end = (daylight.end.day.number(year) * DAY_SECS + i64::from(daylight.end.time))
            .saturating_sub(i64::from(daylight.offset));
        // The end first (a southern rule): daylight lies outside them.
        let on = if start > end {
            secs < end || secs >= start
        } else {
            secs >= start && secs < end
        };
        if on {
            i64::from(daylight.offset)
        } else {
            standard
        }
    }
}

/// A cursor over bytes.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn rest(&self) -> &'a [u8] {
        self.bytes.get(self.at..).unwrap_or(&[])
    }

    fn take(&mut self, count: u64) -> Option<&'a [u8]> {
        let count = usize::try_from(count).ok()?;
        let end = self.at.checked_add(count)?;
        let held = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(held)
    }

    /// A big-endian two's complement integer of `width` bytes, at most eight.
    fn int(&mut self, width: u64) -> Option<i64> {
        let held = self.take(width)?;
        let mut number = i64::from(*held.first()? as i8);
        for byte in held.iter().skip(1) {
            number = (number << 8) | i64::from(*byte);
        }
        Some(number)
    }

    fn eat_newline(&mut self) -> Option<()> {
        let found = self.rest().first() == Some(&b'\n');
        if found {
            self.at += 1;
        }
        found.then_some(())
    }

    fn count(&mut self) -> Option<u64> {
        let held = self.take(4)?;
        Some(
            held.iter()
                .fold(0, |sum, byte| (sum << 8) | u64::from(*byte)),
        )
    }
}

/// A TZif header: the version and the six counts.
struct Header {
    version: u8,
    utc_flags: u64,
    standard_flags: u64,
    leaps: u64,
    times: u64,
    types: u64,
    chars: u64,
}

impl Header {
    fn read(reader: &mut Reader) -> Option<Self> {
        if reader.take(4)? != MAGIC {
            return None;
        }
        let version = *reader.take(1)?.first()?;
        reader.take(15)?;
        Some(Self {
            version,
            utc_flags: reader.count()?,
            standard_flags: reader.count()?,
            leaps: reader.count()?,
            times: reader.count()?,
            types: reader.count()?,
            chars: reader.count()?,
        })
    }

    /// The bytes of the data block that follows, for transition times `width` bytes wide.
    fn data_len(&self, width: u64) -> Option<u64> {
        let times = self.times.checked_mul(width.checked_add(1)?)?;
        let types = self.types.checked_mul(6)?;
        let leaps = self.leaps.checked_mul(width.checked_add(4)?)?;
        times
            .checked_add(types)?
            .checked_add(self.chars)?
            .checked_add(leaps)?
            .checked_add(self.standard_flags)?
            .checked_add(self.utc_flags)
    }
}

/// The data block under a header: transitions and local time types.
fn block(reader: &mut Reader, header: &Header, width: u64) -> Option<Zone> {
    let length = header.data_len(width)?;
    if length > reader.rest().len() as u64 || header.types == 0 {
        return None;
    }
    let mut transitions = Vec::with_capacity(usize::try_from(header.times).ok()?);
    for _ in 0..header.times {
        let at = reader.int(width)?;
        if transitions.last().is_some_and(|last| *last > at) {
            return None;
        }
        transitions.push(at);
    }
    let mut kinds = Vec::with_capacity(transitions.len());
    for _ in 0..header.times {
        let kind = usize::from(*reader.take(1)?.first()?);
        if kind as u64 >= header.types {
            return None;
        }
        kinds.push(kind);
    }
    let mut offsets = Vec::with_capacity(usize::try_from(header.types).ok()?);
    for _ in 0..header.types {
        let offset = i32::try_from(reader.int(4)?).ok()?;
        if offset == i32::MIN {
            return None;
        }
        offsets.push(offset);
        reader.take(2)?;
    }
    reader.take(header.chars)?;
    reader.take(header.leaps.checked_mul(width.checked_add(4)?)?)?;
    reader.take(header.standard_flags)?;
    reader.take(header.utc_flags)?;
    Some(Zone {
        transitions,
        kinds,
        offsets,
        rule: None,
    })
}

/// The footer: a newline, the rule and a newline; `None` unless all three are there.
fn footer<'a>(reader: &mut Reader<'a>) -> Option<&'a [u8]> {
    reader.eat_newline()?;
    let rest = reader.rest();
    let end = rest.iter().position(|byte| *byte == b'\n')?;
    rest.get(..end)
}

/// A text cursor for POSIX rules.
struct Posix<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Posix<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn eat(&mut self, wanted: u8) -> bool {
        let found = self.peek() == Some(wanted);
        if found {
            self.at += 1;
        }
        found
    }

    /// A zone name: three or more letters, or text in angle brackets.
    fn name(&mut self) -> Option<()> {
        let from = self.at;
        if self.eat(b'<') {
            while self.peek().is_some_and(|byte| byte != b'>' && byte != 0) {
                self.at += 1;
            }
            if self.at - from - 1 < 3 || !self.eat(b'>') {
                return None;
            }
        } else {
            while self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
                self.at += 1;
            }
            if self.at - from < 3 {
                return None;
            }
        }
        Some(())
    }

    /// Up to `most` digits, as a number.
    fn number(&mut self, most: usize) -> Option<i32> {
        let from = self.at;
        let mut number: i32 = 0;
        while self.at - from < most && self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            number = number * 10 + i32::from(self.peek()? - b'0');
            self.at += 1;
        }
        (self.at > from).then_some(number)
    }

    /// `[+-]hh[:mm[:ss]]` in seconds, with at most `hours` hours.
    fn clock(&mut self, hours: i32) -> Option<i32> {
        let sign = if self.eat(b'-') {
            -1
        } else {
            self.eat(b'+');
            1
        };
        let hour = self.number(3)?;
        let mut seconds = hour * 3600;
        if self.eat(b':') {
            let minute = self.number(2)?;
            if minute > 59 {
                return None;
            }
            seconds += minute * 60;
            if self.eat(b':') {
                let second = self.number(2)?;
                if second > 59 {
                    return None;
                }
                seconds += second;
            }
        }
        (hour <= hours).then_some(sign * seconds)
    }

    /// `Mm.w.d`, `Jn` or `n`.
    fn day(&mut self) -> Option<Day> {
        if self.eat(b'M') {
            let month = self.number(2)?;
            self.eat(b'.').then_some(())?;
            let week = self.number(1)?;
            self.eat(b'.').then_some(())?;
            let weekday = self.number(1)?;
            if !(1..=12).contains(&month) || !(1..=5).contains(&week) || weekday > 6 {
                return None;
            }
            return Some(Day::Month {
                month: u8::try_from(month).ok()?,
                week: u8::try_from(week).ok()?,
                weekday: u8::try_from(weekday).ok()?,
            });
        }
        if self.eat(b'J') {
            let day = self.number(3)?;
            return (1..=365)
                .contains(&day)
                .then(|| u16::try_from(day).ok().map(Day::Julian))?;
        }
        let day = self.number(3)?;
        (0..=365)
            .contains(&day)
            .then(|| u16::try_from(day).ok().map(Day::Counted))?
    }

    /// A date and an optional `/time`.
    fn date(&mut self) -> Option<RuleDate> {
        let day = self.day()?;
        let time = if self.eat(b'/') {
            self.clock(167)?
        } else {
            2 * 3600
        };
        Some(RuleDate { day, time })
    }

    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }

    /// `std offset [dst [offset] [,start[/time],end[/time]]]`.
    fn rule(&mut self) -> Option<Rule> {
        self.name()?;
        let standard = self.clock(24)?.checked_neg()?;
        if self.done() {
            return Some(Rule {
                standard,
                daylight: None,
            });
        }
        self.name()?;
        let offset = match self.peek() {
            None | Some(b',') => standard.checked_add(3600)?,
            Some(_) => self.clock(24)?.checked_neg()?,
        };
        let (start, end) = if self.done() {
            let mut fallback = Posix {
                bytes: DEFAULT_RULE,
                at: 0,
            };
            fallback.dates()?
        } else {
            self.eat(b',').then_some(())?;
            self.dates()?
        };
        self.done().then_some(Rule {
            standard,
            daylight: Some(Daylight { offset, start, end }),
        })
    }

    /// `start[/time],end[/time]`.
    fn dates(&mut self) -> Option<(RuleDate, RuleDate)> {
        let start = self.date()?;
        self.eat(b',').then_some(())?;
        Some((start, self.date()?))
    }
}

impl Zone {
    /// No offset, whatever the instant.
    pub fn utc() -> Self {
        Self {
            transitions: Vec::new(),
            kinds: Vec::new(),
            offsets: vec![0],
            rule: None,
        }
    }

    /// Reads a TZif file, version 1 from its one block and 2 or later from its 64-bit block and
    /// footer. Anything malformed, truncated or over `MAX_ZONE_BYTES` is `None`.
    pub fn from_tzif(bytes: &[u8]) -> Option<Self> {
        if bytes.len() as u64 > MAX_ZONE_BYTES {
            return None;
        }
        let mut reader = Reader::new(bytes);
        let first = Header::read(&mut reader)?;
        if first.version == 0 {
            return block(&mut reader, &first, 4);
        }
        if !(b'2'..=b'9').contains(&first.version) {
            return None;
        }
        reader.take(first.data_len(4)?)?;
        let second = Header::read(&mut reader)?;
        let mut zone = block(&mut reader, &second, 8)?;
        // A footer cut short, or one that does not parse, is no footer.
        zone.rule = footer(&mut reader).and_then(|text| Posix { bytes: text, at: 0 }.rule());
        Some(zone)
    }

    /// Reads a POSIX rule such as `EST5EDT,M3.2.0,M11.1.0`; `None` when it does not parse.
    pub fn from_posix(text: &str) -> Option<Self> {
        let mut posix = Posix {
            bytes: text.as_bytes(),
            at: 0,
        };
        let rule = posix.rule()?;
        Some(Self {
            transitions: Vec::new(),
            kinds: Vec::new(),
            offsets: vec![rule.standard],
            rule: Some(rule),
        })
    }

    fn offset_secs(&self, secs: i64) -> i64 {
        let after = self.transitions.partition_point(|at| *at <= secs);
        if after == self.transitions.len()
            && let Some(rule) = &self.rule
        {
            return rule.offset_at(secs);
        }
        let kind = after
            .checked_sub(1)
            .and_then(|last| self.kinds.get(last))
            .copied()
            .unwrap_or(0);
        i64::from(self.offsets.get(kind).copied().unwrap_or(0))
    }

    /// Whole minutes east of UTC at `at_ms`, a second's half rounded up.
    pub fn offset_minutes(&self, at_ms: i64) -> i64 {
        let secs = at_ms.div_euclid(1000);
        self.offset_secs(secs).saturating_add(30).div_euclid(60)
    }
}

/// Reads a zone file; `None` when it is missing, not a regular file, unreadable, over
/// `MAX_ZONE_BYTES` or not TZif.
fn read_zone(path: &Path) -> Option<Zone> {
    // A stat does not block on a pipe or a device, an open can.
    if !std::fs::metadata(path).ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(MAX_ZONE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    Zone::from_tzif(&bytes)
}

/// A zone file by name: an absolute path as it is, else under `zone_dir`.
fn named_zone(name: &str, zone_dir: &Path) -> Option<Zone> {
    if name.is_empty() {
        return None;
    }
    let path = Path::new(name);
    let found: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return None;
    } else {
        zone_dir.join(path)
    };
    read_zone(&found)
}

/// Finds a zone as the C library does. `tz` is the `TZ` value, if set: empty is UTC, `:name` a zone
/// file (a path, or a name under `zone_dir`; a bare `:` reads `local_time`), else a name tried as a
/// file and then read as a POSIX rule. Unset reads the file at `local_time`. A zone that cannot be found or read is UTC.
pub fn find_zone(tz: Option<&str>, zone_dir: &Path, local_time: &Path) -> Zone {
    let found = match tz {
        None => read_zone(local_time),
        Some("") => None,
        Some(text) => match text.strip_prefix(':') {
            Some("") => read_zone(local_time),
            Some(name) => named_zone(name, zone_dir),
            None => named_zone(text, zone_dir).or_else(|| Zone::from_posix(text)),
        },
    };
    found.unwrap_or_else(Zone::utc)
}

/// The zone the process runs in: `TZ`, `TZDIR` and `/etc/localtime` as the C library reads them.
/// Read once, at first use, and kept for the life of the process: a change of the machine's zone is
/// seen after a restart. A platform without TZif data is UTC.
pub fn system_zone() -> &'static Zone {
    static ZONE: OnceLock<Zone> = OnceLock::new();
    ZONE.get_or_init(read_system_zone)
}

#[cfg(unix)]
fn read_system_zone() -> Zone {
    let tz = std::env::var_os("TZ");
    let dir = std::env::var_os("TZDIR")
        .filter(|held| !held.is_empty())
        .map_or_else(|| PathBuf::from("/usr/share/zoneinfo"), PathBuf::from);
    find_zone(
        tz.as_deref().map(|held| held.to_string_lossy()).as_deref(),
        &dir,
        Path::new("/etc/localtime"),
    )
}

#[cfg(not(unix))]
fn read_system_zone() -> Zone {
    Zone::utc()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One local time type: seconds east of UTC.
    struct Kind(i32);

    /// A TZif writer for the tests.
    struct Writer {
        version: u8,
        transitions: Vec<(i64, usize)>,
        kinds: Vec<Kind>,
        footer: Option<String>,
    }

    impl Writer {
        fn new(version: u8, kinds: Vec<Kind>) -> Self {
            Self {
                version,
                transitions: Vec::new(),
                kinds,
                footer: None,
            }
        }

        fn at(mut self, at: i64, kind: usize) -> Self {
            self.transitions.push((at, kind));
            self
        }

        fn footer(mut self, text: &str) -> Self {
            self.footer = Some(text.to_owned());
            self
        }

        fn header(&self, out: &mut Vec<u8>) {
            out.extend_from_slice(b"TZif");
            out.push(self.version);
            out.extend_from_slice(&[0; 15]);
            let chars = 4 * self.kinds.len() as u32;
            for count in [
                0,
                0,
                0,
                self.transitions.len() as u32,
                self.kinds.len() as u32,
                chars,
            ] {
                out.extend_from_slice(&count.to_be_bytes());
            }
        }

        fn block(&self, out: &mut Vec<u8>, width: usize) {
            for (at, _) in &self.transitions {
                let bytes = at.to_be_bytes();
                out.extend_from_slice(&bytes[8 - width..]);
            }
            for (_, kind) in &self.transitions {
                out.push(*kind as u8);
            }
            for (at, kind) in self.kinds.iter().enumerate() {
                out.extend_from_slice(&kind.0.to_be_bytes());
                out.push(0);
                out.push((at * 4) as u8);
            }
            for _ in &self.kinds {
                out.extend_from_slice(b"TST\0");
            }
        }

        fn bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            self.header(&mut out);
            self.block(&mut out, 4);
            if self.version != 0 {
                self.header(&mut out);
                self.block(&mut out, 8);
                out.push(b'\n');
                out.extend_from_slice(self.footer.as_deref().unwrap_or("").as_bytes());
                out.push(b'\n');
            }
            out
        }

        fn zone(&self) -> Zone {
            Zone::from_tzif(&self.bytes()).unwrap()
        }
    }

    const HOUR: i32 = 3600;
    /// 2026-01-01T00:00:00Z.
    const NEW_YEAR: i64 = 1_767_225_600;
    const DAY: i64 = 86_400;

    fn minutes(zone: &Zone, secs: i64) -> i64 {
        zone.offset_minutes(secs * 1000)
    }

    #[test]
    fn a_fixed_zone_has_one_offset() {
        let zone = Writer::new(b'2', vec![Kind(3 * HOUR)])
            .footer("<+03>-3")
            .zone();
        for secs in [-5_000_000_000, 0, NEW_YEAR, 9_000_000_000] {
            assert_eq!(minutes(&zone, secs), 180);
        }
        let bare = Writer::new(b'2', vec![Kind(-5 * HOUR - 1800)]).zone();
        assert_eq!(minutes(&bare, NEW_YEAR), -330);
        assert_eq!(minutes(&Zone::utc(), NEW_YEAR), 0);
    }

    #[test]
    fn a_northern_rule_moves_with_the_seasons() {
        let zone = Writer::new(b'2', vec![Kind(-5 * HOUR)])
            .footer("EST5EDT,M3.2.0,M11.1.0")
            .zone();
        let at = |text: &str| minutes(&zone, crate::time::parse_iso_time(text).unwrap() / 1000);
        assert_eq!(at("2026-01-15T12:00:00Z"), -300);
        assert_eq!(at("2026-07-15T12:00:00Z"), -240);
        // 2026-03-08 02:00 EST is 07:00Z; 2026-11-01 02:00 EDT is 06:00Z.
        assert_eq!(at("2026-03-08T06:59:59Z"), -300);
        assert_eq!(at("2026-03-08T07:00:00Z"), -240);
        assert_eq!(at("2026-11-01T05:59:59Z"), -240);
        assert_eq!(at("2026-11-01T06:00:00Z"), -300);
    }

    #[test]
    fn a_southern_rule_spans_the_new_year() {
        let zone = Writer::new(b'2', vec![Kind(10 * HOUR)])
            .footer("AEST-10AEDT,M10.1.0,M4.1.0/3")
            .zone();
        let at = |text: &str| minutes(&zone, crate::time::parse_iso_time(text).unwrap() / 1000);
        assert_eq!(at("2026-01-01T00:00:00Z"), 660);
        assert_eq!(at("2025-12-31T23:59:59Z"), 660);
        assert_eq!(at("2026-07-01T00:00:00Z"), 600);
        assert_eq!(at("2026-12-31T23:59:59Z"), 660);
        // 2026-04-05 03:00 AEDT is 2026-04-04T16:00Z; 2026-10-04 02:00 AEST is 2026-10-03T16:00Z.
        assert_eq!(at("2026-04-04T15:59:59Z"), 660);
        assert_eq!(at("2026-04-04T16:00:00Z"), 600);
        assert_eq!(at("2026-10-03T15:59:59Z"), 600);
        assert_eq!(at("2026-10-03T16:00:00Z"), 660);
    }

    #[test]
    fn a_transition_holds_from_its_own_instant() {
        let zone = Writer::new(
            b'2',
            vec![Kind(-5 * HOUR), Kind(-4 * HOUR), Kind(-5 * HOUR)],
        )
        .at(NEW_YEAR, 1)
        .at(NEW_YEAR + DAY, 2)
        .zone();
        let ms = NEW_YEAR * 1000;
        assert_eq!(zone.offset_minutes(ms - 1), -300);
        assert_eq!(zone.offset_minutes(ms), -240);
        assert_eq!(zone.offset_minutes(ms + 1), -240);
        assert_eq!(zone.offset_minutes(ms + DAY * 1000 - 1), -240);
        assert_eq!(zone.offset_minutes(ms + DAY * 1000), -300);
        assert_eq!(zone.offset_minutes(ms + DAY * 1000 + 1), -300);
        assert_eq!(zone.offset_minutes(i64::MIN), -300);
        assert_eq!(zone.offset_minutes(i64::MAX), -300);
    }

    #[test]
    fn the_first_type_holds_before_the_first_transition() {
        let zone = Writer::new(b'2', vec![Kind(2 * HOUR), Kind(HOUR)])
            .at(NEW_YEAR, 1)
            .zone();
        assert_eq!(minutes(&zone, NEW_YEAR - 1), 120);
        assert_eq!(minutes(&zone, NEW_YEAR), 60);
    }

    #[test]
    fn the_footer_answers_past_the_last_transition() {
        let zone = Writer::new(b'2', vec![Kind(-5 * HOUR), Kind(-4 * HOUR)])
            .at(NEW_YEAR - 100 * DAY, 0)
            .at(NEW_YEAR, 0)
            .footer("EST5EDT,M3.2.0,M11.1.0")
            .zone();
        let at = |text: &str| minutes(&zone, crate::time::parse_iso_time(text).unwrap() / 1000);
        // The table ends on a standard type; only the footer knows of 2030's summer.
        assert_eq!(at("2030-07-01T12:00:00Z"), -240);
        assert_eq!(at("2030-01-01T12:00:00Z"), -300);
        assert_eq!(at("2025-07-01T12:00:00Z"), -300);
    }

    #[test]
    fn a_footer_with_no_rule_keeps_the_last_type() {
        let zone = Writer::new(b'3', vec![Kind(HOUR), Kind(2 * HOUR)])
            .at(NEW_YEAR, 1)
            .footer("")
            .zone();
        assert_eq!(minutes(&zone, NEW_YEAR + 800 * DAY), 120);
        let broken = Writer::new(b'3', vec![Kind(HOUR), Kind(2 * HOUR)])
            .at(NEW_YEAR, 1)
            .footer("not a rule")
            .zone();
        assert_eq!(minutes(&broken, NEW_YEAR + 800 * DAY), 120);
    }

    #[test]
    fn a_version_one_file_reads_its_only_block() {
        let zone = Writer::new(0, vec![Kind(HOUR), Kind(2 * HOUR)])
            .at(NEW_YEAR, 1)
            .zone();
        assert_eq!(minutes(&zone, NEW_YEAR - 1), 60);
        assert_eq!(minutes(&zone, NEW_YEAR), 120);
        assert_eq!(minutes(&zone, NEW_YEAR + 800 * DAY), 120);
    }

    #[test]
    fn a_truncated_file_is_refused_and_a_cut_footer_is_none() {
        for version in [0, b'2'] {
            let writer = Writer::new(version, vec![Kind(HOUR), Kind(2 * HOUR)])
                .at(NEW_YEAR, 1)
                .footer("EST5EDT,M3.2.0,M11.1.0");
            let bytes = writer.bytes();
            assert!(Zone::from_tzif(&bytes).is_some());
            // The footer is a newline, 22 bytes of rule and a newline; v1 has none.
            let blocks = if version == 0 {
                bytes.len()
            } else {
                bytes.len() - 24
            };
            for length in 0..bytes.len() {
                let held = Zone::from_tzif(&bytes[..length]);
                if length < blocks {
                    assert_eq!(held, None, "{version} {length}");
                } else {
                    // The blocks are whole: the zone reads, with no rule.
                    assert_eq!(held.unwrap().rule, None, "{version} {length}");
                }
            }
        }
    }

    #[test]
    fn a_footer_cut_by_one_byte_holds_the_last_type() {
        let writer = Writer::new(b'2', vec![Kind(-5 * HOUR), Kind(-4 * HOUR)])
            .at(NEW_YEAR, 0)
            .footer("EST5EDT,M3.2.0,M11.1.0");
        let bytes = writer.bytes();
        let whole = Zone::from_tzif(&bytes).unwrap();
        let cut = Zone::from_tzif(&bytes[..bytes.len() - 1]).unwrap();
        let summer = NEW_YEAR + 200 * DAY;
        assert_eq!(minutes(&whole, summer), -240);
        // No footer: the last transition's type holds, wrong for a zone that still changes.
        assert_eq!(minutes(&cut, summer), -300);
        assert_eq!(minutes(&cut, NEW_YEAR - 1), -300);
    }

    #[test]
    fn a_bad_file_is_refused() {
        let mut bytes = Writer::new(b'2', vec![Kind(HOUR)])
            .footer("<+01>-1")
            .bytes();
        assert!(Zone::from_tzif(&bytes).is_some());
        // The v1 header claims four billion transitions.
        bytes[32..36].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(Zone::from_tzif(&bytes), None);
        let mut bytes = Writer::new(b'2', vec![Kind(HOUR)]).bytes();
        bytes[0] = b'X';
        assert_eq!(Zone::from_tzif(&bytes), None);
        for version in *b"1Z\n" {
            let mut bytes = Writer::new(b'2', vec![Kind(HOUR)]).bytes();
            bytes[4] = version;
            assert_eq!(Zone::from_tzif(&bytes), None, "{version}");
        }
        assert_eq!(
            Zone::from_tzif(&Writer::new(b'2', Vec::new()).bytes()),
            None
        );
        let unsorted = Writer::new(b'2', vec![Kind(HOUR)])
            .at(NEW_YEAR, 0)
            .at(NEW_YEAR - 1, 0)
            .bytes();
        assert_eq!(Zone::from_tzif(&unsorted), None);
        let stray = Writer::new(b'2', vec![Kind(HOUR)]).at(NEW_YEAR, 1).bytes();
        assert_eq!(Zone::from_tzif(&stray), None);
        let lowest = Writer::new(b'2', vec![Kind(i32::MIN)]).bytes();
        assert_eq!(Zone::from_tzif(&lowest), None);
        let over = vec![0u8; MAX_ZONE_BYTES as usize + 1];
        assert_eq!(Zone::from_tzif(&over), None);
        assert_eq!(Zone::from_tzif(b""), None);
    }

    #[test]
    fn a_damaged_file_reads_without_panicking() {
        for version in [0, b'2'] {
            let bytes = Writer::new(version, vec![Kind(HOUR), Kind(-2 * HOUR)])
                .at(NEW_YEAR, 1)
                .at(NEW_YEAR + DAY, 0)
                .footer("<-02>2<-01>,M3.5.0/-1,M10.5.0/0")
                .bytes();
            for at in 0..bytes.len() {
                for value in [0, 1, b'\n', 0x7f, 0x80, 0xff] {
                    let mut damaged = bytes.clone();
                    damaged[at] = value;
                    if let Some(zone) = Zone::from_tzif(&damaged) {
                        for ms in [i64::MIN, -1, NEW_YEAR * 1000, i64::MAX] {
                            zone.offset_minutes(ms);
                        }
                    }
                }
            }
        }
    }

    fn rule_zone(text: &str) -> Zone {
        Zone::from_posix(text).unwrap()
    }

    fn at_time(zone: &Zone, text: &str) -> i64 {
        minutes(zone, crate::time::parse_iso_time(text).unwrap() / 1000)
    }

    #[test]
    fn a_julian_day_skips_the_leap_day() {
        let zone = rule_zone("EST5EDT,J60,J305");
        for year in [2026, 2028] {
            // March 1 and November 1, at 02:00 local.
            assert_eq!(at_time(&zone, &format!("{year}-03-01T06:59:59Z")), -300);
            assert_eq!(at_time(&zone, &format!("{year}-03-01T07:00:00Z")), -240);
            assert_eq!(at_time(&zone, &format!("{year}-11-01T05:59:59Z")), -240);
            assert_eq!(at_time(&zone, &format!("{year}-11-01T06:00:00Z")), -300);
        }
    }

    #[test]
    fn a_counted_day_keeps_the_leap_day() {
        let zone = rule_zone("EST5EDT,59/1,304/2");
        // Day 59 is March 1, or February 29 in a leap year.
        assert_eq!(at_time(&zone, "2026-03-01T05:59:59Z"), -300);
        assert_eq!(at_time(&zone, "2026-03-01T06:00:00Z"), -240);
        assert_eq!(at_time(&zone, "2028-02-29T05:59:59Z"), -300);
        assert_eq!(at_time(&zone, "2028-02-29T06:00:00Z"), -240);
        // Day 304 is November 1, or October 31 in a leap year.
        assert_eq!(at_time(&zone, "2026-11-01T05:59:59Z"), -240);
        assert_eq!(at_time(&zone, "2026-11-01T06:00:00Z"), -300);
        assert_eq!(at_time(&zone, "2028-10-31T05:59:59Z"), -240);
        assert_eq!(at_time(&zone, "2028-10-31T06:00:00Z"), -300);
    }

    #[test]
    fn a_rule_time_may_be_negative_or_past_a_day() {
        let zone = rule_zone("<-02>2<-01>,M3.5.0/-1,M10.5.0/0");
        // The last Sunday of March at -01:00 standard is Saturday 23:00, 01:00Z on the Sunday.
        assert_eq!(at_time(&zone, "2026-03-29T00:59:59Z"), -120);
        assert_eq!(at_time(&zone, "2026-03-29T01:00:00Z"), -60);
        assert_eq!(at_time(&zone, "2026-10-25T00:59:59Z"), -60);
        assert_eq!(at_time(&zone, "2026-10-25T01:00:00Z"), -120);
        let late = rule_zone("EST5EDT,M3.2.0/26,M11.1.0/25");
        // 26:00 on Sunday March 8 is 02:00 on the Monday; 25:00 EDT on November 1 is 05:00Z the next day.
        assert_eq!(at_time(&late, "2026-03-09T06:59:59Z"), -300);
        assert_eq!(at_time(&late, "2026-03-09T07:00:00Z"), -240);
        assert_eq!(at_time(&late, "2026-11-02T04:59:59Z"), -240);
        assert_eq!(at_time(&late, "2026-11-02T05:00:00Z"), -300);
    }

    #[test]
    fn a_rule_reads_names_offsets_and_defaults() {
        assert_eq!(
            at_time(&rule_zone("<+0545>-5:45"), "2026-06-01T00:00:00Z"),
            345
        );
        assert_eq!(
            at_time(&rule_zone("<+0530>-5:30"), "2026-06-01T00:00:00Z"),
            330
        );
        assert_eq!(
            at_time(&rule_zone("<-0330>3:30:00"), "2026-06-01T00:00:00Z"),
            -210
        );
        assert_eq!(at_time(&rule_zone("UTC0"), "2026-06-01T00:00:00Z"), 0);
        // No dates: the usual northern ones; no daylight offset: an hour ahead.
        let bare = rule_zone("EST5EDT");
        assert_eq!(at_time(&bare, "2026-07-01T00:00:00Z"), -240);
        assert_eq!(at_time(&bare, "2026-01-01T00:00:00Z"), -300);
        let own = rule_zone("AAA-1BBB-3:30,M3.5.0,M10.5.0");
        assert_eq!(at_time(&own, "2026-07-01T00:00:00Z"), 210);
        assert_eq!(at_time(&own, "2026-01-01T00:00:00Z"), 60);
    }

    #[test]
    fn a_bad_rule_is_refused() {
        for text in [
            "",
            "EST",
            "ES5",
            "EST5EDT,M3.2.0",
            "EST5EDT,M3.2.0,",
            "EST5EDT,M3.2.0,M11.1.0,",
            "EST5EDT,M13.1.0,M11.1.0",
            "EST5EDT,M0.1.0,M11.1.0",
            "EST5EDT,M3.0.0,M11.1.0",
            "EST5EDT,M3.6.0,M11.1.0",
            "EST5EDT,M3.2.7,M11.1.0",
            "EST5EDT,J0,J1",
            "EST5EDT,J366,J1",
            "EST5EDT,366,1",
            "EST5EDT,M3.2.0/168,M11.1.0",
            "EST25",
            "EST5:60",
            "EST5:30:60",
            "<+0>5",
            "<+05",
            "EST5EDT,M3.2.0,M11.1.0 trailing",
            "\u{0}\u{0}\u{0}",
        ] {
            assert_eq!(Zone::from_posix(text), None, "{text:?}");
        }
    }

    #[test]
    fn every_prefix_of_a_rule_reads_without_panicking() {
        let text = "<-02>2<-01>,M3.5.0/-1,M10.5.0/0";
        for length in 0..=text.len() {
            if let Some(zone) = Zone::from_posix(&text[..length]) {
                for ms in [i64::MIN, -1, 0, 1_794_528_000_000, i64::MAX] {
                    zone.offset_minutes(ms);
                }
            }
        }
    }

    /// A folder the test owns, gone when it drops.
    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("nyaaai-zone-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_zone_is_found_as_the_c_library_does() {
        let folder = Folder::new("find");
        let dir = folder.0.join("zones");
        std::fs::create_dir_all(dir.join("Test")).unwrap();
        let fixed = |hours: i32, footer: &str| {
            Writer::new(b'2', vec![Kind(hours * HOUR)])
                .footer(footer)
                .bytes()
        };
        std::fs::write(dir.join("Test").join("Zone"), fixed(2, "<+02>-2")).unwrap();
        let local = folder.0.join("localtime");
        std::fs::write(&local, fixed(3, "<+03>-3")).unwrap();
        std::fs::write(folder.0.join("plain"), b"not a zone file").unwrap();
        std::fs::write(
            folder.0.join("huge"),
            vec![0u8; MAX_ZONE_BYTES as usize + 1],
        )
        .unwrap();
        let held = |tz: Option<&str>| find_zone(tz, &dir, &local).offset_minutes(NEW_YEAR * 1000);
        let absolute = local.to_string_lossy().into_owned();
        // Unset reads the local file; empty is UTC.
        assert_eq!(held(None), 180);
        assert_eq!(held(Some("")), 0);
        // A name under the zone folder, with or without the colon; a path as it is.
        assert_eq!(held(Some(":Test/Zone")), 120);
        assert_eq!(held(Some("Test/Zone")), 120);
        assert_eq!(held(Some(&format!(":{absolute}"))), 180);
        assert_eq!(held(Some(&absolute)), 180);
        // A name that is no file is a rule when it has no colon.
        assert_eq!(held(Some("EST5EDT,M3.2.0,M11.1.0")), -300);
        assert_eq!(held(Some("<+0545>-5:45")), 345);
        assert_eq!(held(Some(":EST5EDT,M3.2.0,M11.1.0")), 0);
        // A bare colon is the local file, as the C library reads it.
        assert_eq!(held(Some(":")), 180);
        // Everything else cannot be read.
        for tz in [
            ":nope",
            "nope",
            "../zones/Test/Zone",
            ":../zones/Test/Zone",
            ":Test",
        ] {
            assert_eq!(held(Some(tz)), 0, "{tz}");
        }
        for path in ["plain", "huge", "absent", "zones"] {
            let tz = folder.0.join(path).to_string_lossy().into_owned();
            assert_eq!(held(Some(&format!(":{tz}"))), 0, "{path}");
        }
        let missing = find_zone(None, &dir, &folder.0.join("absent"));
        assert_eq!(missing.offset_minutes(NEW_YEAR * 1000), 0);
    }
}
