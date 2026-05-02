---
title: "OpenAPI Documentation"
weight: 19
summary: "Auto-generated Swagger UI and API documentation"
---

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
