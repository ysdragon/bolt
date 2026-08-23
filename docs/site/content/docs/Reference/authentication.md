---
title: "Authentication"
weight: 16
summary: "JWT token creation and verification, Basic Auth encoding and decoding"
---

### $bolt.jwtEncode(aData, cSecret)
Create JWT token. Secret must be at least 32 bytes. Applies a default 3600-second TTL.

```ring
token = $bolt.jwtEncode([:user_id = 123, :role = "admin"], "my-secret-key-at-least-32-bytes!!")
```

### $bolt.jwtEncodeExp(aData, cSecret, nExpires)
Create JWT with expiration (seconds from now). Secret must be at least 32 bytes.

```ring
token = $bolt.jwtEncodeExp([:user_id = 123], "my-secret-key-at-least-32-bytes!!", 3600)  # 1 hour
```

### $bolt.jwtDecode(cToken, cSecret)
Decode and verify JWT, returns payload.

```ring
data = $bolt.jwtDecode(token, "my-secret-key-at-least-32-bytes!!")
if data != NULL
    userId = data[:user_id]
ok
```

### $bolt.jwtVerify(cToken, cSecret)
Verify JWT validity (returns 1 or 0).

```ring
if $bolt.jwtVerify(token, "my-secret-key-at-least-32-bytes!!")
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
auth = $bolt.basicAuthEncode("user", "pass")  # "Basic dXNlcjpwYXNz"
```
