---
title: "Route Modifiers"
weight: 3
summary: "Parameter constraints, descriptions, tags, prefixes, and per-route middleware"
---

### where(cParamName, cPattern)
Add regex constraint to route parameter.

```ring
@get("/users/:id", func { ... })
where("id", "[0-9]+")  # id must be numeric
```

### whereAll(aConstraints)
Add multiple constraints at once.

```ring
@get("/posts/:year/:month", func { ... })
whereAll([
    ["year", "[0-9]{4}"],
    ["month", "[0-9]{2}"]
])
```

### describe(cDescription)
Add description for OpenAPI docs.

```ring
@get("/users", func { ... })
describe("Get all users")
```

### tag(cTag)
Add tag for OpenAPI docs (can be called multiple times).

```ring
@get("/users", func { ... })
tag("Users")
tag("Admin")
```

### prefix(cPrefix) / endPrefix()
Group routes under a common prefix.

```ring
prefix("/api/v1")
    @get("/users", func { ... })     # /api/v1/users
    @get("/posts", func { ... })     # /api/v1/posts
endPrefix()
```

### before(cMiddlewareName)
Add a before middleware to the last registered route.

```ring
@get("/api/data", func { ... })
before("authMiddleware")
```

### after(cMiddlewareName)
Add an after middleware to the last registered route.

```ring
@get("/api/data", func { ... })
after("logMiddleware")
```

### routeRateLimit(nMax, nWindow)
Add rate limiting to the last registered route.

```ring
@get("/api/data", func { ... })
routeRateLimit(100, 60)
```
