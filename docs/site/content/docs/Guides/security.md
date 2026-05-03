---
title: "Security"
weight: 14
summary: "CSRF protection, rate limiting, IP filtering, and TLS"
---

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
        
        if !$bolt.verifyCsrf(token)
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
