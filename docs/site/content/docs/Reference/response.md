---
title: "Response"
weight: 6
summary: "Send text, JSON, files, binary data, redirects, and error responses"
---

### $bolt.send(cContent)
Send text response (200 OK).

```ring
$bolt.send("Hello World")
```

### $bolt.sendStatus(nStatus)
Send status code only.

```ring
$bolt.sendStatus(204)  # No Content
```

### $bolt.sendWithStatus(nStatus, cContent)
Send text with custom status.

```ring
$bolt.sendWithStatus(201, "Created")
```

### $bolt.json(aData)
Send JSON response.

```ring
$bolt.json([:name = "John", :age = 30])
```

### $bolt.jsonWithStatus(nStatus, aData)
Send JSON with custom status.

```ring
$bolt.jsonWithStatus(201, [:id = 1, :created = true])
```

### $bolt.sendFile(cFilePath)
Send file with auto-detected MIME type.

```ring
$bolt.sendFile("./files/report.pdf")
```

### $bolt.sendFileAs(cFilePath, cContentType)
Send file with explicit MIME type.

```ring
$bolt.sendFileAs("./files/data.bin", "application/octet-stream")
```

### $bolt.sendBinary(cBase64Data)
Send binary data as response.

```ring
$bolt.sendBinary(base64Data)
```

### $bolt.sendBinaryAs(cBase64Data, cContentType)
Send binary data with a custom content type.

```ring
$bolt.sendBinaryAs(base64Data, "image/png")
```

### $bolt.redirect(cUrl)
302 temporary redirect.

```ring
$bolt.redirect("/login")
```

### $bolt.redirectPermanent(cUrl)
301 permanent redirect.

```ring
$bolt.redirectPermanent("/new-url")
```

### $bolt.notFound()
Send 404 Not Found.

```ring
$bolt.notFound()
```

### $bolt.badRequest(cMessage)
Send 400 Bad Request.

```ring
$bolt.badRequest("Invalid input")
```

### $bolt.serverError(cMessage)
Send 500 Internal Server Error.

```ring
$bolt.serverError("Something went wrong")
```

### $bolt.unauthorized()
Send 401 Unauthorized.

```ring
$bolt.unauthorized()
```

### $bolt.forbidden()
Send 403 Forbidden.

```ring
$bolt.forbidden()
```
