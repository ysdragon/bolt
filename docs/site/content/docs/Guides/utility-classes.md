---
title: "Utility Classes"
weight: 18
summary: "Hash, Crypto, Validate, Sanitize, Env, and DateTime utilities"
---

Bolt includes several standalone utility classes for common tasks.

### Environment Variables (Env)

Load and manage `.env` files:

```ring
env = new Env()

# Automatically loads .env from current directory on init
dbUrl = env.getVar("DATABASE_URL")
port = env.getVar("PORT")
secret = env.getOr("SECRET", "default-secret")

# Load from a specific file
env.loadFile("/etc/myapp/env")

# Set a variable
env.setVar("APP_ENV", "production")
```

**Example `.env` file:**
```
DATABASE_URL=postgres://localhost/mydb
SECRET=my-secret-key
PORT=3000
```

### Password Hashing (Hash)

Secure password hashing with Argon2, bcrypt, or scrypt:

```ring
hash = new Hash

# Argon2id (recommended)
hashed = hash.argon2("mypassword")
if hash.verifyArgon2("mypassword", hashed)
    # Password correct
ok

# Bcrypt
hashed = hash.bcrypt("mypassword")
if hash.verifyBcrypt("mypassword", hashed)
    # Password correct
ok

# Scrypt
hashed = hash.scrypt("mypassword")
if hash.verifyScrypt("mypassword", hashed)
    # Password correct
ok
```

**Full authentication example:**
```ring
load "bolt.ring"

hash = new Hash
storedHash = ""

new Bolt() {
    port = 3000

    @post("/register", func {
        data = $bolt.jsonBody()
        storedHash = hash.argon2(data[:password])
        $bolt.json([:registered = true])
    })

    @post("/login", func {
        data = $bolt.jsonBody()
        if hash.verifyArgon2(data[:password], storedHash)
            $bolt.json([:success = true])
        else
            $bolt.unauthorized()
        ok
    })
}
```

### Input Validation (Validate)

Validate common data types:

```ring
v = new Validate

# Email, URL, IP
if v.email(input) { ... }
if v.url(input) { ... }
if v.ip(input) { ... }
if v.ipv4(input) { ... }
if v.ipv6(input) { ... }

# UUID and JSON
if v.uuid(input) { ... }
if v.jsonString(input) { ... }

# String length and number range
if v.length(name, 3, 50) { ... }
if v.range(age, 0, 150) { ... }

# Character classes
if v.alpha(name) { ... }        # letters only
if v.alphanumeric(name) { ... }  # letters + digits
if v.numeric(zip) { ... }       # digits only
```

**Validation middleware example:**
```ring
@post("/users", func {
    v = new Validate
    data = $bolt.jsonBody()

    if !v.email(data[:email])
        $bolt.badRequest("Invalid email")
        return
    ok

    if !v.length(data[:name], 2, 100)
        $bolt.badRequest("Name must be 2-100 characters")
        return
    ok

    # Process valid data...
})
```

### Encryption (Crypto)

AES-256-GCM encryption and HMAC-SHA256:

```ring
crypto = new Crypto

# AES-256-GCM encryption (key must be 32 bytes)
key = "0123456789abcdef0123456789abcdef"
encrypted = crypto.aesEncrypt("secret data", key)
decrypted = crypto.aesDecrypt(encrypted, key)

# HMAC-SHA256 signatures
sig = crypto.hmacSha256("message", "signing-key")
if crypto.hmacVerify("message", "signing-key", sig)
    # Signature valid
ok
```

### Date & Time (DateTime)

Timestamp operations, formatting, and arithmetic:

```ring
dt = new DateTime

# Current time
cNow = dt.now()           # Local ISO string
cUtc = dt.nowUtc()        # UTC ISO string
ts = dt.timestamp()        # Unix seconds
tsMs = dt.timestampMs()   # Unix milliseconds

# Formatting
cDate = dt.formatDate(ts, "%Y-%m-%d %H:%M:%S")
cFriendly = dt.formatDate(ts, "%B %d, %Y")

# Parsing
ts = dt.parseDate("2026-05-02 14:30:00", "%Y-%m-%d %H:%M:%S")

# Arithmetic
nextWeek = dt.addDays(ts, 7)
inTwoHours = dt.addHours(ts, 2)
diff = dt.diff(ts1, ts2)  # Difference in seconds
```

### HTML Sanitization (Sanitize)

Prevent XSS attacks:

```ring
s = new Sanitize

# Strip dangerous tags, keep safe ones
safe = s.html('<script>alert("xss")</script><p>Safe</p>')
# Returns: "<p>Safe</p>"

# Strip ALL tags
text = s.strict('<b>Bold</b> <script>evil()</script>')
# Returns: "Bold evil()"

# Escape HTML entities
escaped = s.escapeHtml('<div>Hello & goodbye</div>')
# Returns: "&lt;div&gt;Hello &amp; goodbye&lt;/div&gt;"
```

**User content rendering example:**
```ring
s = new Sanitize

@post("/comment", func {
    data = $bolt.jsonBody()
    safeComment = s.html(data[:text])
    # Store safeComment in database
})

@get("/comments", func {
    # Sanitize before rendering
    $bolt.renderFile("templates/comments.html", [
        :comments = getComments(),
        :sanitize = s
    ])
})
```
