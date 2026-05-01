// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Environment variable management via dotenvy

use ring_lang_rs::*;

/// bolt_env_load() — load .env from current directory
ring_func!(bolt_env_load, |p| {
    ring_check_paracount!(p, 0);

    match dotenvy::dotenv() {
        Ok(_) => ring_ret_number!(p, 1.0),
        Err(_) => {
            ring_error!(p, "env: failed to load .env file");
        }
    }
});

/// bolt_env_load_file(path) — load specific .env file
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

/// bolt_env_set(key, value) — set env var
ring_func!(bolt_env_set, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let key = ring_get_string!(p, 1);
    let value = ring_get_string!(p, 2);

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
