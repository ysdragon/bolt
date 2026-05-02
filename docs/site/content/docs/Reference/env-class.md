---
title: "Env Class"
weight: 26
summary: "Load and manage environment variables and .env files"
---

The `Env` class manages environment variables and `.env` files.

```ring
env = new Env()
```

### env.loadEnv()
Load `.env` file from the current directory. Called automatically on `init()`.

```ring
env.loadEnv()
```

### env.loadFile(cPath)
Load environment variables from a specific file.

```ring
env.loadFile("/etc/myapp/env")
```

### env.getVar(cKey)
Get an environment variable value. Returns empty string if not found.

```ring
dbUrl = env.getVar("DATABASE_URL")
```

### env.getOr(cKey, cDefault)
Get an environment variable with a default fallback.

```ring
port = env.getVar("PORT")
if port = "" port = "3000" ok

# Or more concisely:
port = env.getOr("PORT", "3000")
```

### env.setVar(cKey, cValue)
Set an environment variable.

```ring
env.setVar("APP_ENV", "production")
```
