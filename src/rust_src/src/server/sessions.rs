// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Session Functions

use ring_lang_rs::*;
use std::collections::HashMap;

use crate::HTTP_SERVER_TYPE;

use super::{HttpServer, PendingResponse, ResponseBody};

/// bolt_session_set(server, key, value) - set session value
ring_func!(bolt_session_set, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let key = ring_get_string!(p, 2);
    let value = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let session_id = {
            let guard = server.current_request.lock();
            guard
                .as_ref()
                .map(|ctx| ctx.session_id.clone())
                .unwrap_or_default()
        };

        if !session_id.is_empty() {
            let mut session = server.sessions.get(&session_id).unwrap_or_default();
            session.insert(key.to_string(), value.to_string());
            server.sessions.insert(session_id.clone(), session);

            let mut response = server.current_response.lock();
            let cookie =
                cookie::Cookie::parse(format!("BOLTSESSION={}; Path=/; HttpOnly", session_id))
                    .map(|c| c.to_string())
                    .unwrap_or_else(|_| format!("BOLTSESSION={}; Path=/; HttpOnly", session_id));
            if let Some(ref mut res) = *response {
                if !res.cookies.iter().any(|c| c.starts_with("BOLTSESSION=")) {
                    res.cookies.push(cookie);
                }
            } else {
                *response = Some(PendingResponse {
                    status: 200,
                    headers: HashMap::new(),
                    cookies: vec![cookie],
                    body: ResponseBody::Bytes(Vec::new()),
                    only_headers: true,
                });
            }
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_session_get(server, key) -> value
ring_func!(bolt_session_get, |p| {
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

        let session_id = {
            let guard = server.current_request.lock();
            guard
                .as_ref()
                .map(|ctx| ctx.session_id.clone())
                .unwrap_or_default()
        };

        if !session_id.is_empty() {
            if let Some(session) = server.sessions.get(&session_id) {
                if let Some(value) = session.get(key) {
                    ring_ret_string!(p, value);
                    return;
                }
            }
        }
        ring_ret_string!(p, "");
    }
});

/// bolt_session_delete(server, key) - delete session key
ring_func!(bolt_session_delete, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let key = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let session_id = {
            let guard = server.current_request.lock();
            guard
                .as_ref()
                .map(|ctx| ctx.session_id.clone())
                .unwrap_or_default()
        };

        if !session_id.is_empty() {
            if let Some(mut session) = server.sessions.get(&session_id) {
                session.remove(key);
                server.sessions.insert(session_id, session);
            }
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_session_clear(server) - clear all session data
ring_func!(bolt_session_clear, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let session_id = {
            let guard = server.current_request.lock();
            guard
                .as_ref()
                .map(|ctx| ctx.session_id.clone())
                .unwrap_or_default()
        };

        if !session_id.is_empty() {
            server.sessions.invalidate(&session_id);
        }
    }

    ring_ret_number!(p, 1.0);
});
