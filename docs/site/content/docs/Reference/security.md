---
title: "Security"
weight: 17
summary: "CSRF protection, SHA-256 hashing, IP whitelisting and blacklisting"
---

### enableCsrf(cSecret)
Enable CSRF protection.

```ring
enableCsrf("my-csrf-secret")
```

### $bolt.csrfToken()
Generate CSRF token for forms.

```ring
token = $bolt.csrfToken()
# Include in form: <input type="hidden" name="_csrf" value="{{ token }}">
```

### $bolt.verifyCsrf(cToken, cExpected)
Verify CSRF token.

```ring
if $bolt.verifyCsrf($bolt.formField("_csrf"), $bolt.csrfToken())
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
