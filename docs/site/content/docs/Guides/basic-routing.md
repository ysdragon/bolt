---
title: "Basic Routing"
weight: 1
summary: "HTTP methods and route prefixes for defining endpoints"
---

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
