---
title: "Middleware"
weight: 7
summary: "Before/after hooks, CORS middleware, and request timing"
---

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
