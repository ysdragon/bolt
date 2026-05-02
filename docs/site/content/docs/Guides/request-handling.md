---
title: "Request Handling"
weight: 3
summary: "Access request data including headers, body, and form fields"
---

### Accessing Request Data

```ring
@post("/api/data", func {
    # HTTP method
    httpMethod = $bolt.method()  # "POST"
    
    # Request path
    requestPath = $bolt.path()   # "/api/data"
    
    # Headers
    contentType = $bolt.header("Content-Type")
    userAgent = $bolt.header("User-Agent")
    auth = $bolt.header("Authorization")
    
    # Raw body
    rawBody = $bolt.body()
    
    # Client info
    ip = $bolt.clientIp()
    reqId = $bolt.requestId()  # Unique request ID
    
    $bolt.json([
        :method = httpMethod,
        :path = requestPath,
        :contentType = contentType,
        :ip = ip,
        :requestId = reqId
    ])
})
```

### JSON Body

```ring
@post("/users", func {
    # Parse JSON body
    data = $bolt.jsonBody()
    
    # Access fields
    name = data[:name]
    email = data[:email]
    
    # Validate
    if name = "" or email = ""
        $bolt.badRequest("Name and email required")
        return
    ok
    
    # Process...
    $bolt.jsonWithStatus(201, [:id = 1, :name = name, :email = email])
})
```

### Form Data

```ring
@post("/login", func {
    username = $bolt.formField("username")
    password = $bolt.formField("password")
    remember = $bolt.formField("remember")
    
    if username = "admin" and password = "secret"
        $bolt.setSession("user", username)
        $bolt.redirect("/dashboard")
    else
        $bolt.badRequest("Invalid credentials")
    ok
})
```
