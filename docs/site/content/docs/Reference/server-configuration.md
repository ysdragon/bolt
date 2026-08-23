---
title: "Server Configuration"
weight: 1
summary: "Port, host, timeouts, body limits, session and cache capacity, TLS, compression, and logging settings"
---

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
Set request timeout in milliseconds. Must be a positive, finite number; zero, negative, NaN, or infinite values are rejected.

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

### setMultipartFieldCountLimit(nMaxFields)
Set maximum number of multipart form fields.

```ring
setMultipartFieldCountLimit(100)  # Max 100 fields
```

**Default:** 1000 fields

### setMultipartFieldSizeLimit(nBytes)
Set maximum size per multipart form field in bytes.

```ring
setMultipartFieldSizeLimit(5 * 1024 * 1024)  # 5MB per field
```

**Default:** 10MB

### ipWhitelist(cIp) / ipBlacklist(cIp)
Add an IP address (or CIDR range) to the whitelist or blacklist.

```ring
ipWhitelist("192.168.1.0/24")
ipBlacklist("10.0.0.5")
```

### proxyWhitelist(cIp)
Add a trusted proxy IP. Requests from these IPs will use X-Forwarded-For / X-Real-IP headers for client IP resolution.

```ring
proxyWhitelist("10.0.0.1")
```

### setWsMaxConnections(nMax)
Set maximum total concurrent WebSocket connections.

```ring
setWsMaxConnections(500)  # Max 500 total connections
```

**Default:** 1000

### setWsMaxPerIp(nMax)
Set maximum WebSocket connections per client IP.

```ring
setWsMaxPerIp(5)  # Max 5 connections per IP
```

**Default:** 10

### setWsMessageRateLimit(nRate)
Set per-client WebSocket message rate limit in messages per second. Set to 0 to disable.

```ring
setWsMessageRateLimit(50)  # Max 50 messages/sec per client
```

**Default:** 100

### forceSecureCookies()
Force the `Secure` flag on session cookies even when TLS is not enabled. Useful when running behind a TLS-terminating proxy.

```ring
forceSecureCookies()
```
