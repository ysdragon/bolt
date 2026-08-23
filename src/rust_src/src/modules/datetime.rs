// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Date/Time utilities via chrono

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use ring_lang_rs::*;

/// True when every item in `fmt` is a valid strftime directive.
fn valid_strftime(fmt: &str) -> bool {
    !chrono::format::strftime::StrftimeItems::new(fmt)
        .any(|item| matches!(item, chrono::format::Item::Error))
}

/// Add `days` to `ts`, returning `None` on out-of-range input or overflow.
fn add_days(ts: i64, days: i64) -> Option<i64> {
    let dt = DateTime::from_timestamp(ts, 0)?;
    let delta = chrono::TimeDelta::try_days(days)?;
    dt.checked_add_signed(delta).map(|d| d.timestamp())
}

/// Add `hours` to `ts`, returning `None` on out-of-range input or overflow.
fn add_hours(ts: i64, hours: i64) -> Option<i64> {
    let dt = DateTime::from_timestamp(ts, 0)?;
    let delta = chrono::TimeDelta::try_hours(hours)?;
    dt.checked_add_signed(delta).map(|d| d.timestamp())
}

/// True when `fmt` contains a timezone-offset directive (`%z`, `%:z`,
/// `%::z`, `%:::z`, `%#z`).
///
/// Detected by scanning the parsed format items rather than by substring
/// search, so escaped sequences like `%%z` (literal "%" followed by "z")
/// are not mistaken for an offset directive. `%#z` maps to chrono's private
/// `TimezoneOffsetPermissive` internal item, which cannot be name-matched,
/// so it is detected textually (also honoring `%%` escapes).
fn fmt_has_offset_directive(fmt: &str) -> bool {
    use chrono::format::{Fixed, Item};
    if chrono::format::strftime::StrftimeItems::new(fmt).any(|item| {
        matches!(
            item,
            Item::Fixed(
                Fixed::TimezoneOffset
                    | Fixed::TimezoneOffsetColon
                    | Fixed::TimezoneOffsetDoubleColon
                    | Fixed::TimezoneOffsetTripleColon
            )
        )
    }) {
        return true;
    }
    contains_permissive_offset_directive(fmt)
}

/// True when `fmt` contains a real (non-`%%`-escaped) `%#z` directive.
fn contains_permissive_offset_directive(fmt: &str) -> bool {
    let b = fmt.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'%' {
            i += 1;
            continue;
        }
        // "%%" is an escaped percent sign — skip the pair.
        if i + 1 < b.len() && b[i + 1] == b'%' {
            i += 2;
            continue;
        }
        // A directive starts at this '%': an optional flag character, then
        // the specifier. Only `%#z` uses chrono's permissive-offset item.
        let flag = b.get(i + 1).copied();
        let spec = b.get(i + 2).copied();
        if flag == Some(b'#') && spec == Some(b'z') {
            return true;
        }
        i += 1;
    }
    false
}

/// Parse `date_str` with `fmt`, returning the UTC timestamp.
///
/// When `fmt` contains a timezone-offset directive the input is parsed as
/// offset-aware and normalized to UTC; otherwise it is parsed as a naive
/// datetime and assumed to be UTC.
fn parse_timestamp(date_str: &str, fmt: &str) -> Option<i64> {
    if fmt_has_offset_directive(fmt) {
        DateTime::parse_from_str(date_str, fmt)
            .ok()
            .map(|dt| dt.timestamp())
    } else {
        NaiveDateTime::parse_from_str(date_str, fmt)
            .ok()
            .map(|ndt| ndt.and_utc().timestamp())
    }
}

/// bolt_datetime_now() → string (ISO 8601 local)
ring_func!(bolt_datetime_now, |p| {
    ring_check_paracount!(p, 0);
    let now = Local::now().to_rfc3339();
    ring_ret_string!(p, &now);
});

/// bolt_datetime_now_utc() → string (ISO 8601 UTC)
ring_func!(bolt_datetime_now_utc, |p| {
    ring_check_paracount!(p, 0);
    let now = Utc::now().to_rfc3339();
    ring_ret_string!(p, &now);
});

/// bolt_datetime_timestamp() → number (unix seconds)
ring_func!(bolt_datetime_timestamp, |p| {
    ring_check_paracount!(p, 0);
    let ts = Utc::now().timestamp() as f64;
    ring_ret_number!(p, ts);
});

/// bolt_datetime_timestamp_ms() → number (unix millis)
ring_func!(bolt_datetime_timestamp_ms, |p| {
    ring_check_paracount!(p, 0);
    let ts = Utc::now().timestamp_millis() as f64;
    ring_ret_number!(p, ts);
});

