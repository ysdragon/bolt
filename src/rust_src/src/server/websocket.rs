// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! WebSocket Handler & Ring Functions

use actix_web::{HttpRequest, HttpResponse};
use std::collections::HashMap;

use crate::HTTP_SERVER_TYPE;
use ring_lang_rs::*;

use super::{AppState, HttpServer, VmWork, WsEventContext, WsOutMessage};

pub(crate) async fn handle_websocket(
    req: HttpRequest,
    stream: actix_web::web::Payload,
    state: actix_web::web::Data<AppState>,
    on_connect: Option<String>,
    on_message: Option<String>,
    on_disconnect: Option<String>,
    ws_path: String,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let ws_params: HashMap<String, String> = req
        .match_info()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let client_id = uuid::Uuid::new_v4().to_string();

    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel::<WsOutMessage>();

    state.ws_clients.insert(client_id.clone(), client_tx);

    let mut broadcast_rx = state.ws_broadcast_tx.subscribe();

    if let Some(ref handler) = on_connect {
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let _ = state
            .vm_tx
            .send(VmWork::WsEvent {
                handler_name: handler.clone(),
                ctx: WsEventContext {
                    client_id: client_id.clone(),
                    event_type: "connect".to_string(),
                    message: String::new(),
                    is_binary: false,
                    binary_data: Vec::new(),
                    path: ws_path.clone(),
                    params: ws_params.clone(),
                },
                done_tx,
            })
            .await;
        let _ = done_rx.await;
    }

    let mut session_clone = session.clone();
    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                result = broadcast_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if session_clone.text(msg).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("[WS] Client lagged, skipped {} messages", n);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                result = client_rx.recv() => {
                    match result {
                        Some(WsOutMessage::Text(msg)) => {
                            if session_clone.text(msg).await.is_err() {
                                break;
                            }
                        }
                        Some(WsOutMessage::Binary(data)) => {
                            if session_clone.binary(data).await.is_err() {
                                break;
                            }
                        }
                        Some(WsOutMessage::Close) => {
                            let _ = session_clone.close(None).await;
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let ws_path_clone = ws_path.clone();
    let ws_params_clone = ws_params.clone();
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                actix_ws::Message::Text(text) => {
                    let trimmed_text = text.to_string();

                    if let Some(ref handler) = on_message {
                        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
                        let _ = state_clone
                            .vm_tx
                            .send(VmWork::WsEvent {
                                handler_name: handler.clone(),
                                ctx: WsEventContext {
                                    client_id: client_id_clone.clone(),
                                    event_type: "message".to_string(),
                                    message: trimmed_text.clone(),
                                    is_binary: false,
                                    binary_data: Vec::new(),
                                    path: ws_path_clone.clone(),
                                    params: ws_params_clone.clone(),
                                },
                                done_tx,
                            })
                            .await;
                        let _ = done_rx.await;
                    }
                }
                actix_ws::Message::Binary(data) => {
                    if let Some(ref handler) = on_message {
                        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
                        let _ = state_clone
                            .vm_tx
                            .send(VmWork::WsEvent {
                                handler_name: handler.clone(),
                                ctx: WsEventContext {
                                    client_id: client_id_clone.clone(),
                                    event_type: "message".to_string(),
                                    message: String::new(),
                                    is_binary: true,
                                    binary_data: data.to_vec(),
                                    path: ws_path_clone.clone(),
                                    params: ws_params_clone.clone(),
                                },
                                done_tx,
                            })
                            .await;
                        let _ = done_rx.await;
                    }
                }
                actix_ws::Message::Close(_) => break,
                _ => {}
            }
        }

        state_clone.ws_clients.remove(&client_id_clone);

        if let Some(client_room_entry) = state_clone.ws_client_rooms.remove(&client_id_clone) {
            let client_rooms = client_room_entry.1;
            for room in client_rooms {
                if let Some(mut members) = state_clone.ws_rooms.get_mut(&room) {
                    members.remove(&client_id_clone);
                    if members.is_empty() {
                        drop(members);
                        state_clone.ws_rooms.remove(&room);
                    }
                }
            }
        }

        if let Some(ref handler) = on_disconnect {
            let (done_tx, done_rx) = tokio::sync::oneshot::channel();
            let _ = state_clone
                .vm_tx
                .send(VmWork::WsEvent {
                    handler_name: handler.clone(),
                    ctx: WsEventContext {
                        client_id: client_id_clone.clone(),
                        event_type: "disconnect".to_string(),
                        message: String::new(),
                        is_binary: false,
                        binary_data: Vec::new(),
                        path: ws_path_clone.clone(),
                        params: ws_params_clone.clone(),
                    },
                    done_tx,
                })
                .await;
            let _ = done_rx.await;
        }
    });

    Ok(response)
}

