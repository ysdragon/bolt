// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! JSON encoding/decoding for Ring

use ring_lang_rs::*;
use serde_json::{Map, Value};

// Convert Ring list to serde_json Value
const MAX_JSON_DEPTH: usize = 128;

/// Sentinel string representing JSON `true` in Ring lists.
pub const JSON_TRUE: &str = "__JSON_TRUE__";
/// Sentinel string representing JSON `false` in Ring lists.
pub const JSON_FALSE: &str = "__JSON_FALSE__";

/// bolt_json_encode(ring_list) → json_string
/// Converts a Ring list to JSON string
ring_func!(bolt_json_encode, |p| {
    ring_check_paracount!(p, 1);

    if !ring_api_islist(p, 1) {
        ring_error!(p, "Expected a list");
        return;
    }

    let list = ring_api_getlist(p, 1);
    match ring_list_to_json(list) {
        Ok(value) => {
            let json_str = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());
            ring_ret_string!(p, &json_str);
        }
        Err(e) => {
            ring_error!(p, &e);
        }
    }
});

/// bolt_json_decode(json_string) → ring_list
/// Parses JSON string to Ring list
ring_func!(bolt_json_decode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let json_str = ring_get_string!(p, 1);

    match serde_json::from_str::<Value>(json_str) {
        Ok(value) => {
            let list = ring_api_newlist(p);
            match json_to_ring_list(list, &value, 0) {
                Ok(()) => ring_ret_list!(p, list),
                Err(e) => ring_error!(p, &e),
            }
        }
        Err(_) => {
            ring_api_retlist(p, ring_api_newlist(p));
        }
    }
});

/// bolt_json_pretty(ring_list) → json_string (formatted)
ring_func!(bolt_json_pretty, |p| {
    ring_check_paracount!(p, 1);

    if !ring_api_islist(p, 1) {
        ring_error!(p, "Expected a list");
        return;
    }

    let list = ring_api_getlist(p, 1);
    match ring_list_to_json(list) {
        Ok(value) => {
            let json_str =
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string());
            ring_ret_string!(p, &json_str);
        }
        Err(e) => {
            ring_error!(p, &e);
        }
    }
});

/// bolt_json_true() → JSON true sentinel string
ring_func!(bolt_json_true, |p| {
    ring_check_paracount!(p, 0);
    ring_ret_string!(p, JSON_TRUE);
});

/// bolt_json_false() → JSON false sentinel string
ring_func!(bolt_json_false, |p| {
    ring_check_paracount!(p, 0);
    ring_ret_string!(p, JSON_FALSE);
});

/// bolt_json_is_true(value) → 1.0 if value is JSON true, else 0.0
ring_func!(bolt_json_is_true, |p| {
    ring_check_paracount!(p, 1);
    if ring_api_isstring(p, 1) {
        let s = ring_get_string!(p, 1);
        ring_ret_number!(p, if s == JSON_TRUE { 1.0 } else { 0.0 });
    } else {
        ring_ret_number!(p, 0.0);
    }
});

/// bolt_json_is_false(value) → 1.0 if value is JSON false, else 0.0
ring_func!(bolt_json_is_false, |p| {
    ring_check_paracount!(p, 1);
    if ring_api_isstring(p, 1) {
        let s = ring_get_string!(p, 1);
        ring_ret_number!(p, if s == JSON_FALSE { 1.0 } else { 0.0 });
    } else {
        ring_ret_number!(p, 0.0);
    }
});

/// bolt_json_tobool(value) → 1.0 for true, 0.0 for false, error otherwise
ring_func!(bolt_json_tobool, |p| {
    ring_check_paracount!(p, 1);
    if ring_api_isstring(p, 1) {
        let s = ring_get_string!(p, 1);
        if s == JSON_TRUE {
            ring_ret_number!(p, 1.0);
            return;
        }
        if s == JSON_FALSE {
            ring_ret_number!(p, 0.0);
            return;
        }
    }
    ring_error!(p, "Not a JSON boolean value.");
});

pub fn ring_list_to_json(list: RingList) -> Result<Value, String> {
    ring_list_to_json_inner(list, 0)
}

fn ring_list_to_json_inner(list: RingList, depth: usize) -> Result<Value, String> {
    if depth >= MAX_JSON_DEPTH {
        return Err("JSON nesting depth exceeds limit (128)".to_string());
    }

    let size = ring_list_getsize(list);

    if size == 0 {
        return Ok(Value::Array(vec![]));
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

                let val = get_list_item_as_json(inner, 2, depth + 1)?;
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
                let val = get_list_item_as_json(list, i + 1, depth + 1)?;
                items.push((key, val));
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
        Ok(Value::Object(map))
    } else {
        // Treat as array
        let mut arr = Vec::new();
        for i in 1..=size {
            arr.push(get_list_item_as_json(list, i, depth + 1)?);
        }
        Ok(Value::Array(arr))
    }
}

