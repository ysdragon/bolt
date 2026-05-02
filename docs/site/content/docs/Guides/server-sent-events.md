---
title: "Server-Sent Events"
weight: 12
summary: "Push real-time updates to clients using SSE"
---

### SSE Server

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # Client page
    @get("/", func {
        html = '<!DOCTYPE html>
        <html>
        <body>
            <div id="updates"></div>
            <script>
                const es = new EventSource("/events");
                es.onmessage = (e) => {
                    document.getElementById("updates").innerHTML += e.data + "<br>";
                };
                es.addEventListener("notification", (e) => {
                    alert("Notification: " + e.data);
                });
            </script>
        </body>
        </html>'
        $bolt.send(html)
    })
    
    # SSE endpoint - clients connect here
    @sse("/events")
    
    # Trigger endpoint - send events from here
    @post("/notify", func {
        message = $bolt.jsonBody()[:message]
        
        # Send to all connected clients
        $bolt.sseBroadcast("/events", message)
        
        $bolt.json([:sent = true])
    })
    
    # Named events
    @post("/alert", func {
        data = $bolt.jsonBody()
        $bolt.sseBroadcastEvent("/events", "notification", data[:text])
        $bolt.json([:sent = true])
    })
}
```
