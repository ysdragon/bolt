---
title: "Complete Example"
weight: 32
summary: "Full working Bolt application demonstrating configuration, middleware, routing, and OpenAPI"
---

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
