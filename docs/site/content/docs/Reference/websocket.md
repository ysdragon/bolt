---
title: "WebSocket"
weight: 14
summary: "Real-time WebSocket endpoints, rooms, broadcast, and per-client messaging"
---

### @ws(cPath, fOnConnect, fOnMessage, fOnDisconnect)
Register event-driven WebSocket endpoint with callbacks. Pass `""` for any callback you don't need.

```ring
@ws("/chat",
    func {
        id = $bolt.wsClientId()
        $bolt.wsRoomJoin("chat", id)
        $bolt.wsSendTo(id, "Welcome!")
    },
    func {
        id = $bolt.wsClientId()
        msg = $bolt.wsEventMessage()
        $bolt.wsRoomBroadcast("chat", "[" + id + "] " + msg)
    },
    func {
        $bolt.wsBroadcast("User left: " + $bolt.wsClientId())
    }
)
```

### Event Context (use inside callbacks)

```ring
$bolt.wsClientId()        # Current client's UUID
$bolt.wsEventType()       # "connect", "message", or "disconnect"
$bolt.wsEventMessage()    # Message text (on_message only)
$bolt.wsEventIsBinary()   # 1 if binary message, 0 otherwise
$bolt.wsEventBinary()     # Binary data as base64 string
$bolt.wsEventPath()       # WebSocket route path
$bolt.wsParam(cName)      # Route parameter from WebSocket event
```

### Per-Client Send

```ring
$bolt.wsSendTo(cClientId, cMessage)             # Send text to specific client
$bolt.wsSendBinaryTo(cClientId, cBase64Data)    # Send binary to specific client
$bolt.wsCloseClient(cClientId)                   # Close a client's connection
$bolt.wsClientList()                             # List of connected client IDs
```

### Rooms

```ring
$bolt.wsRoomJoin(cRoom, cClientId)              # Join a room
$bolt.wsRoomLeave(cRoom, cClientId)             # Leave a room
$bolt.wsRoomBroadcast(cRoom, cMessage)          # Send text to all in room
$bolt.wsRoomBroadcastBinary(cRoom, cBase64Data) # Send binary to all in room
$bolt.wsRoomMembers(cRoom)                       # List of client IDs in room
$bolt.wsRoomCount(cRoom)                         # Number of clients in room
```

### Global Broadcast

```ring
$bolt.wsBroadcast(cMessage)     # Send to ALL connected clients
$bolt.wsConnectionCount()       # Total active connections
```
