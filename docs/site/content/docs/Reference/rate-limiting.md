---
title: "Rate Limiting"
weight: 19
summary: "Per-IP and per-route request rate limiting"
---

### $bolt.rateLimit(nMax, nWindow)
Configure per-IP rate limiting. Each client IP gets its own counter.

```ring
$bolt.rateLimit(100, 60)  # 100 requests per 60 seconds per IP
```

### $bolt.checkRateLimit()
Check if current request is rate limited (returns 1 if allowed, 0 if limited). Rate limiting is per client IP.

```ring
if !$bolt.checkRateLimit()
    $bolt.sendWithStatus(429, "Too many requests")
    return
ok
```

The per-IP rate limiter and per-route governor limiters both run periodic cleanup (every 5 minutes) to remove expired entries, preventing unbounded memory growth from unique IP addresses. The per-IP map is hard-capped at 200,000 entries with fail-open behavior under distributed flood (requests are allowed through when the cap is reached). A warning is logged when a per-route limiter exceeds 10,000 tracked keys.
