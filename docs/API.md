# Bolt API Reference

Complete API documentation for the Bolt framework.

## Table of Contents

- [Server Configuration](#server-configuration)
- [Routing](#routing)
- [Route Modifiers](#route-modifiers)
- [Middleware](#middleware)
- [Request](#request)
- [Response](#response)
- [Headers](#headers)
- [Cookies](#cookies)
- [Sessions](#sessions)
- [File Uploads](#file-uploads)
- [JSON](#json)
- [Templates](#templates)
- [Caching](#caching)
- [WebSocket](#websocket)
- [Server-Sent Events](#server-sent-events)
- [Authentication](#authentication)
- [Security](#security)
- [CORS](#cors)
- [Rate Limiting](#rate-limiting)
- [Validation](#validation)
- [Utilities](#utilities)
- [Health](#health)
- [OpenAPI Documentation](#openapi-documentation)
- [Logging](#logging)
- [Server Control](#server-control)
- [Env Class](#env-class)
- [Hash Class](#hash-class)
- [Validate Class](#validate-class)
- [Crypto Class](#crypto-class)
- [DateTime Class](#datetime-class)
- [Sanitize Class](#sanitize-class)

---

## Server Configuration

### Attributes

```ring
port = 3000              # Server port (default: 3000)
host = "0.0.0.0"         # Bind address (default: "0.0.0.0")
```

### setPort(nValue)
Set the server port.

```ring
setPort(8080)
```

### setHost(cValue)
Set the bind address.

```ring
setHost("127.0.0.1")
```

### setTimeout(nMs)
Set request timeout in milliseconds.

```ring
setTimeout(30000)  # 30 seconds
```

**Default:** 30,000ms (30 seconds)

### setBodyLimit(nBytes)
Set maximum request body size in bytes.

```ring
setBodyLimit(50 * 1024 * 1024)  # 50MB
```

**Default:** 52,428,800 bytes (50MB)

### setSessionCapacity(nMaxEntries)
Set maximum number of session entries.

```ring
setSessionCapacity(50000)  # 50,000 entries
```

**Default:** 10,000 entries

### setSessionTTL(nSeconds)
Set session time-to-live in seconds.

```ring
setSessionTTL(3600)  # 1 hour
```

**Default:** 300 seconds (5 minutes)

### setCacheCapacity(nCapacity)
Set maximum number of cache entries.

```ring
setCacheCapacity(50000)  # 50,000 entries
```

**Default:** 10,000 entries

### setCacheTTL(nSeconds)
Set default cache time-to-live in seconds.

```ring
setCacheTTL(600)  # 10 minutes
```

**Default:** 300 seconds (5 minutes)

### enableTls(cCertPath, cKeyPath)
Enable HTTPS with TLS certificates.

```ring
enableTls("cert.pem", "key.pem")
```

### enableCompression() / disableCompression()
Enable or disable response compression (brotli/gzip).

```ring
enableCompression()
```

### enableLogging() / disableLogging()
Enable or disable request logging.

```ring
enableLogging()
```

---

## Routing

### @get(cPath, fHandler)
Register a GET route.

```ring
@get("/users", func {
    $bolt.json([:users = []])
})
```

### @post(cPath, fHandler)
Register a POST route.

```ring
@post("/users", func {
    data = $bolt.jsonBody()
    $bolt.jsonWithStatus(201, [:created = true])
})
```

### @put(cPath, fHandler)
Register a PUT route.

```ring
@put("/users/:id", func {
    $bolt.json([:updated = $bolt.param("id")])
})
```

### @delete(cPath, fHandler)
Register a DELETE route.

```ring
@delete("/users/:id", func {
    $bolt.sendStatus(204)
})
```

### @patch(cPath, fHandler)
Register a PATCH route.

```ring
@patch("/users/:id", func {
    $bolt.json([:patched = true])
})
```

### @head(cPath, fHandler)
Register a HEAD route.

```ring
@head("/health", func {
    $bolt.sendStatus(200)
})
```

### @options(cPath, fHandler)
Register an OPTIONS route.

```ring
@options("/api", func {
    $bolt.setHeader("Allow", "GET, POST, PUT, DELETE")
    $bolt.sendStatus(200)
})
```

### @route(cMethod, cPath, fHandler)
Register a route with custom HTTP method.

```ring
@route("CUSTOM", "/path", func {
    $bolt.send("Custom method")
})
```

### addRoute(cMethod, cPath, fHandler)
Register a route (used internally by `@route` and the method-specific helpers).

### @static(cUrlPath, cDirPath)
Serve static files from a directory.

```ring
@static("/public", "./static")
@static("/assets", "/var/www/assets")
```

### @error(fHandler)
Register a global error handler.

```ring
@error(func {
    $bolt.serverError("Something went wrong")
})
```

---

## Route Modifiers

### where(cParamName, cPattern)
Add regex constraint to route parameter.

```ring
@get("/users/:id", func { ... })
where("id", "[0-9]+")  # id must be numeric
```

### whereAll(aConstraints)
Add multiple constraints at once.

```ring
@get("/posts/:year/:month", func { ... })
whereAll([
    ["year", "[0-9]{4}"],
    ["month", "[0-9]{2}"]
])
```

### describe(cDescription)
Add description for OpenAPI docs.

```ring
@get("/users", func { ... })
describe("Get all users")
```

### tag(cTag)
Add tag for OpenAPI docs (can be called multiple times).

```ring
@get("/users", func { ... })
tag("Users")
tag("Admin")
```

### prefix(cPrefix) / endPrefix()
Group routes under a common prefix.

```ring
prefix("/api/v1")
    @get("/users", func { ... })     # /api/v1/users
    @get("/posts", func { ... })     # /api/v1/posts
endPrefix()
```

### before(cMiddlewareName)
Add a before middleware to the last registered route.

```ring
@get("/api/data", func { ... })
before("authMiddleware")
```

### after(cMiddlewareName)
Add an after middleware to the last registered route.

```ring
@get("/api/data", func { ... })
after("logMiddleware")
```

### routeRateLimit(nMax, nWindow)
Add rate limiting to the last registered route.

```ring
@get("/api/data", func { ... })
routeRateLimit(100, 60)
```

---

## Middleware

### @before(fHandler)
Register a handler to run before every request.

```ring
@Before(func {
    $bolt.setHeader("X-Request-Time", "" + $bolt.unixtimeMs())
})
```

### @after(fHandler)
Register a handler to run after every request.

```ring
@after(func {
    $bolt.log("Request completed: " + $bolt.path())
})
```

### @use(cMiddlewareName)
Register named middleware by name.

```ring
@use("auth")
```

---

## Request

### $bolt.method()
Get HTTP method.

```ring
m = $bolt.method()  # "GET", "POST", etc.
```

### $bolt.path()
Get request path.

```ring
p = $bolt.path()  # "/users/123"
```

### $bolt.param(cName)
Get URL parameter.

```ring
# Route: /users/:id
id = $bolt.param("id")
```

### $bolt.query(cName)
Get query string parameter.

```ring
# URL: /search?q=hello&page=1
q = $bolt.query("q")        # "hello"
page = $bolt.query("page")  # "1"
```

### $bolt.header(cName)
Get request header.

```ring
auth = $bolt.header("Authorization")
contentType = $bolt.header("Content-Type")
```

### $bolt.body()
Get raw request body as string.

```ring
raw = $bolt.body()
```

### $bolt.jsonBody()
Parse request body as JSON.

```ring
data = $bolt.jsonBody()
name = data[:name]
```

### $bolt.formField(cName)
Get form field value from multipart form data.

```ring
username = $bolt.formField("username")
password = $bolt.formField("password")
```

### $bolt.requestId()
Get unique request ID.

```ring
id = $bolt.requestId()  # "550e8400-e29b-41d4-a716-446655440000"
```

### $bolt.clientIp()
Get client IP address.

```ring
ip = $bolt.clientIp()  # "192.168.1.100"
```

---

## Response

### $bolt.send(cContent)
Send text response (200 OK).

```ring
$bolt.send("Hello World")
```

### $bolt.sendStatus(nStatus)
Send status code only.

```ring
$bolt.sendStatus(204)  # No Content
```

### $bolt.sendWithStatus(nStatus, cContent)
Send text with custom status.

```ring
$bolt.sendWithStatus(201, "Created")
```

### $bolt.json(aData)
Send JSON response.

```ring
$bolt.json([:name = "John", :age = 30])
```

### $bolt.jsonWithStatus(nStatus, aData)
Send JSON with custom status.

```ring
$bolt.jsonWithStatus(201, [:id = 1, :created = true])
```

### $bolt.sendFile(cFilePath)
Send file with auto-detected MIME type.

```ring
$bolt.sendFile("./files/report.pdf")
```

### $bolt.sendFileAs(cFilePath, cContentType)
Send file with explicit MIME type.

```ring
$bolt.sendFileAs("./files/data.bin", "application/octet-stream")
```

### $bolt.sendBinary(cBase64Data)
Send binary data as response.

```ring
$bolt.sendBinary(base64Data)
```

### $bolt.sendBinaryAs(cBase64Data, cContentType)
Send binary data with a custom content type.

```ring
$bolt.sendBinaryAs(base64Data, "image/png")
```

### $bolt.redirect(cUrl)
302 temporary redirect.

```ring
$bolt.redirect("/login")
```

### $bolt.redirectPermanent(cUrl)
301 permanent redirect.

```ring
$bolt.redirectPermanent("/new-url")
```

### $bolt.notFound()
Send 404 Not Found.

```ring
$bolt.notFound()
```

### $bolt.badRequest(cMessage)
Send 400 Bad Request.

```ring
$bolt.badRequest("Invalid input")
```

### $bolt.serverError(cMessage)
Send 500 Internal Server Error.

```ring
$bolt.serverError("Something went wrong")
```

### $bolt.unauthorized()
Send 401 Unauthorized.

```ring
$bolt.unauthorized()
```

### $bolt.forbidden()
Send 403 Forbidden.

```ring
$bolt.forbidden()
```

---

## Headers

### $bolt.setHeader(cName, cValue)
Set response header.

```ring
$bolt.setHeader("X-Custom", "value")
$bolt.setHeader("Content-Type", "text/plain")
```

### $bolt.etag(cContent)
Generate an ETag hash for content.

```ring
$bolt.etag(body)
```

---

## Cookies

### $bolt.setCookie(cName, cValue)
Set a cookie with default path (`/`).

```ring
$bolt.setCookie("user", "john")
```

### $bolt.setCookieEx(cName, cValue, cOptions)
Set cookie with custom options.

```ring
$bolt.setCookieEx("token", "abc123", "Path=/; Max-Age=3600; HttpOnly; Secure")
```

### $bolt.cookie(cName)
Get cookie value.

```ring
user = $bolt.cookie("user")
```

### $bolt.deleteCookie(cName)
Delete a cookie.

```ring
$bolt.deleteCookie("user")
```

### setCookieSecret(cSecret)
Set secret for signed cookies.

```ring
setCookieSecret("my-secret-key")
```

### $bolt.setSignedCookie(cName, cValue)
Set a signed cookie. Requires `setCookieSecret()` to be called first.

```ring
$bolt.setSignedCookie("session", "data")
```

### $bolt.getSignedCookie(cName)
Get and verify signed cookie. Returns empty string if invalid.

```ring
data = $bolt.getSignedCookie("session")
```

---

## Sessions

### $bolt.setSession(cKey, cValue)
Set session value.

```ring
$bolt.setSession("user_id", "123")
$bolt.setSession("role", "admin")
```

### $bolt.getSession(cKey)
Get session value.

```ring
userId = $bolt.getSession("user_id")
```

### $bolt.deleteSession(cKey)
Delete session key.

```ring
$bolt.deleteSession("user_id")
```

### $bolt.clearSession()
Clear all session data.

```ring
$bolt.clearSession()
```

### $bolt.setFlash(cKey, cValue)
Set flash message (one-time session data).

```ring
$bolt.setFlash("success", "User created!")
```

### $bolt.getFlash(cKey)
Get and clear flash message.

```ring
msg = $bolt.getFlash("success")
```

### $bolt.hasFlash(cKey)
Check if flash message exists.

```ring
if $bolt.hasFlash("error")
    # ...
ok
```

---

## File Uploads

### $bolt.filesCount()
Get number of uploaded files.

```ring
count = $bolt.filesCount()
```

### $bolt.file(nIndex)
Get uploaded file by index (1-based). Returns a list with `:name`, `:field`, `:type`, `:size`.

```ring
f = $bolt.file(1)
? f[:name]   # "photo.jpg"
? f[:field]  # "avatar"
? f[:type]   # "image/jpeg"
? f[:size]   # 102400
```

### $bolt.files()
Get all uploaded files as a list of file lists.

```ring
aFiles = $bolt.files()
for f in aFiles
    ? f[:name]
next
```

### $bolt.fileByField(cName)
Get first file matching a form field name.

```ring
f = $bolt.fileByField("avatar")
? f[:name]
```

### $bolt.fileSave(nIndex, cPath)
Save uploaded file to disk.

```ring
f = $bolt.file(1)
$bolt.fileSave(1, "./uploads/" + f[:name])
```

**Example: Handle file upload**
```ring
@post("/upload", func {
    if $bolt.filesCount() > 0
        for i = 1 to $bolt.filesCount()
            f = $bolt.file(i)
            $bolt.fileSave(i, "./uploads/" + f[:name])
        next
        $bolt.json([:uploaded = $bolt.filesCount()])
    else
        $bolt.badRequest("No files uploaded")
    ok
})
```

---

## JSON

### $bolt.jsonEncode(aList)
Encode list/object to JSON string.

```ring
str = $bolt.jsonEncode([:name = "John"])  # '{"name":"John"}'
```

### $bolt.jsonDecode(cJson)
Decode JSON string to list/object.

```ring
data = $bolt.jsonDecode('{"name":"John"}')
? data[:name]  # John
```

### $bolt.jsonPretty(aList)
Encode to pretty-printed JSON.

```ring
str = $bolt.jsonPretty([:name = "John"])
```

---

## Templates

### $bolt.render(cTemplate, aData)
Render inline template string and send as response.

```ring
@get("/", func {
    $bolt.render("<h1>{{ title }}</h1>", [:title = "Hello"])
})
```

### $bolt.renderTemplate(cTemplate, aData)
Render template string and return the result (does not send).

```ring
html = $bolt.renderTemplate("<h1>{{ title }}</h1>", [:title = "Hello"])
```

### $bolt.renderFile(cFilepath, aData)
Render template from file and send as response.

```ring
@get("/", func {
    $bolt.renderFile("templates/home.html", [:user = "John"])
})
```

**Template Syntax (MiniJinja):**
```html
<h1>{{ title }}</h1>

{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}

{% if user %}
    <p>Welcome, {{ user }}!</p>
{% endif %}
```

---

## Caching

### $bolt.cacheSet(cKey, cValue)
Store value in cache (no expiry).

```ring
$bolt.cacheSet("user:123", $bolt.jsonEncode(userData))
```

### $bolt.cacheSetTTL(cKey, cValue, nTTL)
Store value in cache with a TTL in seconds.

```ring
$bolt.cacheSetTTL("user:123", data, 300)  # Expires in 5 minutes
```

### $bolt.cacheGet(cKey)
Retrieve value from cache. Returns empty string if not found.

```ring
data = $bolt.cacheGet("user:123")
if data != ""
    user = $bolt.jsonDecode(data)
ok
```

### $bolt.cacheDelete(cKey)
Delete cache entry.

```ring
$bolt.cacheDelete("user:123")
```

### $bolt.cacheClear()
Clear entire cache.

```ring
$bolt.cacheClear()
```

---

## WebSocket

### @ws(cPath, fOnConnect, fOnMessage, fOnDisconnect)
Register event-driven WebSocket endpoint with callbacks. Pass `""` for any callback you don't need.

```ring
@ws("/chat",
    func {
        id = $bolt.wsClientId()
        $bolt.wsRoomJoin("chat", id)
        $bolt.wsSendTo(id, "Welcome!")
    },
    func {
        id = $bolt.wsClientId()
        msg = $bolt.wsEventMessage()
        $bolt.wsRoomBroadcast("chat", "[" + id + "] " + msg)
    },
    func {
        $bolt.wsBroadcast("User left: " + $bolt.wsClientId())
    }
)
```

### Event Context (use inside callbacks)

```ring
$bolt.wsClientId()        # Current client's UUID
$bolt.wsEventType()       # "connect", "message", or "disconnect"
$bolt.wsEventMessage()    # Message text (on_message only)
$bolt.wsEventIsBinary()   # 1 if binary message, 0 otherwise
$bolt.wsEventBinary()     # Binary data as base64 string
$bolt.wsEventPath()       # WebSocket route path
$bolt.wsParam(cName)      # Route parameter from WebSocket event
```

### Per-Client Send

```ring
$bolt.wsSendTo(cClientId, cMessage)             # Send text to specific client
$bolt.wsSendBinaryTo(cClientId, cBase64Data)    # Send binary to specific client
$bolt.wsCloseClient(cClientId)                   # Close a client's connection
$bolt.wsClientList()                             # List of connected client IDs
```

### Rooms

```ring
$bolt.wsRoomJoin(cRoom, cClientId)              # Join a room
$bolt.wsRoomLeave(cRoom, cClientId)             # Leave a room
$bolt.wsRoomBroadcast(cRoom, cMessage)          # Send text to all in room
$bolt.wsRoomBroadcastBinary(cRoom, cBase64Data) # Send binary to all in room
$bolt.wsRoomMembers(cRoom)                       # List of client IDs in room
$bolt.wsRoomCount(cRoom)                         # Number of clients in room
```

### Global Broadcast

```ring
$bolt.wsBroadcast(cMessage)     # Send to ALL connected clients
$bolt.wsConnectionCount()       # Total active connections
```

---

## Server-Sent Events

### @sse(cPath)
Register SSE endpoint for clients to subscribe.

```ring
@sse("/events")
```

### $bolt.sseBroadcast(cPath, cData)
Send data event to all subscribers.

```ring
$bolt.sseBroadcast("/events", "New notification!")
```

### $bolt.sseBroadcastEvent(cPath, cEventName, cData)
Send named event to all subscribers.

```ring
$bolt.sseBroadcastEvent("/events", "update", '{"count": 42}')
```

**Client-side:**
```javascript
const es = new EventSource('/events');
es.onmessage = (e) => console.log(e.data);
es.addEventListener('update', (e) => console.log(e.data));
```

---

## Authentication

### $bolt.jwtEncode(aData, cSecret)
Create JWT token.

```ring
token = $bolt.jwtEncode([:user_id = 123, :role = "admin"], "secret")
```

### $bolt.jwtEncodeExp(aData, cSecret, nExpires)
Create JWT with expiration (seconds from now).

```ring
token = $bolt.jwtEncodeExp([:user_id = 123], "secret", 3600)  # 1 hour
```

### $bolt.jwtDecode(cToken, cSecret)
Decode and verify JWT, returns payload.

```ring
data = $bolt.jwtDecode(token, "secret")
if data != NULL
    userId = data[:user_id]
ok
```

### $bolt.jwtVerify(cToken, cSecret)
Verify JWT validity (returns 1 or 0).

```ring
if $bolt.jwtVerify(token, "secret")
    # Valid token
ok
```

### $bolt.basicAuthDecode(cHeader)
Decode Basic Auth header. Returns a list with `:username` and `:password`, or NULL if invalid.

```ring
auth = $bolt.header("Authorization")  # "Basic dXNlcjpwYXNz"
creds = $bolt.basicAuthDecode(auth)
if creds != NULL
    user = creds[:username]
    pass = creds[:password]
ok
```

### $bolt.basicAuthEncode(cUsername, cPassword)
Encode Basic Auth credentials.

```ring
auth = $bolt.basicAuthEncode("user", "pass")  # "dXNlcjpwYXNz"
```

---

## Security

### enableCsrf(cSecret)
Enable CSRF protection.

```ring
enableCsrf("my-csrf-secret")
```

### $bolt.csrfToken()
Generate CSRF token for forms.

```ring
token = $bolt.csrfToken()
# Include in form: <input type="hidden" name="_csrf" value="{{ token }}">
```

### $bolt.verifyCsrf(cToken, cExpected)
Verify CSRF token.

```ring
if $bolt.verifyCsrf($bolt.formField("_csrf"), $bolt.csrfToken())
    # Valid request
ok
```

### $bolt.sha256(cData)
Generate SHA-256 hash.

```ring
hash = $bolt.sha256("password123")
```

### ipWhitelist(cIp)
Add IP or CIDR to whitelist.

```ring
ipWhitelist("192.168.1.100")
ipWhitelist("10.0.0.0/8")
```

### ipBlacklist(cIp)
Add IP or CIDR to blacklist.

```ring
ipBlacklist("1.2.3.4")
```

### proxyWhitelist(cIp)
Add IP to the proxy whitelist.

```ring
proxyWhitelist("10.0.0.1")
```

---

## CORS

### enableCors() / disableCors()
Enable or disable CORS.

```ring
enableCors()
```

### corsOrigin(cOrigin)
Set allowed origin (auto-enables CORS).

```ring
corsOrigin("https://example.com")
corsOrigin("*")  # Allow all origins
```

---

## Rate Limiting

### $bolt.rateLimit(nMax, nWindow)
Configure global rate limiting.

```ring
$bolt.rateLimit(100, 60)  # 100 requests per 60 seconds
```

### $bolt.checkRateLimit()
Check if current request is rate limited (returns 1 if allowed, 0 if limited).

```ring
if !$bolt.checkRateLimit()
    $bolt.sendWithStatus(429, "Too many requests")
    return
ok
```

---

## Validation

### $bolt.validateJson(cJson, cSchema)
Validate JSON against JSON Schema (returns 1 if valid).

```ring
schema = '{
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0}
    },
    "required": ["name"]
}'

if $bolt.validateJson($bolt.body(), schema)
    # Valid
ok
```

### $bolt.validateJsonErrors(cJson, cSchema)
Get validation errors as a decoded list.

```ring
errors = $bolt.validateJsonErrors($bolt.body(), schema)
if errors != ""
    $bolt.badRequest(errors)
ok
```

### $bolt.validateParam(cParamName, cPattern)
Validate URL parameter against regex.

```ring
if !$bolt.validateParam("id", "[0-9]+")
    $bolt.badRequest("Invalid ID format")
ok
```

### $bolt.matchRegex(cValue, cPattern)
Match value against regex (returns 1 or 0).

```ring
if $bolt.matchRegex(email, "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
    # Valid email
ok
```

---

## Utilities

### $bolt.uuid()
Generate UUID v4.

```ring
id = $bolt.uuid()  # "550e8400-e29b-41d4-a716-446655440000"
```

### $bolt.unixtime()
Get current Unix timestamp (seconds).

```ring
ts = $bolt.unixtime()  # 1706889600
```

### $bolt.unixtimeMs()
Get current Unix timestamp (milliseconds).

```ring
ts = $bolt.unixtimeMs()  # 1706889600123
```

### $bolt.urlEncode(cStr)
URL-encode a string.

```ring
encoded = $bolt.urlEncode("hello world")  # "hello%20world"
```

### $bolt.urlDecode(cStr)
URL-decode a string.

```ring
decoded = $bolt.urlDecode("hello%20world")  # "hello world"
```

---

## Health

### $bolt.healthCheck()
Perform health check (returns status data).

```ring
@get("/health", func {
    $bolt.send($bolt.healthCheck())
})
```

---

## OpenAPI Documentation

### enableDocs()
Enable Swagger UI at `/docs`.

```ring
enableDocs()
```

### setDocsInfo(cTitle, cVersion, cDescription)
Set API documentation metadata.

```ring
setDocsInfo("My API", "1.0.0", "API for managing users")
```

### setOpenApiSpec(cSpecJson)
Set custom OpenAPI specification.

```ring
spec = $bolt.jsonEncode([:openapi = "3.1.0", ...])
setOpenApiSpec(spec)
```

### homepage()
Add a default homepage at `/` displaying server info and registered routes. Only adds the route if `/` is not already defined.

```ring
homepage()
```

---

## Logging

### $bolt.log(cMessage)
Write to server log at default level.

```ring
$bolt.log("User logged in: " + userId)
```

### $bolt.logWithLevel(cMessage, cLevel)
Write to server log at a specific level.

```ring
$bolt.logWithLevel("Disk space low", "warn")
$bolt.logWithLevel("Connection failed", "error")
```

### $bolt.setLogLevel(cLevel)
Set the minimum log level.

```ring
$bolt.setLogLevel("warn")  # Only warn and error will be logged  # Only warn and error will be logged
```

---

## Server Control

### startServer()
Start the server (called automatically by braceEnd).

```ring
startServer()  # Usually not needed
```

### $bolt.stop()
Stop the server gracefully.

```ring
$bolt.stop()
```

---

## Env Class

The `Env` class manages environment variables and `.env` files.

```ring
env = new Env()
```

### env.loadEnv()
Load `.env` file from the current directory. Called automatically on `init()`.

```ring
env.loadEnv()
```

### env.loadFile(cPath)
Load environment variables from a specific file.

```ring
env.loadFile("/etc/myapp/env")
```

### env.getVar(cKey)
Get an environment variable value. Returns empty string if not found.

```ring
dbUrl = env.getVar("DATABASE_URL")
```

### env.getOr(cKey, cDefault)
Get an environment variable with a default fallback.

```ring
port = env.getVar("PORT")
if port = "" port = "3000" ok

# Or more concisely:
port = env.getOr("PORT", "3000")
```

### env.setVar(cKey, cValue)
Set an environment variable.

```ring
env.setVar("APP_ENV", "production")
```

---

## Hash Class

The `Hash` class provides secure password hashing with Argon2, bcrypt, and scrypt.

```ring
hash = new Hash
```

### hash.argon2(cPassword)
Hash a password using Argon2id. Returns a PHC-formatted hash string.

```ring
hashed = hash.argon2("mypassword")
```

### hash.verifyArgon2(cPassword, cHash)
Verify a password against an Argon2 hash. Returns 1 if valid, 0 otherwise.

```ring
if hash.verifyArgon2("mypassword", storedHash)
    # Password correct
ok
```

### hash.bcrypt(cPassword)
Hash a password using bcrypt.

```ring
hashed = hash.bcrypt("mypassword")
```

### hash.verifyBcrypt(cPassword, cHash)
Verify a password against a bcrypt hash.

```ring
if hash.verifyBcrypt("mypassword", storedHash)
    # Password correct
ok
```

### hash.scrypt(cPassword)
Hash a password using scrypt.

```ring
hashed = hash.scrypt("mypassword")
```

### hash.verifyScrypt(cPassword, cHash)
Verify a password against a scrypt hash.

```ring
if hash.verifyScrypt("mypassword", storedHash)
    # Password correct
ok
```

---

## Validate Class

The `Validate` class provides input validation for common data types.

```ring
v = new Validate
```

### v.email(cStr)
Validate an email address format.

```ring
if v.email("test@example.com") { ... }
```

### v.url(cStr)
Validate a URL format.

```ring
if v.url("https://example.com") { ... }
```

### v.ip(cStr)
Validate an IP address (v4 or v6).

```ring
if v.ip("192.168.1.1") { ... }
```

### v.ipv4(cStr)
Validate an IPv4 address.

```ring
if v.ipv4("10.0.0.1") { ... }
```

### v.ipv6(cStr)
Validate an IPv6 address.

```ring
if v.ipv6("::1") { ... }
```

### v.uuid(cStr)
Validate a UUID format.

```ring
if v.uuid("550e8400-e29b-41d4-a716-446655440000") { ... }
```

### v.jsonString(cStr)
Validate that a string is valid JSON.

```ring
if v.jsonString('{"key": "value"}') { ... }
```

### v.length(cStr, nMin, nMax)
Validate string length within a range.

```ring
if v.length(username, 3, 20) { ... }
```

### v.range(nNum, nMin, nMax)
Validate a number within a range.

```ring
if v.range(age, 0, 150) { ... }
```

### v.alpha(cStr)
Validate that a string contains only alphabetic characters.

```ring
if v.alpha(name) { ... }
```

### v.alphanumeric(cStr)
Validate that a string contains only alphanumeric characters.

```ring
if v.alphanumeric(username) { ... }
```

### v.numeric(cStr)
Validate that a string contains only numeric characters.

```ring
if v.numeric(zipCode) { ... }
```

---

## Crypto Class

The `Crypto` class provides AES-256-GCM encryption and HMAC-SHA256.

```ring
crypto = new Crypto
```

### crypto.aesEncrypt(cPlaintext, cKey)
Encrypt plaintext using AES-256-GCM. Returns base64-encoded ciphertext with IV and tag.

```ring
encrypted = crypto.aesEncrypt("secret data", "0123456789abcdef0123456789abcdef")
```

### crypto.aesDecrypt(cCiphertext, cKey)
Decrypt AES-256-GCM ciphertext. Returns decrypted plaintext.

```ring
decrypted = crypto.aesDecrypt(encrypted, "0123456789abcdef0123456789abcdef")
```

### crypto.hmacSha256(cMessage, cKey)
Compute HMAC-SHA256 signature. Returns hex-encoded signature.

```ring
sig = crypto.hmacSha256("message", "secret-key")
```

### crypto.hmacVerify(cMessage, cKey, cSignature)
Verify an HMAC-SHA256 signature. Returns 1 if valid, 0 otherwise.

```ring
if crypto.hmacVerify("message", "secret-key", sig)
    # Signature valid
ok
```

---

## DateTime Class

The `DateTime` class provides timestamp operations, formatting, and arithmetic.

```ring
dt = new DateTime
```

### dt.now()
Get the current local datetime string in ISO format.

```ring
cNow = dt.now()  # "2026-05-02T14:30:00"
```

### dt.nowUtc()
Get the current UTC datetime string in ISO format.

```ring
cUtc = dt.nowUtc()
```

### dt.timestamp()
Get the current Unix timestamp in seconds.

```ring
ts = dt.timestamp()
```

### dt.timestampMs()
Get the current Unix timestamp in milliseconds.

```ring
tsMs = dt.timestampMs()
```

### dt.formatDate(nTimestamp, cFormat)
Format a Unix timestamp to a string.

```ring
cDate = dt.formatDate(ts, "%Y-%m-%d %H:%M:%S")  # "2026-05-02 14:30:00"
```

### dt.parseDate(cDateStr, cFormat)
Parse a datetime string to a Unix timestamp.

```ring
ts = dt.parseDate("2026-05-02 14:30:00", "%Y-%m-%d %H:%M:%S")
```

### dt.diff(nTs1, nTs2)
Calculate the difference between two timestamps in seconds.

```ring
diff = dt.diff(ts1, ts2)
```

### dt.addDays(nTimestamp, nDays)
Add days to a timestamp.

```ring
future = dt.addDays(ts, 7)  # 7 days from now
```

### dt.addHours(nTimestamp, nHours)
Add hours to a timestamp.

```ring
future = dt.addHours(ts, 24)  # 24 hours from now
```

---

## Sanitize Class

The `Sanitize` class provides HTML and XSS sanitization.

```ring
s = new Sanitize
```

### s.html(cInput)
Sanitize HTML by stripping dangerous tags, keeping safe ones.

```ring
safe = s.html('<script>alert("xss")</script><p>Safe</p>')
# Returns: "<p>Safe</p>"
```

### s.strict(cInput)
Strictly sanitize HTML by stripping all tags.

```ring
text = s.strict('<b>Bold</b> <script>evil()</script>')
# Returns: "Bold evil()"
```

### s.escapeHtml(cInput)
Escape HTML special characters to entities.

```ring
escaped = s.escapeHtml('<div class="test">Hello & goodbye</div>')
# Returns: "&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;"
```

---

## Complete Example

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # Configuration
    setBodyLimit(10 * 1024 * 1024)
    setTimeout(30000)
    enableCors()
    corsOrigin("*")
    enableCompression()
    enableLogging()
    
    # OpenAPI docs
    setDocsInfo("User API", "1.0.0", "User management API")
    enableDocs()
    
    # Middleware
    @before(func {
        $bolt.setHeader("X-Request-Id", $bolt.requestId())
    })
    
    # Routes
    prefix("/api/v1")
    
        @get("/users", func {
            $bolt.json([:users = []])
        })
        describe("Get all users")
        tag("Users")
        
        @get("/users/:id", func {
            $bolt.json([:id = $bolt.param("id")])
        })
        where("id", "[0-9]+")
        describe("Get user by ID")
        tag("Users")
        
        @post("/users", func {
            data = $bolt.jsonBody()
            $bolt.jsonWithStatus(201, [:created = true, :data = data])
        })
        describe("Create new user")
        tag("Users")
        
    endPrefix()
    
    # Static files
    @static("/public", "./static")
    
    # Health check
    @get("/health", func {
        $bolt.json([:status = "ok", :time = $bolt.unixtime()])
    })
}
```
