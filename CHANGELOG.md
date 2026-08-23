# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ⚠️ Breaking Changes

- **JWT default TTL**: `jwt_encode()` now applies a default 3600-second TTL when `expires_in` is omitted. Previously, omitting `expires_in` produced a token with no `exp` claim (never expires). Explicitly pass `expires_in` to control token lifetime. Explicit TTLs that are zero, negative, NaN, or infinite are now rejected with an error instead of saturating to 0 (immediate expiry).
- **CSRF query-string removed**: `_csrf` is no longer accepted via the GET query string. Tokens must be sent via the `X-CSRF-Token` header or the `_csrf` form field. Query-string CSRF tokens leak via server logs, browser history, and referrer headers.
- **AES ciphertext format change**: String keys (anything that is not a raw 32-byte key) are now stretched with Argon2id (m=19MiB, t=2, p=1) using a random 16-byte salt prepended to the output. Ciphertexts produced with string keys by older versions will not decrypt. Raw 32-byte keys continue to use the legacy `nonce || ciphertext` format. A 32-character password is treated as a raw key (no stretching) — use a longer or shorter password to ensure Argon2id key derivation applies.
- **WebSocket origin validation**: Origin check now requires exact host or subdomain match instead of substring containment. `example.com` no longer matches `example.com.attacker.tld`. Previous substring check allowed bypass via domain suffix tricks.

### Added

#### Server Core

- Initial server framework with routing, middleware, WebSocket, SSE, caching, sessions, CSRF protection, and OpenAPI spec generation.
- Default homepage route that renders a styled HTML page displaying Bolt name, version, description, and all registered routes. Skipped if `/` is already defined.
- Support for custom HTTP methods in route registration.
- Wildcard / catch-all route segments: `*name` (named, accessible via `$bolt.param`) and `*` (anonymous) as the last path segment, mapped to actix `{name:.*}` / `{_:.*}`.
- JSON boolean support: `jsonTrue()`, `jsonFalse()`, `jsonIsTrue()`, `jsonIsFalse()`, `jsonToBool()`. Booleans now round-trip correctly through encode/decode using sentinel strings.
- `index.html` resolution for static file routes.
- Platform-specific guidance for port-in-use errors.
- Template fuel limits via minijinja fuel feature (100k instruction cap on template rendering).
- Raw request URI exposed via `bolt_req_uri` and `uri` field on `RequestContext`, distinct from the matched route pattern.
- WebSocket close frame with optional code and reason fields forwarded to actix-ws close frames.
- Support for multiple TLS private key formats — tries PKCS#8, RSA (PKCS#1), and EC (SEC1) in sequence.
- `bolt_escape_js_dq` for escaping JavaScript in double-quoted string contexts (complements `bolt_escape_js` for single-quoted contexts).
- Default `ringvm_errorhandler` registered when no `@error()` handler is set, preventing Ring runtime errors from killing the server process. Unhandled errors return 500 Internal Server Error instead of crashing.
- VM thread handle stored in `VM_THREAD` and joined on next `bolt_listen` to prevent two threads racing on the same non-thread-safe Ring VM.
- `Done(None)` (handler error, VM alive) now returns 500 Internal Server Error; `ChannelDown` (VM thread gone) returns 503 Service Unavailable — previously both returned 503.
- Request timeout via `tokio::time::timeout` around VM send+response_wait, returning 504 Gateway Timeout on deadline expiry.

#### Security

