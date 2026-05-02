---
title: "Health"
weight: 22
summary: "Built-in health check endpoint"
---

### $bolt.healthCheck()
Perform health check (returns status data).

```ring
@get("/health", func {
    $bolt.send($bolt.healthCheck())
})
```
