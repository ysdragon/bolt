---
title: "Middleware"
weight: 4
summary: "Before and after request handlers, named middleware"
---

### @before(fHandler)
Register a handler to run before every request.

```ring
@Before(func {
    $bolt.setHeader("X-Request-Time", "" + $bolt.unixtimeMs())
})
```

### @after(fHandler)
Register a handler to run after every request.

```ring
@after(func {
    $bolt.log("Request completed: " + $bolt.path())
})
```

### @use(cMiddlewareName)
Register named middleware by name.

```ring
@use("auth")
```
