# Bolt Usage Guide

A comprehensive guide to building web applications with Bolt.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Routing](#basic-routing)
3. [Route Parameters](#route-parameters)
4. [Request Handling](#request-handling)
5. [Response Methods](#response-methods)
6. [JSON APIs](#json-apis)
7. [Static Files](#static-files)
8. [Middleware](#middleware)
9. [Cookies & Sessions](#cookies--sessions)
10. [File Uploads](#file-uploads)
11. [Templates](#templates)
12. [WebSocket](#websocket)
13. [Server-Sent Events](#server-sent-events)
14. [Authentication](#authentication)
15. [Security](#security)
16. [Caching](#caching)
17. [Error Handling](#error-handling)
18. [Configuration](#configuration)
19. [Utility Classes](#utility-classes)
20. [OpenAPI Documentation](#openapi-documentation)
21. [Testing & Deployment](#testing--deployment)

---

## Getting Started

### Installation

```bash
ringpm install bolt from ysdragon
```

1. Create your first app:
```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    @get("/", func {
        $bolt.send("Hello, World!")
    })
}
```

2. Run it:
```bash
ring app.ring
```

---

## Basic Routing

### HTTP Methods

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # GET - Retrieve data
    @get("/users", func {
        $bolt.json([:users = [[:id = 1, :name = "Alice"]]])
    })
    
    # POST - Create data
    @post("/users", func {
        data = $bolt.jsonBody()
        $bolt.jsonWithStatus(201, [:created = true, :user = data])
    })
    
    # PUT - Update/replace data
    @put("/users/:id", func {
        data = $bolt.jsonBody()
        $bolt.json([:updated = $bolt.param("id"), :data = data])
    })
    
    # PATCH - Partial update
    @patch("/users/:id", func {
        data = $bolt.jsonBody()
        $bolt.json([:patched = $bolt.param("id")])
    })
    
    # DELETE - Remove data
    @delete("/users/:id", func {
        $bolt.sendStatus(204)  # No content
    })
    
    # HEAD - Headers only (no body)
    @head("/health", func {
        $bolt.setHeader("X-Status", "healthy")
        $bolt.sendStatus(200)
    })
    
    # OPTIONS - CORS preflight
    @options("/api/*", func {
        $bolt.setHeader("Allow", "GET, POST, PUT, DELETE, OPTIONS")
        $bolt.sendStatus(200)
    })
}
```

### Route Prefixes

Group related routes:

```ring
new Bolt() {
    port = 3000
    
    # API v1
    prefix("/api/v1")
        @get("/users", func { $bolt.json([:version = 1]) })
        @get("/posts", func { $bolt.json([:version = 1]) })
    endPrefix()
    
    # API v2
    prefix("/api/v2")
        @get("/users", func { $bolt.json([:version = 2]) })
        @get("/posts", func { $bolt.json([:version = 2]) })
    endPrefix()
    
    # Admin routes
    prefix("/admin")
        @get("/dashboard", func { $bolt.send("Admin Dashboard") })
        @get("/users", func { $bolt.send("Admin Users") })
    endPrefix()
}
```

---

## Route Parameters

### URL Parameters

```ring
# Single parameter
@get("/users/:id", func {
    userId = $bolt.param("id")
    $bolt.json([:id = userId])
})

# Multiple parameters
@get("/posts/:postId/comments/:commentId", func {
    postId = $bolt.param("postId")
    commentId = $bolt.param("commentId")
    $bolt.json([
        :postId = postId,
        :commentId = commentId
    ])
})

# Example: /users/123 → {"id": "123"}
# Example: /posts/1/comments/5 → {"postId": "1", "commentId": "5"}
```

### Route Constraints

Validate parameters with regex:

```ring
# Numeric ID only
@get("/users/:id", func {
    $bolt.json([:id = $bolt.param("id")])
})
where("id", "[0-9]+")
# ✓ /users/123
# ✗ /users/abc (404)

# UUID format
@get("/items/:uuid", func {
    $bolt.json([:uuid = $bolt.param("uuid")])
})
where("uuid", "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")

# Multiple constraints
@get("/archive/:year/:month", func {
    $bolt.json([:year = $bolt.param("year"), :month = $bolt.param("month")])
})
whereAll([
    ["year", "[0-9]{4}"],
    ["month", "(0[1-9]|1[0-2])"]
])
# ✓ /archive/2024/03
# ✗ /archive/2024/15 (404)
```

### Query Strings

```ring
@get("/search", func {
    q = $bolt.query("q")
    page = $bolt.query("page")
    limit = $bolt.query("limit")
    
    if page = "" page = "1" ok
    if limit = "" limit = "10" ok
    
    $bolt.json([
        :query = q,
        :page = page,
        :limit = limit
    ])
})

# /search?q=hello&page=2&limit=20
# → {"query": "hello", "page": "2", "limit": "20"}
```

---

## Request Handling

### Accessing Request Data

```ring
@post("/api/data", func {
    # HTTP method
    httpMethod = $bolt.method()  # "POST"
    
    # Request path
    requestPath = $bolt.path()   # "/api/data"
    
    # Headers
    contentType = $bolt.header("Content-Type")
    userAgent = $bolt.header("User-Agent")
    auth = $bolt.header("Authorization")
    
    # Raw body
    rawBody = $bolt.body()
    
    # Client info
    ip = $bolt.clientIp()
    reqId = $bolt.requestId()  # Unique request ID
    
    $bolt.json([
        :method = httpMethod,
        :path = requestPath,
        :contentType = contentType,
        :ip = ip,
        :requestId = reqId
    ])
})
```

### JSON Body

```ring
@post("/users", func {
    # Parse JSON body
    data = $bolt.jsonBody()
    
    # Access fields
    name = data[:name]
    email = data[:email]
    
    # Validate
    if name = "" or email = ""
        $bolt.badRequest("Name and email required")
        return
    ok
    
    # Process...
    $bolt.jsonWithStatus(201, [:id = 1, :name = name, :email = email])
})
```

### Form Data

```ring
@post("/login", func {
    username = $bolt.formField("username")
    password = $bolt.formField("password")
    remember = $bolt.formField("remember")
    
    if username = "admin" and password = "secret"
        $bolt.setSession("user", username)
        $bolt.redirect("/dashboard")
    else
        $bolt.badRequest("Invalid credentials")
    ok
})
```

---

## Response Methods

### Text Responses

```ring
# Simple text
$bolt.send("Hello, World!")

# With custom status
$bolt.sendWithStatus(201, "Resource created")

# Status code only
$bolt.sendStatus(204)  # No content
```

### JSON Responses

```ring
# Standard JSON
$bolt.json([:name = "John", :age = 30])

# With status code
$bolt.jsonWithStatus(201, [:id = 1, :created = true])

# Nested objects
$bolt.json([
    :user = [
        :id = 1,
        :name = "John",
        :email = "john@example.com"
    ],
    :roles = ["admin", "user"]
])
```

### File Responses

```ring
# Auto-detect MIME type
$bolt.sendFile("./files/report.pdf")

# Explicit MIME type
$bolt.sendFileAs("./files/data.bin", "application/octet-stream")
```

### Binary Responses

```ring
# Send binary data (base64-encoded)
$bolt.sendBinary(base64EncodedData)

# Send binary with explicit MIME type
$bolt.sendBinaryAs(base64EncodedData, "image/png")
```

### Redirects

```ring
# Temporary redirect (302)
$bolt.redirect("/login")

# Permanent redirect (301)
$bolt.redirectPermanent("/new-location")
```

### Error Responses

```ring
$bolt.notFound()           # 404 Not Found
$bolt.badRequest("msg")    # 400 Bad Request
$bolt.unauthorized()       # 401 Unauthorized
$bolt.forbidden()          # 403 Forbidden
$bolt.serverError("msg")   # 500 Internal Server Error
```

### Custom Headers

```ring
@get("/download", func {
    $bolt.setHeader("Content-Disposition", "attachment; filename=data.csv")
    $bolt.setHeader("Cache-Control", "no-cache")
    $bolt.sendFile("./exports/data.csv")
})
```

---

## JSON APIs

### RESTful API Example

```ring
load "bolt.ring"

# In-memory data store
users = [
    [:id = 1, :name = "Alice", :email = "alice@example.com"],
    [:id = 2, :name = "Bob", :email = "bob@example.com"]
]
nextId = 3

new Bolt() {
    port = 3000
    enableCors()
    
    prefix("/api")
    
        # List all users
        @get("/users", func {
            $bolt.json([:data = users, :count = len(users)])
        })
        
        # Get single user
        @get("/users/:id", func {
            id = number($bolt.param("id"))
            for user in users
                if user[:id] = id
                    $bolt.json(user)
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
        
        # Create user
        @post("/users", func {
            data = $bolt.jsonBody()
            
            if data[:name] = "" or data[:email] = ""
                $bolt.badRequest("Name and email required")
                return
            ok
            
            user = [
                :id = nextId,
                :name = data[:name],
                :email = data[:email]
            ]
            add(users, user)
            nextId++
            
            $bolt.jsonWithStatus(201, user)
        })
        
        # Update user
        @put("/users/:id", func {
            id = number($bolt.param("id"))
            data = $bolt.jsonBody()
            
            for i = 1 to len(users)
                if users[i][:id] = id
                    users[i][:name] = data[:name]
                    users[i][:email] = data[:email]
                    $bolt.json(users[i])
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
        
        # Delete user
        @delete("/users/:id", func {
            id = number($bolt.param("id"))
            
            for i = 1 to len(users)
                if users[i][:id] = id
                    del(users, i)
                    $bolt.sendStatus(204)
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
    
    endPrefix()
}
```

### JSON Validation

```ring
@post("/users", func {
    schema = '{
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 1},
            "email": {"type": "string", "format": "email"},
            "age": {"type": "integer", "minimum": 0}
        },
        "required": ["name", "email"]
    }'
    
    errors = $bolt.validateJsonErrors($bolt.body(), schema)
    if errors != ""
        $bolt.jsonWithStatus(400, [:error = "Validation failed", :details = errors])
        return
    ok
    
    data = $bolt.jsonBody()
    # Process valid data...
})
```

---

## Static Files

### Basic Static Serving

```ring
# Serve ./static directory at /public URL
@static("/public", "./static")

# Multiple directories
@static("/css", "./assets/css")
@static("/js", "./assets/js")
@static("/images", "./assets/images")
```

### Directory Structure

```
project/
├── app.ring
└── static/
    ├── css/
    │   └── style.css
    ├── js/
    │   └── app.js
    └── images/
        └── logo.png
```

Access URLs:
- `/public/css/style.css`
- `/public/js/app.js`
- `/public/images/logo.png`

---

## Middleware

### Before/After Hooks

```ring
new Bolt() {
    port = 3000
    
    # Runs BEFORE every request
    @before(func {
        # Add request ID header
        $bolt.setHeader("X-Request-Id", $bolt.requestId())
        
        # Log request
        $bolt.log("[" + $bolt.method() + "] " + $bolt.path())
        
        # Authentication check
        auth = $bolt.header("Authorization")
        if $bolt.path() != "/login" and $bolt.path() != "/public" and auth = ""
            $bolt.unauthorized()
            return
        ok
    })
    
    # Runs AFTER every request
    @after(func {
        # Add timing header
        $bolt.setHeader("X-Response-Time", "" + $bolt.unixtimeMs())
        
        # Log completion
        $bolt.log("Request completed: " + $bolt.path())
    })
    
    @get("/", func { $bolt.send("Home") })
    @get("/login", func { $bolt.send("Login page") })
    @get("/dashboard", func { $bolt.send("Dashboard (protected)") })
}
```

### CORS Middleware Example

```ring
new Bolt() {
    port = 3000
    
    @before(func {
        # Set CORS headers for all responses
        $bolt.setHeader("Access-Control-Allow-Origin", "*")
        $bolt.setHeader("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        $bolt.setHeader("Access-Control-Allow-Headers", "Content-Type, Authorization")
        
        # Handle preflight
        if $bolt.method() = "OPTIONS"
            $bolt.sendStatus(204)
            return
        ok
    })
    
    # Your routes...
}
```

### Request Timing

```ring
@before(func {
    $bolt.setSession("_start_time", "" + $bolt.unixtimeMs())
})

@after(func {
    startTime = number($bolt.getSession("_start_time"))
    duration = $bolt.unixtimeMs() - startTime
    $bolt.setHeader("X-Response-Time", "" + duration + "ms")
})
```

---

## Cookies & Sessions

### Basic Cookies

```ring
# Set cookie
@get("/set-cookie", func {
    $bolt.setCookie("username", "john")
    $bolt.send("Cookie set!")
})

# Read cookie
@get("/get-cookie", func {
    username = $bolt.cookie("username")
    if username != ""
        $bolt.send("Hello, " + username)
    else
        $bolt.send("No cookie found")
    ok
})

# Delete cookie
@get("/delete-cookie", func {
    $bolt.deleteCookie("username")
    $bolt.send("Cookie deleted!")
})
```

### Cookie Options

```ring
# Set cookie with options
$bolt.setCookieEx("session", "abc123", "path=/; httponly; secure; max-age=3600; samesite=strict")

# Options explained:
# - path=/        : Cookie valid for all paths
# - httponly      : Not accessible via JavaScript
# - secure        : Only sent over HTTPS
# - max-age=3600  : Expires in 1 hour
# - samesite=strict : CSRF protection
```

### Signed Cookies

```ring
new Bolt() {
    port = 3000
    
    # Set cookie signing secret
    setCookieSecret("my-super-secret-key-32chars!")
    
    @get("/set", func {
        $bolt.setSignedCookie("user_id", "12345")
        $bolt.send("Signed cookie set")
    })
    
    @get("/get", func {
        userId = $bolt.getSignedCookie("user_id")
        if userId != NULL
            $bolt.send("User ID: " + userId)
        else
            $bolt.send("Invalid or missing cookie")
        ok
    })
}
```

### Sessions

```ring
@post("/login", func {
    username = $bolt.formField("username")
    password = $bolt.formField("password")
    
    if username = "admin" and password = "secret"
        $bolt.setSession("user_id", "1")
        $bolt.setSession("username", username)
        $bolt.setSession("role", "admin")
        $bolt.redirect("/dashboard")
    else
        $bolt.badRequest("Invalid credentials")
    ok
})

@get("/dashboard", func {
    userId = $bolt.getSession("user_id")
    if userId = ""
        $bolt.redirect("/login")
        return
    ok
    
    username = $bolt.getSession("username")
    $bolt.send("Welcome, " + username + "!")
})

@get("/logout", func {
    $bolt.clearSession()
    $bolt.redirect("/login")
})
```

### Flash Messages

```ring
@post("/action", func {
    # Do something...
    $bolt.setFlash("success", "Action completed successfully!")
    $bolt.redirect("/result")
})

@get("/result", func {
    if $bolt.hasFlash("success")
        msg = $bolt.getFlash("success")  # Automatically cleared after reading
        $bolt.send("Message: " + msg)
    else
        $bolt.send("No message")
    ok
})
```

---

## File Uploads

### Single File Upload

```ring
@post("/upload", func {
    if $bolt.filesCount() < 1
        $bolt.badRequest("No file uploaded")
        return
    ok
    
    # Get file info (1-indexed!)
    f = $bolt.file(1)
    name = f[:name]
    field = f[:field]
    type = f[:type]
    size = f[:size]
    
    # Validate
    if size > 5 * 1024 * 1024
        $bolt.badRequest("File too large (max 5MB)")
        return
    ok
    
    # Save file
    savePath = "./uploads/" + name
    $bolt.fileSave(1, savePath)
    
    $bolt.json([
        :success = true,
        :file = [
            :name = name,
            :type = type,
            :size = size
        ]
    ])
})
```

### Multiple File Upload

```ring
@post("/upload-multiple", func {
    count = $bolt.filesCount()
    
    if count < 1
        $bolt.badRequest("No files uploaded")
        return
    ok
    
    uploaded = []
    
    for i = 1 to count
        f = $bolt.file(i)
        name = f[:name]
        size = f[:size]
        
        # Save each file
        $bolt.fileSave(i, "./uploads/" + name)
        
        add(uploaded, [:name = name, :size = size])
    next
    
    $bolt.json([:uploaded = uploaded, :count = count])
})
```

### HTML Form

```html
<form action="/upload" method="POST" enctype="multipart/form-data">
    <input type="file" name="file" />
    <button type="submit">Upload</button>
</form>

<!-- Multiple files -->
<form action="/upload-multiple" method="POST" enctype="multipart/form-data">
    <input type="file" name="files" multiple />
    <button type="submit">Upload</button>
</form>
```

---

## Templates

### MiniJinja Syntax

Bolt uses MiniJinja, a Jinja2-compatible template engine.

**templates/layout.html:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <header>
        <h1>{{ title }}</h1>
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        <p>&copy; {{ year }}</p>
    </footer>
</body>
</html>
```

**templates/home.html:**
```html
{% extends "layout.html" %}

{% block content %}
    <p>Welcome, {{ user.name }}!</p>
    
    <h2>Your Items</h2>
    <ul>
    {% for item in items %}
        <li>{{ item.name }} - ${{ item.price }}</li>
    {% endfor %}
    </ul>
    
    {% if user.admin %}
        <a href="/admin">Admin Panel</a>
    {% endif %}
{% endblock %}
```

### Rendering Templates

```ring
@get("/", func {
    html = $bolt.renderFile("templates/home.html", [
        :title = "Welcome",
        :year = 2024,
        :user = [
            :name = "John",
            :admin = true
        ],
        :items = [
            [:name = "Widget", :price = 9.99],
            [:name = "Gadget", :price = 19.99]
        ]
    ])
    $bolt.send(html)
})
```

### Inline Templates

```ring
@get("/greeting/:name", func {
    html = $bolt.render("<h1>Hello, {{ name }}!</h1>", [
        :name = $bolt.param("name")
    ])
    $bolt.send(html)
})
```

---

## WebSocket

### Basic WebSocket Server

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # Serve HTML client
    @get("/", func {
        html = '<!DOCTYPE html>
        <html>
        <body>
            <div id="messages"></div>
            <input id="msg" placeholder="Type message...">
            <button onclick="sendMsg()">Send</button>
            <script>
                const ws = new WebSocket("ws://localhost:3000/ws");
                ws.onmessage = (e) => {
                    document.getElementById("messages").innerHTML += e.data + "<br>";
                };
                function sendMsg() {
                    ws.send(document.getElementById("msg").value);
                    document.getElementById("msg").value = "";
                }
            </script>
        </body>
        </html>'
        $bolt.send(html)
    })
    
    # WebSocket endpoint with event-driven callbacks
    @ws("/ws",
        func {
            id = $bolt.wsClientId()
            $bolt.wsSendTo(id, "Welcome!")
        },
        func {
            $bolt.wsBroadcast("[User] " + $bolt.wsEventMessage())
        },
        func { }
    )
}
```

### Chat Application with Rooms

```ring
@ws("/chat",
    func {
        # on_connect: join chat room
        id = $bolt.wsClientId()
        $bolt.wsRoomJoin("chat", id)
        $bolt.wsRoomBroadcast("chat", $bolt.jsonEncode([
            :type = "join", :id = id
        ]))
    },
    func {
        # on_message: broadcast to room
        data = $bolt.jsonDecode($bolt.wsEventMessage())
        broadcast = $bolt.jsonEncode([
            :type = "message",
            :user = data[:user],
            :text = data[:text],
            :time = $bolt.unixtime()
        ])
        $bolt.wsRoomBroadcast("chat", broadcast)
    },
    func {
        # on_disconnect: notify room
        $bolt.wsRoomBroadcast("chat", $bolt.jsonEncode([
            :type = "leave", :id = $bolt.wsClientId()
        ]))
    }
)

# Get connection count
@get("/chat/stats", func {
    $bolt.json([
        :connections = $bolt.wsConnectionCount(),
        :room_members = $bolt.wsRoomCount("chat")
    ])
})
```

---

## Server-Sent Events

### SSE Server

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # Client page
    @get("/", func {
        html = '<!DOCTYPE html>
        <html>
        <body>
            <div id="updates"></div>
            <script>
                const es = new EventSource("/events");
                es.onmessage = (e) => {
                    document.getElementById("updates").innerHTML += e.data + "<br>";
                };
                es.addEventListener("notification", (e) => {
                    alert("Notification: " + e.data);
                });
            </script>
        </body>
        </html>'
        $bolt.send(html)
    })
    
    # SSE endpoint - clients connect here
    @sse("/events")
    
    # Trigger endpoint - send events from here
    @post("/notify", func {
        message = $bolt.jsonBody()[:message]
        
        # Send to all connected clients
        $bolt.sseBroadcast("/events", message)
        
        $bolt.json([:sent = true])
    })
    
    # Named events
    @post("/alert", func {
        data = $bolt.jsonBody()
        $bolt.sseBroadcastEvent("/events", "notification", data[:text])
        $bolt.json([:sent = true])
    })
}
```

---

## Authentication

### JWT Authentication

```ring
load "bolt.ring"

SECRET = "your-super-secret-key"

new Bolt() {
    port = 3000
    
    # Login - get token
    @post("/login", func {
        data = $bolt.jsonBody()
        
        if data[:username] = "admin" and data[:password] = "secret"
            # Create token with 1 hour expiry (seconds from now)
            token = $bolt.jwtEncodeExp([
                :user_id = 1,
                :username = data[:username],
                :role = "admin"
            ], SECRET, 3600)
            
            $bolt.json([:token = token])
        else
            $bolt.unauthorized()
        ok
    })
    
    # Protected route
    @get("/profile", func {
        auth = $bolt.header("Authorization")
        
        if auth = ""
            $bolt.unauthorized()
            return
        ok
        
        # Extract token (Bearer xxx)
        token = substr(auth, 8)  # Skip "Bearer "
        
        if !$bolt.jwtVerify(token, SECRET)
            $bolt.unauthorized()
            return
        ok
        
        payload = $bolt.jwtDecode(token, SECRET)
        $bolt.json([
            :user_id = payload[:user_id],
            :username = payload[:username],
            :role = payload[:role]
        ])
    })
}
```

### Basic Auth

```ring
@before(func {
    # Skip public routes
    if $bolt.path() = "/" or $bolt.path() = "/public"
        return
    ok
    
    auth = $bolt.header("Authorization")
    if auth = "" or left(auth, 6) != "Basic "
        $bolt.setHeader("WWW-Authenticate", 'Basic realm="Protected"')
        $bolt.unauthorized()
        return
    ok
    
    # Decode credentials
    creds = $bolt.basicAuthDecode(auth)
    
    if creds = NULL
        $bolt.unauthorized()
        return
    ok
    
    username = creds[:username]
    password = creds[:password]
    
    if username != "admin" or password != "secret"
        $bolt.unauthorized()
    ok
})
```

---

## Security

### CSRF Protection

```ring
new Bolt() {
    port = 3000
    
    enableCsrf("csrf-secret-key")
    
    # Form with CSRF token
    @get("/form", func {
        token = $bolt.csrfToken()
        html = '<form method="POST" action="/submit">
            <input type="hidden" name="_csrf" value="' + token + '">
            <input type="text" name="data">
            <button>Submit</button>
        </form>'
        $bolt.send(html)
    })
    
    # Verify CSRF on submit
    @post("/submit", func {
        token = $bolt.formField("_csrf")
        expected = $bolt.csrfToken()
        
        if !$bolt.verifyCsrf(token, expected)
            $bolt.forbidden()
            return
        ok
        
        # Process form...
        $bolt.send("Success!")
    })
}
```

### Rate Limiting

```ring
new Bolt() {
    port = 3000
    
    # Global: 100 requests per minute
    $bolt.rateLimit(100, 60)
    
    @before(func {
        if !$bolt.checkRateLimit()
            $bolt.setHeader("Retry-After", "60")
            $bolt.sendWithStatus(429, "Too many requests")
        ok
    })
    
    @get("/api/data", func {
        $bolt.json([:data = "..."])
    })

    # Per-route rate limiting
    @post("/api/login", func {
        # Login logic...
    })
    routeRateLimit(5, 60)  # 5 attempts per minute
}
```

### IP Filtering

```ring
new Bolt() {
    port = 3000
    
    # Allow only specific IPs
    ipWhitelist("192.168.1.0/24")  # Local network
    ipWhitelist("10.0.0.5")         # Specific IP
    
    # Block bad actors
    ipBlacklist("1.2.3.4")
    ipBlacklist("5.6.7.0/24")
    
    # Routes...
}
```

### HTTPS/TLS

```ring
new Bolt() {
    port = 443
    
    # Enable TLS
    enableTls("./certs/server.crt", "./certs/server.key")
    
    # Force HTTPS redirect
    @before(func {
        if $bolt.header("X-Forwarded-Proto") = "http"
            $bolt.redirectPermanent("https://" + $bolt.header("Host") + $bolt.path())
        ok
    })
    
    # Routes...
}
```

---

## Caching

### In-Memory Cache

```ring
# Cache expensive database queries
@get("/users", func {
    cached = $bolt.cacheGet("users_list")
    
    if cached != ""
        $bolt.json($bolt.jsonDecode(cached))
        return
    ok
    
    # Expensive operation...
    users = fetchUsersFromDatabase()
    
    # Cache for 5 minutes
    $bolt.cacheSet("users_list", $bolt.jsonEncode(users))
    
    $bolt.json(users)
})

# Cache with per-key TTL
@get("/stats", func {
    cached = $bolt.cacheGet("site_stats")
    if cached != ""
        $bolt.json($bolt.jsonDecode(cached))
        return
    ok
    
    stats = computeExpensiveStats()
    $bolt.cacheSetTTL("site_stats", $bolt.jsonEncode(stats), 600)  # 10 min TTL
    $bolt.json(stats)
})

# Invalidate cache on update
@post("/users", func {
    # Create user...
    $bolt.cacheDelete("users_list")
    $bolt.json([:created = true])
})

# Clear all cache
@post("/admin/clear-cache", func {
    $bolt.cacheClear()
    $bolt.json([:cleared = true])
})
```

### ETag Caching

```ring
@get("/data", func {
    data = getExpensiveData()
    content = $bolt.jsonEncode(data)
    
    # Generate ETag
    $bolt.etag(content)
    
    $bolt.json(data)
})
```

---

## Error Handling

### Global Error Handler

```ring
new Bolt() {
    port = 3000
    
    @error(func {
        $bolt.json([
            :error = true,
            :message = "An unexpected error occurred",
            :path = $bolt.path(),
            :method = $bolt.method()
        ])
    })
    
    @get("/crash", func {
        # This will trigger error handler
        x = 1/0
    })
}
```

### Custom Error Responses

```ring
@get("/users/:id", func {
    id = $bolt.param("id")
    
    # Not found
    user = findUser(id)
    if user = NULL
        $bolt.jsonWithStatus(404, [
            :error = "User not found",
            :code = "USER_NOT_FOUND"
        ])
        return
    ok
    
    $bolt.json(user)
})

@post("/users", func {
    data = $bolt.jsonBody()
    
    # Validation error
    if data[:name] = ""
        $bolt.jsonWithStatus(400, [
            :error = "Validation failed",
            :code = "VALIDATION_ERROR",
            :details = [[:field = "name", :message = "Name is required"]]
        ])
        return
    ok
    
    # Conflict
    if userExists(data[:email])
        $bolt.jsonWithStatus(409, [
            :error = "User already exists",
            :code = "DUPLICATE_EMAIL"
        ])
        return
    ok
    
    # Success
    $bolt.jsonWithStatus(201, [:id = createUser(data)])
})
```

---

## Configuration

### Full Configuration Example

```ring
load "bolt.ring"

new Bolt() {
    # Server
    port = 3000
    host = "0.0.0.0"
    
    # Timeouts & Limits
    setTimeout(30000)                  # 30 second timeout
    setBodyLimit(50 * 1024 * 1024)     # 50MB max body
    setSessionCapacity(100000)         # 100k sessions
    setSessionTTL(3600)                # 1 hour session TTL
    setCacheCapacity(50000)            # 50k cache entries
    setCacheTTL(600)                   # 10 minute cache TTL
    
    # Security
    enableCors()
    corsOrigin("https://myapp.com")
    corsOrigin("https://admin.myapp.com")
    
    ipWhitelist("10.0.0.0/8")
    ipBlacklist("1.2.3.4")
    
    $bolt.rateLimit(1000, 60)  # 1000 req/min
    
    enableCsrf("my-csrf-secret")
    setCookieSecret("my-cookie-secret-key-32chars!")
    
    # Features
    enableCompression()
    enableLogging()
    
    # TLS (production)
    # enableTls("cert.pem", "key.pem")
    
    # Documentation
    setDocsInfo("My API", "1.0.0", "Production API")
    enableDocs()
    
    # Routes...
}
```

---

## Utility Classes

Bolt includes several standalone utility classes for common tasks.

### Environment Variables (Env)

Load and manage `.env` files:

```ring
env = new Env()

# Automatically loads .env from current directory on init
dbUrl = env.getVar("DATABASE_URL")
port = env.getVar("PORT")
secret = env.getOr("SECRET", "default-secret")

# Load from a specific file
env.loadFile("/etc/myapp/env")

# Set a variable
env.setVar("APP_ENV", "production")
```

**Example `.env` file:**
```
DATABASE_URL=postgres://localhost/mydb
SECRET=my-secret-key
PORT=3000
```

### Password Hashing (Hash)

Secure password hashing with Argon2, bcrypt, or scrypt:

```ring
hash = new Hash

# Argon2id (recommended)
hashed = hash.argon2("mypassword")
if hash.verifyArgon2("mypassword", hashed)
    # Password correct
ok

# Bcrypt
hashed = hash.bcrypt("mypassword")
if hash.verifyBcrypt("mypassword", hashed)
    # Password correct
ok

# Scrypt
hashed = hash.scrypt("mypassword")
if hash.verifyScrypt("mypassword", hashed)
    # Password correct
ok
```

**Full authentication example:**
```ring
load "bolt.ring"

hash = new Hash
storedHash = ""

new Bolt() {
    port = 3000

    @post("/register", func {
        data = $bolt.jsonBody()
        storedHash = hash.argon2(data[:password])
        $bolt.json([:registered = true])
    })

    @post("/login", func {
        data = $bolt.jsonBody()
        if hash.verifyArgon2(data[:password], storedHash)
            $bolt.json([:success = true])
        else
            $bolt.unauthorized()
        ok
    })
}
```

### Input Validation (Validate)

Validate common data types:

```ring
v = new Validate

# Email, URL, IP
if v.email(input) { ... }
if v.url(input) { ... }
if v.ip(input) { ... }
if v.ipv4(input) { ... }
if v.ipv6(input) { ... }

# UUID and JSON
if v.uuid(input) { ... }
if v.jsonString(input) { ... }

# String length and number range
if v.length(name, 3, 50) { ... }
if v.range(age, 0, 150) { ... }

# Character classes
if v.alpha(name) { ... }        # letters only
if v.alphanumeric(name) { ... }  # letters + digits
if v.numeric(zip) { ... }       # digits only
```

**Validation middleware example:**
```ring
@post("/users", func {
    v = new Validate
    data = $bolt.jsonBody()

    if !v.email(data[:email])
        $bolt.badRequest("Invalid email")
        return
    ok

    if !v.length(data[:name], 2, 100)
        $bolt.badRequest("Name must be 2-100 characters")
        return
    ok

    # Process valid data...
})
```

### Encryption (Crypto)

AES-256-GCM encryption and HMAC-SHA256:

```ring
crypto = new Crypto

# AES-256-GCM encryption (key must be 32 bytes)
key = "0123456789abcdef0123456789abcdef"
encrypted = crypto.aesEncrypt("secret data", key)
decrypted = crypto.aesDecrypt(encrypted, key)

# HMAC-SHA256 signatures
sig = crypto.hmacSha256("message", "signing-key")
if crypto.hmacVerify("message", "signing-key", sig)
    # Signature valid
ok
```

### Date & Time (DateTime)

Timestamp operations, formatting, and arithmetic:

```ring
dt = new DateTime

# Current time
cNow = dt.now()           # Local ISO string
cUtc = dt.nowUtc()        # UTC ISO string
ts = dt.timestamp()        # Unix seconds
tsMs = dt.timestampMs()   # Unix milliseconds

# Formatting
cDate = dt.formatDate(ts, "%Y-%m-%d %H:%M:%S")
cFriendly = dt.formatDate(ts, "%B %d, %Y")

# Parsing
ts = dt.parseDate("2026-05-02 14:30:00", "%Y-%m-%d %H:%M:%S")

# Arithmetic
nextWeek = dt.addDays(ts, 7)
inTwoHours = dt.addHours(ts, 2)
diff = dt.diff(ts1, ts2)  # Difference in seconds
```

### HTML Sanitization (Sanitize)

Prevent XSS attacks:

```ring
s = new Sanitize

# Strip dangerous tags, keep safe ones
safe = s.html('<script>alert("xss")</script><p>Safe</p>')
# Returns: "<p>Safe</p>"

# Strip ALL tags
text = s.strict('<b>Bold</b> <script>evil()</script>')
# Returns: "Bold evil()"

# Escape HTML entities
escaped = s.escapeHtml('<div>Hello & goodbye</div>')
# Returns: "&lt;div&gt;Hello &amp; goodbye&lt;/div&gt;"
```

**User content rendering example:**
```ring
s = new Sanitize

@post("/comment", func {
    data = $bolt.jsonBody()
    safeComment = s.html(data[:text])
    # Store safeComment in database
})

@get("/comments", func {
    # Sanitize before rendering
    $bolt.renderFile("templates/comments.html", [
        :comments = getComments(),
        :sanitize = s
    ])
})
```

---

## OpenAPI Documentation

### Auto-Generated Docs

```ring
new Bolt() {
    port = 3000
    
    # Set API info
    setDocsInfo("User API", "2.0.0", "API for managing users")
    
    # Enable Swagger UI at /docs
    enableDocs()
    
    # Document routes
    @get("/users", func {
        $bolt.json([:users = []])
    })
    describe("List all users")
    tag("Users")
    
    @get("/users/:id", func {
        $bolt.json([:id = $bolt.param("id")])
    })
    where("id", "[0-9]+")
    describe("Get user by ID")
    tag("Users")
    
    @post("/users", func {
        data = $bolt.jsonBody()
        $bolt.jsonWithStatus(201, [:id = 1])
    })
    describe("Create new user")
    tag("Users")
}
```

Visit `http://localhost:3000/docs` for interactive Swagger UI.

### Default Homepage

Add an auto-generated homepage at `/` that displays server info and links to the API docs:

```ring
new Bolt() {
    port = 3000

    setDocsInfo("My API", "1.0.0", "My awesome API")
    enableDocs()
    homepage()  # Adds "/" with server info (skips if "/" already defined)

    # Your routes...
}
```

---

## Testing & Deployment

### Testing with curl

```bash
# GET request
curl http://localhost:3000/users

# POST JSON
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com"}'

# With authentication
curl http://localhost:3000/profile \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9..."

# File upload
curl -X POST http://localhost:3000/upload \
  -F "file=@photo.jpg"
```

### Production Checklist

1. **Enable HTTPS**
   ```ring
   enableTls("cert.pem", "key.pem")
   ```

2. **Set appropriate limits**
   ```ring
   setBodyLimit(10 * 1024 * 1024)  # Limit body size
   $bolt.rateLimit(100, 60)               # Rate limiting
   ```

3. **Configure CORS properly**
   ```ring
   enableCors()
   corsOrigin("https://yourdomain.com")  # Not "*" in production
   ```

4. **Enable security features**
   ```ring
   enableCsrf("strong-secret")
   setCookieSecret("another-strong-secret")
   ```

5. **Use systemd service**
   ```ini
   [Unit]
   Description=Bolt API Server
   After=network.target

   [Service]
   Type=simple
   User=www-data
   WorkingDirectory=/var/www/myapp
   ExecStart=/usr/bin/ring app.ring
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

6. **Reverse proxy with nginx**
   ```nginx
   server {
       listen 443 ssl;
       server_name api.yourdomain.com;
       
       ssl_certificate /etc/ssl/certs/cert.pem;
       ssl_certificate_key /etc/ssl/private/key.pem;
       
       location / {
           proxy_pass http://127.0.0.1:3000;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection "upgrade";
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

---

## Next Steps

- Browse [Examples](../examples/) for more use cases
- Read the [API Reference](API.md) for all methods