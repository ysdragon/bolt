// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Environment variable management via dotenvy

use ring_lang_rs::*;

/// bolt_env_load() → number (1 on success, 0 if .env not found) — load .env from current directory
ring_func!(bolt_env_load, |p| {
    ring_check_paracount!(p, 0);

    match dotenvy::dotenv() {
        Ok(_) => ring_ret_number!(p, 1.0),
        Err(_) => ring_ret_number!(p, 0.0),
    }
});

/// bolt_env_load_file(path) → number (1 on success) — load specific .env file
ring_func!(bolt_env_load_file, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let path = ring_get_string!(p, 1);

    match dotenvy::from_filename(path) {
        Ok(_) => ring_ret_number!(p, 1.0),
        Err(_) => {
            ring_error!(p, "env: failed to load .env file from path");
        }
    }
});

/// bolt_env_get(key) → string — get env var (empty string if not found)
ring_func!(bolt_env_get, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let key = ring_get_string!(p, 1);

    let value = std::env::var(key).unwrap_or_default();
    ring_ret_string!(p, &value);
});

pub static ENV_SET_ALLOWED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// True when `set_var` can safely be called with this key/value pair.
/// `std::env::set_var` panics on `=` or NUL bytes in the key and on NUL
/// bytes in the value.
fn valid_env_pair(key: &str, value: &str) -> bool {
    !key.is_empty() && !key.contains(['=', '\0']) && !value.contains('\0')
}

/// bolt_env_set(key, value) → number (1 on success) — set env var (only before bolt_listen is called)
ring_func!(bolt_env_set, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    if !ENV_SET_ALLOWED.load(std::sync::atomic::Ordering::SeqCst) {
        ring_error!(
            p,
            "env: cannot set environment variables after server has started"
        );
        return;
    }

    let key = ring_get_string!(p, 1);
    let value = ring_get_string!(p, 2);

    if !valid_env_pair(key, value) {
        ring_error!(p, "env: invalid key or value");
        return;
    }

    unsafe {
        std::env::set_var(key, value);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_env_get_or(key, default) → string — get with fallback
ring_func!(bolt_env_get_or, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let key = ring_get_string!(p, 1);
    let default = ring_get_string!(p, 2);

    let value = std::env::var(key).unwrap_or_else(|_| default.to_string());
    ring_ret_string!(p, &value);
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_env_pair_good() {
        assert!(valid_env_pair("BOLT_TEST", "some value"));
        assert!(valid_env_pair("A", "1"));
    }

    #[test]
    fn test_valid_env_pair_empty_key() {
        assert!(!valid_env_pair("", "value"));
    }

    #[test]
    fn test_valid_env_pair_equals_in_key() {
        assert!(!valid_env_pair("a=b", "value"));
    }

    #[test]
    fn test_valid_env_pair_nul_byte() {
        assert!(!valid_env_pair("a\0b", "value"));
        assert!(!valid_env_pair("key", "va\0lue"));
    }
}
