# Bolt Tests

Integration test suite for the Bolt framework. Each test module spawns a real
Ring server (from `servers/`) and exercises it over HTTP with `httpx`.

## Layout

```
├── conftest.py          # fixtures: server lifecycle, per-module ports, TLS
├── pytest.ini
├── requirements.txt
├── certs/               # self-signed TLS certificates for HTTPS tests
├── static_test/         # static file assets
├── templates/           # minijinja templates for rendering tests
├── servers/             # Ring server fixtures (one per test module)
└── test_*.py            # pytest modules
```

## Running

From the repository root:

```bash
pip install -r requirements.txt
python -m pytest
```

Run a single module:

```bash
python -m pytest test_wildcard_routes.py
```

The `ring` command must be on `PATH` (override with `RING_CMD`). Each module
gets its own port (`8801`–`8829`, see `_MODULE_PORTS` in `conftest.py`), so
modules can run in parallel; servers are health-checked via `GET /health`
before tests start.

## Writing a Test

1. Add a fixture `servers/<name>.ring` that reads `BOLT_TEST_PORT`,
   registers `@get("/health")`, and exposes the routes under test.
2. Add `test_<name>.py` using the `client` fixture (an `httpx.Client`
   bound to the server) and `unwrap()` to strip Bolt's `{"Ok": ...}` wrapper.
3. Register the module's port in `_MODULE_PORTS` in `conftest.py`.

## Coverage

HTTP methods, routing (params, constraints, prefixes, wildcard/catch-all
segments), middleware, responses, cookies/sessions, auth (JWT, basic),
caching, WebSocket, SSE, uploads, TLS, OpenAPI spec generation, security
(NUL-byte validation, path traversal, limits), environment, logging, panics,
and JSON utilities (booleans, schema validation, templates).
