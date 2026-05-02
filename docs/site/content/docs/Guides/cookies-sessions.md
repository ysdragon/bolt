---
title: "Cookies & Sessions"
weight: 8
summary: "Cookie management, signed cookies, sessions, and flash messages"
---

### Basic Cookies

```ring
# Set cookie
@get("/set-cookie", func {
    $bolt.setCookie("username", "john")
    $bolt.send("Cookie set!")
})

# Read cookie
@get("/get-cookie", func {
    username = $bolt.cookie("username")
    if username != ""
        $bolt.send("Hello, " + username)
    else
        $bolt.send("No cookie found")
    ok
})

# Delete cookie
@get("/delete-cookie", func {
    $bolt.deleteCookie("username")
    $bolt.send("Cookie deleted!")
})
```

### Cookie Options

```ring
# Set cookie with options
$bolt.setCookieEx("session", "abc123", "path=/; httponly; secure; max-age=3600; samesite=strict")

# Options explained:
# - path=/        : Cookie valid for all paths
# - httponly      : Not accessible via JavaScript
# - secure        : Only sent over HTTPS
# - max-age=3600  : Expires in 1 hour
# - samesite=strict : CSRF protection
```

### Signed Cookies

```ring
new Bolt() {
    port = 3000
    
    # Set cookie signing secret
    setCookieSecret("my-super-secret-key-32chars!")
    
    @get("/set", func {
        $bolt.setSignedCookie("user_id", "12345")
        $bolt.send("Signed cookie set")
    })
    
    @get("/get", func {
        userId = $bolt.getSignedCookie("user_id")
        if userId != NULL
            $bolt.send("User ID: " + userId)
        else
            $bolt.send("Invalid or missing cookie")
        ok
    })
}
```

### Sessions

```ring
@post("/login", func {
    username = $bolt.formField("username")
    password = $bolt.formField("password")
    
    if username = "admin" and password = "secret"
        $bolt.setSession("user_id", "1")
        $bolt.setSession("username", username)
        $bolt.setSession("role", "admin")
        $bolt.redirect("/dashboard")
    else
        $bolt.badRequest("Invalid credentials")
    ok
})

@get("/dashboard", func {
    userId = $bolt.getSession("user_id")
    if userId = ""
        $bolt.redirect("/login")
        return
    ok
    
    username = $bolt.getSession("username")
    $bolt.send("Welcome, " + username + "!")
})

@get("/logout", func {
    $bolt.clearSession()
    $bolt.redirect("/login")
})
```

### Flash Messages

```ring
@post("/action", func {
    # Do something...
    $bolt.setFlash("success", "Action completed successfully!")
    $bolt.redirect("/result")
})

@get("/result", func {
    if $bolt.hasFlash("success")
        msg = $bolt.getFlash("success")  # Automatically cleared after reading
        $bolt.send("Message: " + msg)
    else
        $bolt.send("No message")
    ok
})
```
