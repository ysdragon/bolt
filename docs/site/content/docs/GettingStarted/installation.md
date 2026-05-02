---
title: "Installation"
weight: 1
summary: "Install Bolt via Ring Package Manager"
---

## Install via Ring Package Manager

```bash
ringpm install bolt from ysdragon
```

## Requirements

- Ring 1.25+
- Pre-built binaries included for Windows, Linux (glibc / musl), macOS, and FreeBSD

## Verify Installation

Create a file called `app.ring`:

```ring
load "bolt.ring"

new Bolt() {
    port = 3000

    @get("/", func {
        $bolt.send("Hello, World!")
    })
}
```

Run it:

```bash
ring app.ring
```

You should see:

```
[bolt] Server running on http://0.0.0.0:3000
```

Visit `http://localhost:3000` in your browser.
