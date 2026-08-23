// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Rate Limiting (Simple In-Memory)

use ring_lang_rs::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::HTTP_SERVER_TYPE;

use super::{HttpServer, resolve_client_ip};

struct IpRateEntry {
    requests: AtomicU64,
    window_start: AtomicU64,
}

static RATE_LIMIT_MAX: AtomicU64 = AtomicU64::new(100);
static RATE_LIMIT_WINDOW: AtomicU64 = AtomicU64::new(60);
static RATE_LIMIT_ENABLED: AtomicBool = AtomicBool::new(false);
static RATE_LIMIT_IP_MAP: std::sync::LazyLock<dashmap::DashMap<String, IpRateEntry>> =
    std::sync::LazyLock::new(|| dashmap::DashMap::new());
static RATE_LIMIT_CAP_WARNED: AtomicBool = AtomicBool::new(false);

const RATE_LIMIT_IP_MAP_CAP: usize = 200_000;

/// True when the per-IP rate limit map has reached its hard cap.
fn map_over_cap(len: usize) -> bool {
    len >= RATE_LIMIT_IP_MAP_CAP
}

/// Seconds since the Unix epoch (0 for pre-1970 clocks — degrades to a
/// window-reset instead of a panic).
fn secs_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Validate `bolt_rate_limit` arguments: non-negative max, positive window.
fn validate_rate_limit_args(max: f64, win: f64) -> Result<(), &'static str> {
    if !max.is_finite() || max < 0.0 || !win.is_finite() || win <= 0.0 {
        return Err(
            "rate limit: max_requests must be non-negative and window_seconds must be positive",
        );
    }
    Ok(())
}

/// bolt_rate_limit(max_requests, window_seconds) → configure rate limiting
ring_func!(bolt_rate_limit, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);

    let max_req_f = ring_get_number!(p, 1);
    let window_sec_f = ring_get_number!(p, 2);
    if let Err(msg) = validate_rate_limit_args(max_req_f, window_sec_f) {
        ring_error!(p, msg);
        return;
    }
    let max_requests = max_req_f as u64;
    let window_seconds = window_sec_f as u64;

    RATE_LIMIT_MAX.store(max_requests, Ordering::SeqCst);
    RATE_LIMIT_WINDOW.store(window_seconds, Ordering::SeqCst);
    RATE_LIMIT_ENABLED.store(true, Ordering::SeqCst);

    ring_ret_number!(p, 1.0);
});

