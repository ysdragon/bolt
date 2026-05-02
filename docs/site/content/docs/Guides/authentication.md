---
title: "Authentication"
weight: 13
summary: "JWT tokens and Basic Auth for securing routes"
---

### JWT Authentication

```ring
load "bolt.ring"

SECRET = "your-super-secret-key"

new Bolt() {
    port = 3000
    
    # Login - get token
    @post("/login", func {
        data = $bolt.jsonBody()
        
        if data[:username] = "admin" and data[:password] = "secret"
            # Create token with 1 hour expiry (seconds from now)
            token = $bolt.jwtEncodeExp([
                :user_id = 1,
                :username = data[:username],
                :role = "admin"
            ], SECRET, 3600)
            
            $bolt.json([:token = token])
        else
            $bolt.unauthorized()
        ok
    })
    
    # Protected route
    @get("/profile", func {
        auth = $bolt.header("Authorization")
        
        if auth = ""
            $bolt.unauthorized()
            return
        ok
        
        # Extract token (Bearer xxx)
        token = substr(auth, 8)  # Skip "Bearer "
        
        if !$bolt.jwtVerify(token, SECRET)
            $bolt.unauthorized()
            return
        ok
        
        payload = $bolt.jwtDecode(token, SECRET)
        $bolt.json([
            :user_id = payload[:user_id],
            :username = payload[:username],
            :role = payload[:role]
        ])
    })
}
```

### Basic Auth

```ring
@before(func {
    # Skip public routes
    if $bolt.path() = "/" or $bolt.path() = "/public"
        return
    ok
    
    auth = $bolt.header("Authorization")
    if auth = "" or left(auth, 6) != "Basic "
        $bolt.setHeader("WWW-Authenticate", 'Basic realm="Protected"')
        $bolt.unauthorized()
        return
    ok
    
    # Decode credentials
    creds = $bolt.basicAuthDecode(auth)
    
    if creds = NULL
        $bolt.unauthorized()
        return
    ok
    
    username = creds[:username]
    password = creds[:password]
    
    if username != "admin" or password != "secret"
        $bolt.unauthorized()
    ok
})
```
