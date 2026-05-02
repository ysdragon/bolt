---
title: "Caching"
weight: 15
summary: "In-memory cache and ETag-based HTTP caching"
---

### In-Memory Cache

```ring
# Cache expensive database queries
@get("/users", func {
    cached = $bolt.cacheGet("users_list")
    
    if cached != ""
        $bolt.json($bolt.jsonDecode(cached))
        return
    ok
    
    # Expensive operation...
    users = fetchUsersFromDatabase()
    
    # Cache for 5 minutes
    $bolt.cacheSet("users_list", $bolt.jsonEncode(users))
    
    $bolt.json(users)
})

# Cache with per-key TTL
@get("/stats", func {
    cached = $bolt.cacheGet("site_stats")
    if cached != ""
        $bolt.json($bolt.jsonDecode(cached))
        return
    ok
    
    stats = computeExpensiveStats()
    $bolt.cacheSetTTL("site_stats", $bolt.jsonEncode(stats), 600)  # 10 min TTL
    $bolt.json(stats)
})

# Invalidate cache on update
@post("/users", func {
    # Create user...
    $bolt.cacheDelete("users_list")
    $bolt.json([:created = true])
})

# Clear all cache
@post("/admin/clear-cache", func {
    $bolt.cacheClear()
    $bolt.json([:cleared = true])
})
```

### ETag Caching

```ring
@get("/data", func {
    data = getExpensiveData()
    content = $bolt.jsonEncode(data)
    
    # Generate ETag
    $bolt.etag(content)
    
    $bolt.json(data)
})
```
