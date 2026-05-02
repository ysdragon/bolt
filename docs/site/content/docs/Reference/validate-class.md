---
title: "Validate Class"
weight: 28
summary: "Input validation for emails, URLs, IPs, UUIDs, strings, and numbers"
---

The `Validate` class provides input validation for common data types.

```ring
v = new Validate
```

### v.email(cStr)
Validate an email address format.

```ring
if v.email("test@example.com") { ... }
```

### v.url(cStr)
Validate a URL format.

```ring
if v.url("https://example.com") { ... }
```

### v.ip(cStr)
Validate an IP address (v4 or v6).

```ring
if v.ip("192.168.1.1") { ... }
```

### v.ipv4(cStr)
Validate an IPv4 address.

```ring
if v.ipv4("10.0.0.1") { ... }
```

### v.ipv6(cStr)
Validate an IPv6 address.

```ring
if v.ipv6("::1") { ... }
```

### v.uuid(cStr)
Validate a UUID format.

```ring
if v.uuid("550e8400-e29b-41d4-a716-446655440000") { ... }
```

### v.jsonString(cStr)
Validate that a string is valid JSON.

```ring
if v.jsonString('{"key": "value"}') { ... }
```

### v.length(cStr, nMin, nMax)
Validate string length within a range.

```ring
if v.length(username, 3, 20) { ... }
```

### v.range(nNum, nMin, nMax)
Validate a number within a range.

```ring
if v.range(age, 0, 150) { ... }
```

### v.alpha(cStr)
Validate that a string contains only alphabetic characters.

```ring
if v.alpha(name) { ... }
```

### v.alphanumeric(cStr)
Validate that a string contains only alphanumeric characters.

```ring
if v.alphanumeric(username) { ... }
```

### v.numeric(cStr)
Validate that a string contains only numeric characters.

```ring
if v.numeric(zipCode) { ... }
```
