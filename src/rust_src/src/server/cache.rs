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
        server
            .cache
            .insert(key.to_string(), (value.to_string(), ttl_seconds));
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
        match server.cache.get(&key.to_string()) {
            Some(entry) => ring_ret_string!(p, &entry.0),
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
    }

    ring_ret_number!(p, 1.0);
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("key1".into(), ("value1".into(), 0));
        let entry = server.cache.get("key1").unwrap();
        assert_eq!(entry.0, "value1");
        assert_eq!(entry.1, 0);
    }

    #[test]
    fn test_cache_get_missing() {
        let server = HttpServer::new(std::ptr::null_mut());
        assert!(server.cache.get("missing").is_none());
    }

    #[test]
    fn test_cache_delete() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("key1".into(), ("value1".into(), 0));
        server.cache.invalidate("key1");
        assert!(server.cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("a".into(), ("1".into(), 0));
        server.cache.insert("b".into(), ("2".into(), 0));
        server.cache.invalidate_all();
        assert!(server.cache.get("a").is_none());
        assert!(server.cache.get("b").is_none());
    }

    #[test]
    fn test_cache_set_with_ttl() {
        let server = HttpServer::new(std::ptr::null_mut());
        server
            .cache
            .insert("ttl_key".into(), ("value".into(), 3600));
        let entry = server.cache.get("ttl_key").unwrap();
        assert_eq!(entry.0, "value");
        assert_eq!(entry.1, 3600);
    }

    #[test]
    fn test_cache_set_no_ttl() {
        let server = HttpServer::new(std::ptr::null_mut());
        server.cache.insert("perm_key".into(), ("value".into(), 0));
        let entry = server.cache.get("perm_key").unwrap();
        assert_eq!(entry.0, "value");
        assert_eq!(entry.1, 0);
    }
}