- Context-aware HTML escape functions using the `html_escape` crate: `bolt_escape_html`, `bolt_escape_attr`, `bolt_escape_js`, `bolt_escape_url`.
- Input size limits on base64 decode (16 MiB) and JSON nesting depth (128).
- Environment variable mutation prevention after server start via `bolt_env_set`; allowlist restrictions removed from `bolt_env_get`.
- Minimum 32-byte JWT secrets enforced with required `exp` claim.
- Rate limit parameter validation with atomic CAS for window reset.
- WebSocket origin validation with IP whitelist/blacklist enforcement; bounded channels with `try_send`.
- Path traversal, absolute path, and NUL byte rejection in uploads, templates, and `respond_file`.
- Auto-escaping enabled in minijinja templates.
- SSE event name and log message sanitization (control characters stripped).
- Unicode-aware length validation in `bolt_validate_length`; non-finite numeric values rejected.
- Default `Content-Type` set to `text/plain`; headers preserved across `respond_redirect`, `respond_file`, and `respond_status`.
- Session IDs signed with HMAC-SHA256 to prevent fixation attacks; `__Host-` prefixed session cookies over TLS; `force_secure_cookies` option and proper session clear.
- Multipart field count and per-field size limits.
- Security headers: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`, HSTS over TLS.
- Canonicalized static file paths to prevent symlink directory traversal.
- Error propagation replacing `process::exit` in `bolt_listen`; `Result` return from `run_server`.
- Actix payload limit (100 MiB) and template cache limit (1000 entries).
- After-middleware skipped when request is already aborted.
- Error handler and middleware target route name validation.
- Thread-safe session access via `Arc<Mutex<HashMap>>`; silent error returns replaced with proper `ring_error!` messages.
- PBKDF2-HMAC-SHA256 (600k iterations) for AES-GCM key derivation.
- Full SHA-256 HMAC for CSRF tokens instead of truncated 16-char signature; `SameSite=Lax` and conditional `Secure` flags on session cookies.
- `bolt_session_regenerate` to prevent session fixation attacks.
- Panic safety via `catch_unwind` wrapping request handler execution; poisoned `RwLock` recovery in template cache.
- `body_size_limit` made thread-safe via `Arc<AtomicUsize>`.
- Per-instance CSRF secret storage replacing global `OnceLock`, enabling independent server configurations.
- AES key handling: SHA-256 hash keys that are not exactly 32 bytes instead of silently truncating.
- Middleware abort support so before-handlers can short-circuit request processing when a response is already set.
- Cache and session data preserved across server restarts by migrating entries to new cache instances.
- Early log skip when logging is disabled.
- WebSocket text messages no longer trimmed, preserving original payload.
- Middleware abort support for before-handler short-circuiting.
- Email/URL validation regexes hardened against consecutive dots, authority `@`, and length limits.
- Strict UTF-8 for base64 decode results; `f64`→`i64` overflow boundary fixed.
- Atomic session regeneration via remove+insert instead of get+insert+invalidate.
- SSE `id`/`retry` fields added; PII redaction in logs; full 32-byte ETag hashes.
- Regex cache lock scope narrowed in constraint checking.
- `keep_alive` separated from `client_request_timeout`.
- User OpenAPI spec sanitized (servers/externalDocs stripped, descriptions escaped).
- Template cache mtime precision improved from seconds to milliseconds.
- Constant-time CSRF verification via the `subtle` crate, replacing direct string equality checks.
- Rate limiting refactored from global `AtomicU64` counters to per-client `DashMap<IpRateEntry>` tracking.
- Request query and form field types changed from `HashMap<String, String>` to `HashMap<String, Vec<String>>` for multi-value support.
- Session cookies hardened with HMAC-signed session IDs, `__Host-` prefix conditional on TLS, and `SameSite=Lax`.
- Per-route SSE subscriber caps via `DashMap<String, usize>` with automatic decrement on `SseStream` drop.
- OpenAPI endpoint gated behind IP allowlist/blacklist.
- `bolt_force_secure_cookies` for enforcing secure session cookies.
- `bolt_ws_dropped_count` metric for WebSocket backpressure monitoring.
- `valid_jwt_ttl` rejects zero, negative, NaN, and infinite TTL values.
- `valid_env_pair` rejects `=` and NUL bytes in env keys and NUL bytes in env values before calling `std::env::set_var` (prevents panic under `panic=abort`).
- Upload index validation rejects non-finite or sub-1.0 values before integer conversion (prevents saturation to index 0).
- Per-route rate-limit window validation: `window_seconds` must be strictly positive (prevents division-by-zero in quota period calculation).
- Request timeout rejection of non-positive or non-finite milliseconds in `bolt_set_timeout`.
- Rate limit map hard cap at 200,000 IPs with fail-open behavior under distributed flood; warns once when cap is reached, re-arms after cleanup.
- Log redaction regex hardened: multi-token quantifier replaced with single-token match to prevent log-line erasure; chained auth schemes (`Bearer token: ...`) handled correctly.
- JSON boolean sentinel escaping: literal strings equal to `__JSON_TRUE__`/`__JSON_FALSE__` are backslash-prefixed on decode and unescaped on encode, so they round-trip as strings instead of being coerced to booleans.
- Datetime format validation: invalid `strftime` patterns return 0 instead of panicking; timezone offset directives (`%z`, `%:z`, `%::z`, `%:::z`, `%#z`) detected via parsed items, with `%%` escapes honored; `datetime_parse` normalizes offset-aware inputs to UTC.
- `add_days`/`add_hours` use `checked_add_signed` to prevent silent overflow on extreme inputs.
- SSE `data` field splitting now handles bare `\r` (CR without LF) to prevent raw CR injection into the wire format.
- WebSocket close frame forwarded with `code: None, reason: None` on `AggregatedMessage::Close` (previously silently dropped).

