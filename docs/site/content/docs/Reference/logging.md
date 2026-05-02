---
title: "Logging"
weight: 24
summary: "Write log messages at different levels and configure log filtering"
---

### $bolt.log(cMessage)
Write to server log at default level.

```ring
$bolt.log("User logged in: " + userId)
```

### $bolt.logWithLevel(cMessage, cLevel)
Write to server log at a specific level.

```ring
$bolt.logWithLevel("Disk space low", "warn")
$bolt.logWithLevel("Connection failed", "error")
```

### $bolt.setLogLevel(cLevel)
Set the minimum log level.

```ring
$bolt.setLogLevel("warn")  # Only warn and error will be logged
```
