---
title: "Crypto Class"
weight: 29
summary: "AES-256-GCM encryption and HMAC-SHA256 signatures"
---

The `Crypto` class provides AES-256-GCM encryption and HMAC-SHA256.

```ring
crypto = new Crypto
```

### crypto.aesEncrypt(cPlaintext, cKey)
Encrypt plaintext using AES-256-GCM. A raw 32-byte key is used directly; any other string key is stretched with Argon2id using a random salt. Returns base64-encoded ciphertext with IV and tag.

```ring
encrypted = crypto.aesEncrypt("secret data", "0123456789abcdef0123456789abcdef")
```

### crypto.aesDecrypt(cCiphertext, cKey)
Decrypt AES-256-GCM ciphertext. Key must match the one used for encryption. Returns base64-encoded plaintext (decode with `$bolt.base64Decode()`). Raises an error on wrong key, corrupted data, or invalid base64.

```ring
cB64 = crypto.aesDecrypt(encrypted, "0123456789abcdef0123456789abcdef")
decrypted = $bolt.base64Decode(cB64)
```

### crypto.hmacSha256(cMessage, cKey)
Compute HMAC-SHA256 signature. Returns hex-encoded signature.

```ring
sig = crypto.hmacSha256("message", "secret-key")
```

### crypto.hmacVerify(cMessage, cKey, cSignature)
Verify an HMAC-SHA256 signature. Returns 1 if valid, 0 otherwise.

```ring
if crypto.hmacVerify("message", "secret-key", sig)
    # Signature valid
ok
```