#### WebSocket

- Global and per-IP connection limits with per-client message rate limiting.
- `AggregatedMessageStream` for continuation frames; ping/pong handling; close handshake.
- Fire-and-forget `on_message` via `try_send`; VM priority drain (HTTP backlog before WS backlog).
- `wsEventAbort()` support for cancelling events from before middleware.
- WebSocket origin validation with IP whitelist/blacklist enforcement; bounded channels.
- WS drop counter for backpressure monitoring.
- WebSocket client existence validated before room join operations.
- Rate limiter guarded against empty client IP strings.
- H1 half-closed connections disabled; `peer_addr` defaults to empty string instead of `"unknown"`.
- Per-IP WebSocket connection quotas and per-client message throttling exposed to the Ring API.
- `$bolt_last_route` now correctly assigned in WebSocket route registration.

#### SSE

- Subscriber-side param filtering with subset matching.
- `sseMaxSubscribers()` for per-route subscriber caps with 503 fallback.
- Broadcast functions accept Ring list directly (no manual JSON encoding): `sseBroadcastParams` and `sseBroadcastEventParams`.
- SSE `id`/`retry` fields added.
- Per-route SSE subscriber caps via `DashMap` with automatic decrement on drop.
- `bolt_sse_filter_params` for optional SSE subscriber parameter filtering.

#### Ring API

- `$bolt.bodyBase64()` for binary-safe request body access (base64-encoded, no UTF-8 lossy conversion).
- `$bolt.uri()` returning the actual request path with query string, distinct from `$bolt.path()` which yields the matched route pattern.
- `$bolt.queryAll()` and `$bolt.formFieldAll()` for retrieving all values of multi-valued parameters.
- `$bolt.csrfAutoVerify()` for automatic CSRF token verification on state-changing HTTP methods.
- `$bolt.regenerateSession()` for session ID rotation after authentication or privilege changes.
- `$bolt.forceSecureCookies()` exposed to enforce secure session cookies; accepts an explicit enabled flag.
- WS drop count and SSE param filter APIs exposed.
- `ring_list_to_json` exposed as a public FFI helper for converting Ring lists to JSON strings.
- Native ring lists accepted as input in JWT encode and JSON respond functions.
- Native ring lists accepted for template data instead of JSON strings.

#### Sanitize Class

- `escapeHtmlBody`, `escapeAttr`, `escapeJs`, `escapeUrl` methods exposing low-level context-specific encoding routines for HTML body, attribute, JavaScript, and URL contexts.

#### OpenAPI

- Path parameters extracted from route paths (e.g., `/users/{id}/posts/{pid}`) and emitted as required path parameters in the generated spec.
- User-provided `servers`/`externalDocs` stripped from spec; descriptions escaped.

#### Package & Build

- Main entry point with styled ASCII banner displaying package metadata, version, description, author credit, and GitHub URL.
- Package manifest defining version 1.0.0, cross-platform native library targets, Rust source module listings, and setup/remove/run commands.
- Example scripts and documentation files included in the distribution manifest.
- Package list loaded before `bolt.ring` during initialization.

#### CI/CD

- Cross-platform compilation and artifact publishing for Alpine (musl), FreeBSD, macOS, Ubuntu, and Windows across amd64, arm64, and i386 architectures.
- GitHub Actions workflow for Hugo documentation site deployment to GitHub Pages.
- Deployment restricted to `docs/site` path changes.
- Library update job trigger relaxed to run on all non-cancelled events regardless of branch.

#### Examples

