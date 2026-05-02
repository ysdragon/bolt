---
title: "Cookies"
weight: 8
summary: "Get, set, delete, and sign cookies"
---

### $bolt.setCookie(cName, cValue)
Set a cookie with default path (`/`).

```ring
$bolt.setCookie("user", "john")
```

### $bolt.setCookieEx(cName, cValue, cOptions)
Set cookie with custom options.

```ring
$bolt.setCookieEx("token", "abc123", "Path=/; Max-Age=3600; HttpOnly; Secure")
```

### $bolt.cookie(cName)
Get cookie value.

```ring
user = $bolt.cookie("user")
```

### $bolt.deleteCookie(cName)
Delete a cookie.

```ring
$bolt.deleteCookie("user")
```

### setCookieSecret(cSecret)
Set secret for signed cookies.

```ring
setCookieSecret("my-secret-key")
```

### $bolt.setSignedCookie(cName, cValue)
Set a signed cookie. Requires `setCookieSecret()` to be called first.

```ring
$bolt.setSignedCookie("session", "data")
```

### $bolt.getSignedCookie(cName)
Get and verify signed cookie. Returns empty string if invalid.

```ring
data = $bolt.getSignedCookie("session")
```
