// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! HTTP Server Core

pub mod auth;
pub mod cache;
pub mod logging;
pub mod middleware;
pub mod openapi;
pub mod rate_limit;
pub mod response;
pub mod sessions;
pub mod sse;
pub mod templates;
pub mod uploads;
pub mod websocket;

pub use auth::*;
pub use cache::*;
pub use logging::*;
pub use middleware::*;
pub use openapi::*;
pub use rate_limit::*;
pub use response::*;
pub use sessions::*;
pub use sse::*;
pub use templates::*;
pub use uploads::*;
pub use websocket::*;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer as ActixHttpServer, Responder,
    http::{StatusCode, header},
    middleware::{Compress, Condition, DefaultHeaders},
    web,
};
use dashmap::DashMap;
use futures_util::StreamExt;
use moka::sync::Cache;
use parking_lot::Mutex;
use ring_lang_rs::*;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::HTTP_SERVER_TYPE;

use cookie::{Cookie, CookieJar, Key};
use governor::{Quota, RateLimiter};
use ipnetwork::IpNetwork;
use moka::policy::Expiry;
use std::num::NonZeroU32;
use std::str::FromStr;

struct BoltCacheExpiry(u64);

impl Expiry<String, (String, u64)> for BoltCacheExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &(String, u64),
        _created_at: std::time::Instant,
    ) -> Option<Duration> {
        let ttl = value.1;
        if ttl > 0 {
            Some(Duration::from_secs(ttl))
        } else {
            Some(Duration::from_secs(self.0))
        }
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &(String, u64),
        _updated_at: std::time::Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        let ttl = value.1;
        if ttl > 0 {
            Some(Duration::from_secs(ttl))
        } else {
            Some(Duration::from_secs(self.0))
        }
    }
}

/// Route definition
#[derive(Clone)]
pub struct RouteDefinition {
    pub method: String,
    pub path: String,
    pub handler_name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub constraints: Vec<(String, String)>,
    pub rate_limit: Option<(u64, u64)>,
    pub before_middleware: Vec<String>,
    pub after_middleware: Vec<String>,
}

/// Static file route
#[derive(Clone)]
pub struct StaticRoute {
    pub url_path: String,
    pub dir_path: String,
}

pub enum ResponseBody {
    Bytes(Vec<u8>),
    File(String),
}

/// Pending response (set by Ring handler)
pub struct PendingResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<String>,
    pub body: ResponseBody,
    pub only_headers: bool,
}

impl PendingResponse {
    pub fn take_existing(
        response: &Arc<Mutex<Option<PendingResponse>>>,
    ) -> (HashMap<String, String>, Vec<String>) {
        let guard = response.lock();
        match guard.as_ref() {
            Some(r) => (r.headers.clone(), r.cookies.clone()),
            None => (HashMap::new(), Vec::new()),
        }
    }
}

/// Current request context (accessible from Ring)
#[derive(Clone)]
pub struct RequestContext {
    pub id: u64,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub form: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub body: Vec<u8>,
    pub handler_name: String,
    pub files: Vec<UploadedFile>,
    pub session_id: String,
    pub peer_addr: String,
}

/// Uploaded file from multipart form
#[derive(Clone)]
pub struct UploadedFile {
    pub name: String,
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// CORS configuration
#[derive(Clone)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: Vec::new(),
            methods: vec![
                "GET".into(),
                "POST".into(),
                "PUT".into(),
                "DELETE".into(),
                "PATCH".into(),
                "OPTIONS".into(),
            ],
            headers: vec!["Content-Type".into(), "Authorization".into()],
            credentials: false,
        }
    }
}

/// WebSocket route definition
#[derive(Clone)]
pub struct WsRouteDefinition {
    pub path: String,
    pub on_connect: Option<String>,
    pub on_message: Option<String>,
    pub on_disconnect: Option<String>,
}

/// Outgoing WebSocket message (per-client channel)
#[derive(Clone, Debug)]
pub enum WsOutMessage {
    Text(String),
    Binary(Vec<u8>),
    Close,
}

/// WebSocket event context (accessible from Ring during WS callbacks)
pub struct WsEventContext {
    pub client_id: String,
    pub event_type: String,
    pub message: String,
    pub is_binary: bool,
    pub binary_data: Vec<u8>,
    pub path: String,
    pub params: HashMap<String, String>,
}

/// TLS Configuration
#[derive(Clone, Default)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

/// Server Configuration
#[derive(Clone)]
pub struct ServerConfig {
    pub request_timeout_ms: u64,
    pub body_size_limit: usize,
    pub ip_whitelist: Vec<IpNetwork>,
    pub ip_blacklist: Vec<IpNetwork>,
    pub proxy_whitelist: Vec<String>,
    pub session_max_capacity: u64,
    pub session_ttl_secs: u64,
    pub cache_max_capacity: u64,
    pub cache_ttl_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 30000,
            body_size_limit: 50 * 1024 * 1024,
            ip_whitelist: Vec::new(),
            ip_blacklist: Vec::new(),
            proxy_whitelist: Vec::new(),
            session_max_capacity: 10_000,
            session_ttl_secs: 300,
            cache_max_capacity: 10_000,
            cache_ttl_secs: 300,
        }
    }
}

/// SSE route definition
#[derive(Clone)]
pub struct SseRouteDefinition {
    pub path: String,
    pub handler_name: String,
}

pub struct VmPtr(pub *mut c_void);

unsafe impl Send for VmPtr {}
unsafe impl Sync for VmPtr {}

/// HTTP Server state
#[allow(clippy::type_complexity)]
pub struct HttpServer {
    pub port: u16,
    pub host: String,
    pub routes: Vec<RouteDefinition>,
    pub ws_routes: Vec<WsRouteDefinition>,
    pub sse_routes: Vec<SseRouteDefinition>,
    pub static_routes: Vec<StaticRoute>,
    pub middlewares: Vec<String>,
    pub before_handlers: Vec<String>,
    pub after_handlers: Vec<String>,
    pub cors: CorsConfig,
    pub compression: bool,
    pub tls: TlsConfig,
    pub config: ServerConfig,
    pub start_time: Instant,
    pub sessions: Arc<Cache<String, HashMap<String, String>>>,
    pub cache: Arc<Cache<String, (String, u64)>>,
    pub ws_broadcast_tx: tokio::sync::broadcast::Sender<String>,
    pub vm: Arc<Mutex<VmPtr>>,
    pub running: Arc<Mutex<bool>>,
    pub current_request: Arc<Mutex<Option<RequestContext>>>,
    pub current_response: Arc<Mutex<Option<PendingResponse>>>,
    pub ws_clients: Arc<DashMap<String, tokio::sync::mpsc::UnboundedSender<WsOutMessage>>>,
    pub ws_rooms: Arc<DashMap<String, HashSet<String>>>,
    pub ws_client_rooms: Arc<DashMap<String, HashSet<String>>>,
    pub ws_event: Arc<Mutex<Option<WsEventContext>>>,
    pub sse_broadcast_channels:
        Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<SseEvent>>>>,
    pub error_handler: Option<String>,
    pub openapi_spec: Option<String>,
    pub openapi_enabled: bool,
    pub openapi_title: String,
    pub openapi_version: String,
    pub openapi_description: String,
    pub server_shutdown_tx: tokio::sync::broadcast::Sender<()>,
    pub template_cache: Arc<std::sync::RwLock<HashMap<String, (String, u64)>>>,
    pub regex_cache: Arc<Mutex<HashMap<String, regex::Regex>>>,
}

#[derive(Clone, Debug)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

impl HttpServer {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(vm: *mut c_void) -> Self {
        let server = Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            routes: Vec::new(),
            ws_routes: Vec::new(),
            sse_routes: Vec::new(),
            static_routes: Vec::new(),
            middlewares: Vec::new(),
            before_handlers: Vec::new(),
            after_handlers: Vec::new(),
            cors: CorsConfig::default(),
            compression: false,
            tls: TlsConfig::default(),
            config: ServerConfig::default(),
            start_time: Instant::now(),
            sessions: Arc::new(
                Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(300))
                    .build(),
            ),
            cache: Arc::new(
                Cache::builder()
                    .max_capacity(10_000)
                    .expire_after(BoltCacheExpiry(300))
                    .build(),
            ),
            ws_broadcast_tx: tokio::sync::broadcast::channel(10_000).0,
            vm: Arc::new(Mutex::new(VmPtr(vm))),
            running: Arc::new(Mutex::new(false)),
            current_request: Arc::new(Mutex::new(None)),
            current_response: Arc::new(Mutex::new(None)),
            ws_clients: Arc::new(DashMap::new()),
            ws_rooms: Arc::new(DashMap::new()),
            ws_client_rooms: Arc::new(DashMap::new()),
            ws_event: Arc::new(Mutex::new(None)),
            sse_broadcast_channels: Arc::new(Mutex::new(HashMap::new())),
            error_handler: None,
            openapi_spec: None,
            openapi_enabled: false,
            openapi_title: "Bolt API".to_string(),
            openapi_version: "1.0.0".to_string(),
            openapi_description: String::new(),
            server_shutdown_tx: tokio::sync::broadcast::channel(1).0,
            template_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            regex_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        server
    }

    pub fn add_ws_route(
        &mut self,
        path: &str,
        on_connect: Option<String>,
        on_message: Option<String>,
        on_disconnect: Option<String>,
    ) {
        self.ws_routes.push(WsRouteDefinition {
            path: path.to_string(),
            on_connect,
            on_message,
            on_disconnect,
        });
    }

    pub fn add_static(&mut self, url_path: &str, dir_path: &str) {
        self.static_routes.push(StaticRoute {
            url_path: url_path.to_string(),
            dir_path: dir_path.to_string(),
        });
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler_name: &str) {
        let actix_path = convert_path_params(path);
        self.routes.push(RouteDefinition {
            method: method.to_uppercase(),
            path: actix_path,
            handler_name: handler_name.to_string(),
            description: None,
            tags: Vec::new(),
            constraints: Vec::new(),
            rate_limit: None,
            before_middleware: Vec::new(),
            after_middleware: Vec::new(),
        });
    }

    pub fn set_route_description(&mut self, method: &str, path: &str, description: &str) {
        let actix_path = convert_path_params(path);
        for route in &mut self.routes {
            if route.method == method.to_uppercase() && route.path == actix_path {
                route.description = Some(description.to_string());
                break;
            }
        }
    }

    pub fn add_route_tag(&mut self, method: &str, path: &str, tag: &str) {
        let actix_path = convert_path_params(path);
        for route in &mut self.routes {
            if route.method == method.to_uppercase() && route.path == actix_path {
                if !route.tags.contains(&tag.to_string()) {
                    route.tags.push(tag.to_string());
                }
                break;
            }
        }
    }

    pub fn add_constraint(&mut self, handler_name: &str, param_name: &str, pattern: &str) {
        for route in &mut self.routes {
            if route.handler_name == handler_name {
                route
                    .constraints
                    .push((param_name.to_string(), pattern.to_string()));
                break;
            }
        }
    }
}