// ========================================
// WebSocket Ring Functions
// ========================================

/// bolt_ws_route(server, path, on_connect, on_message, on_disconnect)
ring_func!(bolt_ws_route, |p| {
    ring_check_paracount!(p, 5);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);
    ring_check_string!(p, 5);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let path = ring_get_string!(p, 2);
    let on_connect = ring_get_string!(p, 3);
    let on_message = ring_get_string!(p, 4);
    let on_disconnect = ring_get_string!(p, 5);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.add_ws_route(
            path,
            if on_connect.is_empty() {
                None
            } else {
                Some(on_connect.to_string())
            },
            if on_message.is_empty() {
                None
            } else {
                Some(on_message.to_string())
            },
            if on_disconnect.is_empty() {
                None
            } else {
                Some(on_disconnect.to_string())
            },
        );
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_ws_client_id(server) -> current WS event's client ID
ring_func!(bolt_ws_client_id, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.client_id);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_event_type(server) -> "connect", "message", or "disconnect"
ring_func!(bolt_ws_event_type, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.event_type);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_event_message(server) -> current WS message text
ring_func!(bolt_ws_event_message, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.message);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_event_is_binary(server) -> 1 if binary message, 0 otherwise
ring_func!(bolt_ws_event_is_binary, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_number!(p, if ctx.is_binary { 1.0 } else { 0.0 });
        } else {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_ws_event_binary(server) -> binary data as base64 string
ring_func!(bolt_ws_event_binary, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&ctx.binary_data);
            ring_ret_string!(p, &encoded);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_event_path(server) -> WS route path
ring_func!(bolt_ws_event_path, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.path);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_param(server, name) -> get WS route path parameter
ring_func!(bolt_ws_param, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "");
        return;
    }

    let name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.ws_event.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.params.get(name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_ws_send_to(server, client_id, message) -> send text to specific client
ring_func!(bolt_ws_send_to, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let client_id = ring_get_string!(p, 2);
    let message = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        if let Some(tx) = server.ws_clients.get(client_id) {
            match tx.send(WsOutMessage::Text(message.to_string())) {
                Ok(_) => ring_ret_number!(p, 1.0),
                Err(_) => ring_ret_number!(p, 0.0),
            }
        } else {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_ws_send_binary_to(server, client_id, base64_data) -> send binary to specific client
ring_func!(bolt_ws_send_binary_to, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let client_id = ring_get_string!(p, 2);
    let b64_data = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(b64_data) {
            Ok(d) => d,
            Err(_) => {
                ring_ret_number!(p, 0.0);
                return;
            }
        };
        if let Some(tx) = server.ws_clients.get(client_id) {
            match tx.send(WsOutMessage::Binary(data)) {
                Ok(_) => ring_ret_number!(p, 1.0),
                Err(_) => ring_ret_number!(p, 0.0),
            }
        } else {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_ws_close_client(server, client_id) -> close a specific client connection
ring_func!(bolt_ws_close_client, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let client_id = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        if let Some((_, tx)) = server.ws_clients.remove(client_id) {
            let _ = tx.send(WsOutMessage::Close);
            // clean up rooms
            if let Some(client_room_entry) = server.ws_client_rooms.remove(client_id) {
                let client_rooms = client_room_entry.1;
                for room in client_rooms {
                    if let Some(mut members) = server.ws_rooms.get_mut(&room) {
                        members.remove(client_id);
                        if members.is_empty() {
                            drop(members);
                            server.ws_rooms.remove(&room);
                        }
                    }
                }
            }
            ring_ret_number!(p, 1.0);
        } else {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_ws_client_list(server) -> JSON array of connected client IDs
ring_func!(bolt_ws_client_list, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "[]");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let ids: Vec<String> = server.ws_clients.iter().map(|e| e.key().clone()).collect();
        let json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
        ring_ret_string!(p, &json);
    }
});

// ========================================
// WebSocket Rooms
// ========================================

/// bolt_ws_room_join(server, room, client_id) -> join a room
ring_func!(bolt_ws_room_join, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let room = ring_get_string!(p, 2);
    let client_id = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        server
            .ws_rooms
            .entry(room.to_string())
            .or_default()
            .insert(client_id.to_string());
        server
            .ws_client_rooms
            .entry(client_id.to_string())
            .or_default()
            .insert(room.to_string());
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_ws_room_leave(server, room, client_id) -> leave a room
ring_func!(bolt_ws_room_leave, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let room = ring_get_string!(p, 2);
    let client_id = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        if let Some(mut members) = server.ws_rooms.get_mut(room) {
            members.remove(client_id);
            if members.is_empty() {
                drop(members);
                server.ws_rooms.remove(room);
            }
        }
        if let Some(mut rooms) = server.ws_client_rooms.get_mut(client_id) {
            rooms.remove(room);
            if rooms.is_empty() {
                drop(rooms);
                server.ws_client_rooms.remove(client_id);
            }
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_ws_room_broadcast(server, room, message) -> send text to all clients in a room
ring_func!(bolt_ws_room_broadcast, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let room = ring_get_string!(p, 2);
    let message = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let members: Vec<String> = server
            .ws_rooms
            .get(room)
            .map(|m| m.iter().cloned().collect())
            .unwrap_or_default();
        let mut count = 0usize;
        for member_id in members {
            if let Some(tx) = server.ws_clients.get(&member_id) {
                if tx.send(WsOutMessage::Text(message.to_string())).is_ok() {
                    count += 1;
                }
            }
        }
        ring_ret_number!(p, count as f64);
    }
});

/// bolt_ws_room_broadcast_binary(server, room, base64_data) -> send binary to all clients in a room
ring_func!(bolt_ws_room_broadcast_binary, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let room = ring_get_string!(p, 2);
    let b64_data = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(b64_data) {
            Ok(d) => d,
            Err(_) => {
                ring_ret_number!(p, 0.0);
                return;
            }
        };
        let members: Vec<String> = server
            .ws_rooms
            .get(room)
            .map(|m| m.iter().cloned().collect())
            .unwrap_or_default();
        let mut count = 0usize;
        for member_id in members {
            if let Some(tx) = server.ws_clients.get(&member_id) {
                if tx.send(WsOutMessage::Binary(data.clone())).is_ok() {
                    count += 1;
                }
            }
        }
        ring_ret_number!(p, count as f64);
    }
});

/// bolt_ws_room_members(server, room) -> JSON array of client IDs in a room
ring_func!(bolt_ws_room_members, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, "[]");
        return;
    }

    let room = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        if let Some(members) = server.ws_rooms.get(room) {
            let ids: Vec<&String> = members.iter().collect();
            let json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
            ring_ret_string!(p, &json);
        } else {
            ring_ret_string!(p, "[]");
        }
    }
});

/// bolt_ws_room_count(server, room) -> number of clients in a room
ring_func!(bolt_ws_room_count, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_number!(p, 0.0);
        return;
    }

    let room = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let count = server.ws_rooms.get(room).map(|m| m.len()).unwrap_or(0);
        ring_ret_number!(p, count as f64);
    }
});

/// bolt_ws_broadcast(server, message) - send to all WebSocket connections
ring_func!(bolt_ws_broadcast, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let message = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        match server.ws_broadcast_tx.send(message.to_string()) {
            Ok(count) => ring_ret_number!(p, count as f64),
            Err(_) => ring_ret_number!(p, -1.0),
        }
    }
});

/// bolt_ws_connection_count(server) -> number of active WebSocket connections
ring_func!(bolt_ws_connection_count, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let count = server.ws_clients.len();
        ring_ret_number!(p, count as f64);
    }
});
