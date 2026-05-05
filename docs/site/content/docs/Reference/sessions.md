---
title: "Sessions"
weight: 9
summary: "Session storage, flash messages, and session management"
---

### $bolt.setSession(cKey, cValue)
Set session value.

```ring
$bolt.setSession("user_id", "123")
$bolt.setSession("role", "admin")
```

### $bolt.getSession(cKey)
Get session value.

```ring
userId = $bolt.getSession("user_id")
```

### $bolt.deleteSession(cKey)
Delete session key.

```ring
$bolt.deleteSession("user_id")
```

### $bolt.clearSession()
Clear all session data.

```ring
$bolt.clearSession()
```

### $bolt.regenerateSession()
Regenerate session ID, migrate data, and invalidate the old session. Prevents session fixation attacks. Call after login or privilege escalation.

```ring
$bolt.regenerateSession()
```

### $bolt.setFlash(cKey, cValue)
Set flash message (one-time session data).

```ring
$bolt.setFlash("success", "User created!")
```

### $bolt.getFlash(cKey)
Get and clear flash message.

```ring
msg = $bolt.getFlash("success")
```

### $bolt.hasFlash(cKey)
Check if flash message exists.

```ring
if $bolt.hasFlash("error")
    # ...
ok
```
