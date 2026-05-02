---
title: "WebSocket"
weight: 11
summary: "Real-time WebSocket communication with rooms and broadcasting"
---

### Basic WebSocket Server

```ring
load "bolt.ring"

new Bolt() {
    port = 3000
    
    # Serve HTML client
    @get("/", func {
        html = '<!DOCTYPE html>
        <html>
        <body>
            <div id="messages"></div>
            <input id="msg" placeholder="Type message...">
            <button onclick="sendMsg()">Send</button>
            <script>
                const ws = new WebSocket("ws://localhost:3000/ws");
                ws.onmessage = (e) => {
                    document.getElementById("messages").innerHTML += e.data + "<br>";
                };
                function sendMsg() {
                    ws.send(document.getElementById("msg").value);
                    document.getElementById("msg").value = "";
                }
            </script>
        </body>
        </html>'
        $bolt.send(html)
    })
    
    # WebSocket endpoint with event-driven callbacks
    @ws("/ws",
        func {
            id = $bolt.wsClientId()
            $bolt.wsSendTo(id, "Welcome!")
        },
        func {
            $bolt.wsBroadcast("[User] " + $bolt.wsEventMessage())
        },
        func { }
    )
}
```

### Chat Application with Rooms

```ring
@ws("/chat",
    func {
        # on_connect: join chat room
        id = $bolt.wsClientId()
        $bolt.wsRoomJoin("chat", id)
        $bolt.wsRoomBroadcast("chat", $bolt.jsonEncode([
            :type = "join", :id = id
        ]))
    },
    func {
        # on_message: broadcast to room
        data = $bolt.jsonDecode($bolt.wsEventMessage())
        broadcast = $bolt.jsonEncode([
            :type = "message",
            :user = data[:user],
            :text = data[:text],
            :time = $bolt.unixtime()
        ])
        $bolt.wsRoomBroadcast("chat", broadcast)
    },
    func {
        # on_disconnect: notify room
        $bolt.wsRoomBroadcast("chat", $bolt.jsonEncode([
            :type = "leave", :id = $bolt.wsClientId()
        ]))
    }
)

# Get connection count
@get("/chat/stats", func {
    $bolt.json([
        :connections = $bolt.wsConnectionCount(),
        :room_members = $bolt.wsRoomCount("chat")
    ])
})
```
