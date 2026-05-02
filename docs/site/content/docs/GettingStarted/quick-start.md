---
title: "Quick Start"
weight: 2
summary: "One-line hello world and a real app"
---

## Hello World

One line is all it takes:

```ring
load "bolt.ring"
new Bolt() { $bolt.send("Hello, World!") }
```

Run it:

```bash
ring app.ring
# [bolt] Server running on http://0.0.0.0:3000
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

```bash
ring app.ring
# [bolt] Server running on http://0.0.0.0:3000
```
