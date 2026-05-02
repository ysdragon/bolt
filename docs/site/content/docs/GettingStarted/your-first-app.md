---
title: "Your First App"
weight: 2
summary: "Create a basic Bolt application"
---

## Hello World

The simplest Bolt app:

```ring
load "bolt.ring"
new Bolt() { $bolt.send("Hello, World!") }
```

## A Real App

A more complete example with routing, params, and JSON:

```ring
load "bolt.ring"

new Bolt() {
    port = 3000

    @get("/", func {
        $bolt.send("Hello from Bolt!")
    })

    @get("/users/:id", func {
        $bolt.json([
            :id   = $bolt.param("id"),
            :name = "User " + $bolt.param("id")
        ])
    })
    where("id", "[0-9]+")

    @post("/users", func {
        data = $bolt.jsonBody()
        $bolt.jsonWithStatus(201, [:created = true, :data = data])
    })
}
```

Run it:

```bash
ring app.ring
```

## What's Happening

1. `load "bolt.ring"` — loads the Bolt framework
2. `new Bolt() { ... }` — creates a new server instance
3. `port = 3000` — sets the listening port
4. `@get("/", func { ... })` — defines a GET route
5. `$bolt.send(...)` — sends a text response
6. `$bolt.json(...)` — sends a JSON response
7. `$bolt.param("id")` — reads a URL parameter
8. `$bolt.jsonBody()` — parses the request body as JSON
9. `where("id", "[0-9]+")` — constrains the `:id` param to digits