fn get_list_item_as_json(list: RingList, index: u32, depth: usize) -> Result<Value, String> {
    if ring_list_isstring(list, index) {
        let s = ring_list_getstring_str(list, index);
        if s == JSON_TRUE {
            return Ok(Value::Bool(true));
        }
        if s == JSON_FALSE {
            return Ok(Value::Bool(false));
        }
        Ok(Value::String(s))
    } else if ring_list_isnumber(list, index) {
        let n = ring_list_getdouble(list, index);
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n < i64::MAX as f64 {
            Ok(Value::Number(serde_json::Number::from(n as i64)))
        } else {
            Ok(serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::Null))
        }
    } else if ring_list_islist(list, index) {
        let inner = ring_list_getlist(list, index);
        ring_list_to_json_inner(inner, depth)
    } else {
        Ok(Value::Null)
    }
}

// Convert serde_json Value to Ring list
fn json_to_ring_list(list: RingList, value: &Value, depth: usize) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                // Use ring_list_addstring which properly handles Ring's internal format
                // Key should be just the name without colon for Ring's hash access
                let item = ring_list_newlist(list);
                // Add key as string (Ring will handle hash access)
                ring_list_addstring_str(item, key);
                add_json_value_to_list(item, val, depth + 1)?;
            }
        }
        Value::Array(arr) => {
            for val in arr {
                add_json_value_to_list(list, val, depth + 1)?;
            }
        }
        _ => {
            add_json_value_to_list(list, value, depth + 1)?;
        }
    }
    Ok(())
}

fn add_json_value_to_list(list: RingList, value: &Value, depth: usize) -> Result<(), String> {
    if depth >= MAX_JSON_DEPTH {
        return Err("JSON nesting depth exceeds limit (128)".to_string());
    }
    match value {
        Value::Null => {
            ring_list_addstring_str(list, "");
        }
        Value::Bool(b) => {
            ring_list_addstring_str(list, if *b { JSON_TRUE } else { JSON_FALSE });
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
                add_json_value_to_list(inner, item, depth + 1)?;
            }
        }
        Value::Object(map) => {
            let inner = ring_list_newlist(list);
            for (key, val) in map {
                let pair = ring_list_newlist(inner);
                ring_list_addstring_str(pair, key);
                add_json_value_to_list(pair, val, depth + 1)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(list: RingList) -> String {
        serde_json::to_string(&ring_list_to_json(list).unwrap()).unwrap()
    }

    fn decode(json: &str) -> RingList {
        let value: Value = serde_json::from_str(json).unwrap();
        let list = ring_list_new(0);
        json_to_ring_list(list, &value, 0).unwrap();
        list
    }

    #[test]
    fn test_encode_boolean_true() {
        let list = ring_list_new(0);
        let pair = ring_list_newlist(list);
        ring_list_addstring_str(pair, "active");
        ring_list_addstring_str(pair, JSON_TRUE);
        assert_eq!(encode(list), r#"{"active":true}"#);
    }

    #[test]
    fn test_encode_boolean_false() {
        let list = ring_list_new(0);
        let pair = ring_list_newlist(list);
        ring_list_addstring_str(pair, "disabled");
        ring_list_addstring_str(pair, JSON_FALSE);
        assert_eq!(encode(list), r#"{"disabled":false}"#);
    }

    #[test]
    fn test_encode_boolean_in_array() {
        let list = ring_list_new(0);
        ring_list_addstring_str(list, JSON_TRUE);
        ring_list_addstring_str(list, JSON_FALSE);
        assert_eq!(encode(list), r#"[true,false]"#);
    }

    #[test]
    fn test_decode_boolean_true() {
        let list = decode(r#"{"active":true}"#);
        let inner = ring_list_getlist(list, 1);
        assert!(ring_list_isstring(inner, 2));
        assert_eq!(ring_list_getstring_str(inner, 2), JSON_TRUE);
    }

    #[test]
    fn test_decode_boolean_false() {
        let list = decode(r#"{"disabled":false}"#);
        let inner = ring_list_getlist(list, 1);
        assert_eq!(ring_list_getstring_str(inner, 2), JSON_FALSE);
    }

    #[test]
    fn test_decode_boolean_in_array() {
        let list = decode(r#"[true,false]"#);
        assert_eq!(ring_list_getstring_str(list, 1), JSON_TRUE);
        assert_eq!(ring_list_getstring_str(list, 2), JSON_FALSE);
    }

    #[test]
    fn test_round_trip_boolean() {
        let original = r#"{"active":true,"disabled":false,"count":1}"#;
        let list = decode(original);
        let restored = encode(list);
        // Compare as parsed JSON values (key order may differ due to serde_json's
        // BTreeMap, but booleans and numbers are preserved).
        let a: Value = serde_json::from_str(original).unwrap();
        let b: Value = serde_json::from_str(&restored).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_encode_number_not_boolean() {
        let list = ring_list_new(0);
        let pair = ring_list_newlist(list);
        ring_list_addstring_str(pair, "count");
        ring_list_adddouble(pair, 1.0);
        assert_eq!(encode(list), r#"{"count":1}"#);
    }

    #[test]
    fn test_sentinel_string_escaped_as_literal() {
        let list = ring_list_new(0);
        ring_list_addstring_str(list, JSON_TRUE);
        assert_eq!(encode(list), "[true]");
    }
}
