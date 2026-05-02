---
title: "Configuration"
weight: 17
summary: "Server settings, timeouts, security options, and feature flags"
---

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