/// bolt_datetime_format(timestamp, format_str) → string (0 on invalid timestamp or format)
ring_func!(bolt_datetime_format, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_string!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let fmt = ring_get_string!(p, 2);
    if !valid_strftime(fmt) {
        ring_ret_number!(p, 0.0);
        return;
    }
    match DateTime::from_timestamp(ts, 0) {
        Some(dt) => {
            let formatted = dt.format(fmt).to_string();
            ring_ret_string!(p, &formatted);
        }
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_datetime_parse(date_str, format_str) → number (timestamp, 0 on failure)
ring_func!(bolt_datetime_parse, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let date_str = ring_get_string!(p, 1);
    let fmt = ring_get_string!(p, 2);
    match parse_timestamp(date_str, fmt) {
        Some(ts) => ring_ret_number!(p, ts as f64),
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_datetime_diff(ts1, ts2) → number (seconds between, ts1 - ts2)
ring_func!(bolt_datetime_diff, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    let ts1 = ring_get_number!(p, 1) as i64;
    let ts2 = ring_get_number!(p, 2) as i64;
    let diff = (ts1 - ts2) as f64;
    ring_ret_number!(p, diff);
});

/// bolt_datetime_add_days(timestamp, days) → number (0 on invalid input or overflow)
ring_func!(bolt_datetime_add_days, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let days = ring_get_number!(p, 2) as i64;
    match add_days(ts, days) {
        Some(new_ts) => ring_ret_number!(p, new_ts as f64),
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_datetime_add_hours(timestamp, hours) → number (0 on invalid input or overflow)
ring_func!(bolt_datetime_add_hours, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let hours = ring_get_number!(p, 2) as i64;
    match add_hours(ts, hours) {
        Some(new_ts) => ring_ret_number!(p, new_ts as f64),
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_strftime() {
        assert!(valid_strftime("%Y-%m-%d"));
        assert!(valid_strftime("%Y-%m-%d %H:%M:%S"));
        assert!(!valid_strftime("%Q"));
        assert!(!valid_strftime("%Y %"));
        assert!(!valid_strftime("%"));
    }

    #[test]
    fn test_add_days_normal() {
        assert_eq!(add_days(86_400, 7), Some(86_400 + 7 * 86_400));
        assert_eq!(add_days(86_400, -1), Some(0));
    }

    #[test]
    fn test_add_days_extreme_returns_none() {
        assert_eq!(add_days(86_400, i64::MAX / 2), None);
        assert_eq!(add_days(86_400, 1_000_000_000_000_000), None);
        assert_eq!(add_days(i64::MAX, 1), None);
    }

    #[test]
    fn test_add_hours_normal_and_extreme() {
        assert_eq!(add_hours(86_400, 48), Some(86_400 + 48 * 3_600));
        assert_eq!(add_hours(86_400, i64::MAX), None);
        assert_eq!(add_hours(86_400, -48), Some(-86_400));
    }

    #[test]
    fn test_parse_naive_without_offset() {
        assert_eq!(
            parse_timestamp("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S"),
            Some(1_767_225_600 + 12 * 3_600)
        );
    }

    #[test]
    fn test_parse_with_offset_normalizes_to_utc() {
        assert_eq!(
            parse_timestamp("2026-01-01 12:00:00 +0530", "%Y-%m-%d %H:%M:%S %z"),
            Some(1_767_249_000)
        );
    }

    #[test]
    fn test_parse_escaped_percent_z_is_naive_not_offset() {
        // "%%z" is an escaped percent sign followed by a literal 'z' — NOT an
        // offset directive. A substring check would misroute this format to
        // offset-aware parsing and fail.
        assert!(fmt_has_offset_directive("%z"));
        assert!(fmt_has_offset_directive("%:z"));
        assert!(fmt_has_offset_directive("%::z"));
        // %#z maps to chrono's private TimezoneOffsetPermissive item.
        assert!(fmt_has_offset_directive("%#z"));
        assert!(!fmt_has_offset_directive("%%z"));
        assert!(!fmt_has_offset_directive("%%#z"));
        assert!(!fmt_has_offset_directive("%Y-%m-%d"));
        // `%%z` is parsed literally (not as an offset), so the input must
        // contain the literal characters "%z". NaiveDateTime parsing requires
        // time fields, so the format carries %H:%M:%S as well.
        assert_eq!(
            parse_timestamp("2026-01-01 12:00:00 %z", "%Y-%m-%d %H:%M:%S %%z"),
            Some(1_767_225_600 + 12 * 3_600)
        );
    }

    #[test]
    fn test_parse_permissive_offset_directive() {
        // %#z accepts offsets with or without colons/minutes.
        assert_eq!(
            parse_timestamp("2026-01-01 12:00:00 +05:30", "%Y-%m-%d %H:%M:%S%#z"),
            Some(1_767_249_000)
        );
        assert_eq!(
            parse_timestamp("2026-01-01 12:00:00 +0530", "%Y-%m-%d %H:%M:%S%#z"),
            Some(1_767_249_000)
        );
    }

    #[test]
    fn test_parse_invalid_returns_none() {
        assert_eq!(parse_timestamp("not a date", "%Y-%m-%d"), None);
    }
}
