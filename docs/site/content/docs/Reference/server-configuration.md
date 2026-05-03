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
