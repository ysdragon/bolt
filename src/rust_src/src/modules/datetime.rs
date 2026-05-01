// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Date/Time utilities via chrono

use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use ring_lang_rs::*;

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

/// bolt_datetime_format(timestamp, format_str) → string
ring_func!(bolt_datetime_format, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_string!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let fmt = ring_get_string!(p, 2);
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

/// bolt_datetime_parse(date_str, format_str) → number (timestamp)
ring_func!(bolt_datetime_parse, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let date_str = ring_get_string!(p, 1);
    let fmt = ring_get_string!(p, 2);
    match NaiveDateTime::parse_from_str(date_str, fmt) {
        Ok(ndt) => {
            let ts = ndt.and_utc().timestamp() as f64;
            ring_ret_number!(p, ts);
        }
        Err(_) => {
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

/// bolt_datetime_add_days(timestamp, days) → number
ring_func!(bolt_datetime_add_days, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let days = ring_get_number!(p, 2) as i64;
    match DateTime::from_timestamp(ts, 0) {
        Some(dt) => {
            let new_dt = dt + Duration::days(days);
            ring_ret_number!(p, new_dt.timestamp() as f64);
        }
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_datetime_add_hours(timestamp, hours) → number
ring_func!(bolt_datetime_add_hours, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    let ts = ring_get_number!(p, 1) as i64;
    let hours = ring_get_number!(p, 2) as i64;
    match DateTime::from_timestamp(ts, 0) {
        Some(dt) => {
            let new_dt = dt + Duration::hours(hours);
            ring_ret_number!(p, new_dt.timestamp() as f64);
        }
        None => {
            ring_ret_number!(p, 0.0);
        }
    }
});
