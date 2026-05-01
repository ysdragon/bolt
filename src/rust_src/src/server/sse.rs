// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Server-Sent Events (SSE)

use actix_web::{HttpRequest, HttpResponse};
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

use crate::HTTP_SERVER_TYPE;
use ring_lang_rs::*;

use super::{AppState, HttpServer, SseEvent, SseRouteDefinition, convert_path_params};

pub(crate) async fn handle_sse(
    req: HttpRequest,
    state: actix_web::web::Data<AppState>,
) -> HttpResponse {
    let path_str = req.match_pattern().unwrap_or_default();

    let broadcast_rx = {
        let channels = state.sse_broadcast_channels.lock();
        channels.get(&path_str).map(|tx| tx.subscribe())
    };

    if let Some(rx) = broadcast_rx {
        struct SseStream {
            inner: BroadcastStream<SseEvent>,
            interval: tokio::time::Interval,
        }

        impl Stream for SseStream {
            type Item = Result<actix_web::web::Bytes, actix_web::Error>;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                match Pin::new(&mut self.inner).poll_next(cx) {
                    Poll::Ready(Some(Ok(evt))) => {
                        let mut event_str = String::new();
                        if let Some(event_name) = evt.event {
                            event_str.push_str(&format!("event: {}\n", event_name));
                        }
                        for line in evt.data.lines() {
                            event_str.push_str(&format!("data: {}\n", line));
                        }
                        event_str.push('\n');
                        return Poll::Ready(Some(Ok(actix_web::web::Bytes::from(event_str))));
                    }
                    Poll::Ready(Some(Err(_))) => {}
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Pending => {}
                }

                match self.interval.poll_tick(cx) {
                    Poll::Ready(_) => {
                        Poll::Ready(Some(Ok(actix_web::web::Bytes::from(":ping\n\n"))))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        let stream = SseStream {
            inner: BroadcastStream::new(rx),
            interval: tokio::time::interval(Duration::from_secs(15)),
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

/// bolt_sse_route(server, path, handler) - add SSE route
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
        });

        let mut channels = server.sse_broadcast_channels.lock();
        if !channels.contains_key(&path_converted) {
            let (tx, _) = tokio::sync::broadcast::channel::<SseEvent>(1000);
            channels.insert(path_converted.to_string(), tx);
        }
    }

    ring_ret_number!(p, 1.0);
});

ring_func!(bolt_sse_broadcast, |p| {
    ring_check_paracount!(p, 3);
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

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let channels = server.sse_broadcast_channels.lock();

        if let Some(tx) = channels.get(&path_converted) {
            let evt = SseEvent {
                event: None,
                data: data.to_string(),
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

ring_func!(bolt_sse_broadcast_event, |p| {
    ring_check_paracount!(p, 4);
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

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let channels = server.sse_broadcast_channels.lock();

        if let Some(tx) = channels.get(&path_converted) {
            let evt = SseEvent {
                event: Some(event_name.to_string()),
                data: data.to_string(),
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