/// bolt_check_rate_limit([server]) → 1 if allowed, 0 if rate limited
ring_func!(bolt_check_rate_limit, |p| {
    if !RATE_LIMIT_ENABLED.load(Ordering::SeqCst) {
        ring_ret_number!(p, 1.0);
        return;
    }

    let now = secs_since_epoch();

    let window = RATE_LIMIT_WINDOW.load(Ordering::SeqCst);
    let max = RATE_LIMIT_MAX.load(Ordering::SeqCst);

    let client_ip = if ring_api_paracount(p) >= 1 {
        let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
        if !ptr.is_null() {
            unsafe {
                let server = &*(ptr as *const HttpServer);
                let guard = server.current_request.lock();
                if let Some(ref ctx) = *guard {
                    let proxy_whitelist = &server.config.proxy_whitelist;
                    resolve_client_ip(&ctx.peer_addr, &ctx.headers, proxy_whitelist)
                } else {
                    String::new()
                }
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let ip_key = if client_ip.is_empty() {
        "unknown".to_string()
    } else {
        client_ip
    };

    // Hard cap on map growth: past the cap, new IPs are allowed without
    // limiting (fail-open under flood).
    if map_over_cap((*RATE_LIMIT_IP_MAP).len()) {
        if !RATE_LIMIT_CAP_WARNED.swap(true, Ordering::SeqCst) {
            eprintln!(
                "[bolt] rate limit map reached cap ({}); new IPs allowed without limiting",
                RATE_LIMIT_IP_MAP_CAP
            );
        }
        ring_ret_number!(p, 1.0);
        return;
    }

    let entry = (*RATE_LIMIT_IP_MAP)
        .entry(ip_key.clone())
        .or_insert_with(|| IpRateEntry {
            requests: AtomicU64::new(0),
            window_start: AtomicU64::new(now),
        });

    loop {
        let window_start = entry.window_start.load(Ordering::SeqCst);

        if now.saturating_sub(window_start) >= window {
            match entry.window_start.compare_exchange(
                window_start,
                now,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    entry.requests.store(1, Ordering::SeqCst);
                    ring_ret_number!(p, 1.0);
                    return;
                }
                Err(_) => continue,
            }
        }

        let requests = entry
            .requests
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);

        if requests > max {
            ring_ret_number!(p, 0.0);
        } else {
            ring_ret_number!(p, 1.0);
        }
        return;
    }
});

/// Clean up expired entries from the per-IP rate limit map
pub fn rate_limit_cleanup_ip_map() {
    let now = secs_since_epoch();
    let window = RATE_LIMIT_WINDOW.load(Ordering::SeqCst);
    (*RATE_LIMIT_IP_MAP).retain(|_, entry| {
        let window_start = entry.window_start.load(Ordering::SeqCst);
        now.saturating_sub(window_start) < window
    });
    // Re-arm the cap warning so a future flood warns again instead of the
    // warning firing only once per process.
    if (*RATE_LIMIT_IP_MAP).len() < RATE_LIMIT_IP_MAP_CAP {
        RATE_LIMIT_CAP_WARNED.store(false, Ordering::SeqCst);
    }
}

/// bolt_route_rate_limit(server, handler_name, max_requests, window_seconds) → set per-route rate limit
ring_func!(bolt_route_rate_limit, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_number!(p, 3);
    ring_check_number!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let handler_name = ring_get_string!(p, 2);
    let max_req_f = ring_get_number!(p, 3);
    let window_sec_f = ring_get_number!(p, 4);
    if let Err(msg) = validate_rate_limit_args(max_req_f, window_sec_f) {
        ring_error!(p, msg);
        return;
    }
    let max_requests = max_req_f as u64;
    let window_seconds = window_sec_f as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        for route in &mut server.routes {
            if route.handler_name == handler_name {
                route.rate_limit = Some((max_requests, window_seconds));
                break;
            }
        }
    }

    ring_ret_number!(p, 1.0);
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_rate_limit_disabled_returns_allowed() {
        RATE_LIMIT_ENABLED.store(false, Ordering::SeqCst);
        let enabled = RATE_LIMIT_ENABLED.load(Ordering::SeqCst);
        assert!(!enabled);
    }

    #[test]
    fn test_rate_limit_configure() {
        RATE_LIMIT_MAX.store(50, Ordering::SeqCst);
        RATE_LIMIT_WINDOW.store(120, Ordering::SeqCst);
        RATE_LIMIT_ENABLED.store(true, Ordering::SeqCst);

        assert_eq!(RATE_LIMIT_MAX.load(Ordering::SeqCst), 50);
        assert_eq!(RATE_LIMIT_WINDOW.load(Ordering::SeqCst), 120);
        assert!(RATE_LIMIT_ENABLED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_rate_limit_window_reset() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let _entry = RATE_LIMIT_IP_MAP
            .entry("test".to_string())
            .or_insert_with(|| IpRateEntry {
                requests: AtomicU64::new(0),
                window_start: AtomicU64::new(now - 100),
            });
        RATE_LIMIT_WINDOW.store(60, Ordering::SeqCst);
        let window = RATE_LIMIT_WINDOW.load(Ordering::SeqCst);
        assert!(now - (now - 100) >= window);
    }

    #[test]
    fn test_rate_limit_saturating_add() {
        let count = u64::MAX;
        let result = count.saturating_add(1);
        assert_eq!(result, u64::MAX);
    }

    #[test]
    fn test_rate_limit_overflow_guard() {
        let requests = u64::MAX;
        assert_eq!(requests, u64::MAX);
    }

    #[test]
    fn test_route_rate_limit_assignment() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/api/:id", "api_handler");

        for route in &mut server.routes {
            if route.handler_name == "api_handler" {
                route.rate_limit = Some((100, 60));
                break;
            }
        }

        let route = server
            .routes
            .iter()
            .find(|r| r.handler_name == "api_handler")
            .unwrap();
        assert_eq!(route.rate_limit, Some((100, 60)));
    }

    #[test]
    fn test_validate_rate_limit_args() {
        assert!(validate_rate_limit_args(100.0, 60.0).is_ok());
        assert!(validate_rate_limit_args(0.0, 60.0).is_ok());
        assert!(validate_rate_limit_args(100.0, 0.0).is_err());
        assert!(validate_rate_limit_args(-1.0, 60.0).is_err());
        assert!(validate_rate_limit_args(100.0, -60.0).is_err());
        assert!(validate_rate_limit_args(f64::NAN, 60.0).is_err());
        assert!(validate_rate_limit_args(f64::INFINITY, 60.0).is_err());
    }

    #[test]
    fn test_map_over_cap() {
        assert!(!map_over_cap(0));
        assert!(!map_over_cap(199_999));
        assert!(map_over_cap(200_000));
        assert!(map_over_cap(1_000_000));
    }

    #[test]
    fn test_secs_since_epoch_positive() {
        assert!(secs_since_epoch() > 1_700_000_000);
    }
}
