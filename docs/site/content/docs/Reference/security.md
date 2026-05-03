---
title: "Security"
weight: 17
summary: "CSRF protection, SHA-256 hashing, IP whitelisting and blacklisting"
---

### enableCsrf(cSecret)
Enable CSRF protection. Must be called before defining routes.

```ring
enableCsrf("my-csrf-secret")
```

### $bolt.csrfToken()
Generate a session-bound CSRF token (format: `session_id.timestamp.hmac`). Also sets a `BOLTSESSION` cookie if the client doesn't already have one.

```ring
token = $bolt.csrfToken()
# Include in form: <input type="hidden" name="_csrf" value="{{ token }}">
```

### $bolt.verifyCsrf(cToken)
Verify CSRF token. Checks session binding, HMAC signature, and 1-hour expiry. Returns 1 if valid, 0 otherwise.

```ring
if $bolt.verifyCsrf($bolt.formField("_csrf"))
    # Valid request
ok
```

### $bolt.sha256(cData)
Generate SHA-256 hash.

```ring
hash = $bolt.sha256("password123")
```

### ipWhitelist(cIp)
Add IP or CIDR to whitelist.

```ring
ipWhitelist("192.168.1.100")
ipWhitelist("10.0.0.0/8")
```

### ipBlacklist(cIp)
Add IP or CIDR to blacklist.

```ring
ipBlacklist("1.2.3.4")
```

### proxyWhitelist(cIp)
Add IP to the proxy whitelist.

```ring
proxyWhitelist("10.0.0.1")
```