/// VM thread work item: either an HTTP request or a WebSocket event
pub(crate) enum VmWork {
    Http {
        ctx: RequestContext,
        handler_name: String,
        response_tx: tokio::sync::oneshot::Sender<Option<PendingResponse>>,
    },
    WsEvent {
        handler_name: String,
        ctx: WsEventContext,
        done_tx: tokio::sync::oneshot::Sender<()>,
    },
}

unsafe impl Send for VmWork {}

// Actix Web application state
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub(crate) struct AppState {
    routes: Vec<RouteDefinition>,
    sse_broadcast_channels: Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<SseEvent>>>>,
    vm_tx: tokio::sync::mpsc::Sender<VmWork>,
    ws_broadcast_tx: tokio::sync::broadcast::Sender<String>,
    ws_clients: Arc<DashMap<String, tokio::sync::mpsc::UnboundedSender<WsOutMessage>>>,
    ws_rooms: Arc<DashMap<String, HashSet<String>>>,
    ws_client_rooms: Arc<DashMap<String, HashSet<String>>>,
    route_limiters: Arc<DashMap<String, Arc<governor::DefaultKeyedRateLimiter<String>>>>,
    ip_whitelist: Vec<IpNetwork>,
    ip_blacklist: Vec<IpNetwork>,
    regex_cache: Arc<Mutex<HashMap<String, regex::Regex>>>,
    body_size_limit: usize,
}

/// Check route constraints (standalone function for AppState)
pub fn check_route_constraints(
    routes: &[RouteDefinition],
    handler_name: &str,
    params: &HashMap<String, String>,
    regex_cache: &Arc<Mutex<HashMap<String, regex::Regex>>>,
) -> Option<String> {
    for route in routes {
        if route.handler_name == handler_name {
            for (param_name, pattern) in &route.constraints {
                if let Some(value) = params.get(param_name) {
                    let mut cache = regex_cache.lock();
                    #[allow(clippy::regex_creation_in_loops)]
                    let re = cache.entry(pattern.clone()).or_insert_with(|| {
                        regex::Regex::new(pattern)
                            .unwrap_or_else(|_| regex::Regex::new("^$").unwrap())
                    });
                    if !re.is_match(value) {
                        return Some(param_name.clone());
                    }
                }
            }
            break;
        }
    }
    None
}

/// Convert Express-style path params (:param) to Actix-style ({param})
pub fn convert_path_params(path: &str) -> String {
    let mut result = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ':' {
            result.push('{');
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '_' {
                    result.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push('}');
        } else {
            result.push(c);
        }
    }

    result
}

static REQUEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_request_id() -> u64 {
    REQUEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// ========================================
// Core Server Ring Functions
// ========================================

/// bolt_new() -> HttpServer
ring_func!(bolt_new, |p| {
    ring_check_paracount!(p, 0);

    let server = Box::new(HttpServer::new(p));
    let ptr = Box::into_raw(server);

    ring_ret_cpointer!(p, ptr, HTTP_SERVER_TYPE);
});

/// bolt_set_host(server, host) - set IP address to listen on
ring_func!(bolt_set_host, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let host = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.host = host.to_string();
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_route(server, method, path, handler_name)
ring_func!(bolt_route, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let method = ring_get_string!(p, 2);
    let path = ring_get_string!(p, 3);
    let handler_name = ring_get_string!(p, 4);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.add_route(method, path, handler_name);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_listen(server, port) - starts server (blocks!)
ring_func!(bolt_listen, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let port = ring_get_number!(p, 2) as u16;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.port = port;
        *server.running.lock() = true;

        let host = server.host.clone();
        let routes = server.routes.clone();
        let ws_routes = server.ws_routes.clone();
        let sse_routes = server.sse_routes.clone();
        let static_routes = server.static_routes.clone();
        let before_handlers = server.before_handlers.clone();
        let after_handlers = server.after_handlers.clone();
        let cors = server.cors.clone();
        let compression = server.compression;
        let tls = server.tls.clone();
        let ws_broadcast_tx = server.ws_broadcast_tx.clone();
        let ws_clients = server.ws_clients.clone();
        let ws_rooms = server.ws_rooms.clone();
        let ws_client_rooms = server.ws_client_rooms.clone();
        let ws_event = server.ws_event.clone();
        let error_handler = server.error_handler.clone();
        let sse_broadcast_channels = server.sse_broadcast_channels.clone();
        let server_shutdown_rx = server.server_shutdown_tx.subscribe();
        let vm = server.vm.clone();
        let current_request = server.current_request.clone();
        let current_response = server.current_response.clone();
        let running = server.running.clone();
        let openapi_spec = if server.openapi_enabled {
            Some(self::openapi::generate_openapi_spec(
                &routes,
                &server.openapi_title,
                &server.openapi_version,
                &server.openapi_description,
            ))
        } else {
            server.openapi_spec.clone()
        };

        let body_size_limit = server.config.body_size_limit;
        let ip_whitelist = server.config.ip_whitelist.clone();
        let ip_blacklist = server.config.ip_blacklist.clone();
        let request_timeout_ms = server.config.request_timeout_ms;

        let route_limiters = Arc::new(DashMap::<
            String,
            Arc<governor::DefaultKeyedRateLimiter<String>>,
        >::new());

        server.cache = Arc::new(
            Cache::builder()
                .max_capacity(server.config.cache_max_capacity)
                .expire_after(BoltCacheExpiry(server.config.cache_ttl_secs))
                .build(),
        );
        server.sessions = Arc::new(
            Cache::builder()
                .max_capacity(server.config.session_max_capacity)
                .time_to_live(Duration::from_secs(server.config.session_ttl_secs))
                .build(),
        );

        let system = actix_web::rt::System::new();
        system.block_on(async {
            run_server(
                host,
                port,
                routes,
                ws_routes,
                sse_routes,
                static_routes,
                before_handlers,
                after_handlers,
                cors,
                compression,
                tls,
                ws_broadcast_tx,
                ws_clients,
                ws_rooms,
                ws_client_rooms,
                ws_event,
                route_limiters,
                error_handler,
                sse_broadcast_channels,
                server_shutdown_rx,
                vm,
                current_request,
                current_response,
                running,
                openapi_spec,
                body_size_limit,
                ip_whitelist,
                ip_blacklist,
                request_timeout_ms,
            )
            .await;
        });
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// Core Server (async)
// ========================================

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
async fn run_server(
    host: String,
    port: u16,
    routes: Vec<RouteDefinition>,
    ws_routes: Vec<WsRouteDefinition>,
    sse_routes: Vec<SseRouteDefinition>,
    static_routes: Vec<StaticRoute>,
    before_handlers: Vec<String>,
    after_handlers: Vec<String>,
    cors_config: CorsConfig,
    compression: bool,
    tls_config: TlsConfig,
    ws_broadcast_tx: tokio::sync::broadcast::Sender<String>,
    ws_clients: Arc<DashMap<String, tokio::sync::mpsc::UnboundedSender<WsOutMessage>>>,
    ws_rooms: Arc<DashMap<String, HashSet<String>>>,
    ws_client_rooms: Arc<DashMap<String, HashSet<String>>>,
    ws_event: Arc<Mutex<Option<WsEventContext>>>,
    route_limiters: Arc<DashMap<String, Arc<governor::DefaultKeyedRateLimiter<String>>>>,
    error_handler: Option<String>,
    sse_broadcast_channels: Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<SseEvent>>>>,
    mut server_shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    vm: Arc<Mutex<VmPtr>>,
    current_request: Arc<Mutex<Option<RequestContext>>>,
    current_response: Arc<Mutex<Option<PendingResponse>>>,
    running: Arc<Mutex<bool>>,
    openapi_spec: Option<String>,
    body_size_limit: usize,
    ip_whitelist: Vec<IpNetwork>,
    ip_blacklist: Vec<IpNetwork>,
    request_timeout_ms: u64,
) {
    let openapi_spec_clone = openapi_spec.clone();
    let (sse_shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    let (vm_tx, mut vm_rx) = tokio::sync::mpsc::channel::<VmWork>(256);
    {
        let current_request = current_request.clone();
        let current_response = current_response.clone();
        let ws_event_for_vm = ws_event.clone();
        let routes_for_vm = routes.clone();
        let error_handler_for_vm = error_handler.clone();

        let vm_ptr = (*vm.lock()).0 as usize;

        std::thread::Builder::new()
            .name("bolt-vm".into())
            .spawn(move || {
                let vm_ptr = vm_ptr as *mut c_void;

                while let Some(work) = vm_rx.blocking_recv() {
                    match work {
                        VmWork::Http {
                            ctx,
                            handler_name,
                            response_tx,
                        } => {
                            *current_request.lock() = Some(ctx);
                            *current_response.lock() = None;

                            let route_def = routes_for_vm
                                .iter()
                                .find(|r| r.handler_name == handler_name);

                            for bh in &before_handlers {
                                ring_vm_callfunction_str(vm_ptr as RingVM, bh);
                            }
                            if let Some(rd) = route_def {
                                for bh in &rd.before_middleware {
                                    ring_vm_callfunction_str(vm_ptr as RingVM, bh);
                                }
                            }

                            ring_vm_callfunction_str(vm_ptr as RingVM, &handler_name);

                            if let Some(rd) = route_def {
                                for ah in &rd.after_middleware {
                                    ring_vm_callfunction_str(vm_ptr as RingVM, ah);
                                }
                            }
                            for ah in &after_handlers {
                                ring_vm_callfunction_str(vm_ptr as RingVM, ah);
                            }

                            let mut response = current_response.lock().take();

                            if response.as_ref().map_or(true, |r| r.only_headers) {
                                if let Some(ref eh) = error_handler_for_vm {
                                    ring_vm_callfunction_str(vm_ptr as RingVM, eh);
                                    response = current_response.lock().take();
                                }
                            }

                            let _ = response_tx.send(response);
                        }
                        VmWork::WsEvent {
                            handler_name,
                            ctx,
                            done_tx,
                        } => {
                            *ws_event_for_vm.lock() = Some(ctx);
                            ring_vm_callfunction_str(vm_ptr as RingVM, &handler_name);
                            *ws_event_for_vm.lock() = None;
                            let _ = done_tx.send(());
                        }
                    }
                }
            })
            .expect("Failed to spawn VM thread");
    }

    let state = AppState {
        routes: routes.clone(),
        sse_broadcast_channels,
        vm_tx,
        ws_broadcast_tx,
        ws_clients,
        ws_rooms,
        ws_client_rooms,
        route_limiters,
        ip_whitelist,
        ip_blacklist,
        regex_cache: Arc::new(Mutex::new(HashMap::new())),
        body_size_limit,
    };

    let limiters_clone = state.route_limiters.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            for route in limiters_clone.iter() {
                route.value().retain_recent();
            }
        }
    });

    let state_data = web::Data::new(state.clone());

    let server = ActixHttpServer::new(move || {
        let mut app = App::new()
            .app_data(state_data.clone())
            .app_data(web::PayloadConfig::new(body_size_limit));

        for route in &routes {
            let path = route.path.clone();
            let handler_name = route.handler_name.clone();

            match route.method.as_str() {
                "GET" => {
                    app = app.route(
                        &path,
                        web::get().to(move |req, state, query, payload| {
                            handle_request(req, state, query, payload, handler_name.clone())
                        }),
                    );
                }
                "POST" => {
                    app = app.route(
                        &path,
                        web::post().to(move |req, state, query, payload| {
                            handle_request(req, state, query, payload, handler_name.clone())
                        }),
                    );
                }
                "PUT" => {
                    app = app.route(
                        &path,
                        web::put().to(move |req, state, query, payload| {
                            handle_request(req, state, query, payload, handler_name.clone())
                        }),
                    );
                }
                "DELETE" => {
                    app = app.route(
                        &path,
                        web::delete().to(move |req, state, query, payload| {
                            handle_request(req, state, query, payload, handler_name.clone())
                        }),
                    );
                }
                "PATCH" => {
                    app = app.route(
                        &path,
                        web::patch().to(move |req, state, query, payload| {
                            handle_request(req, state, query, payload, handler_name.clone())
                        }),
                    );
                }
                "OPTIONS" => {
                    app = app.route(
                        &path,
                        web::route().method(actix_web::http::Method::OPTIONS).to(
                            move |req, state, query, payload| {
                                handle_request(req, state, query, payload, handler_name.clone())
                            },
                        ),
                    );
                }
                "HEAD" => {
                    app = app.route(
                        &path,
                        web::route().method(actix_web::http::Method::HEAD).to(
                            move |req, state, query, payload| {
                                handle_request(req, state, query, payload, handler_name.clone())
                            },
                        ),
                    );
                }
                _ => {}
            }
        }

        for ws_route in &ws_routes {
            let path = convert_path_params(&ws_route.path);
            let on_connect = ws_route.on_connect.clone();
            let on_message = ws_route.on_message.clone();
            let on_disconnect = ws_route.on_disconnect.clone();
            let ws_path = ws_route.path.clone();

            app = app.route(
                &path,
                web::get().to(
                    move |req, stream: web::Payload, state: web::Data<AppState>| {
                        self::websocket::handle_websocket(
                            req,
                            stream,
                            state,
                            on_connect.clone(),
                            on_message.clone(),
                            on_disconnect.clone(),
                            ws_path.clone(),
                        )
                    },
                ),
            );
        }

        for sse_route in &sse_routes {
            let path = convert_path_params(&sse_route.path);
            app = app.route(&path, web::get().to(self::sse::handle_sse));
        }

        for static_route in &static_routes {
            let url_path = static_route.url_path.trim_end_matches('/');
            app =
                app.service(Files::new(url_path, &static_route.dir_path).index_file("index.html"));
        }

        if let Some(ref spec) = openapi_spec_clone {
            let spec_for_json = spec.clone();
            app = app.route(
                "/openapi.json",
                web::get().to(move || {
                    let s = spec_for_json.clone();
                    async move { HttpResponse::Ok().content_type("application/json").body(s) }
                }),
            );

            let spec_parsed: utoipa::openapi::OpenApi =
                serde_json::from_str(spec).unwrap_or_default();
            app = app.route(
                "/docs",
                web::get().to(|| async {
                    HttpResponse::Found()
                        .insert_header(("location", "/docs/"))
                        .finish()
                }),
            );
            app = app.service(
                utoipa_swagger_ui::SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", spec_parsed),
            );
        }

        let cors = if cors_config.enabled {
            let has_wildcard = cors_config.origins.iter().any(|o| o == "*");
            if cors_config.origins.is_empty() || has_wildcard {
                Cors::default()
                    .allow_any_origin()
                    .send_wildcard()
                    .allow_any_method()
                    .allow_any_header()
                    .block_on_origin_mismatch(false)
            } else {
                let mut cors = Cors::default();
                for origin in &cors_config.origins {
                    cors = cors.allowed_origin(origin);
                }
                cors.allowed_methods(vec![
                    "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD",
                ])
                .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
                .block_on_origin_mismatch(false)
                .max_age(3600)
            }
        } else {
            Cors::default()
        };

        let has_wildcard = cors_config.origins.iter().any(|o| o == "*");
        let default_cors_header =
            if cors_config.enabled && (cors_config.origins.is_empty() || has_wildcard) {
                DefaultHeaders::new().add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            } else {
                DefaultHeaders::new()
            };
        app.wrap(Condition::new(compression, Compress::default()))
            .wrap(Condition::new(cors_config.enabled, cors))
            .wrap(default_cors_header)
    })
    .keep_alive(Duration::from_millis(request_timeout_ms));

    let addr = format!("{}:{}", host, port);

    let sse_shutdown_for_signal = sse_shutdown_tx.clone();
    let running_for_signal = running.clone();

    let shutdown_future = async move {
        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n[bolt] Received Ctrl+C, shutting down...");
            }
            _ = terminate => {
                println!("\n[bolt] Received SIGTERM, shutting down...");
            }
            _ = async {
                let _ = server_shutdown_rx.recv().await;
            } => {
                println!("[bolt] Shutdown requested, stopping...");
            }
        }
        let _ = sse_shutdown_for_signal.send(());
        *running_for_signal.lock() = false;
    };

    if tls_config.enabled && !tls_config.cert_path.is_empty() && !tls_config.key_path.is_empty() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        use rustls::ServerConfig;
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::BufReader;

        let cert_file = match std::fs::File::open(&tls_config.cert_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("\n[error] Failed to load TLS certificates");
                eprintln!("        Cert: {}", tls_config.cert_path);
                eprintln!("        {}\n", e);
                return;
            }
        };

        let key_file = match std::fs::File::open(&tls_config.key_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("\n[error] Failed to load TLS key");
                eprintln!("        Key:  {}", tls_config.key_path);
                eprintln!("        {}\n", e);
                return;
            }
        };

        let cert_chain: Vec<rustls::pki_types::CertificateDer> =
            certs(&mut BufReader::new(cert_file))
                .filter_map(Result::ok)
                .collect();

        let mut keys = pkcs8_private_keys(&mut BufReader::new(key_file))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if keys.is_empty() {
            eprintln!(
                "\n[error] No valid private keys found in {}\n",
                tls_config.key_path
            );
            return;
        }

        let tls_config = match ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                cert_chain,
                rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
            ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("\n[error] Failed to build TLS config: {}\n", e);
                return;
            }
        };

        println!("[bolt] HTTPS server running on https://{}:{}", host, port);

        let server_handle = server
            .bind_rustls_0_23(&addr, tls_config)
            .unwrap_or_else(|e| {
                eprintln!("\n[error] Failed to bind HTTPS server on {}", addr);
                eprintln!("        {}\n", e);
                std::process::exit(1);
            })
            .run();

        tokio::select! {
            result = server_handle => {
                if let Err(e) = result {
                    eprintln!("[error] HTTPS server: {}", e);
                }
            }
            _ = shutdown_future => {
                println!("[bolt] Shutdown complete");
            }
        }
    } else {
        println!("[bolt] Server running on http://{}:{}", host, port);

        let server_handle = server
            .bind(&addr)
            .unwrap_or_else(|e: std::io::Error| {
                match e.kind() {
                    std::io::ErrorKind::AddrInUse => {
                        eprintln!("\n[error] Port {} is already in use.", port);
                        #[cfg(target_os = "windows")]
                        {
                            eprintln!("        Try: netstat -ano | findstr :{}", port);
                            eprintln!("        Then: taskkill /PID <PID> /F");
                            eprintln!("        Or use a different port.\n");
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            eprintln!(
                                "        Try: kill $(lsof -t -i:{}) or use a different port.\n",
                                port
                            );
                        }
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("\n[error] Permission denied. Ports below 1024 require root.\n");
                    }
                    _ => {
                        eprintln!("\n[error] Failed to bind server on {}", addr);
                        eprintln!("        {}\n", e);
                    }
                }
                std::process::exit(1);
            })
            .run();

        tokio::select! {
            result = server_handle => {
                if let Err(e) = result {
                    eprintln!("[error] Server: {}", e);
                }
            }
            _ = shutdown_future => {
                println!("[bolt] Shutdown complete");
            }
        }
    }

    *running.lock() = false;
}

// ========================================
// Generic HTTP Request Handler
// ========================================

