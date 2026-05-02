---
title: "Server-Sent Events"
weight: 15
summary: "SSE endpoints for server-push streaming to clients"
---

### @sse(cPath)
Register SSE endpoint for clients to subscribe.

```ring
@sse("/events")
```

### $bolt.sseBroadcast(cPath, cData)
Send data event to all subscribers.

```ring
$bolt.sseBroadcast("/events", "New notification!")
```

### $bolt.sseBroadcastEvent(cPath, cEventName, cData)
Send named event to all subscribers.

```ring
$bolt.sseBroadcastEvent("/events", "update", '{"count": 42}')
```

**Client-side:**
```javascript
const es = new EventSource('/events');
es.onmessage = (e) => console.log(e.data);
es.addEventListener('update', (e) => console.log(e.data));
```
