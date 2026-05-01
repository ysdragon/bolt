// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! HTML/XSS sanitization via ammonia

use ring_lang_rs::*;

/// bolt_sanitize_html(input) → string (strips dangerous tags, keeps safe ones)
ring_func!(bolt_sanitize_html, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let clean = ammonia::clean(input);
    ring_ret_string!(p, &clean);
});

/// bolt_sanitize_strict(input) → string (strips ALL HTML)
ring_func!(bolt_sanitize_strict, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let clean = ammonia::Builder::new()
        .tags(std::collections::HashSet::new())
        .clean(input)
        .to_string();
    ring_ret_string!(p, &clean);
});

/// bolt_escape_html(input) → string (escapes < > & " ')
ring_func!(bolt_escape_html, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let escaped = input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    ring_ret_string!(p, &escaped);
});
