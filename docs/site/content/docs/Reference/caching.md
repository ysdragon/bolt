---
title: "Caching"
weight: 13
summary: "In-memory cache with TTL support for storing and retrieving values"
---

### $bolt.cacheSet(cKey, cValue)
Store value in cache (no expiry).

```ring
$bolt.cacheSet("user:123", $bolt.jsonEncode(userData))
```

### $bolt.cacheSetTTL(cKey, cValue, nTTL)
Store value in cache with a TTL in seconds.

```ring
$bolt.cacheSetTTL("user:123", data, 300)  # Expires in 5 minutes
```

### $bolt.cacheGet(cKey)
Retrieve value from cache. Returns empty string if not found.

```ring
data = $bolt.cacheGet("user:123")
if data != ""
    user = $bolt.jsonDecode(data)
ok
```

### $bolt.cacheDelete(cKey)
Delete cache entry.

```ring
$bolt.cacheDelete("user:123")
```

### $bolt.cacheClear()
Clear entire cache.

```ring
$bolt.cacheClear()
```
