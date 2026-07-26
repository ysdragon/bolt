---
title: "Route Parameters"
weight: 2
summary: "URL parameters, route constraints, and query string handling"
---

### URL Parameters

```ring
# Single parameter
@get("/users/:id", func {
    userId = $bolt.param("id")
    $bolt.json([:id = userId])
})

# Multiple parameters
@get("/posts/:postId/comments/:commentId", func {
    postId = $bolt.param("postId")
    commentId = $bolt.param("commentId")
    $bolt.json([
        :postId = postId,
        :commentId = commentId
    ])
})

# Example: /users/123 → {"id": "123"}
# Example: /posts/1/comments/5 → {"postId": "1", "commentId": "5"}
```

### Wildcard / Catch-All Routes

A `*` segment matches the rest of the URL path (including `/`), enabling
catch-all routes such as SPA fallbacks, proxies, or custom 404 handlers.

```ring
# Named wildcard — capture the matched tail
@get("/files/*path", func {
    # /files/a/b/c.txt → path = "a/b/c.txt"
    $bolt.json([:path = $bolt.param("path")])
})

# Anonymous wildcard — match anything, no param captured
@get("/*", func {
    $bolt.html(renderTemplate("index.html"))
})
```

A wildcard must be the **last** segment. Named wildcards (`*path`) are
accessible via `$bolt.param("path")`; anonymous wildcards (`*`) are not
exposed as a parameter.

### Route Constraints

Validate parameters with regex:

```ring
# Numeric ID only
@get("/users/:id", func {
    $bolt.json([:id = $bolt.param("id")])
})
where("id", "[0-9]+")
# ✓ /users/123
# ✗ /users/abc (404)

# UUID format
@get("/items/:uuid", func {
    $bolt.json([:uuid = $bolt.param("uuid")])
})
where("uuid", "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")

# Multiple constraints
@get("/archive/:year/:month", func {
    $bolt.json([:year = $bolt.param("year"), :month = $bolt.param("month")])
})
whereAll([
    ["year", "[0-9]{4}"],
    ["month", "(0[1-9]|1[0-2])"]
])
# ✓ /archive/2024/03
# ✗ /archive/2024/15 (404)
```

### Query Strings

```ring
@get("/search", func {
    q = $bolt.query("q")
    page = $bolt.query("page")
    limit = $bolt.query("limit")
    
    if page = "" page = "1" ok
    if limit = "" limit = "10" ok
    
    $bolt.json([
        :query = q,
        :page = page,
        :limit = limit
    ])
})

# /search?q=hello&page=2&limit=20
# → {"query": "hello", "page": "2", "limit": "20"}
```

### Multi-value Query Parameters

When the same key appears multiple times (e.g., `?tag=rust&tag=web`), use `queryAll()` to get all values as a list:

```ring
@get("/search", func {
    tags = $bolt.queryAll("tag")  # ["rust", "web"]
    
    $bolt.json([:tags = tags])
})

# /search?tag=rust&tag=web
# → {"tags": ["rust", "web"]}
```
