---
title: "Static Files"
weight: 6
summary: "Serving static assets like CSS, JavaScript, and images"
---

### Basic Static Serving

```ring
# Serve ./static directory at /public URL
@static("/public", "./static")

# Multiple directories
@static("/css", "./assets/css")
@static("/js", "./assets/js")
@static("/images", "./assets/images")
```

### Directory Structure

```
project/
├── app.ring
└── static/
    ├── css/
    │   └── style.css
    ├── js/
    │   └── app.js
    └── images/
        └── logo.png
```

Access URLs:
- `/public/css/style.css`
- `/public/js/app.js`
- `/public/images/logo.png`
