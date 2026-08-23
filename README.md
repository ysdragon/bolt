<div align="center">
  <img src="assets/logo.png" alt="Bolt Logo" width="150">
  <h1>Bolt</h1>
  <p>A blazing-fast web framework for <a href="https://ring-lang.github.io/">Ring</a></p>

  [![](https://img.shields.io/github/license/ysdragon/bolt?style=for-the-badge&label=License&labelColor=414868&color=7aa2f7)](LICENSE)
  [![](https://img.shields.io/badge/language-Ring-2D54CB.svg?style=for-the-badge&labelColor=414868)](https://ring-lang.github.io/)
  [![](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS%20|%20FreeBSD-7aa2f7.svg?style=for-the-badge&labelColor=414868)](#)
  [![](https://img.shields.io/badge/dynamic/regex?url=https%3A%2F%2Fraw.githubusercontent.com%2Fysdragon%2Fbolt%2Fmaster%2Fpackage.ring&search=%3Aversion\s*%3D\s*%22([^%22]%2B)%22&replace=%241&style=for-the-badge&label=package&labelColor=414868&color=bb9af7)](#)
  [![](https://img.shields.io/github/actions/workflow/status/ysdragon/bolt/test.yml?style=for-the-badge&label=tests&labelColor=414868&color=9ece6a)](https://github.com/ysdragon/bolt/actions/workflows/test.yml)
</div>

---

> [!NOTE]
> Bolt is currently in the final stages of development.

---

## Overview

Bolt brings modern web development to Ring. It pairs an **Express.js-like DSL** with a **Rust-powered HTTP engine**, giving you a framework that is both approachable and fast. Write routes in Ring, run them on a Rust async runtime.

---

## 🚀 Quick Start

One line is all it takes:

```ring
load "bolt.ring" new Bolt() { @get("/", func { $bolt.send("Hello, World!") }) }
```

A real app with routing, params, and JSON:

```ring
load "bolt.ring"

new Bolt() {
    port = 3000

    @get("/", func {
        $bolt.send("Hello from Bolt!")
    })

    @get("/users/:id", func {
        $bolt.json([
            :id   = $bolt.param("id"),
            :name = "User " + $bolt.param("id")
        ])
    })
    where("id", "[0-9]+")

    @post("/users", func {
        data = $bolt.jsonBody()
        $bolt.jsonWithStatus(201, [:created = true, :data = data])
    })
}
```

```bash
ring app.ring
# [bolt] Server running on http://0.0.0.0:3000
```

---

## ✨ Features

**Routing & HTTP**
- All HTTP methods: `@get`, `@post`, `@put`, `@patch`, `@delete`, `@head`, `@options`
- URL parameters (`:id`), query strings, and regex constraints (`where()`)
- Route prefixes for clean API versioning (`prefix()` / `endPrefix()`)
- Static file serving with automatic MIME detection

**Middleware & Lifecycle**
- Global `@before` / `@after` hooks
- Named middleware via `@use`
- Per-route middleware and rate limiting

**Real-time**
- WebSocket endpoints with per-client send, rooms, and broadcast
- Server-Sent Events for push updates

**Security**
- JWT encode / decode / verify with expiry
- Basic Auth, signed & flash cookies, CSRF tokens
- Rate limiting, IP whitelist / blacklist, CORS
- Built-in TLS / HTTPS support

**Data & Templates**
- In-memory sessions and key-value cache with TTL
- Multipart file uploads with save-to-disk
- MiniJinja templates (Jinja2-compatible)
- JSON parsing and encoding

**Developer Experience**
- Auto-generated OpenAPI docs at `/docs`
- Built-in brotli/gzip compression
- Request logging with configurable levels
- Homepage helper for instant landing pages

**Utilities**
- `Hash` — Argon2id, bcrypt, scrypt
- `Crypto` — AES-256-GCM, HMAC-SHA256
- `Validate` — Email, URL, IP, UUID, regex, JSON Schema
- `Sanitize` — HTML / XSS safe output
- `Env` — `.env` file loading
- `DateTime` — Formatting, parsing, arithmetic

---

## 📊 Benchmarks

Hello-world endpoint tested with `wrk -t16 -c1000 -d30s` on a dedicated server with AMD Ryzen 9 9950X (32 CPUs, 128 GiB RAM), Ubuntu 26.04.

| Framework | Language | Requests/sec | Avg Latency | vs Bolt |
|-----------|----------|-------------|-------------|---------|
| Actix-web | Rust | 4,209,699 | 282.71us | 7.2x faster |
| Java Virtual Threads | Java | 3,874,620 | 262.46us | 6.6x faster |
| ASP.NET | .NET | 1,859,957 | 617.27us | 3.2x faster |
| Gin | Go | 1,126,549 | 1.08ms | 1.9x faster |
| Fiber | Go | 1,034,599 | 1.29ms | 1.8x faster |
| **Bolt** | **Ring/Rust** | **584,918** | **1.62ms** | **—** |
| Bun | JS | 430,449 | 2.41ms | 1.4x slower |
| Elysia | Bun | 396,082 | 2.70ms | 1.5x slower |
| NestJS+Fastify/Node | JS | 128,475 | 21.58ms | 4.6x slower |
| Express/Bun | JS | 115,804 | 8.75ms | 5.1x slower |
| Express/Node | JS | 91,038 | 18.62ms | 6.4x slower |
| Flask | Python | 70,951 | 18.86ms | 8.2x slower |
| FastAPI | Python | 21,820 | 45.37ms | 26.8x slower |

---

## 📦 Installation

```bash
ringpm install bolt from ysdragon
```

**Requirements**
- Ring 1.27+
- Pre-built binaries included for Windows, Linux (glibc / musl), macOS, and FreeBSD

---

## 🎯 Examples

| # | Example | What it shows |
|---|---------|-------------|
| 01 | [hello](examples/basic/01_hello.ring) | Basic routes, JSON, params |
| 02 | [http_methods](examples/basic/02_http_methods.ring) | All HTTP verbs |
| 03 | [route_params](examples/basic/03_route_params.ring) | URL params, query strings, constraints |
| 04 | [request_response](examples/basic/04_request_response.ring) | Headers, body, cookies |
| 05 | [json_api](examples/basic/05_json_api.ring) | RESTful CRUD API |
| 06 | [static_files](examples/basic/06_static_files.ring) | Serving directories |

Browse the full set in [`examples/`](examples/).

---

## 📚 Documentation

- [Usage Guide](docs/USAGE.md) — Feature-by-feature guide with complete code examples
- [API Reference](docs/API.md) — Every method, every class, every parameter

---

## ⚙️ Common Configuration

```ring
new Bolt() {
    port = 3000
    host = "0.0.0.0"

    # Limits
    setBodyLimit(50 * 1024 * 1024)
    setTimeout(30000)
    setSessionCapacity(10000)
    setSessionTTL(300)
    setCacheCapacity(50000)
    setCacheTTL(600)

    # Security & features
    enableCors()
    enableCompression()
    enableLogging()
    enableDocs()
    homepage()

    # Routes...
}
```

---

## 🤝 Contributing

Issues and pull requests are welcome!

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.