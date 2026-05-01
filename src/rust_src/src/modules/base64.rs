// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Base64 encode/decode

use base64::Engine;
use ring_lang_rs::*;

/// bolt_base64_encode(str) → string
ring_func!(bolt_base64_encode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let encoded = base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
    ring_ret_string!(p, &encoded);
});

/// bolt_base64_decode(str) → string
ring_func!(bolt_base64_decode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    match base64::engine::general_purpose::STANDARD.decode(s) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).to_string();
            ring_ret_string!(p, &decoded);
        }
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_base64_url_encode(str) → string (URL-safe)
ring_func!(bolt_base64_url_encode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    let encoded = base64::engine::general_purpose::URL_SAFE.encode(s.as_bytes());
    ring_ret_string!(p, &encoded);
});

/// bolt_base64_url_decode(str) → string
ring_func!(bolt_base64_url_decode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let s = ring_get_string!(p, 1);
    match base64::engine::general_purpose::URL_SAFE.decode(s) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).to_string();
            ring_ret_string!(p, &decoded);
        }
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});