- Intermediate tutorial suite (examples 07–19) covering cookie/session management, middleware, file uploads, template rendering, form handling, error pages, route grouping, OpenAPI docs, logging, input validation, and environment variables.
- Advanced tutorial suite (examples 20–43) covering WebSocket, SSE, JWT authentication, CORS, CSRF, TLS/HTTPS, route constraints, compression, caching, JSON Schema validation, password hashing, AES encryption, HTML sanitization, and custom OpenAPI specs.

#### Tests

- Comprehensive integration test suite with 25+ pytest modules, 27+ Ring server fixtures, TLS certificates, and per-module port allocation covering HTTP methods, routing, middleware, security, caching, WebSocket, SSE, uploads, and edge cases.
- Integration tests for wildcard/catch-all routes (`test_wildcard_routes`): named and anonymous wildcard matching, param extraction, docs endpoints surviving catch-alls, and OpenAPI path-param emission for wildcard segments (`test_openapi_wildcards`).
- Integration tests for JSON boolean sentinels (`test_json_booleans`): real boolean encoding, predicate helpers on both branches, and encode/decode round-trip.

#### Documentation Site

- Hextra-powered Hugo documentation site with three sections (Getting Started, Guides, API Reference) spanning 60+ pages covering routing, middleware, authentication, WebSocket, SSE, caching, security, templates, and utility classes.
- Custom CSS for benchmark tables, dark-mode support, Tailwind build output, favicon set, Hugo layouts/shortcodes, and a homepage with hero section and feature grid.

#### Documentation

- Comprehensive API reference (`docs/API.md`) and usage guide (`docs/USAGE.md`).
- `$bolt.uri()` endpoint documentation with refined `$bolt.path()` examples showing route pattern output vs resolved paths.
- `$bolt.bodyBase64()` reference with clarification that `$bolt.body()` replaces non-UTF-8 bytes with U+FFFD replacement characters.
- `queryAll()`, `formFieldAll()`, `csrfAutoVerify()`, `sseMaxSubscribers()` reference docs; per-IP rate limiting clarified; ReDoS protection documented with `size_limit` and `dfa_size_limit`.
- Secure cookie behavior, WS drop counter, and SSE param filter documentation.
- WebSocket quotas, SSE param filters, and event abort reference documentation.
- `regenerateSession()` usage documented across API reference, sessions reference, and cookies-sessions guide.
- Context-specific escaping methods documented for html, attr, js, and url contexts.
- CSRF and basic auth examples updated to match the new session-bound CSRF token API and corrected `basicAuthEncode` output.
- Compression algorithm references updated from deflate to brotli.
- Benchmark results updated with dedicated server results (AMD Ryzen 9 9950X, 32 CPUs, 128 GiB RAM, Ubuntu 26.04); revised test configuration (16 threads, 1000 connections, 30s duration), added Flask, reordered by throughput, and recalculated all ratios.
- Ring version requirement updated to 1.27.
- Basic usage examples added: hello world, HTTP methods, route params, request/response, JSON API, and static files.
- README updated with badges, feature catalog, framework comparison benchmarks, and Express.js-style quick start snippets.
- Note added about final development stages.
- Rust doc comments standardized across server and core modules (unicode arrows, return type annotations).

### Changed

#### Style

- Hash comments replaced with double-slash comments in utility scripts (`src/utils/color.ring`, `install.ring`, `uninstall.ring`).

#### Server

- Custom destructor removed in favor of unmanaged pointer (`Box::into_raw`/`from_raw`) for `HttpServer`.
- Server fields renamed to idiomatic names with legacy aliases for backward compatibility.
- Dead code removed: unused `HTTP_REQUEST_TYPE`/`HTTP_RESPONSE_TYPE` identifiers, unused `methods`/`headers` fields from `CorsConfig`, unused `sse_shutdown_tx` channel, query extractor from route handlers.
- Duplicated SSE params parsing extracted into `extract_sse_params` helper; duplicated file-to-list conversion extracted into `add_file_to_list!` macro.
- `MAX_TEMPLATE_CACHE` hoisted to module level; `expire_after_update` delegates to `expire_after_create` in cache expiry.
- Redundant JSON encoding removed in response and JWT helpers on the Ring side.
- Intermediate `bolt_json_encode` call eliminated in `render`, `renderFile`, and `renderTemplate` — native Rust layer now accepts ring lists directly.
- Form body parsing and header normalization restructured: `form_urlencoded` crate added for `application/x-www-form-urlencoded` decoding; header keys lowercased at collection time for O(1) lookup; session cookie construction delegates to the `cookie` crate parser.
- Unused `request_id` variable removed from test setup.
- `process::exit` replaced with error propagation in `bolt_listen`; `run_server` returns `Result`.
- After-middleware skipped when request is aborted.
- Error handler and middleware target route names validated.
- CORS config simplified: wildcard origin (`*`) treated as allow-any.
- `wsRouteBefore`/`wsRouteAfter` removed in favor of unified `routeBefore`/`routeAfter` with `(handler_name, middleware)` tuples for WS routes.
- IP allow/deny check moved before body buffering (cheap rejection of forbidden IPs).
- Cookie priority: `__Host-BOLTSESSION` preferred over `BOLTSESSION`.

