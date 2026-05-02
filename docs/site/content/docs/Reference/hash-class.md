---
title: "Hash Class"
weight: 27
summary: "Secure password hashing with Argon2, bcrypt, and scrypt"
---

The `Hash` class provides secure password hashing with Argon2, bcrypt, and scrypt.

```ring
hash = new Hash
```

### hash.argon2(cPassword)
Hash a password using Argon2id. Returns a PHC-formatted hash string.

```ring
hashed = hash.argon2("mypassword")
```

### hash.verifyArgon2(cPassword, cHash)
Verify a password against an Argon2 hash. Returns 1 if valid, 0 otherwise.

```ring
if hash.verifyArgon2("mypassword", storedHash)
    # Password correct
ok
```

### hash.bcrypt(cPassword)
Hash a password using bcrypt.

```ring
hashed = hash.bcrypt("mypassword")
```

### hash.verifyBcrypt(cPassword, cHash)
Verify a password against a bcrypt hash.

```ring
if hash.verifyBcrypt("mypassword", storedHash)
    # Password correct
ok
```

### hash.scrypt(cPassword)
Hash a password using scrypt.

```ring
hashed = hash.scrypt("mypassword")
```

### hash.verifyScrypt(cPassword, cHash)
Verify a password against a scrypt hash.

```ring
if hash.verifyScrypt("mypassword", storedHash)
    # Password correct
ok
```
