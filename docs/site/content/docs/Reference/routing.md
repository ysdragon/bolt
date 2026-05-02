---
title: "Routing"
weight: 2
summary: "HTTP method routes, static files, error handlers, and custom methods"
---

### @get(cPath, fHandler)
Register a GET route.

```ring
@get("/users", func {
    $bolt.json([:users = []])
})
```

### @post(cPath, fHandler)
Register a POST route.

```ring
@post("/users", func {
    data = $bolt.jsonBody()
    $bolt.jsonWithStatus(201, [:created = true])
})
```

### @put(cPath, fHandler)
Register a PUT route.

```ring
@put("/users/:id", func {
    $bolt.json([:updated = $bolt.param("id")])
})
```

### @delete(cPath, fHandler)
Register a DELETE route.

```ring
@delete("/users/:id", func {
    $bolt.sendStatus(204)
})
```

### @patch(cPath, fHandler)
Register a PATCH route.

```ring
@patch("/users/:id", func {
    $bolt.json([:patched = true])
})
```

### @head(cPath, fHandler)
Register a HEAD route.

```ring
@head("/health", func {
    $bolt.sendStatus(200)
})
```

### @options(cPath, fHandler)
Register an OPTIONS route.

```ring
@options("/api", func {
    $bolt.setHeader("Allow", "GET, POST, PUT, DELETE")
    $bolt.sendStatus(200)
})
```

### @route(cMethod, cPath, fHandler)
Register a route with custom HTTP method.

```ring
@route("CUSTOM", "/path", func {
    $bolt.send("Custom method")
})
```

### addRoute(cMethod, cPath, fHandler)
Register a route (used internally by `@route` and the method-specific helpers).

### @static(cUrlPath, cDirPath)
Serve static files from a directory.

```ring
@static("/public", "./static")
@static("/assets", "/var/www/assets")
```

### @error(fHandler)
Register a global error handler.

```ring
@error(func {
    $bolt.serverError("Something went wrong")
})
```
