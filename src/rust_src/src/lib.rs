// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

#![allow(unused_doc_comments)]
#![allow(clippy::manual_range_contains)]

use ring_lang_rs::*;

mod modules;
mod server;

pub use modules::base64::*;
pub use modules::crypto::*;
pub use modules::datetime::*;
pub use modules::env::*;
pub use modules::hash::*;
pub use modules::json::*;
pub use modules::sanitize::*;
pub use modules::validate::*;
pub use server::*;

// Type identifiers for Ring C pointers
pub const HTTP_SERVER_TYPE: &[u8] = b"HttpServer\0";
pub const HTTP_REQUEST_TYPE: &[u8] = b"HttpRequest\0";
pub const HTTP_RESPONSE_TYPE: &[u8] = b"HttpResponse\0";

// Register all functions with Ring
ring_libinit! {
    // Server functions
    "bolt_new" => bolt_new,
    "bolt_listen" => bolt_listen,
    "bolt_set_host" => bolt_set_host,
    "bolt_stop" => bolt_stop,
    "bolt_route" => bolt_route,
    "bolt_static" => bolt_static,
    "bolt_use" => bolt_use,
    "bolt_cors" => bolt_cors,
    "bolt_cors_origin" => bolt_cors_origin,

    // Response functions
    "bolt_respond" => bolt_respond,
    "bolt_respond_json" => bolt_respond_json,
    "bolt_respond_file" => bolt_respond_file,
    "bolt_respond_redirect" => bolt_respond_redirect,
    "bolt_respond_status" => bolt_respond_status,
    "bolt_respond_binary" => bolt_respond_binary,

    // Request info functions
    "bolt_req_request_id" => bolt_req_request_id,
    "bolt_req_method" => bolt_req_method,
    "bolt_req_path" => bolt_req_path,
    "bolt_req_param" => bolt_req_param,
    "bolt_req_query" => bolt_req_query,
    "bolt_req_header" => bolt_req_header,
    "bolt_req_body" => bolt_req_body,
    "bolt_req_form_field" => bolt_req_form_field,
    "bolt_req_client_ip" => bolt_req_client_ip,
    "bolt_req_handler" => bolt_req_handler,
    "bolt_req_cookie" => bolt_req_cookie,

    // Cookie functions
    "bolt_set_cookie" => bolt_set_cookie,
    "bolt_sign_cookie" => bolt_sign_cookie,
    "bolt_verify_cookie" => bolt_verify_cookie,

    // File upload functions
    "bolt_req_files_count" => bolt_req_files_count,
    "bolt_req_file" => bolt_req_file,
    "bolt_req_files" => bolt_req_files,
    "bolt_req_file_by_field" => bolt_req_file_by_field,
    "bolt_req_file_save" => bolt_req_file_save,

    // Session functions
    "bolt_session_set" => bolt_session_set,
    "bolt_session_get" => bolt_session_get,
    "bolt_session_delete" => bolt_session_delete,
    "bolt_session_clear" => bolt_session_clear,

    // Compression
    "bolt_compression" => bolt_compression,

    // Custom headers
    "bolt_set_header" => bolt_set_header,

    // Template engine
    "bolt_render_template" => bolt_render_template,
    "bolt_render_file" => bolt_render_file,

    // JWT Authentication
    "bolt_jwt_encode" => bolt_jwt_encode,
    "bolt_jwt_decode" => bolt_jwt_decode,
    "bolt_jwt_verify" => bolt_jwt_verify,

    // Logging
    "bolt_logging" => bolt_logging,
    "bolt_log" => bolt_log,
    "bolt_set_log_level" => bolt_set_log_level,

    // Rate Limiting
    "bolt_rate_limit" => bolt_rate_limit,
    "bolt_check_rate_limit" => bolt_check_rate_limit,
    "bolt_route_rate_limit" => bolt_route_rate_limit,

    // Error Handler
    "bolt_set_error_handler" => bolt_set_error_handler,

    // Per-Route Middleware
    "bolt_route_before" => bolt_route_before,
    "bolt_route_after" => bolt_route_after,

    // Basic Auth
    "bolt_basic_auth_decode" => bolt_basic_auth_decode,
    "bolt_basic_auth_encode" => bolt_basic_auth_encode,

    // Hashing & UUID
    "bolt_hash_sha256" => bolt_hash_sha256,
    "bolt_uuid" => bolt_uuid,

    // TLS/HTTPS
    "bolt_tls" => bolt_tls,

    // WebSocket
    "bolt_ws_route" => bolt_ws_route,
    "bolt_ws_broadcast" => bolt_ws_broadcast,
    "bolt_ws_connection_count" => bolt_ws_connection_count,

    // WebSocket Client IDs & Per-Client Send
    "bolt_ws_client_id" => bolt_ws_client_id,
    "bolt_ws_event_type" => bolt_ws_event_type,
    "bolt_ws_event_message" => bolt_ws_event_message,
    "bolt_ws_event_is_binary" => bolt_ws_event_is_binary,
    "bolt_ws_event_binary" => bolt_ws_event_binary,
    "bolt_ws_event_path" => bolt_ws_event_path,
    "bolt_ws_param" => bolt_ws_param,
    "bolt_ws_send_to" => bolt_ws_send_to,
    "bolt_ws_send_binary_to" => bolt_ws_send_binary_to,
    "bolt_ws_close_client" => bolt_ws_close_client,
    "bolt_ws_client_list" => bolt_ws_client_list,

    // WebSocket Rooms
    "bolt_ws_room_join" => bolt_ws_room_join,
    "bolt_ws_room_leave" => bolt_ws_room_leave,
    "bolt_ws_room_broadcast" => bolt_ws_room_broadcast,
    "bolt_ws_room_broadcast_binary" => bolt_ws_room_broadcast_binary,
    "bolt_ws_room_members" => bolt_ws_room_members,
    "bolt_ws_room_count" => bolt_ws_room_count,

    // Caching
    "bolt_cache_set" => bolt_cache_set,
    "bolt_cache_get" => bolt_cache_get,
    "bolt_cache_delete" => bolt_cache_delete,
    "bolt_cache_clear" => bolt_cache_clear,

    // Server Configuration
    "bolt_set_timeout" => bolt_set_timeout,
    "bolt_set_body_limit" => bolt_set_body_limit,
    "bolt_set_session_capacity" => bolt_set_session_capacity,
    "bolt_set_session_ttl" => bolt_set_session_ttl,
    "bolt_set_cache_capacity" => bolt_set_cache_capacity,
    "bolt_set_cache_ttl" => bolt_set_cache_ttl,
    "bolt_ip_whitelist" => bolt_ip_whitelist,
    "bolt_ip_blacklist" => bolt_ip_blacklist,
    "bolt_proxy_whitelist" => bolt_proxy_whitelist,

    // CSRF Protection
    "bolt_enable_csrf" => bolt_enable_csrf,
    "bolt_csrf_token" => bolt_csrf_token,
    "bolt_verify_csrf" => bolt_verify_csrf,

    "bolt_health_status" => bolt_health_status,

    // JSON Schema Validation
    "bolt_validate_json" => bolt_validate_json,
    "bolt_validate_json_errors" => bolt_validate_json_errors,

    // ETag
    "bolt_etag" => bolt_etag,

    // SSE (broadcast pattern)
    "bolt_sse_route" => bolt_sse_route,
    "bolt_sse_broadcast" => bolt_sse_broadcast,
    "bolt_sse_broadcast_event" => bolt_sse_broadcast_event,

    // Global Middleware
    "bolt_before" => bolt_before,
    "bolt_after" => bolt_after,

    // OpenAPI
    "bolt_openapi_spec" => bolt_openapi_spec,
    "bolt_openapi_route" => bolt_openapi_route,
    "bolt_openapi_info" => bolt_openapi_info,
    "bolt_route_describe" => bolt_route_describe,
    "bolt_route_tag" => bolt_route_tag,
    "bolt_add_constraint" => bolt_add_constraint,

    // Route Constraints (Regex validation)
    "bolt_validate_param" => bolt_validate_param,
    "bolt_validate_regex" => bolt_validate_regex,

    // URL Encoding & Time
    "bolt_urlencode" => bolt_urlencode,
    "bolt_urldecode" => bolt_urldecode,
    "bolt_unixtime" => bolt_unixtime,
    "bolt_unixtime_ms" => bolt_unixtime_ms,

    // Base64
    "bolt_base64_encode" => bolt_base64_encode,
    "bolt_base64_decode" => bolt_base64_decode,
    "bolt_base64_url_encode" => bolt_base64_url_encode,
    "bolt_base64_url_decode" => bolt_base64_url_decode,

    // JSON functions
    "bolt_json_encode" => bolt_json_encode,
    "bolt_json_decode" => bolt_json_decode,
    "bolt_json_pretty" => bolt_json_pretty,

    // Environment (.env)
    "bolt_env_load" => bolt_env_load,
    "bolt_env_load_file" => bolt_env_load_file,
    "bolt_env_get" => bolt_env_get,
    "bolt_env_set" => bolt_env_set,
    "bolt_env_get_or" => bolt_env_get_or,

    // Password hashing
    "bolt_hash_argon2" => bolt_hash_argon2,
    "bolt_verify_argon2" => bolt_verify_argon2,
    "bolt_hash_bcrypt" => bolt_hash_bcrypt,
    "bolt_verify_bcrypt" => bolt_verify_bcrypt,
    "bolt_hash_scrypt" => bolt_hash_scrypt,
    "bolt_verify_scrypt" => bolt_verify_scrypt,

    // Validation
    "bolt_validate_email" => bolt_validate_email,
    "bolt_validate_url" => bolt_validate_url,
    "bolt_validate_ip" => bolt_validate_ip,
    "bolt_validate_ipv4" => bolt_validate_ipv4,
    "bolt_validate_ipv6" => bolt_validate_ipv6,
    "bolt_validate_uuid" => bolt_validate_uuid,
    "bolt_validate_json_string" => bolt_validate_json_string,
    "bolt_validate_length" => bolt_validate_length,
    "bolt_validate_range" => bolt_validate_range,
    "bolt_validate_alpha" => bolt_validate_alpha,
    "bolt_validate_alphanumeric" => bolt_validate_alphanumeric,
    "bolt_validate_numeric" => bolt_validate_numeric,

    // Crypto (AES-GCM + HMAC)
    "bolt_aes_encrypt" => bolt_aes_encrypt,
    "bolt_aes_decrypt" => bolt_aes_decrypt,
    "bolt_hmac_sha256" => bolt_hmac_sha256,
    "bolt_hmac_verify" => bolt_hmac_verify,

    // DateTime
    "bolt_datetime_now" => bolt_datetime_now,
    "bolt_datetime_now_utc" => bolt_datetime_now_utc,
    "bolt_datetime_timestamp" => bolt_datetime_timestamp,
    "bolt_datetime_timestamp_ms" => bolt_datetime_timestamp_ms,
    "bolt_datetime_format" => bolt_datetime_format,
    "bolt_datetime_parse" => bolt_datetime_parse,
    "bolt_datetime_diff" => bolt_datetime_diff,
    "bolt_datetime_add_days" => bolt_datetime_add_days,
    "bolt_datetime_add_hours" => bolt_datetime_add_hours,

    // Sanitize
    "bolt_sanitize_html" => bolt_sanitize_html,
    "bolt_sanitize_strict" => bolt_sanitize_strict,
    "bolt_escape_html" => bolt_escape_html,

}
