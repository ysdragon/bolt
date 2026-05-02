---
title: "Routing Basics"
weight: 3
summary: "Learn how to define routes"
---

## HTTP Methods

Bolt supports all standard HTTP methods:

```ring
new Bolt() {
    port = 3000

    # GET - Retrieve data
    @get("/users", func {
        $bolt.json([:users = [[:id = 1, :name = "Alice"]]])
    })

    # POST - Create data
    @post("/users", func {
        data = $bolt.jsonBody()
        $bolt.jsonWithStatus(201, [:created = true, :user = data])
    })

    # PUT - Update/replace data
    @put("/users/:id", func {
        data = $bolt.jsonBody()
        $bolt.json([:updated = $bolt.param("id"), :data = data])
    })

    # PATCH - Partial update
    @patch("/users/:id", func {
        data = $bolt.jsonBody()
        $bolt.json([:patched = $bolt.param("id")])
    })

    # DELETE - Remove data
    @delete("/users/:id", func {
        $bolt.sendStatus(204)  # No content
    })

    # HEAD - Headers only (no body)
    @head("/health", func {
        $bolt.setHeader("X-Status", "healthy")
        $bolt.sendStatus(200)
    })

    # OPTIONS - CORS preflight
    @options("/api/*", func {
        $bolt.setHeader("Allow", "GET, POST, PUT, DELETE, OPTIONS")
        $bolt.sendStatus(200)
    })
}
```

## Route Parameters

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
```

## Route Constraints

Validate parameters with regex:

```ring
# Numeric ID only
@get("/users/:id", func {
    $bolt.json([:id = $bolt.param("id")])
})
where("id", "[0-9]+")
# /users/123 → OK
# /users/abc → 404

# Multiple constraints
@get("/archive/:year/:month", func {
    $bolt.json([:year = $bolt.param("year"), :month = $bolt.param("month")])
})
whereAll([
    ["year", "[0-9]{4}"],
    ["month", "(0[1-9]|1[0-2])"]
])
```

## Route Prefixes

Group related routes:

```ring
new Bolt() {
    port = 3000

    prefix("/api/v1")
        @get("/users", func { $bolt.json([:version = 1]) })
        @get("/posts", func { $bolt.json([:version = 1]) })
    endPrefix()

    prefix("/api/v2")
        @get("/users", func { $bolt.json([:version = 2]) })
        @get("/posts", func { $bolt.json([:version = 2]) })
    endPrefix()
}
```
