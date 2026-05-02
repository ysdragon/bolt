---
title: "Rate Limiting"
weight: 19
summary: "Global and per-route request rate limiting"
---

### $bolt.rateLimit(nMax, nWindow)
Configure global rate limiting.

```ring
$bolt.rateLimit(100, 60)  # 100 requests per 60 seconds
```

### $bolt.checkRateLimit()
Check if current request is rate limited (returns 1 if allowed, 0 if limited).

```ring
if !$bolt.checkRateLimit()
    $bolt.sendWithStatus(429, "Too many requests")
    return
ok
```
