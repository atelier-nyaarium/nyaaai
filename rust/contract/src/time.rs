//! Millisecond and ISO times.

const DAY_MS: i64 = 86_400_000;

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

/// Parses ISO times with millisecond precision.
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

/// ECMAScript Date time limit.
const LAST_MS: i64 = 8_640_000_000_000_000;

/// Formats valid ECMAScript Date times.
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
