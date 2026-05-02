---
title: "JSON APIs"
weight: 5
summary: "Building RESTful APIs with JSON validation"
---

### RESTful API Example

```ring
load "bolt.ring"

# In-memory data store
users = [
    [:id = 1, :name = "Alice", :email = "alice@example.com"],
    [:id = 2, :name = "Bob", :email = "bob@example.com"]
]
nextId = 3

new Bolt() {
    port = 3000
    enableCors()
    
    prefix("/api")
    
        # List all users
        @get("/users", func {
            $bolt.json([:data = users, :count = len(users)])
        })
        
        # Get single user
        @get("/users/:id", func {
            id = number($bolt.param("id"))
            for user in users
                if user[:id] = id
                    $bolt.json(user)
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
        
        # Create user
        @post("/users", func {
            data = $bolt.jsonBody()
            
            if data[:name] = "" or data[:email] = ""
                $bolt.badRequest("Name and email required")
                return
            ok
            
            user = [
                :id = nextId,
                :name = data[:name],
                :email = data[:email]
            ]
            add(users, user)
            nextId++
            
            $bolt.jsonWithStatus(201, user)
        })
        
        # Update user
        @put("/users/:id", func {
            id = number($bolt.param("id"))
            data = $bolt.jsonBody()
            
            for i = 1 to len(users)
                if users[i][:id] = id
                    users[i][:name] = data[:name]
                    users[i][:email] = data[:email]
                    $bolt.json(users[i])
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
        
        # Delete user
        @delete("/users/:id", func {
            id = number($bolt.param("id"))
            
            for i = 1 to len(users)
                if users[i][:id] = id
                    del(users, i)
                    $bolt.sendStatus(204)
                    return
                ok
            next
            $bolt.notFound()
        })
        where("id", "[0-9]+")
    
    endPrefix()
}
```

### JSON Validation

```ring
@post("/users", func {
    schema = '{
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 1},
            "email": {"type": "string", "format": "email"},
            "age": {"type": "integer", "minimum": 0}
        },
        "required": ["name", "email"]
    }'
    
    errors = $bolt.validateJsonErrors($bolt.body(), schema)
    if errors != ""
        $bolt.jsonWithStatus(400, [:error = "Validation failed", :details = errors])
        return
    ok
    
    data = $bolt.jsonBody()
    # Process valid data...
})
```
