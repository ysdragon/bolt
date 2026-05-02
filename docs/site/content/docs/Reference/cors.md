---
title: "CORS"
weight: 18
summary: "Enable and configure Cross-Origin Resource Sharing"
---

### enableCors() / disableCors()
Enable or disable CORS.

```ring
enableCors()
```

### corsOrigin(cOrigin)
Set allowed origin (auto-enables CORS).

```ring
corsOrigin("https://example.com")
corsOrigin("*")  # Allow all origins
```
