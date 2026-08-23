// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Request Logging

use ring_lang_rs::*;

static LOG_INIT: std::sync::Once = std::sync::Once::new();
static LOGGING_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LOG_LEVEL: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new({
    // Initialize from BOLT_LOG_LEVEL env var if set
    // Can't use match in const, so default to 1 (info)
    1
});

fn log_level_num(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "debug" => 0,
        "info" => 1,
        "warn" | "warning" => 2,
        "error" => 3,
        _ => 1,
    }
}

/// Redact common PII patterns from a log message
fn redact_pii(msg: &str) -> String {
    // Redact email addresses
    static EMAIL_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").expect("valid regex")
    });
    // Redact common token/key patterns (Bearer tokens, API keys, etc.)
    //
    // Consumes the keyword, its separator, any chained keywords/auth schemes
    // ("Bearer eyJ...", "bearer token: x"), and exactly ONE following token.
    // Everything after the secret is preserved — do not replace `\S+` with a
    // multi-token quantifier like `(\S+\s*)*`, which erases the rest of the
    // log line (and subsequent lines) whenever a message merely contains the
    // word "token"/"password"/etc.
    static TOKEN_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"(?i)\b(bearer|api[_-]?key|token|secret|password|authorization)\b(?:[:=]\s*|\s+)(?:(?:bearer|basic|digest|negotiate|api[_-]?key|token|secret|password|authorization)\b(?:[:=]\s*|\s+))*\S+",
        )
        .expect("valid regex")
    });
    let result = EMAIL_RE.replace_all(msg, "[REDACTED_EMAIL]");
    let result = TOKEN_RE.replace_all(&result, "${1}: [REDACTED]");
    result.into_owned()
}

/// bolt_logging(enabled) → enable/disable request logging
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

/// bolt_log(message, level?) → log a message with optional level (debug/info/warn/error)
ring_func!(bolt_log, |p| {
    ring_check_paracount_range!(p, 1, 2);
    ring_check_string!(p, 1);

    LOG_INIT.call_once(|| {
        if let Ok(level) = std::env::var("BOLT_LOG_LEVEL") {
            LOG_LEVEL.store(log_level_num(&level), std::sync::atomic::Ordering::SeqCst);
        }
    });

    if !LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
        ring_ret_number!(p, 1.0);
        return;
    }

    let message = ring_get_string!(p, 1);
    let level = if ring_api_paracount(p) >= 2 && ring_api_isstring(p, 2) {
        ring_get_string!(p, 2).to_string()
    } else {
        "info".to_string()
    };

    let level_num = log_level_num(&level);
    let min_level = LOG_LEVEL.load(std::sync::atomic::Ordering::SeqCst);

    let message = redact_pii(&message);
    let message: String = message
        .chars()
        .filter(|c| *c != '\r' && *c != '\n' && *c != '\x1b' && *c != '\0')
        .collect();

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

        // VM thread: use stderr to avoid blocking on stdout buffer flush
        eprintln!("[{}] [{}] {}", timestamp, prefix, message);
    }
    ring_ret_number!(p, 1.0);
});

/// bolt_set_log_level(level) → set minimum log level (debug/info/warn/error)
ring_func!(bolt_set_log_level, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let level = ring_get_string!(p, 1);
    LOG_LEVEL.store(log_level_num(level), std::sync::atomic::Ordering::SeqCst);
    ring_ret_number!(p, 1.0);
});

#[cfg(test)]
mod tests {
    use super::redact_pii;

    #[test]
    fn test_redact_bearer_header() {
        let out = redact_pii("Authorization: Bearer eyJ.abc");
        assert!(!out.contains("eyJ"), "got: {out}");
        assert!(!out.contains("abc"), "got: {out}");
    }

    #[test]
    fn test_redact_api_key_equals() {
        let out = redact_pii("api_key=xyz");
        assert!(!out.contains("xyz"), "got: {out}");
    }

    #[test]
    fn test_redact_token_colon() {
        let out = redact_pii("token: t0k");
        assert!(!out.contains("t0k"), "got: {out}");
    }

    #[test]
    fn test_redact_bearer_then_token() {
        let out = redact_pii("bearer token: B");
        assert!(!out.contains("B"), "got: {out}");
    }

    #[test]
    fn test_redact_password_and_secret() {
        // Two independent key/value pairs in one line: each secret follows
        // its own keyword, so both are redacted.
        let out = redact_pii("password=supersecret and secret: hidden");
        assert!(!out.contains("supersecret"), "got: {out}");
        assert!(!out.contains("hidden"), "got: {out}");
    }

    #[test]
    fn test_redaction_preserves_surrounding_log_data() {
        // Only the single token after a keyword may be redacted; a message
        // that merely CONTAINS the word "token" keeps its remaining content.
        let out = redact_pii("Invalid token provided for user alice from 10.0.0.5");
        assert!(out.contains("for user alice"), "got: {out}");
        assert!(out.contains("10.0.0.5"), "got: {out}");

        let out = redact_pii("GET /api?page=1 for user bob");
        assert_eq!(out, "GET /api?page=1 for user bob");
    }

    #[test]
    fn test_redact_basic_auth_scheme() {
        let out = redact_pii("Authorization: Basic dXNlcjpwYXNz");
        assert!(!out.contains("dXNlcjpwYXNz"), "got: {out}");
    }

    #[test]
    fn test_keyword_alone_not_redacted() {
        // Nothing follows the keyword, so there is nothing to consume.
        assert_eq!(redact_pii("secret"), "secret");
        assert_eq!(redact_pii("token rotation done"), "token: [REDACTED] done");
    }

    #[test]
    fn test_no_false_positive_monkey() {
        assert_eq!(redact_pii("monkey"), "monkey");
    }
}
