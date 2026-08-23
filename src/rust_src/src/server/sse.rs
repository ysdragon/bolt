// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Server-Sent Events (SSE)

/// Serialize an SSE event's `data` field, splitting on `\n` AND `\r` so no
/// raw CR byte can be injected into the wire format.
fn sse_data_lines(data: &str) -> Vec<String> {
    data.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(|line| format!("data: {}\n", line))
        .collect()
}

use actix_web::{HttpRequest, HttpResponse};
use dashmap::DashMap;
use futures_util::stream::Stream;
use std::collections::HashMap;
use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

use crate::HTTP_SERVER_TYPE;
use ring_lang_rs::*;

use super::{AppState, HttpServer, SseEvent, SseRouteDefinition, convert_path_params};
use crate::modules::json::ring_list_to_json;

fn extract_sse_params(p: *mut c_void, param_index: i32) -> HashMap<String, String> {
    if ring_api_paracount(p) >= param_index && ring_api_islist(p, param_index) {
        let list = ring_api_getlist(p, param_index);
        match ring_list_to_json(list) {
            Ok(serde_json::Value::Object(map)) => map
                .into_iter()
                .filter_map(|(k, v)| {
                    if v.is_string() {
                        Some((k, v.as_str().unwrap().to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            Ok(_) => HashMap::new(),
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    }
}

pub(crate) async fn handle_sse(
    req: HttpRequest,
    state: actix_web::web::Data<AppState>,
) -> HttpResponse {
    let path_str = req.match_pattern().unwrap_or_default();

    let mut current_count = state
        .sse_subscriber_counts
        .entry(path_str.clone())
        .or_insert(0);
    if *current_count >= state.sse_max_subscribers {
        drop(current_count);
        return HttpResponse::ServiceUnavailable()
            .insert_header(("Retry-After", "5"))
            .body("SSE subscriber limit reached");
    }
    *current_count += 1;
    drop(current_count);

    let filter_params = state
        .sse_routes
        .iter()
        .find(|r| r.path == path_str)
        .map(|r| r.filter_params)
        .unwrap_or(false);

    let subscriber_params: HashMap<String, String> = if filter_params {
        req.match_info()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    } else {
        HashMap::new()
    };

    let broadcast_rx = {
        let channels = state.sse_broadcast_channels.lock();
        channels.get(&path_str).map(|tx| tx.subscribe())
    };

    if let Some(rx) = broadcast_rx {
        struct SseStream {
            inner: BroadcastStream<SseEvent>,
            interval: tokio::time::Interval,
            filter_params: bool,
            subscriber_params: HashMap<String, String>,
            path: String,
            subscriber_counts: Arc<DashMap<String, usize>>,
            done: bool,
        }

        impl Stream for SseStream {
            type Item = Result<actix_web::web::Bytes, actix_web::Error>;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                loop {
                    match Pin::new(&mut self.inner).poll_next(cx) {
                        Poll::Ready(Some(Ok(evt))) => {
                            if self.filter_params
                                && !self.subscriber_params.is_empty()
                                && !self
                                    .subscriber_params
                                    .iter()
                                    .all(|(k, v)| evt.params.get(k) == Some(v))
                            {
                                continue;
                            }
                            let mut event_str = String::new();
                            if let Some(ref event_name) = evt.event {
                                let sanitized: String = event_name
                                    .chars()
                                    .filter(|c| *c != '\r' && *c != '\n' && *c != ':')
                                    .collect();
                                if !sanitized.is_empty() {
                                    event_str.push_str(&format!("event: {}\n", sanitized));
                                }
                            }
                            if let Some(ref id) = evt.id {
                                let sanitized_id: String =
                                    id.chars().filter(|c| *c != '\r' && *c != '\n').collect();
                                if !sanitized_id.is_empty() {
                                    event_str.push_str(&format!("id: {}\n", sanitized_id));
                                }
                            }
                            if let Some(retry) = evt.retry {
                                event_str.push_str(&format!("retry: {}\n", retry));
                            }
                            for line in sse_data_lines(&evt.data) {
                                event_str.push_str(&line);
                            }
                            event_str.push('\n');
                            return Poll::Ready(Some(Ok(actix_web::web::Bytes::from(event_str))));
                        }
                        Poll::Ready(Some(Err(_))) => continue,
                        Poll::Ready(None) => {
                            self.done = true;
                            return Poll::Ready(None);
                        }
                        Poll::Pending => break,
                    }
                }

                match self.interval.poll_tick(cx) {
                    Poll::Ready(_) => {
                        Poll::Ready(Some(Ok(actix_web::web::Bytes::from(":ping\n\n"))))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        impl Drop for SseStream {
            fn drop(&mut self) {
                if !self.done {
                    self.done = true;
                }
                if let Some(mut count) = self.subscriber_counts.get_mut(&self.path) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        let stream = SseStream {
            inner: BroadcastStream::new(rx),
            interval: tokio::time::interval(Duration::from_secs(15)),
            filter_params,
            subscriber_params,
            path: path_str.clone(),
            subscriber_counts: state.sse_subscriber_counts.clone(),
            done: false,
        };

        HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("X-Accel-Buffering", "no"))
            .streaming(stream)
    } else {
        HttpResponse::NotFound().body("SSE endpoint not found")
    }
}

/// bolt_sse_route(server, path, handler) → add SSE route
ring_func!(bolt_sse_route, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let path = ring_get_string!(p, 2);
    let handler = ring_get_string!(p, 3);

    let path_converted = convert_path_params(path);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.sse_routes.push(SseRouteDefinition {
            path: path_converted.to_string(),
            handler_name: handler.to_string(),
            filter_params: false,
        });

        let mut channels = server.sse_broadcast_channels.lock();
        if !channels.contains_key(&path_converted) {
            let (tx, _) = tokio::sync::broadcast::channel::<SseEvent>(1000);
            channels.insert(path_converted.to_string(), tx);
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_sse_broadcast(server, path, data[, params]) → number of clients notified (-1 on error)
ring_func!(bolt_sse_broadcast, |p| {
    ring_check_paracount_range!(p, 3, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, -1.0);
        return;
    }

    let path = ring_get_string!(p, 2);
    let data = ring_get_string!(p, 3);
    let path_converted = convert_path_params(path);

    let params = extract_sse_params(p, 4);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let channels = server.sse_broadcast_channels.lock();

        if let Some(tx) = channels.get(&path_converted) {
            let evt = SseEvent {
                event: None,
                data: data.to_string(),
                params,
                id: None,
                retry: None,
            };
            match tx.send(evt) {
                Ok(count) => ring_ret_number!(p, count as f64),
                Err(_) => {
                    ring_ret_number!(p, 0.0);
                }
            }
        } else {
            ring_ret_number!(p, -1.0);
        }
    }
});

/// bolt_sse_broadcast_event(server, path, event_name, data[, params]) → number of clients notified (-1 on error)
ring_func!(bolt_sse_broadcast_event, |p| {
    ring_check_paracount_range!(p, 4, 5);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, -1.0);
        return;
    }

    let path = ring_get_string!(p, 2);
    let event_name = ring_get_string!(p, 3);
    let data = ring_get_string!(p, 4);
    let path_converted = convert_path_params(path);

    let params = extract_sse_params(p, 5);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let channels = server.sse_broadcast_channels.lock();

        if let Some(tx) = channels.get(&path_converted) {
            let evt = SseEvent {
                event: Some(event_name.to_string()),
                data: data.to_string(),
                params,
                id: None,
                retry: None,
            };
            match tx.send(evt) {
                Ok(count) => ring_ret_number!(p, count as f64),
                Err(_) => {
                    ring_ret_number!(p, 0.0);
                }
            }
        } else {
            ring_ret_number!(p, -1.0);
        }
    }
});

/// bolt_sse_filter_params(server, path) → enable param-based event filtering for an SSE route
ring_func!(bolt_sse_filter_params, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let path = ring_get_string!(p, 2);
    let path_converted = convert_path_params(path);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        if let Some(route) = server
            .sse_routes
            .iter_mut()
            .find(|r| r.path == path_converted)
        {
            route.filter_params = true;
            ring_ret_number!(p, 1.0);
        } else {
            ring_error!(p, "sse_filter_params: no SSE route found for path");
        }
    }
});

/// bolt_sse_max_subscribers(server, max) → set max concurrent SSE subscribers per route
ring_func!(bolt_sse_max_subscribers, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let max = ring_get_number!(p, 2) as usize;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        if max > 0 {
            server.sse_max_subscribers = max;
        }
    }

    ring_ret_number!(p, 1.0);
});

#[cfg(test)]
mod tests {
    use super::sse_data_lines;

    #[test]
    fn test_sse_data_lines_bare_cr_split() {
        let out = sse_data_lines("a\rb");
        assert_eq!(out, vec!["data: a\n", "data: b\n"]);
        assert!(out.concat().chars().all(|c| c != '\r'));
    }

    #[test]
    fn test_sse_data_lines_crlf_produces_two_lines() {
        let out = sse_data_lines("a\r\nb");
        assert_eq!(out, vec!["data: a\n", "data: b\n"]);
    }

    #[test]
    fn test_sse_data_lines_no_raw_cr_anywhere() {
        let out = sse_data_lines("x\r\ny\rz\nw");
        let joined = out.concat();
        assert!(!joined.contains('\r'));
        assert_eq!(joined, "data: x\ndata: y\ndata: z\ndata: w\n");
    }
}
