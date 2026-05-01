// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! JSON encoding/decoding for Ring

use ring_lang_rs::*;
use serde_json::{Map, Value};

/// bolt_json_encode(ring_list) -> json_string
/// Converts a Ring list to JSON string
ring_func!(bolt_json_encode, |p| {
    ring_check_paracount!(p, 1);

    if !ring_api_islist(p, 1) {
        ring_error!(p, "Expected a list");
        return;
    }

    let list = ring_api_getlist(p, 1);
    let value = ring_list_to_json(list);
    let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());

    ring_ret_string!(p, &json_str);
});

/// bolt_json_decode(json_string) -> ring_list
/// Parses JSON string to Ring list
ring_func!(bolt_json_decode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let json_str = ring_get_string!(p, 1);

    match serde_json::from_str::<Value>(json_str) {
        Ok(value) => {
            let list = ring_api_newlist(p);
            json_to_ring_list(list, &value);
            ring_ret_list!(p, list);
        }
        Err(_) => {
            ring_api_retlist(p, ring_api_newlist(p));
        }
    }
});

/// bolt_json_pretty(ring_list) -> json_string (formatted)
ring_func!(bolt_json_pretty, |p| {
    ring_check_paracount!(p, 1);

    if !ring_api_islist(p, 1) {
        ring_error!(p, "Expected a list");
        return;
    }

    let list = ring_api_getlist(p, 1);
    let value = ring_list_to_json(list);
    let json_str = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string());

    ring_ret_string!(p, &json_str);
});

// Convert Ring list to serde_json Value
fn ring_list_to_json(list: RingList) -> Value {
    let size = ring_list_getsize(list);

    if size == 0 {
        return Value::Array(vec![]);
    }

    // Check if it's a hash-like list (list of [key, value] pairs where key is string starting with :)
    // or an object-like list with :key = value syntax
    let mut is_object = true;
    let mut items: Vec<(String, Value)> = Vec::new();

    for i in 1..=size {
        let idx = i;
        if ring_list_islist(list, idx) {
            let inner = ring_list_getlist(list, idx);
            let inner_size = ring_list_getsize(inner);

            // Check for [:key, value] format
            if inner_size == 2 && ring_list_isstring(inner, 1) {
                let key = ring_list_getstring_str(inner, 1);
                // Remove : prefix if present
                let clean_key = key.strip_prefix(':').unwrap_or(&key).to_string();

                let val = get_list_item_as_json(inner, 2);
                items.push((clean_key, val));
            } else {
                is_object = false;
                break;
            }
        } else if ring_list_isstring(list, idx) {
            // Check if it looks like a key (starts with :)
            let s = ring_list_getstring_str(list, idx);
            if s.starts_with(':') && i < size {
                let key = s.strip_prefix(':').unwrap_or(&s).to_string();
                let val = get_list_item_as_json(list, i + 1);
                items.push((key, val));
                // Skip the value in next iteration - but this pattern doesn't work well
                // Better to just treat as array
            }
            is_object = false;
            break;
        } else {
            is_object = false;
            break;
        }
    }

    if is_object && !items.is_empty() {
        let map: Map<String, Value> = items.into_iter().collect();
        Value::Object(map)
    } else {
        // Treat as array
        let mut arr = Vec::new();
        for i in 1..=size {
            arr.push(get_list_item_as_json(list, i));
        }
        Value::Array(arr)
    }
}

fn get_list_item_as_json(list: RingList, index: u32) -> Value {
    if ring_list_isstring(list, index) {
        let s = ring_list_getstring_str(list, index);
        Value::String(s)
    } else if ring_list_isnumber(list, index) {
        let n = ring_list_getdouble(list, index);
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            Value::Number(serde_json::Number::from(n as i64))
        } else {
            serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
    } else if ring_list_islist(list, index) {
        let inner = ring_list_getlist(list, index);
        ring_list_to_json(inner)
    } else {
        Value::Null
    }
}

// Convert serde_json Value to Ring list
fn json_to_ring_list(list: RingList, value: &Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                // Use ring_list_addstring which properly handles Ring's internal format
                // Key should be just the name without colon for Ring's hash access
                let item = ring_list_newlist(list);
                // Add key as string (Ring will handle hash access)
                ring_list_addstring_str(item, key);
                add_json_value_to_list(item, val);
            }
        }
        Value::Array(arr) => {
            for val in arr {
                add_json_value_to_list(list, val);
            }
        }
        _ => {
            add_json_value_to_list(list, value);
        }
    }
}

fn add_json_value_to_list(list: RingList, value: &Value) {
    match value {
        Value::Null => {
            ring_list_addstring_str(list, "");
        }
        Value::Bool(b) => {
            ring_list_adddouble(list, if *b { 1.0 } else { 0.0 });
        }
        Value::Number(n) => {
            ring_list_adddouble(list, n.as_f64().unwrap_or(0.0));
        }
        Value::String(s) => {
            ring_list_addstring_str(list, s);
        }
        Value::Array(arr) => {
            let inner = ring_list_newlist(list);
            for item in arr {
                add_json_value_to_list(inner, item);
            }
        }
        Value::Object(map) => {
            let inner = ring_list_newlist(list);
            for (key, val) in map {
                let pair = ring_list_newlist(inner);
                ring_list_addstring_str(pair, key);
                add_json_value_to_list(pair, val);
            }
        }
    }
}
