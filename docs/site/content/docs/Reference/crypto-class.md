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
Encrypt plaintext using AES-256-GCM. Returns base64-encoded ciphertext with IV and tag.

```ring
encrypted = crypto.aesEncrypt("secret data", "0123456789abcdef0123456789abcdef")
```

### crypto.aesDecrypt(cCiphertext, cKey)
Decrypt AES-256-GCM ciphertext. Returns decrypted plaintext.

```ring
decrypted = crypto.aesDecrypt(encrypted, "0123456789abcdef0123456789abcdef")
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
