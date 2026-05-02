---
title: "Headers"
weight: 7
summary: "Set response headers and generate ETags"
---

### $bolt.setHeader(cName, cValue)
Set response header.

```ring
$bolt.setHeader("X-Custom", "value")
$bolt.setHeader("Content-Type", "text/plain")
```

### $bolt.etag(cContent)
Generate an ETag hash for content.

```ring
$bolt.etag(body)
```