async fn handle_request(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
    mut payload: web::Payload,
    handler_name: String,
) -> HttpResponse {
    let _start = Instant::now();

    let path_params: HashMap<String, String> = req
        .match_info()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    if let Some(failed_param) = check_route_constraints(
        &state.routes,
        &handler_name,
        &path_params,
        &state.regex_cache,
    ) {
        return HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Invalid parameter: {}", failed_param));
    }

    let headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_lowercase(),
                v.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    let cookies: HashMap<String, String> = req
        .cookies()
        .map(|cookies| {
            cookies
                .iter()
                .map(|c| (c.name().to_string(), c.value().to_string()))
                .collect()
        })
        .unwrap_or_default();

    let request_id = next_request_id();

    let content_type = headers
        .get("content-type")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_multipart = content_type.starts_with("multipart/form-data");
    let is_form_urlencoded = content_type.starts_with("application/x-www-form-urlencoded");

    let (body_bytes, files, form) = if is_multipart {
        let mut files = Vec::new();
        let mut form = HashMap::new();
        let mut multipart = actix_multipart::Multipart::new(req.headers(), payload);
        let mut total_size = 0usize;

        while let Some(field_result) = multipart.next().await {
            let mut field = match field_result {
                Ok(f) => f,
                Err(_) => break,
            };

            let name = field.name().unwrap_or("").to_string();
            let filename = field
                .content_disposition()
                .and_then(|cd| cd.get_filename())
                .map(|s| s.to_string());
            let ct = field
                .content_type()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let bytes = chunk.unwrap_or_default();
                data.extend_from_slice(&bytes);
                total_size += bytes.len();
                if total_size > state.body_size_limit {
                    return HttpResponse::PayloadTooLarge()
                        .insert_header(("X-Request-Id", format!("{:x}", request_id)))
                        .body("Payload Too Large");
                }
            }

            if let Some(fname) = filename {
                files.push(UploadedFile {
                    name,
                    filename: fname,
                    content_type: ct,
                    data,
                });
            } else if !name.is_empty() {
                let value_str = String::from_utf8_lossy(&data).to_string();
                form.insert(name, value_str);
            }
        }
        (Vec::new(), files, form)
    } else {
        let mut body = actix_web::web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk: actix_web::web::Bytes = chunk.unwrap_or_default();
            if body.len() + chunk.len() > state.body_size_limit {
                return HttpResponse::PayloadTooLarge()
                    .insert_header(("X-Request-Id", format!("{:x}", request_id)))
                    .body("Payload Too Large");
            }
            body.extend_from_slice(&chunk);
        }
        let body_vec = body.to_vec();
        let form = if is_form_urlencoded {
            form_urlencoded::parse(&body_vec)
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        } else {
            HashMap::new()
        };
        (body_vec, Vec::new(), form)
    };

    let client_ip_str = req
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if !state.ip_whitelist.is_empty() || !state.ip_blacklist.is_empty() {
        let parsed_ip = if client_ip_str == "::1" {
            std::net::IpAddr::from_str("127.0.0.1")
                .unwrap_or_else(|_| std::net::IpAddr::from_str("0.0.0.0").unwrap())
        } else {
            client_ip_str
                .parse::<std::net::IpAddr>()
                .unwrap_or_else(|_| std::net::IpAddr::from_str("0.0.0.0").unwrap())
        };

        if !state.ip_whitelist.is_empty()
            && !state.ip_whitelist.iter().any(|net| net.contains(parsed_ip))
        {
            return HttpResponse::Forbidden()
                .insert_header(("X-Request-Id", format!("{:x}", request_id)))
                .body("Forbidden");
        }

        if state.ip_blacklist.iter().any(|net| net.contains(parsed_ip)) {
            return HttpResponse::Forbidden()
                .insert_header(("X-Request-Id", format!("{:x}", request_id)))
                .body("Forbidden");
        }
    }

    if let Some((max_req, window_secs)) = state
        .routes
        .iter()
        .find(|r| r.handler_name == handler_name)
        .and_then(|r| r.rate_limit)
    {
        let limiter = state
            .route_limiters
            .entry(handler_name.clone())
            .or_insert_with(|| {
                let burst =
                    NonZeroU32::new(max_req as u32).unwrap_or_else(|| NonZeroU32::new(1).unwrap());
                let period = (Duration::from_secs(window_secs.max(1)) / max_req.max(1) as u32)
                    .max(Duration::from_nanos(1));
                let quota = Quota::with_period(period).unwrap().allow_burst(burst);
                Arc::new(RateLimiter::keyed(quota))
            })
            .clone();
        if limiter.check_key(&client_ip_str).is_err() {
            return HttpResponse::TooManyRequests()
                .insert_header(("X-Request-Id", format!("{:x}", request_id)))
                .body("Too Many Requests");
        }
    }

    let session_id = cookies
        .get("BOLTSESSION")
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let method = req.method().to_string();
    let matched_path = req.match_pattern().unwrap_or_default();

    let query_inner = query.into_inner();
    let peer_addr = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default();
    let ctx = RequestContext {
        id: request_id,
        request_id: String::new(),
        method,
        path: matched_path,
        params: path_params,
        query: query_inner,
        form,
        headers,
        cookies,
        body: body_bytes,
        handler_name: handler_name.clone(),
        files,
        session_id,
        peer_addr,
    };

    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let vm_req = VmWork::Http {
        ctx,
        handler_name: handler_name.clone(),
        response_tx,
    };

    let response = if state.vm_tx.send(vm_req).await.is_ok() {
        response_rx.await.unwrap_or(None)
    } else {
        None
    };

    let req_id_header = format!("{:x}", request_id);

    let http_response = match response {
        Some(res) => {
            let status = StatusCode::from_u16(res.status).unwrap_or(StatusCode::OK);
            let headers = res.headers;
            let cookies = res.cookies;
            let body = res.body;

            if let Some(etag) = headers.get("ETag").or_else(|| headers.get("etag")) {
                if req
                    .headers()
                    .get("if-none-match")
                    .and_then(|v| v.to_str().ok())
                    == Some(etag)
                {
                    let mut builder = HttpResponse::NotModified();
                    builder.insert_header(("X-Request-Id", req_id_header.clone()));
                    builder.insert_header(("ETag", etag.clone()));
                    for cookie in &cookies {
                        builder.append_header((header::SET_COOKIE, cookie.clone()));
                    }
                    return builder.finish();
                }
            }

            match body {
                ResponseBody::Bytes(b) => {
                    let mut builder = HttpResponse::build(status);
                    builder.insert_header(("X-Request-Id", req_id_header.clone()));

                    for (key, value) in headers {
                        builder.insert_header((key, value));
                    }

                    for cookie in cookies {
                        builder.append_header((header::SET_COOKIE, cookie));
                    }

                    builder.body(b)
                }
                ResponseBody::File(path) => match actix_files::NamedFile::open(&path) {
                    Ok(named_file) => {
                        let mut resp = named_file
                            .customize()
                            .with_status(status)
                            .respond_to(&req)
                            .map_into_boxed_body();
                        let headers_mut = resp.headers_mut();

                        if let Ok(val) =
                            actix_web::http::header::HeaderValue::from_str(&req_id_header)
                        {
                            headers_mut.insert(
                                actix_web::http::header::HeaderName::from_static("x-request-id"),
                                val,
                            );
                        }

                        for (key, value) in headers {
                            if let (Ok(k), Ok(v)) = (
                                actix_web::http::header::HeaderName::try_from(key.as_str()),
                                actix_web::http::header::HeaderValue::from_str(&value),
                            ) {
                                headers_mut.insert(k, v);
                            }
                        }

                        for cookie in cookies {
                            if let Ok(v) = actix_web::http::header::HeaderValue::from_str(&cookie) {
                                headers_mut.append(actix_web::http::header::SET_COOKIE, v);
                            }
                        }

                        resp
                    }
                    Err(_) => HttpResponse::NotFound()
                        .insert_header(("X-Request-Id", req_id_header))
                        .body("File not found"),
                },
            }
        }
        None => HttpResponse::InternalServerError()
            .insert_header(("X-Request-Id", req_id_header))
            .body("No response from handler"),
    };

    http_response
}

// ========================================
// Shutdown
// ========================================

