---
title: "Response Methods"
weight: 4
summary: "Text, JSON, file, binary, redirect, and error responses"
---

### Text Responses

```ring
# Simple text
$bolt.send("Hello, World!")

# With custom status
$bolt.sendWithStatus(201, "Resource created")

# Status code only
$bolt.sendStatus(204)  # No content
```

### JSON Responses

```ring
# Standard JSON
$bolt.json([:name = "John", :age = 30])

# With status code
$bolt.jsonWithStatus(201, [:id = 1, :created = true])

# Nested objects
$bolt.json([
    :user = [
        :id = 1,
        :name = "John",
        :email = "john@example.com"
    ],
    :roles = ["admin", "user"]
])
```

### File Responses

```ring
# Auto-detect MIME type
$bolt.sendFile("./files/report.pdf")

# Explicit MIME type
$bolt.sendFileAs("./files/data.bin", "application/octet-stream")
```

### Binary Responses

```ring
# Send binary data (base64-encoded)
$bolt.sendBinary(base64EncodedData)

# Send binary with explicit MIME type
$bolt.sendBinaryAs(base64EncodedData, "image/png")
```

### Redirects

```ring
# Temporary redirect (302)
$bolt.redirect("/login")

# Permanent redirect (301)
$bolt.redirectPermanent("/new-location")
```

### Error Responses

```ring
$bolt.notFound()           # 404 Not Found
$bolt.badRequest("msg")    # 400 Bad Request
$bolt.unauthorized()       # 401 Unauthorized
$bolt.forbidden()          # 403 Forbidden
$bolt.serverError("msg")   # 500 Internal Server Error
```

### Custom Headers

```ring
@get("/download", func {
    $bolt.setHeader("Content-Disposition", "attachment; filename=data.csv")
    $bolt.setHeader("Cache-Control", "no-cache")
    $bolt.sendFile("./exports/data.csv")
})
```
