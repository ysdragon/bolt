// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Caching Functions

use ring_lang_rs::*;

use crate::HTTP_SERVER_TYPE;

use super::HttpServer;

/// bolt_cache_set(server, key, value, ttl_seconds) - cache a value
ring_func!(bolt_cache_set, |p| {
    ring_check_paracount_range!(p, 3, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let key = ring_get_string!(p, 2);
    let value = ring_get_string!(p, 3);
    let ttl_seconds = if ring_api_paracount(p) >= 4 && ring_api_isnumber(p, 4) {
        ring_get_number!(p, 4) as u64
    } else {
        0
    };

    unsafe {
        let server = &*(ptr as *const HttpServer);
        server.cache.insert(key.to_string(), value.to_string());
        let mut expiry = server.cache_expiry.lock();
        if ttl_seconds > 0 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            expiry.insert(key.to_string(), now + ttl_seconds);
        } else {
            expiry.remove(key);
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_cache_get(server, key) -> cached value or empty string
ring_func!(bolt_cache_get, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let key = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        {
            let expiry = server.cache_expiry.lock();
            if let Some(expires_at) = expiry.get(key).cloned() {
                if now >= expires_at {
                    drop(expiry);
                    let mut expiry = server.cache_expiry.lock();
                    if expiry.get(key).map(|&t| now >= t).unwrap_or(false) {
                        expiry.remove(key);
                        server.cache.invalidate(&key.to_string());
                    }
                    ring_ret_string!(p, "");
                    return;
                }
            }
        }

        match server.cache.get(&key.to_string()) {
            Some(value) => ring_ret_string!(p, &value),
            None => {
                ring_ret_string!(p, "");
            }
        }
    }
});

/// bolt_cache_delete(server, key) - remove from cache
ring_func!(bolt_cache_delete, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let key = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        server.cache.invalidate(&key.to_string());
        let mut expiry = server.cache_expiry.lock();
        expiry.remove(key);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_cache_clear(server) - clear all cache
ring_func!(bolt_cache_clear, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        server.cache.invalidate_all();
        let mut expiry = server.cache_expiry.lock();
        expiry.clear();
    }

    ring_ret_number!(p, 1.0);
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_cache_set_and_get() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("key1".into(), "value1".into());
        assert_eq!(server.cache.get("key1").unwrap(), "value1");
    }

    #[test]
    fn test_cache_get_missing() {
        let server = HttpServer::new(std::ptr::null_mut());
        assert!(server.cache.get("missing").is_none());
    }

    #[test]
    fn test_cache_delete() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("key1".into(), "value1".into());
        server.cache.invalidate("key1");
        assert!(server.cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("a".into(), "1".into());
        server.cache.insert("b".into(), "2".into());
        server.cache.invalidate_all();
        assert!(server.cache.get("a").is_none());
        assert!(server.cache.get("b").is_none());
    }

    #[test]
    fn test_cache_expiry_set_and_get() {
        let server = HttpServer::new(std::ptr::null_mut());
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        server.cache.insert("key1".into(), "value1".into());
        server.cache_expiry.lock().insert("key1".into(), now + 3600);

        let expiry = server.cache_expiry.lock();
        assert_eq!(expiry.get("key1").cloned(), Some(now + 3600));
    }

    #[test]
    fn test_cache_expiry_check_expired() {
        let server = HttpServer::new(std::ptr::null_mut());
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        server.cache.insert("expired_key".into(), "value".into());
        server
            .cache_expiry
            .lock()
            .insert("expired_key".into(), now - 1);

        let expiry = server.cache_expiry.lock();
        if let Some(expires_at) = expiry.get("expired_key").cloned() {
            assert!(now >= expires_at);
        }
    }

    #[test]
    fn test_cache_expiry_remove() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache_expiry.lock().insert("key".into(), 9999);
        server.cache_expiry.lock().remove("key");
        assert!(server.cache_expiry.lock().get("key").is_none());
    }

    #[test]
    fn test_cache_expiry_clear() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache_expiry.lock().insert("a".into(), 1);
        server.cache_expiry.lock().insert("b".into(), 2);
        server.cache_expiry.lock().clear();
        assert!(server.cache_expiry.lock().is_empty());
    }
}
