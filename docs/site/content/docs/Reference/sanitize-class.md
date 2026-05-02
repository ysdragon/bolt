---
title: "Sanitize Class"
weight: 31
summary: "HTML and XSS sanitization for user input"
---

The `Sanitize` class provides HTML and XSS sanitization.

```ring
s = new Sanitize
```

### s.html(cInput)
Sanitize HTML by stripping dangerous tags, keeping safe ones.

```ring
safe = s.html('<script>alert("xss")</script><p>Safe</p>')
# Returns: "<p>Safe</p>"
```

### s.strict(cInput)
Strictly sanitize HTML by stripping all tags.

```ring
text = s.strict('<b>Bold</b> <script>evil()</script>')
# Returns: "Bold evil()"
```

### s.escapeHtml(cInput)
Escape HTML special characters to entities.

```ring
escaped = s.escapeHtml('<div class="test">Hello & goodbye</div>')
# Returns: "&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;"
```
