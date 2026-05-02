---
title: "OpenAPI Documentation"
weight: 23
summary: "Swagger UI, API metadata, custom specs, and auto-generated homepage"
---

### enableDocs()
Enable Swagger UI at `/docs`.

```ring
enableDocs()
```

### setDocsInfo(cTitle, cVersion, cDescription)
Set API documentation metadata.

```ring
setDocsInfo("My API", "1.0.0", "API for managing users")
```

### setOpenApiSpec(cSpecJson)
Set custom OpenAPI specification.

```ring
spec = $bolt.jsonEncode([:openapi = "3.1.0", ...])
setOpenApiSpec(spec)
```

### homepage()
Add a default homepage at `/` displaying server info and registered routes. Only adds the route if `/` is not already defined.

```ring
homepage()
```
