// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Input validation

use regex::Regex;
use ring_lang_rs::*;
use std::sync::LazyLock;

/// bolt_validate_email(str) → 0/1
ring_func!(bolt_validate_email, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$").unwrap()
    });
    ring_ret_number!(p, if RE.is_match(s) { 1.0 } else { 0.0 });
});

/// bolt_validate_url(str) → 0/1
ring_func!(bolt_validate_url, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());
    ring_ret_number!(p, if RE.is_match(s) { 1.0 } else { 0.0 });
});

/// bolt_validate_ip(str) → 0/1
ring_func!(bolt_validate_ip, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = s.parse::<std::net::IpAddr>().is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_ipv4(str) → 0/1
ring_func!(bolt_validate_ipv4, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = s.parse::<std::net::Ipv4Addr>().is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_ipv6(str) → 0/1
ring_func!(bolt_validate_ipv6, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = s.parse::<std::net::Ipv6Addr>().is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_uuid(str) → 0/1
ring_func!(bolt_validate_uuid, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
            .unwrap()
    });
    ring_ret_number!(p, if RE.is_match(s) { 1.0 } else { 0.0 });
});

/// bolt_validate_json_string(str) → 0/1
ring_func!(bolt_validate_json_string, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = serde_json::from_str::<serde_json::Value>(s).is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_length(str, min, max) → 0/1
ring_func!(bolt_validate_length, |p| {
    ring_check_paracount!(p, 3);
    ring_check_string!(p, 1);
    ring_check_number!(p, 2);
    ring_check_number!(p, 3);
    let s = ring_get_string!(p, 1);
    let min = ring_get_number!(p, 2) as usize;
    let max = ring_get_number!(p, 3) as usize;
    let len = s.len();
    ring_ret_number!(p, if len >= min && len <= max { 1.0 } else { 0.0 });
});

/// bolt_validate_range(num, min, max) → 0/1
ring_func!(bolt_validate_range, |p| {
    ring_check_paracount!(p, 3);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);
    ring_check_number!(p, 3);
    let num = ring_get_number!(p, 1);
    let min = ring_get_number!(p, 2);
    let max = ring_get_number!(p, 3);
    ring_ret_number!(p, if num >= min && num <= max { 1.0 } else { 0.0 });
});

/// bolt_validate_alpha(str) → 0/1
ring_func!(bolt_validate_alpha, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic());
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_alphanumeric(str) → 0/1
ring_func!(bolt_validate_alphanumeric, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric());
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_validate_numeric(str) → 0/1
ring_func!(bolt_validate_numeric, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let ok = !s.is_empty() && s.parse::<f64>().is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});
