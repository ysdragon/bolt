// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! HTTP Response functions

use super::{HttpServer, PendingResponse, ResponseBody};
use crate::HTTP_SERVER_TYPE;
use ring_lang_rs::*;
use std::collections::HashMap;

/// bolt_respond(server, status, body)
ring_func!(bolt_respond, |p| {
    ring_check_paracount_range!(p, 2, 3);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let status = ring_get_number!(p, 2) as u16;

    let body = if ring_api_paracount(p) >= 3 && ring_api_isstring(p, 3) {
        ring_get_string!(p, 3).as_bytes().to_vec()
    } else {
        Vec::new()
    };

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let (mut existing_headers, existing_cookies) =
            PendingResponse::take_existing(&server.current_response);

        existing_headers
            .entry("Content-Type".to_string())
            .or_insert_with(|| "text/html; charset=utf-8".to_string());

        *server.current_response.lock() = Some(PendingResponse {
            status,
            headers: existing_headers,
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::Bytes(body),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_respond_json(server, status, json_string)
ring_func!(bolt_respond_json, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let status = ring_get_number!(p, 2) as u16;
    let json_body = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let (mut existing_headers, existing_cookies) =
            PendingResponse::take_existing(&server.current_response);

        existing_headers.insert("Content-Type".to_string(), "application/json".to_string());

        *server.current_response.lock() = Some(PendingResponse {
            status,
            headers: existing_headers,
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::Bytes(json_body.as_bytes().to_vec()),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_respond_file(server, file_path, content_type?)
ring_func!(bolt_respond_file, |p| {
    ring_check_paracount_range!(p, 2, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let file_path = ring_get_string!(p, 2);

    let content_type = if ring_api_paracount(p) >= 3 && ring_api_isstring(p, 3) {
        ring_get_string!(p, 3).to_string()
    } else {
        mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string()
    };

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type);

        let existing_cookies = server
            .current_response
            .lock()
            .as_ref()
            .map(|r| r.cookies.clone())
            .unwrap_or_default();

        *server.current_response.lock() = Some(PendingResponse {
            status: 200,
            headers,
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::File(file_path.to_string()),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_respond_redirect(server, url, permanent?)
ring_func!(bolt_respond_redirect, |p| {
    ring_check_paracount_range!(p, 2, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let url = ring_get_string!(p, 2);

    let permanent = if ring_api_paracount(p) >= 3 && ring_api_isnumber(p, 3) {
        ring_get_number!(p, 3) != 0.0
    } else {
        false
    };

    let status = if permanent { 301 } else { 302 };

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url.to_string());

        // Preserve existing cookies
        let existing_cookies = server
            .current_response
            .lock()
            .as_ref()
            .map(|r| r.cookies.clone())
            .unwrap_or_default();

        *server.current_response.lock() = Some(PendingResponse {
            status,
            headers,
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::Bytes(Vec::new()),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_respond_status(server, status)
ring_func!(bolt_respond_status, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let status = ring_get_number!(p, 2) as u16;

    unsafe {
        let server = &*(ptr as *const HttpServer);

        // Preserve existing cookies
        let existing_cookies = server
            .current_response
            .lock()
            .as_ref()
            .map(|r| r.cookies.clone())
            .unwrap_or_default();

        *server.current_response.lock() = Some(PendingResponse {
            status,
            headers: HashMap::new(),
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::Bytes(Vec::new()),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_respond_binary(server, data_base64, content_type?) - send raw binary response
ring_func!(bolt_respond_binary, |p| {
    ring_check_paracount_range!(p, 2, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let b64_data = ring_get_string!(p, 2);

    let content_type = if ring_api_paracount(p) >= 3 && ring_api_isstring(p, 3) {
        ring_get_string!(p, 3).to_string()
    } else {
        "application/octet-stream".to_string()
    };

    use base64::Engine;
    let body = match base64::engine::general_purpose::STANDARD.decode(b64_data) {
        Ok(d) => d,
        Err(e) => {
            ring_error!(p, &format!("Invalid base64 data: {}", e));
            return;
        }
    };

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let (mut existing_headers, existing_cookies) =
            PendingResponse::take_existing(&server.current_response);

        existing_headers.insert("Content-Type".to_string(), content_type);

        *server.current_response.lock() = Some(PendingResponse {
            status: 200,
            headers: existing_headers,
            cookies: existing_cookies,
            only_headers: false,
            body: ResponseBody::Bytes(body),
        });
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_urlencode(string) -> encoded_string
ring_func!(bolt_urlencode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let input = ring_get_string!(p, 1);
    let encoded = urlencoding::encode(input);
    ring_ret_string!(p, &encoded);
});

/// bolt_urldecode(string) -> decoded_string
ring_func!(bolt_urldecode, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let input = ring_get_string!(p, 1);
    match urlencoding::decode(input) {
        Ok(decoded) => ring_ret_string!(p, &decoded),
        Err(_) => ring_ret_string!(p, input),
    }
});
