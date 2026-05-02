---
title: "Error Handling"
weight: 16
summary: "Global error handlers and custom error responses"
---

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
