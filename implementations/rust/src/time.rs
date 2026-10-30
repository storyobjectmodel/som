//! The small amount of RFC 3339 handling SOM needs: stamping an envelope's
//! `timestamp`, and ordering two `date-time` values by the instant they name
//! rather than by their spelling.
//!
//! This is deliberately not a date library. It parses and formats the one
//! profile the schemas use (`format: date-time`, RFC 3339 §5.6) and nothing
//! else.

use std::time::{SystemTime, UNIX_EPOCH};

/// An RFC 3339 `date-time` resolved to the instant it names, UTC, to the
/// nanosecond. Two spellings of one instant compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    /// Seconds since 1970-01-01T00:00:00Z.
    pub unix_seconds: i64,
    /// Nanoseconds within the second.
    pub nanos: u32,
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let year_of_era = y - era * 400;
    let shifted_month = (i64::from(month) + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The civil date of a day count since 1970-01-01. The month and day are
/// bounded by construction (1..=12 and 1..=31), so the narrowing cannot fail.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = u32::try_from(day_of_year - (153 * shifted_month + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    })
    .unwrap_or(1);
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        _ => 28,
    }
}

fn digits(s: &[u8]) -> Option<u32> {
    if s.is_empty() || !s.iter().all(u8::is_ascii_digit) {
        return None;
    }
    s.iter().try_fold(0u32, |acc, b| {
        acc.checked_mul(10)?.checked_add(u32::from(b - b'0'))
    })
}

/// Parses an RFC 3339 `date-time`, such as `2026-09-11T16:41:00.5-04:00`.
/// Returns `None` for anything else. A leap second (`:60`) is read as the
/// first instant of the following second; fractional digits beyond the ninth
/// are ignored.
#[must_use]
pub fn parse_rfc3339(value: &str) -> Option<DateTime> {
    let b = value.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || !matches!(b[10], b'T' | b't')
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let year = i64::from(digits(&b[0..4])?);
    let month = digits(&b[5..7])?;
    let day = digits(&b[8..10])?;
    let hour = digits(&b[11..13])?;
    let minute = digits(&b[14..16])?;
    let second = digits(&b[17..19])?;
    if !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut rest = &b[19..];
    let mut nanos = 0u32;
    if let Some(frac) = rest.strip_prefix(b".") {
        let len = frac.iter().take_while(|c| c.is_ascii_digit()).count();
        if len == 0 {
            return None;
        }
        let mut scale = 100_000_000u32;
        for d in &frac[..len.min(9)] {
            nanos += u32::from(d - b'0') * scale;
            scale /= 10;
        }
        rest = &frac[len..];
    }

    let offset_seconds: i64 = match rest {
        [b'Z' | b'z'] => 0,
        [sign @ (b'+' | b'-'), h1, h2, b':', m1, m2] => {
            let oh = digits(&[*h1, *h2])?;
            let om = digits(&[*m1, *m2])?;
            if oh > 23 || om > 59 {
                return None;
            }
            let magnitude = i64::from(oh * 3600 + om * 60);
            if *sign == b'+' { magnitude } else { -magnitude }
        }
        _ => return None,
    };

    let local =
        days_from_civil(year, month, day) * 86_400 + i64::from(hour * 3600 + minute * 60 + second);
    Some(DateTime {
        unix_seconds: local - offset_seconds,
        nanos,
    })
}

/// Formats `time` as an RFC 3339 UTC `date-time` with microsecond precision,
/// which the envelope schema recommends for `timestamp`. A time before the
/// Unix epoch formats as the epoch.
#[must_use]
pub fn format_rfc3339_micros(time: SystemTime) -> String {
    let since = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = i64::try_from(since.as_secs()).unwrap_or(i64::MAX);
    let (year, month, day) = civil_from_days(secs.div_euclid(86_400));
    let in_day = secs.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:06}Z",
        in_day / 3600,
        (in_day % 3600) / 60,
        in_day % 60,
        since.subsec_micros()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn offsets_and_fractions_compare_by_instant() {
        let z = parse_rfc3339("2026-09-11T16:41:00Z").unwrap();
        let half = parse_rfc3339("2026-09-11T16:41:00.5Z").unwrap();
        let ny = parse_rfc3339("2026-09-11T12:41:00-04:00").unwrap();
        assert!(half > z);
        assert_eq!(ny, z);
        assert_eq!(z.unix_seconds, 1_789_144_860);
    }

    #[test]
    fn fractions_are_read_to_the_nanosecond_and_no_further() {
        let nine = parse_rfc3339("2026-09-11T16:41:00.123456789Z").unwrap();
        let twelve = parse_rfc3339("2026-09-11T16:41:00.123456789999Z").unwrap();
        assert_eq!(nine.nanos, 123_456_789);
        assert_eq!(nine, twelve);
        assert_eq!(
            parse_rfc3339("2026-09-11T16:41:00.1Z").unwrap().nanos,
            100_000_000
        );
    }

    #[test]
    fn refuses_what_is_not_a_date_time() {
        for bad in [
            "11 September 2026, 16:41",
            "2026-02-30T00:00:00Z",
            "2026-09-11T24:00:00Z",
            "2026-09-11T16:41:00",
            "2026-09-11T16:41:00.Z",
            "2026-09-11T16:41:00+0400",
            "2026-09-11T16:41:00+24:00",
            "2026-13-01T00:00:00Z",
        ] {
            assert!(parse_rfc3339(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn leap_days_and_leap_seconds() {
        assert!(parse_rfc3339("2024-02-29T00:00:00Z").is_some());
        assert!(parse_rfc3339("2100-02-29T00:00:00Z").is_none());
        assert!(parse_rfc3339("2000-02-29T00:00:00Z").is_some());
        let leap = parse_rfc3339("2016-12-31T23:59:60Z").unwrap();
        let next = parse_rfc3339("2017-01-01T00:00:00Z").unwrap();
        assert_eq!(leap, next);
    }

    #[test]
    fn formats_and_parses_back() {
        let t = UNIX_EPOCH + Duration::new(1_789_144_860, 123_456_789);
        let s = format_rfc3339_micros(t);
        assert_eq!(s, "2026-09-11T16:41:00.123456Z");
        assert_eq!(
            parse_rfc3339(&s),
            Some(DateTime {
                unix_seconds: 1_789_144_860,
                nanos: 123_456_000
            })
        );
        assert_eq!(
            format_rfc3339_micros(UNIX_EPOCH),
            "1970-01-01T00:00:00.000000Z"
        );
    }
}
