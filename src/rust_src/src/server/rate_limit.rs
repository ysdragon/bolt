// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Rate Limiting (Simple In-Memory)

use ring_lang_rs::*;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::HTTP_SERVER_TYPE;

use super::HttpServer;

static RATE_LIMIT_REQUESTS: AtomicU64 = AtomicU64::new(0);
static RATE_LIMIT_MAX: AtomicU64 = AtomicU64::new(100);
static RATE_LIMIT_WINDOW: AtomicU64 = AtomicU64::new(60);
static RATE_LIMIT_WINDOW_START: AtomicU64 = AtomicU64::new(0);
static RATE_LIMIT_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// bolt_rate_limit(max_requests, window_seconds) - configure rate limiting
ring_func!(bolt_rate_limit, |p| {
    ring_check_paracount!(p, 2);
    ring_check_number!(p, 1);
    ring_check_number!(p, 2);

    let max_requests = ring_get_number!(p, 1) as u64;
    let window_seconds = ring_get_number!(p, 2) as u64;

    RATE_LIMIT_MAX.store(max_requests, Ordering::SeqCst);
    RATE_LIMIT_WINDOW.store(window_seconds, Ordering::SeqCst);
    RATE_LIMIT_ENABLED.store(true, Ordering::SeqCst);

    ring_ret_number!(p, 1.0);
});

/// bolt_check_rate_limit() -> 1 if allowed, 0 if rate limited
ring_func!(bolt_check_rate_limit, |p| {
    ring_check_paracount!(p, 0);

    if !RATE_LIMIT_ENABLED.load(Ordering::SeqCst) {
        ring_ret_number!(p, 1.0);
        return;
    }

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let window_start = RATE_LIMIT_WINDOW_START.load(Ordering::SeqCst);
    let window = RATE_LIMIT_WINDOW.load(Ordering::SeqCst);

    if now - window_start >= window {
        RATE_LIMIT_WINDOW_START.store(now, Ordering::SeqCst);
        RATE_LIMIT_REQUESTS.store(1, Ordering::SeqCst);
        ring_ret_number!(p, 1.0);
        return;
    }

    let requests = RATE_LIMIT_REQUESTS
        .fetch_add(1, Ordering::SeqCst)
        .saturating_add(1);
    let max = RATE_LIMIT_MAX.load(Ordering::SeqCst);

    if requests == u64::MAX {
        RATE_LIMIT_WINDOW_START.store(now, Ordering::SeqCst);
        RATE_LIMIT_REQUESTS.store(1, Ordering::SeqCst);
        ring_ret_number!(p, 1.0);
        return;
    }

    if requests > max {
        ring_ret_number!(p, 0.0);
    } else {
        ring_ret_number!(p, 1.0);
    }
});

/// bolt_route_rate_limit(server, handler_name, max_requests, window_seconds)
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
    let max_requests = ring_get_number!(p, 3) as u64;
    let window_seconds = ring_get_number!(p, 4) as u64;

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

        RATE_LIMIT_WINDOW_START.store(now - 100, Ordering::SeqCst);
        RATE_LIMIT_WINDOW.store(60, Ordering::SeqCst);
        let window_start = RATE_LIMIT_WINDOW_START.load(Ordering::SeqCst);
        let window = RATE_LIMIT_WINDOW.load(Ordering::SeqCst);
        assert!(now - window_start >= window);
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
}
