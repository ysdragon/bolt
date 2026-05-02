---
title: "Authentication"
weight: 16
summary: "JWT token creation and verification, Basic Auth encoding and decoding"
---

### $bolt.jwtEncode(aData, cSecret)
Create JWT token.

```ring
token = $bolt.jwtEncode([:user_id = 123, :role = "admin"], "secret")
```

### $bolt.jwtEncodeExp(aData, cSecret, nExpires)
Create JWT with expiration (seconds from now).

```ring
token = $bolt.jwtEncodeExp([:user_id = 123], "secret", 3600)  # 1 hour
```

### $bolt.jwtDecode(cToken, cSecret)
Decode and verify JWT, returns payload.

```ring
data = $bolt.jwtDecode(token, "secret")
if data != NULL
    userId = data[:user_id]
ok
```

### $bolt.jwtVerify(cToken, cSecret)
Verify JWT validity (returns 1 or 0).

```ring
if $bolt.jwtVerify(token, "secret")
    # Valid token
ok
```

### $bolt.basicAuthDecode(cHeader)
Decode Basic Auth header. Returns a list with `:username` and `:password`, or NULL if invalid.

```ring
auth = $bolt.header("Authorization")  # "Basic dXNlcjpwYXNz"
creds = $bolt.basicAuthDecode(auth)
if creds != NULL
    user = creds[:username]
    pass = creds[:password]
ok
```

### $bolt.basicAuthEncode(cUsername, cPassword)
Encode Basic Auth credentials.

```ring
auth = $bolt.basicAuthEncode("user", "pass")  # "dXNlcjpwYXNz"
```
