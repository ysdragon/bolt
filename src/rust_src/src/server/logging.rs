// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Request Logging

use ring_lang_rs::*;

static LOGGING_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LOG_LEVEL: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(1);

fn log_level_num(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "debug" => 0,
        "info" => 1,
        "warn" | "warning" => 2,
        "error" => 3,
        _ => 1,
    }
}

/// bolt_logging(enabled) - enable/disable request logging
ring_func!(bolt_logging, |p| {
    ring_check_paracount_range!(p, 0, 1);

    let enabled = if ring_api_paracount(p) >= 1 && ring_api_isnumber(p, 1) {
        ring_get_number!(p, 1) != 0.0
    } else {
        true
    };

    LOGGING_ENABLED.store(enabled, std::sync::atomic::Ordering::SeqCst);
    ring_ret_number!(p, 1.0);
});

/// bolt_log(message, level?) - log a message with optional level (debug/info/warn/error)
ring_func!(bolt_log, |p| {
    ring_check_paracount_range!(p, 1, 2);
    ring_check_string!(p, 1);

    let message = ring_get_string!(p, 1);
    let level = if ring_api_paracount(p) >= 2 && ring_api_isstring(p, 2) {
        ring_get_string!(p, 2).to_string()
    } else {
        "info".to_string()
    };

    let level_num = log_level_num(&level);
    let min_level = LOG_LEVEL.load(std::sync::atomic::Ordering::SeqCst);

    if level_num >= min_level {
        let prefix = match level_num {
            0 => "DEBUG",
            1 => "INFO",
            2 => "WARN",
            3 => "ERROR",
            _ => "INFO",
        };

        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!("[{}] [{}] {}", timestamp, prefix, message);
    }
    ring_ret_number!(p, 1.0);
});

/// bolt_set_log_level(level) - set minimum log level (debug/info/warn/error)
ring_func!(bolt_set_log_level, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let level = ring_get_string!(p, 1);
    LOG_LEVEL.store(log_level_num(level), std::sync::atomic::Ordering::SeqCst);
    ring_ret_number!(p, 1.0);
});