#### Performance

- Manual TTL tracking and background cleanup thread (60-second polling) eliminated by leveraging moka's built-in `Expiry` trait with `BoltCacheExpiry` struct.
- Template cache mtime precision improved from seconds to milliseconds.

### Fixed

- Ring runtime errors (e.g., `0 + "abc"` type errors) no longer kill the server process when no `@error()` handler is registered.
- VM thread handle not joined on server restart (could race on the same non-thread-safe Ring VM).
- WebSocket origin validation bypass via domain suffix tricks (`example.com.attacker.tld`).
- Rate-limit division-by-zero when `window_seconds` was 0 (quota period calculation).
- `std::env::set_var` panic on keys containing `=` or NUL bytes (fatal under `panic=abort`).
- Upload index saturation to 0 on NaN/negative input (could return wrong file).
- SSE raw CR injection via bare `\r` in data field.
- JWT immediate-expiry tokens from NaN/negative `expires_in` saturating to 0 via float→u64 cast.
- Datetime overflow on extreme `add_days`/`add_hours` inputs (silent wraparound to wrong timestamps).
- Email validation regex now allows dots in local part but rejects consecutive dots (`..`).
- JSON sentinel strings (`__JSON_TRUE__`/`__JSON_FALSE__`) now round-trip correctly through decode→encode instead of being coerced to booleans.
- Race conditions in WebSocket connection counting — global and per-IP counters now increment before handshake and roll back on failure.
- Per-client rate limiting now correctly uses the request handle instead of a global check.
- IO protocol errors handled silently in the WebSocket message loop.
- Rate limit interval integer division fixed (caused panic at rates >1000/sec); now uses microseconds with `.max(1)`.
- Deprecated `from_slice` calls updated to `from` in aes-gcm constructors.
- Non-32-byte AES keys SHA-256 hashed instead of silently truncated.
- PBKDF2-HMAC-SHA256 replaced with SHA-256 digest for non-32-byte AES keys; `pbkdf2` dependency removed.
- `ring_list_to_json` errors now properly propagated in JWT encoding, JSON response, and template rendering paths.
- JWT `exp` claim made optional instead of required.
- `bolt_force_secure_cookies` updated to accept an explicit enabled flag.
- NUL byte injection prevention: file paths validated in `fileSave`, `sendFile`, `sendFileAs`, and `renderFile` methods; returns error status when NUL bytes detected.
- Catch-all routes (`/*`, `/*name`) are now registered after framework endpoints and static file mounts, so they no longer shadow `/docs`, `/openapi.json`, or static files. Known limitation: plain leading params (e.g. `/:id`) still shadow docs endpoints. Covered by Rust unit tests and `test_wildcard_routes`.
- File upload switched from `create_new` to `create` + `truncate` to allow overwriting existing files.
- Cache expiry fallback behavior corrected; header key casing preserved.
- JSON encoding returns errors on depth limit exceeded.
- `.env` file loading returns an error on failure instead of silently ignoring.
- Missing `ws_message_rate_limit` and `proxy_whitelist` fields added to server test fixture.
- Form field added to request struct initializations in tests.
- CORS config tests aligned with simplified struct fields.

### Removed

- Axum and Ring HTTPLib entries from benchmark comparisons.
- `forceSecureCookies` setter — replaced by automatic TLS-aware Secure cookie handling.
- Allowlist restrictions on `bolt_env_get`.