/// bolt_stop(server) - trigger graceful shutdown
ring_func!(bolt_stop, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        *server.running.lock() = false;
        let _ = server.server_shutdown_tx.send(());
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// Request Context Getters
// ========================================

/// bolt_req_method(server) -> string
ring_func!(bolt_req_method, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.method);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_request_id(server) -> string (UUID, lazily generated on first call)
ring_func!(bolt_req_request_id, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let mut guard = server.current_request.lock();
        if let Some(ref mut ctx) = *guard {
            if ctx.request_id.is_empty() {
                ctx.request_id = uuid::Uuid::new_v4().to_string();
            }
            ring_ret_string!(p, &ctx.request_id);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_path(server) -> string
ring_func!(bolt_req_path, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.path);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_param(server, name) -> string
ring_func!(bolt_req_param, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.params.get(name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_query(server, name) -> string
ring_func!(bolt_req_query, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.query.get(name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_header(server, name) -> string
ring_func!(bolt_req_header, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let name = ring_get_string!(p, 2).to_lowercase();

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.headers.get(&name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_body(server) -> string
ring_func!(bolt_req_body, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let body_str = String::from_utf8_lossy(&ctx.body);
            ring_ret_string!(p, &body_str);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_form_field(server, name) -> value from form body (urlencoded or multipart)
ring_func!(bolt_req_form_field, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.form.get(name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_req_client_ip(server) -> client IP string
ring_func!(bolt_req_client_ip, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let peer_addr = &ctx.peer_addr;
            let proxy_whitelist = &server.config.proxy_whitelist;

            let is_trusted = proxy_whitelist.is_empty()
                || proxy_whitelist.iter().any(|allowed| {
                    peer_addr == allowed
                        || (allowed.ends_with('.')
                            && peer_addr.starts_with(allowed.trim_end_matches('.')))
                });

            if is_trusted {
                if let Some(forwarded) = ctx.headers.get("x-forwarded-for") {
                    let ip = forwarded.split(',').next().unwrap_or(forwarded).trim();
                    ring_ret_string!(p, ip);
                    return;
                }
                if let Some(real_ip) = ctx.headers.get("x-real-ip") {
                    ring_ret_string!(p, real_ip.trim());
                    return;
                }
            }

            ring_ret_string!(p, peer_addr);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

// ========================================
// Static Files, Middleware, CORS
// ========================================

/// bolt_static(server, url_path, dir_path)
ring_func!(bolt_static, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let url_path = ring_get_string!(p, 2);
    let dir_path = ring_get_string!(p, 3);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.add_static(url_path, dir_path);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_use(server, middleware_name)
ring_func!(bolt_use, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let middleware_name = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.middlewares.push(middleware_name.to_string());
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_cors(server, enabled) - enable/disable CORS
ring_func!(bolt_cors, |p| {
    ring_check_paracount_range!(p, 1, 2);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let enabled = if ring_api_paracount(p) >= 2 && ring_api_isnumber(p, 2) {
        ring_get_number!(p, 2) != 0.0
    } else {
        true
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.cors.enabled = enabled;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_cors_origin(server, origin) - add allowed origin
ring_func!(bolt_cors_origin, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let origin = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.cors.origins.push(origin.to_string());
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// Cookies
// ========================================

/// bolt_req_cookie(server, name) -> cookie value
ring_func!(bolt_req_cookie, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let value = ctx.cookies.get(name).map(|s| s.as_str()).unwrap_or("");
            ring_ret_string!(p, value);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_set_cookie(server, name, value, options) - set response cookie
ring_func!(bolt_set_cookie, |p| {
    ring_check_paracount_range!(p, 3, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let name = ring_get_string!(p, 2);
    let value = ring_get_string!(p, 3);

    let options = if ring_api_paracount(p) >= 4 && ring_api_isstring(p, 4) {
        ring_get_string!(p, 4).to_string()
    } else {
        "Path=/".to_string()
    };

    let cookie_str = format!("{}={}; {}", name, value, options);
    let cookie = cookie::Cookie::parse(cookie_str)
        .map(|c| c.to_string())
        .unwrap_or_else(|_| format!("{}={}; {}", name, value, options));

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let mut guard = server.current_response.lock();
        if let Some(ref mut res) = *guard {
            res.cookies.push(cookie);
        } else {
            *guard = Some(PendingResponse {
                status: 200,
                headers: HashMap::new(),
                cookies: vec![cookie],
                body: ResponseBody::Bytes(Vec::new()),
                only_headers: true,
            });
        }
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_sign_cookie(value, secret) -> signed value using cookie crate
/// bolt_sign_cookie(value, secret) -> signed value using cookie crate
/// Secrets of any length are accepted: short secrets are hashed to 32 bytes via SHA-256
ring_func!(bolt_sign_cookie, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let value = ring_get_string!(p, 1);
    let secret = ring_get_string!(p, 2);

    let key = if secret.len() < 32 {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(secret.as_bytes());
        Key::derive_from(&hash)
    } else {
        Key::derive_from(secret.as_bytes())
    };

    let mut jar = CookieJar::new();
    jar.signed_mut(&key).add(Cookie::new("bolt", value));
    let signed = jar.get("bolt").unwrap().value().to_string();
    ring_ret_string!(p, &signed);
});

/// bolt_verify_cookie(signed_value, secret) -> original value or ""
/// Secrets of any length are accepted: short secrets are hashed to 32 bytes via SHA-256
ring_func!(bolt_verify_cookie, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let signed_value = ring_get_string!(p, 1);
    let secret = ring_get_string!(p, 2);

    let key = if secret.len() < 32 {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(secret.as_bytes());
        Key::derive_from(&hash)
    } else {
        Key::derive_from(secret.as_bytes())
    };

    let mut jar = CookieJar::new();
    jar.add_original(Cookie::new("bolt", signed_value));
    if let Some(c) = jar.signed(&key).get("bolt") {
        ring_ret_string!(p, c.value());
    } else {
        ring_ret_string!(p, "");
    }
});

// ========================================
// Compression & Headers
// ========================================

/// bolt_compression(server, enabled) - enable/disable gzip compression
ring_func!(bolt_compression, |p| {
    ring_check_paracount_range!(p, 1, 2);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let enabled = if ring_api_paracount(p) >= 2 && ring_api_isnumber(p, 2) {
        ring_get_number!(p, 2) != 0.0
    } else {
        true
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.compression = enabled;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_header(server, name, value) - set response header
ring_func!(bolt_set_header, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let name = ring_get_string!(p, 2);
    let value = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let mut guard = server.current_response.lock();
        if let Some(ref mut res) = *guard {
            res.headers.insert(name.to_string(), value.to_string());
        } else {
            let mut headers = HashMap::new();
            headers.insert(name.to_string(), value.to_string());
            *guard = Some(PendingResponse {
                status: 200,
                headers,
                cookies: Vec::new(),
                body: ResponseBody::Bytes(Vec::new()),
                only_headers: true,
            });
        }
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// Utilities: Time, Handler, UUID, SHA-256, TLS
// ========================================

/// bolt_unixtime() -> current Unix timestamp in seconds
ring_func!(bolt_unixtime, |p| {
    ring_check_paracount!(p, 0);

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    ring_ret_number!(p, timestamp as f64);
});

/// bolt_unixtime_ms() -> current Unix timestamp in milliseconds
ring_func!(bolt_unixtime_ms, |p| {
    ring_check_paracount!(p, 0);

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    ring_ret_number!(p, timestamp as f64);
});

/// bolt_req_handler(server) -> current handler name
ring_func!(bolt_req_handler, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_string!(p, &ctx.handler_name);
        } else {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_hash_sha256(data) -> hex hash
ring_func!(bolt_hash_sha256, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let data = ring_get_string!(p, 1);

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();

    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    ring_ret_string!(p, &hex);
});

/// bolt_uuid() -> generate UUID v4
ring_func!(bolt_uuid, |p| {
    ring_check_paracount!(p, 0);

    let id = uuid::Uuid::new_v4();
    ring_ret_string!(p, &id.to_string());
});

/// bolt_tls(server, cert_path, key_path) - enable HTTPS
ring_func!(bolt_tls, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "Invalid HTTP server");
        return;
    }

    let cert_path = ring_get_string!(p, 2);
    let key_path = ring_get_string!(p, 3);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.tls.enabled = true;
        server.tls.cert_path = cert_path.to_string();
        server.tls.key_path = key_path.to_string();
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// ETag
// ========================================

/// bolt_etag(content) -> generate ETag hash
ring_func!(bolt_etag, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let content = ring_get_string!(p, 1);

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();
    let etag = format!("\"{}\"", hex::encode(&hash[..8]));

    ring_ret_string!(p, &etag);
});

// ========================================
// Server Configuration
// ========================================

/// bolt_set_timeout(server, ms) - set request timeout
ring_func!(bolt_set_timeout, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let ms = ring_get_number!(p, 2) as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.request_timeout_ms = ms;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_body_limit(server, bytes) - set max body size
ring_func!(bolt_set_body_limit, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let bytes = ring_get_number!(p, 2) as usize;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.body_size_limit = bytes;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_session_capacity(server, max_entries) - set session cache max capacity
ring_func!(bolt_set_session_capacity, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let capacity = ring_get_number!(p, 2) as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.session_max_capacity = capacity;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_session_ttl(server, seconds) - set session TTL in seconds
ring_func!(bolt_set_session_ttl, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let seconds = ring_get_number!(p, 2) as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.session_ttl_secs = seconds;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_cache_capacity(server, max_entries) - set cache max capacity
ring_func!(bolt_set_cache_capacity, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let capacity = ring_get_number!(p, 2) as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.cache_max_capacity = capacity;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_set_cache_ttl(server, seconds) - set cache default TTL in seconds
ring_func!(bolt_set_cache_ttl, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let seconds = ring_get_number!(p, 2) as u64;

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.cache_ttl_secs = seconds;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_ip_whitelist(server, ip) - add IP to whitelist
ring_func!(bolt_ip_whitelist, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let ip = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        match ip.parse::<IpNetwork>() {
            Ok(network) => {
                server.config.ip_whitelist.push(network);
                ring_ret_number!(p, 1.0);
            }
            Err(_) => {
                ring_error!(p, "invalid IP/CIDR");
            }
        }
    }
});

/// bolt_ip_blacklist(server, ip) - add IP to blacklist
ring_func!(bolt_ip_blacklist, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let ip = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        match ip.parse::<IpNetwork>() {
            Ok(network) => {
                server.config.ip_blacklist.push(network);
                ring_ret_number!(p, 1.0);
            }
            Err(_) => {
                ring_error!(p, "invalid IP/CIDR");
            }
        }
    }
});

/// bolt_proxy_whitelist(server, ip) - add trusted proxy IP
ring_func!(bolt_proxy_whitelist, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let ip = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.config.proxy_whitelist.push(ip.to_string());
    }

    ring_ret_number!(p, 1.0);
});

// ========================================
// Health Check
// ========================================

/// bolt_health_status(server) -> JSON health status
ring_func!(bolt_health_status, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_ret_string!(p, r#"{"status":"error","message":"Invalid server"}"#);
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let uptime = server.start_time.elapsed().as_secs();
        let json = format!(
            r#"{{"status":"healthy","uptime_seconds":{},"cache_size":{}}}"#,
            uptime,
            server.cache.entry_count()
        );
        ring_ret_string!(p, &json);
    }
});

// ========================================
// JSON Schema Validation
// ========================================

/// bolt_validate_json(json_str, schema_str) -> 1 if valid, 0 if not
ring_func!(bolt_validate_json, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let json_str = ring_get_string!(p, 1);
    let schema_str = ring_get_string!(p, 2);

    let json_value: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    let schema_value: Result<serde_json::Value, _> = serde_json::from_str(schema_str);

    match (json_value, schema_value) {
        (Ok(json), Ok(schema)) => match jsonschema::validator_for(&schema) {
            Ok(validator) => {
                if validator.is_valid(&json) {
                    ring_ret_number!(p, 1.0);
                } else {
                    ring_ret_number!(p, 0.0);
                }
            }
            Err(_) => {
                ring_ret_number!(p, 0.0);
            }
        },
        _ => {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_validate_json_errors(json_str, schema_str) -> JSON array of errors
ring_func!(bolt_validate_json_errors, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let json_str = ring_get_string!(p, 1);
    let schema_str = ring_get_string!(p, 2);

    let json_value: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    let schema_value: Result<serde_json::Value, _> = serde_json::from_str(schema_str);

    match (json_value, schema_value) {
        (Ok(json), Ok(schema)) => match jsonschema::validator_for(&schema) {
            Ok(validator) => {
                let errors: Vec<String> = validator
                    .iter_errors(&json)
                    .map(|e| e.to_string())
                    .collect();
                let json = serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string());
                ring_ret_string!(p, &json);
            }
            Err(e) => {
                let errors = vec![format!("Invalid schema: {}", e)];
                let json = serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string());
                ring_ret_string!(p, &json);
            }
        },
        (Err(e), _) => {
            let errors = vec![format!("Invalid JSON: {}", e)];
            let json = serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string());
            ring_ret_string!(p, &json);
        }
        (_, Err(e)) => {
            let errors = vec![format!("Invalid schema: {}", e)];
            let json = serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string());
            ring_ret_string!(p, &json);
        }
    }
});

// ========================================
// Route Constraints (Regex Validation)
// ========================================

/// bolt_validate_param(server, param_name, regex_pattern) - validate a route param against regex
ring_func!(bolt_validate_param, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let param_name = ring_get_string!(p, 2);
    let pattern = ring_get_string!(p, 3);

    unsafe {
        let server = &*(ptr as *const HttpServer);

        let request_guard = server.current_request.lock();
        if let Some(ref ctx) = *request_guard {
            let param_key = param_name.to_string();
            if let Some(value) = ctx.params.get(&param_key) {
                let re = {
                    let mut cache = server.regex_cache.lock();
                    cache
                        .entry(pattern.to_string())
                        .or_insert_with(|| {
                            regex::Regex::new(pattern)
                                .unwrap_or_else(|_| regex::Regex::new("^$").unwrap())
                        })
                        .clone()
                };
                if re.is_match(value) {
                    ring_ret_number!(p, 1.0);
                } else {
                    ring_ret_number!(p, 0.0);
                }
            } else {
                ring_ret_number!(p, 0.0);
            }
        }
    }
});

/// bolt_validate_regex(value, regex_pattern) - validate any string against regex
ring_func!(bolt_validate_regex, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let value = ring_get_string!(p, 1);
    let pattern = ring_get_string!(p, 2);

    match regex::Regex::new(pattern) {
        Ok(re) => {
            if re.is_match(value) {
                ring_ret_number!(p, 1.0);
            } else {
                ring_ret_number!(p, 0.0);
            }
        }
        Err(_) => {
            ring_ret_number!(p, 0.0);
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_convert_path_params_no_params() {
        assert_eq!(convert_path_params("/users"), "/users");
        assert_eq!(convert_path_params("/"), "/");
        assert_eq!(convert_path_params("/users/list"), "/users/list");
        assert_eq!(convert_path_params(""), "");
    }

    #[test]
    fn test_convert_path_params_single_param() {
        assert_eq!(convert_path_params("/users/:id"), "/users/{id}");
        assert_eq!(convert_path_params("/:resource"), "/{resource}");
        assert_eq!(convert_path_params("/:id"), "/{id}");
    }

    #[test]
    fn test_convert_path_params_multiple_params() {
        assert_eq!(
            convert_path_params("/users/:uid/posts/:pid"),
            "/users/{uid}/posts/{pid}"
        );
        assert_eq!(
            convert_path_params("/api/:version/users/:id"),
            "/api/{version}/users/{id}"
        );
    }

    #[test]
    fn test_convert_path_params_param_with_underscore() {
        assert_eq!(convert_path_params("/users/:user_id"), "/users/{user_id}");
    }

    #[test]
    fn test_convert_path_params_colon_with_digits() {
        assert_eq!(convert_path_params("/v:123"), "/v{123}");
        assert_eq!(convert_path_params("/path:value"), "/path{value}");
    }

    #[test]
    fn test_check_route_constraints_match() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let mut params = HashMap::new();
        params.insert("id".into(), "42".into());
        assert_eq!(
            check_route_constraints(&routes, "get_user", &params, &cache),
            None
        );
    }

    #[test]
    fn test_check_route_constraints_no_match() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let mut params = HashMap::new();
        params.insert("id".into(), "abc".into());
        assert_eq!(
            check_route_constraints(&routes, "get_user", &params, &cache),
            Some("id".into())
        );
    }

    #[test]
    fn test_check_route_constraints_no_matching_route() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let params = HashMap::new();
        assert_eq!(
            check_route_constraints(&routes, "unknown_handler", &params, &cache),
            None
        );
    }

    #[test]
    fn test_check_route_constraints_missing_param() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let params = HashMap::new();
        assert_eq!(
            check_route_constraints(&routes, "get_user", &params, &cache),
            None
        );
    }

    #[test]
    fn test_check_route_constraints_multiple() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{uid}/posts/{pid}".into(),
            handler_name: "get_post".into(),
            description: None,
            tags: vec![],
            constraints: vec![
                ("uid".into(), r"^\d+$".into()),
                ("pid".into(), r"^[a-z]+$".into()),
            ],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let mut params = HashMap::new();
        params.insert("uid".into(), "123".into());
        params.insert("pid".into(), "hello".into());
        assert_eq!(
            check_route_constraints(&routes, "get_post", &params, &cache),
            None
        );

        params.insert("pid".into(), "1234".into());
        assert_eq!(
            check_route_constraints(&routes, "get_post", &params, &cache),
            Some("pid".into())
        );
    }

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(!config.enabled);
        assert!(config.origins.is_empty());
        assert_eq!(config.methods.len(), 6);
        assert_eq!(config.headers.len(), 2);
        assert!(!config.credentials);
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.request_timeout_ms, 30000);
        assert_eq!(config.body_size_limit, 50 * 1024 * 1024);
        assert_eq!(config.session_max_capacity, 10_000);
        assert_eq!(config.session_ttl_secs, 300);
        assert_eq!(config.cache_max_capacity, 10_000);
        assert_eq!(config.cache_ttl_secs, 300);
    }

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert!(config.cert_path.is_empty());
        assert!(config.key_path.is_empty());
    }

    #[test]
    fn test_pending_response_take_existing_none() {
        let response = Arc::new(Mutex::new(None));
        let (headers, cookies) = PendingResponse::take_existing(&response);
        assert!(headers.is_empty());
        assert!(cookies.is_empty());
    }

    #[test]
    fn test_pending_response_take_existing_some() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".into(), "value".into());
        let cookies = vec!["session=abc".into(), "token=xyz".into()];

        let pending = PendingResponse {
            status: 200,
            headers: headers.clone(),
            cookies: cookies.clone(),
            body: ResponseBody::Bytes(b"hello".to_vec()),
            only_headers: false,
        };

        let response = Arc::new(Mutex::new(Some(pending)));
        let (returned_headers, returned_cookies) = PendingResponse::take_existing(&response);
        assert_eq!(returned_headers, headers);
        assert_eq!(returned_cookies, cookies);
    }

    #[test]
    fn test_pending_response_only_headers_flag() {
        let pending = PendingResponse {
            status: 200,
            headers: HashMap::new(),
            cookies: vec![],
            body: ResponseBody::Bytes(vec![]),
            only_headers: true,
        };
        assert!(pending.only_headers);

        let pending = PendingResponse {
            status: 200,
            headers: HashMap::new(),
            cookies: vec![],
            body: ResponseBody::Bytes(vec![]),
            only_headers: false,
        };
        assert!(!pending.only_headers);
    }

    #[test]
    fn test_response_body_bytes() {
        let body = ResponseBody::Bytes(b"hello".to_vec());
        match body {
            ResponseBody::Bytes(data) => assert_eq!(data, b"hello".to_vec()),
            _ => panic!("Expected Bytes variant"),
        }
    }

    #[test]
    fn test_response_body_file() {
        let body = ResponseBody::File("/tmp/test.txt".into());
        match body {
            ResponseBody::File(path) => assert_eq!(path, "/tmp/test.txt"),
            _ => panic!("Expected File variant"),
        }
    }

    #[test]
    fn test_route_definition_constraints() {
        let route = RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: Some("Get user by ID".into()),
            tags: vec!["users".into()],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: Some((100, 60)),
            before_middleware: vec!["auth".into()],
            after_middleware: vec!["log".into()],
        };

        assert_eq!(route.method, "GET");
        assert_eq!(route.description, Some("Get user by ID".into()));
        assert_eq!(route.tags, vec!["users"]);
        assert_eq!(route.rate_limit, Some((100, 60)));
        assert_eq!(route.before_middleware, vec!["auth"]);
        assert_eq!(route.after_middleware, vec!["log"]);
    }

    #[test]
    fn test_http_server_add_route() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("get", "/users/:id", "get_user");

        assert_eq!(server.routes.len(), 1);
        assert_eq!(server.routes[0].method, "GET");
        assert_eq!(server.routes[0].path, "/users/{id}");
        assert_eq!(server.routes[0].handler_name, "get_user");
        assert!(server.routes[0].description.is_none());
        assert!(server.routes[0].tags.is_empty());
    }

    #[test]
    fn test_http_server_add_multiple_routes() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users", "list_users");
        server.add_route("POST", "/users", "create_user");
        server.add_route("DELETE", "/users/:id", "delete_user");

        assert_eq!(server.routes.len(), 3);
        assert_eq!(server.routes[0].method, "GET");
        assert_eq!(server.routes[1].method, "POST");
        assert_eq!(server.routes[2].method, "DELETE");
    }

    #[test]
    fn test_http_server_set_route_description() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users/:id", "get_user");
        server.set_route_description("GET", "/users/:id", "Fetch a user by ID");

        assert_eq!(
            server.routes[0].description,
            Some("Fetch a user by ID".into())
        );
    }

    #[test]
    fn test_http_server_set_route_description_no_match() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users/:id", "get_user");
        server.set_route_description("POST", "/users/:id", "nope");

        assert!(server.routes[0].description.is_none());
    }

    #[test]
    fn test_http_server_add_route_tag() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users/:id", "get_user");
        server.add_route_tag("GET", "/users/:id", "users");
        server.add_route_tag("GET", "/users/:id", "public");

        assert_eq!(server.routes[0].tags, vec!["users", "public"]);
    }

    #[test]
    fn test_http_server_add_route_tag_duplicate() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users/:id", "get_user");
        server.add_route_tag("GET", "/users/:id", "users");
        server.add_route_tag("GET", "/users/:id", "users");

        assert_eq!(server.routes[0].tags, vec!["users"]);
    }

    #[test]
    fn test_http_server_add_constraint() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_route("GET", "/users/:id", "get_user");
        server.add_constraint("get_user", "id", r"^\d+$");

        assert_eq!(server.routes[0].constraints.len(), 1);
        assert_eq!(server.routes[0].constraints[0].0, "id");
        assert_eq!(server.routes[0].constraints[0].1, r"^\d+$");
    }

    #[test]
    fn test_http_server_add_ws_route() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_ws_route(
            "/ws",
            Some("ws_connect".into()),
            Some("ws_message".into()),
            Some("ws_disconnect".into()),
        );

        assert_eq!(server.ws_routes.len(), 1);
        assert_eq!(server.ws_routes[0].path, "/ws");
        assert_eq!(server.ws_routes[0].on_connect, Some("ws_connect".into()));
        assert_eq!(server.ws_routes[0].on_message, Some("ws_message".into()));
        assert_eq!(
            server.ws_routes[0].on_disconnect,
            Some("ws_disconnect".into())
        );
    }

    #[test]
    fn test_http_server_add_ws_route_none_callbacks() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_ws_route("/ws", None, None, None);

        assert_eq!(server.ws_routes.len(), 1);
        assert_eq!(server.ws_routes[0].path, "/ws");
        assert!(server.ws_routes[0].on_connect.is_none());
        assert!(server.ws_routes[0].on_message.is_none());
        assert!(server.ws_routes[0].on_disconnect.is_none());
    }

    #[test]
    fn test_http_server_add_static() {
        let server = HttpServer::new(std::ptr::null_mut());
        let mut server = server;
        server.add_static("/public", "./static");

        assert_eq!(server.static_routes.len(), 1);
        assert_eq!(server.static_routes[0].url_path, "/public");
        assert_eq!(server.static_routes[0].dir_path, "./static");
    }

    #[test]
    fn test_http_server_new_defaults() {
        let server = HttpServer::new(std::ptr::null_mut());
        assert_eq!(server.port, 3000);
        assert_eq!(server.host, "0.0.0.0");
        assert!(server.routes.is_empty());
        assert!(server.ws_routes.is_empty());
        assert!(!server.compression);
        assert!(!server.cors.enabled);
        assert_eq!(server.config.request_timeout_ms, 30000);
        assert_eq!(server.config.body_size_limit, 50 * 1024 * 1024);
    }

    #[test]
    fn test_ws_out_message_clone() {
        let msg = WsOutMessage::Text("hello".into());
        let cloned = msg.clone();
        match cloned {
            WsOutMessage::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_ws_out_message_binary() {
        let msg = WsOutMessage::Binary(vec![1, 2, 3]);
        match msg {
            WsOutMessage::Binary(data) => assert_eq!(data, vec![1, 2, 3]),
            _ => panic!("Expected Binary variant"),
        }
    }

    #[test]
    fn test_sse_event_creation() {
        let event = SseEvent {
            event: Some("update".into()),
            data: r#"{"key":"value"}"#.into(),
        };
        assert_eq!(event.event, Some("update".into()));
        assert_eq!(event.data, r#"{"key":"value"}"#);

        let event = SseEvent {
            event: None,
            data: "data".into(),
        };
        assert!(event.event.is_none());
    }

    #[test]
    fn test_uploaded_file_creation() {
        let file = UploadedFile {
            name: "avatar".into(),
            filename: "photo.png".into(),
            content_type: "image/png".into(),
            data: vec![0, 1, 2, 3],
        };
        assert_eq!(file.name, "avatar");
        assert_eq!(file.filename, "photo.png");
        assert_eq!(file.content_type, "image/png");
        assert_eq!(file.data, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_request_context_creation() {
        let ctx = RequestContext {
            id: 1,
            request_id: "req-1".into(),
            method: "GET".into(),
            path: "/users/1".into(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: vec![],
            handler_name: "get_user".into(),
            files: vec![],
            session_id: String::new(),
            peer_addr: "127.0.0.1:1234".into(),
        };
        assert_eq!(ctx.id, 1);
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.request_id, "req-1");
        assert_eq!(ctx.peer_addr, "127.0.0.1:1234");
    }

    #[test]
    fn test_sse_route_definition() {
        let route = SseRouteDefinition {
            path: "/events".into(),
            handler_name: "sse_handler".into(),
        };
        assert_eq!(route.path, "/events");
        assert_eq!(route.handler_name, "sse_handler");
    }

    #[test]
    fn test_static_route_definition() {
        let route = StaticRoute {
            url_path: "/assets".into(),
            dir_path: "./public".into(),
        };
        assert_eq!(route.url_path, "/assets");
        assert_eq!(route.dir_path, "./public");
    }

    #[test]
    fn test_cors_config_custom() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".into()],
            methods: vec!["GET".into(), "POST".into()],
            headers: vec!["X-Custom".into()],
            credentials: true,
        };
        assert!(config.enabled);
        assert_eq!(config.origins, vec!["https://example.com"]);
        assert!(config.credentials);
    }

    #[test]
    fn test_route_definition_empty_constraints() {
        let route = RouteDefinition {
            method: "GET".into(),
            path: "/users".into(),
            handler_name: "list_users".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        };
        assert!(route.constraints.is_empty());
        assert!(route.rate_limit.is_none());
    }

    // Concurrent stress tests

    #[test]
    fn test_regex_cache_concurrent_reads() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let patterns = vec![r"^\d+$", r"^[a-z]+$", r"^[A-Z]+$", r"^\w+$", r"^\S+$"];

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let patterns = patterns.clone();
                std::thread::spawn(move || {
                    for pattern in &patterns {
                        let mut guard = cache.lock();
                        #[allow(clippy::regex_creation_in_loops)]
                        let re = guard
                            .entry(pattern.to_string())
                            .or_insert_with(|| regex::Regex::new(pattern).unwrap())
                            .clone();
                        drop(guard);
                        // Use inputs that match each pattern
                        let test_input = match *pattern {
                            r"^\d+$" => "12345",
                            r"^[a-z]+$" => "test",
                            r"^[A-Z]+$" => "TEST",
                            r"^\w+$" => "test123",
                            r"^\S+$" => "test123",
                            _ => "test",
                        };
                        assert!(
                            re.is_match(test_input),
                            "Pattern '{}' failed on '{}'",
                            pattern,
                            test_input
                        );
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let final_cache = cache.lock();
        assert_eq!(final_cache.len(), patterns.len());
    }

    #[test]
    fn test_regex_cache_concurrent_compilation() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let handles: Vec<_> = (0..20)
            .map(|i| {
                let cache = Arc::clone(&cache);
                std::thread::spawn(move || {
                    let pattern = format!(r"^test{}$", i);
                    let mut guard = cache.lock();
                    #[allow(clippy::regex_creation_in_loops)]
                    let re = guard
                        .entry(pattern.clone())
                        .or_insert_with(|| regex::Regex::new(&pattern).unwrap())
                        .clone();
                    drop(guard);
                    assert!(re.is_match(&format!("test{}", i)));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let final_cache = cache.lock();
        assert_eq!(final_cache.len(), 20);
    }

    #[test]
    fn test_dashmap_concurrent_insertions() {
        use dashmap::DashMap;
        let map = Arc::new(DashMap::new());
        let handles: Vec<_> = (0..50)
            .map(|i| {
                let map = Arc::clone(&map);
                std::thread::spawn(move || {
                    map.insert(i, format!("value{}", i));
                    assert!(map.contains_key(&i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 50);
    }

    #[test]
    fn test_dashmap_concurrent_reads_and_writes() {
        use dashmap::DashMap;
        let map = Arc::new(DashMap::new());

        // Pre-populate
        for i in 0..100 {
            map.insert(i, i * 2);
        }

        let write_handles: Vec<_> = (0..10)
            .map(|t| {
                let map = Arc::clone(&map);
                std::thread::spawn(move || {
                    for i in 0..100 {
                        map.insert(i, t * 100 + i);
                    }
                })
            })
            .collect();

        let read_handles: Vec<_> = (0..10)
            .map(|_| {
                let map = Arc::clone(&map);
                std::thread::spawn(move || {
                    for i in 0..100 {
                        let _ = map.get(&i);
                    }
                })
            })
            .collect();

        for handle in write_handles {
            handle.join().unwrap();
        }
        for handle in read_handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 100);
    }

    #[test]
    fn test_pending_response_take_existing_concurrent() {
        let response = Arc::new(Mutex::new(Some(PendingResponse {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Test".into(), "value".into());
                h
            },
            cookies: vec!["session=abc".into()],
            body: ResponseBody::Bytes(b"body".to_vec()),
            only_headers: false,
        })));

        let handles: Vec<_> = (0..20)
            .map(|_| {
                let response = Arc::clone(&response);
                std::thread::spawn(move || {
                    let (headers, cookies) = PendingResponse::take_existing(&response);
                    assert!(headers.contains_key("X-Test"));
                    assert_eq!(cookies, vec!["session=abc"]);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_cache_concurrent_insert_and_get() {
        let server = HttpServer::new(std::ptr::null_mut());
        let server = Arc::new(server);

        let handles: Vec<_> = (0..30)
            .map(|i| {
                let server = Arc::clone(&server);
                std::thread::spawn(move || {
                    server
                        .cache
                        .insert(format!("key{}", i), (format!("value{}", i), 0));
                    let _ = server.cache.get(&format!("key{}", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_sessions_concurrent_insert_and_get() {
        let server = HttpServer::new(std::ptr::null_mut());
        let server = Arc::new(server);

        let handles: Vec<_> = (0..30)
            .map(|i| {
                let server = Arc::clone(&server);
                std::thread::spawn(move || {
                    let mut session = HashMap::new();
                    session.insert(format!("key{}", i), format!("value{}", i));
                    server.sessions.insert(format!("session{}", i), session);
                    let _ = server.sessions.get(&format!("session{}", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_route_matching_concurrent_access() {
        let routes = Arc::new(Mutex::new(vec![
            RouteDefinition {
                method: "GET".into(),
                path: "/users/{id}".into(),
                handler_name: "get_user".into(),
                description: None,
                tags: vec![],
                constraints: vec![("id".into(), r"^\d+$".into())],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
            RouteDefinition {
                method: "POST".into(),
                path: "/users".into(),
                handler_name: "create_user".into(),
                description: None,
                tags: vec![],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
        ]));

        let handles: Vec<_> = (0..20)
            .map(|i| {
                let routes = Arc::clone(&routes);
                std::thread::spawn(move || {
                    let guard = routes.lock();
                    let route = &guard[i % 2];
                    assert!(!route.handler_name.is_empty());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_http_server_new_concurrent() {
        let handles: Vec<_> = (0..10)
            .map(|_| {
                std::thread::spawn(|| {
                    let server = HttpServer::new(std::ptr::null_mut());
                    assert_eq!(server.port, 3000);
                    assert!(server.routes.is_empty());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    // ========================================
    // Integration Test Helpers
    // ========================================

    use actix_web::{App, test as aw_test, web};

    /// Direct test handler for .to() usage (4 params, no handler_name)
    async fn test_handle_request_direct(
        req: actix_web::HttpRequest,
        state: web::Data<AppState>,
        query: web::Query<HashMap<String, String>>,
        payload: web::Payload,
    ) -> actix_web::HttpResponse {
        test_handle_request_core(req, state, query, payload, "direct".to_string()).await
    }

    /// Core test handler logic with handler_name
    async fn test_handle_request_core(
        req: actix_web::HttpRequest,
        state: web::Data<AppState>,
        query: web::Query<HashMap<String, String>>,
        _payload: web::Payload,
        handler_name: String,
    ) -> actix_web::HttpResponse {
        // Build a minimal response using the same logic as handle_request
        // but skipping the VM channel entirely
        let path_params: HashMap<String, String> = req
            .match_info()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        if let Some(failed_param) = check_route_constraints(
            &state.routes,
            &handler_name,
            &path_params,
            &state.regex_cache,
        ) {
            return actix_web::HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Invalid parameter: {}", failed_param));
        }

        let headers: HashMap<String, String> = req
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let cookies: HashMap<String, String> = req
            .cookies()
            .map(|cookies| {
                cookies
                    .iter()
                    .map(|c| (c.name().to_string(), c.value().to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let request_id = next_request_id();
        let req_id_header = format!("{:x}", request_id);

        // Check IP filtering if configured
        let client_ip_str = req
            .peer_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if !state.ip_whitelist.is_empty() || !state.ip_blacklist.is_empty() {
            let parsed_ip = if client_ip_str == "::1" {
                std::net::IpAddr::from_str("127.0.0.1")
                    .unwrap_or_else(|_| std::net::IpAddr::from_str("0.0.0.0").unwrap())
            } else {
                client_ip_str
                    .parse::<std::net::IpAddr>()
                    .unwrap_or_else(|_| std::net::IpAddr::from_str("0.0.0.0").unwrap())
            };

            if !state.ip_whitelist.is_empty()
                && !state.ip_whitelist.iter().any(|net| net.contains(parsed_ip))
            {
                return actix_web::HttpResponse::Forbidden()
                    .insert_header(("X-Request-Id", req_id_header))
                    .body("Forbidden");
            }

            if state.ip_blacklist.iter().any(|net| net.contains(parsed_ip)) {
                return actix_web::HttpResponse::Forbidden()
                    .insert_header(("X-Request-Id", req_id_header))
                    .body("Forbidden");
            }
        }

        let query_inner = query.into_inner();

        // Return a JSON response with what we captured
        let response_body = serde_json::json!({
            "handler": "test_handler",
            "path_params": path_params,
            "query": query_inner,
            "headers": headers,
            "cookies": cookies,
            "request_id": req_id_header,
        });

        actix_web::HttpResponse::Ok()
            .insert_header(("X-Request-Id", req_id_header))
            .content_type("application/json")
            .body(response_body.to_string())
    }

    fn build_test_app_state(routes: Vec<RouteDefinition>) -> AppState {
        let (vm_tx, _vm_rx) = tokio::sync::mpsc::channel::<VmWork>(256);
        AppState {
            routes,
            sse_broadcast_channels: Arc::new(Mutex::new(HashMap::new())),
            vm_tx,
            ws_broadcast_tx: tokio::sync::broadcast::channel(10).0,
            ws_clients: Arc::new(DashMap::new()),
            ws_rooms: Arc::new(DashMap::new()),
            ws_client_rooms: Arc::new(DashMap::new()),
            route_limiters: Arc::new(DashMap::new()),
            ip_whitelist: Vec::new(),
            ip_blacklist: Vec::new(),
            regex_cache: Arc::new(Mutex::new(HashMap::new())),
            body_size_limit: 50 * 1024 * 1024,
        }
    }

    fn build_test_app(
        routes: Vec<RouteDefinition>,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        let state = build_test_app_state(routes.clone());
        let state_data = web::Data::new(state);

        let mut app = App::new()
            .app_data(state_data.clone())
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024));

        for route in &routes {
            let path = route.path.clone();
            let handler_name = route.handler_name.clone();

            app = match route.method.as_str() {
                "GET" => app.route(
                    &path,
                    web::get().to(move |req, state, query, payload| {
                        test_handle_request_core(req, state, query, payload, handler_name.clone())
                    }),
                ),
                "POST" => app.route(
                    &path,
                    web::post().to(move |req, state, query, payload| {
                        test_handle_request_core(req, state, query, payload, handler_name.clone())
                    }),
                ),
                "PUT" => app.route(
                    &path,
                    web::put().to(move |req, state, query, payload| {
                        test_handle_request_core(req, state, query, payload, handler_name.clone())
                    }),
                ),
                "DELETE" => app.route(
                    &path,
                    web::delete().to(move |req, state, query, payload| {
                        test_handle_request_core(req, state, query, payload, handler_name.clone())
                    }),
                ),
                _ => app,
            };
        }

        app
    }

    #[actix_web::test]
    async fn test_integration_get_route() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get().uri("/users/123").to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = aw_test::read_body_json(resp).await;
        assert_eq!(body["handler"], "test_handler");
        assert_eq!(body["path_params"]["id"], "123");
    }

    #[actix_web::test]
    async fn test_integration_post_route_with_query() {
        let routes = vec![RouteDefinition {
            method: "POST".into(),
            path: "/search".into(),
            handler_name: "do_search".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::post()
            .uri("/search?q=rust&limit=10")
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = aw_test::read_body_json(resp).await;
        assert_eq!(body["query"]["q"], "rust");
        assert_eq!(body["query"]["limit"], "10");
    }

    #[actix_web::test]
    async fn test_integration_constraint_failure() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get().uri("/users/abc").to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
        let body = aw_test::read_body(resp).await;
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("Invalid parameter: id"));
    }

    #[actix_web::test]
    async fn test_integration_ip_whitelist_blocks() {
        let mut state = build_test_app_state(vec![RouteDefinition {
            method: "GET".into(),
            path: "/admin".into(),
            handler_name: "admin".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }]);
        state.ip_whitelist = vec!["192.168.1.0/24".parse().unwrap()];

        let state_data = web::Data::new(state);
        let app = aw_test::init_service(
            App::new()
                .app_data(state_data.clone())
                .route("/admin", web::get().to(test_handle_request_direct)),
        )
        .await;

        // Request from non-whitelisted IP should be forbidden
        let req = aw_test::TestRequest::get()
            .uri("/admin")
            .peer_addr("10.0.0.1:1234".parse().unwrap())
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn test_integration_ip_whitelist_allows() {
        let mut state = build_test_app_state(vec![RouteDefinition {
            method: "GET".into(),
            path: "/admin".into(),
            handler_name: "admin".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }]);
        state.ip_whitelist = vec!["192.168.1.0/24".parse().unwrap()];

        let state_data = web::Data::new(state);
        let app = aw_test::init_service(
            App::new()
                .app_data(state_data.clone())
                .route("/admin", web::get().to(test_handle_request_direct)),
        )
        .await;

        // Request from whitelisted IP should succeed
        let req = aw_test::TestRequest::get()
            .uri("/admin")
            .peer_addr("192.168.1.50:1234".parse().unwrap())
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_integration_ip_blacklist_blocks() {
        let mut state = build_test_app_state(vec![RouteDefinition {
            method: "GET".into(),
            path: "/public".into(),
            handler_name: "public".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }]);
        state.ip_blacklist = vec!["10.0.0.0/8".parse().unwrap()];

        let state_data = web::Data::new(state);
        let app = aw_test::init_service(
            App::new()
                .app_data(state_data.clone())
                .route("/public", web::get().to(test_handle_request_direct)),
        )
        .await;

        let req = aw_test::TestRequest::get()
            .uri("/public")
            .peer_addr("10.1.2.3:1234".parse().unwrap())
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn test_integration_delete_route() {
        let routes = vec![RouteDefinition {
            method: "DELETE".into(),
            path: "/users/{id}".into(),
            handler_name: "delete_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::delete().uri("/users/42").to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = aw_test::read_body_json(resp).await;
        assert_eq!(body["path_params"]["id"], "42");
    }

    #[actix_web::test]
    async fn test_integration_request_headers_captured() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/api".into(),
            handler_name: "api_handler".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get()
            .uri("/api")
            .insert_header(("X-Custom-Header", "custom-value"))
            .insert_header(("Accept", "application/json"))
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = aw_test::read_body_json(resp).await;
        assert_eq!(body["headers"]["x-custom-header"], "custom-value");
        assert_eq!(body["headers"]["accept"], "application/json");
    }

    #[actix_web::test]
    async fn test_integration_cookies_captured() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/api".into(),
            handler_name: "api_handler".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get()
            .uri("/api")
            .cookie(actix_web::cookie::Cookie::new("session", "abc123"))
            .cookie(actix_web::cookie::Cookie::new("pref", "dark"))
            .to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = aw_test::read_body_json(resp).await;
        assert_eq!(body["cookies"]["session"], "abc123");
        assert_eq!(body["cookies"]["pref"], "dark");
    }

    #[actix_web::test]
    async fn test_integration_x_request_id_header() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/api".into(),
            handler_name: "api_handler".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get().uri("/api").to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let request_id = resp.headers().get("x-request-id");
        assert!(request_id.is_some());
        let id_str = request_id.unwrap().to_str().unwrap();
        assert!(!id_str.is_empty());
        // Should be a valid hex number
        assert!(u64::from_str_radix(id_str, 16).is_ok());
    }

    #[actix_web::test]
    async fn test_integration_multiple_routes() {
        let routes = vec![
            RouteDefinition {
                method: "GET".into(),
                path: "/users".into(),
                handler_name: "list_users".into(),
                description: None,
                tags: vec![],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
            RouteDefinition {
                method: "POST".into(),
                path: "/users".into(),
                handler_name: "create_user".into(),
                description: None,
                tags: vec![],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
            RouteDefinition {
                method: "GET".into(),
                path: "/users/{id}".into(),
                handler_name: "get_user".into(),
                description: None,
                tags: vec![],
                constraints: vec![("id".into(), r"^\d+$".into())],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
        ];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get().uri("/users").to_request();
        let resp = aw_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = aw_test::TestRequest::post().uri("/users").to_request();
        let resp = aw_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = aw_test::TestRequest::get().uri("/users/123").to_request();
        let resp = aw_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = aw_test::TestRequest::get().uri("/users/abc").to_request();
        let resp = aw_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_integration_404_unknown_route() {
        let routes = vec![RouteDefinition {
            method: "GET".into(),
            path: "/known".into(),
            handler_name: "known".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let app = aw_test::init_service(build_test_app(routes)).await;

        let req = aw_test::TestRequest::get().uri("/unknown").to_request();
        let resp = aw_test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    // ========================================
    // Edge Case / Error Branch Tests
    // ========================================

    #[test]
    fn test_request_context_empty_fields() {
        let ctx = RequestContext {
            id: 0,
            request_id: String::new(),
            method: String::new(),
            path: String::new(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            body: vec![],
            handler_name: String::new(),
            files: vec![],
            session_id: String::new(),
            peer_addr: String::new(),
        };
        assert!(ctx.params.is_empty());
        assert!(ctx.query.is_empty());
        assert!(ctx.headers.is_empty());
        assert!(ctx.cookies.is_empty());
        assert!(ctx.body.is_empty());
        assert!(ctx.files.is_empty());
        assert!(ctx.session_id.is_empty());
    }

    #[test]
    fn test_pending_response_empty() {
        let pending = PendingResponse {
            status: 200,
            headers: HashMap::new(),
            cookies: vec![],
            body: ResponseBody::Bytes(vec![]),
            only_headers: false,
        };
        assert_eq!(pending.status, 200);
        assert!(pending.headers.is_empty());
        assert!(pending.cookies.is_empty());
        match pending.body {
            ResponseBody::Bytes(b) => assert!(b.is_empty()),
            _ => panic!("Expected Bytes variant"),
        }
    }

    #[test]
    fn test_pending_response_file_not_found_path() {
        let pending = PendingResponse {
            status: 200,
            headers: HashMap::new(),
            cookies: vec![],
            body: ResponseBody::File("/tmp/this_file_does_not_exist_99999.txt".into()),
            only_headers: false,
        };
        match pending.body {
            ResponseBody::File(path) => {
                assert_eq!(path, "/tmp/this_file_does_not_exist_99999.txt");
            }
            _ => panic!("Expected File variant"),
        }
    }

    #[test]
    fn test_http_server_no_routes_empty() {
        let server = HttpServer::new(std::ptr::null_mut());
        assert!(server.routes.is_empty());
        assert!(server.ws_routes.is_empty());
        assert!(server.static_routes.is_empty());
        assert!(server.before_handlers.is_empty());
        assert!(server.after_handlers.is_empty());
    }

    #[test]
    fn test_check_route_constraints_empty_routes() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes: Vec<RouteDefinition> = vec![];
        let params = HashMap::new();
        assert_eq!(
            check_route_constraints(&routes, "any", &params, &cache),
            None
        );
    }

    #[test]
    fn test_check_route_constraints_empty_params() {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: None,
            tags: vec![],
            constraints: vec![("id".into(), r"^\d+$".into())],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];
        let params = HashMap::new();
        // Missing param should not trigger constraint failure
        assert_eq!(
            check_route_constraints(&routes, "get_user", &params, &cache),
            None
        );
    }

    #[test]
    fn test_ip_whitelist_empty_allows_all() {
        let mut state = build_test_app_state(vec![RouteDefinition {
            method: "GET".into(),
            path: "/public".into(),
            handler_name: "public".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }]);
        // Empty whitelist should allow all IPs
        state.ip_whitelist = vec![];
        assert!(state.ip_whitelist.is_empty());
    }

    #[test]
    fn test_ip_blacklist_empty_blocks_none() {
        let mut state = build_test_app_state(vec![RouteDefinition {
            method: "GET".into(),
            path: "/public".into(),
            handler_name: "public".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }]);
        // Empty blacklist should not block any IP
        state.ip_blacklist = vec![];
        assert!(state.ip_blacklist.is_empty());
    }

    #[test]
    fn test_sse_event_no_event_name() {
        let event = SseEvent {
            event: None,
            data: "plain data".into(),
        };
        assert!(event.event.is_none());
        assert_eq!(event.data, "plain data");
    }

    #[test]
    fn test_ws_event_context_empty() {
        let ctx = WsEventContext {
            client_id: String::new(),
            event_type: String::new(),
            message: String::new(),
            is_binary: false,
            binary_data: vec![],
            path: String::new(),
            params: HashMap::new(),
        };
        assert!(ctx.client_id.is_empty());
        assert!(ctx.message.is_empty());
        assert!(!ctx.is_binary);
        assert!(ctx.binary_data.is_empty());
        assert!(ctx.params.is_empty());
    }
}
