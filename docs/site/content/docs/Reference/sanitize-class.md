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

### s.escapeAttr(cInput)
Escape string for safe use in HTML attribute values (including unquoted).

```ring
escaped = s.escapeAttr('x onerror=alert(1)')
# Returns: "x onerror&#x3D;alert(1)"
```

### s.escapeJs(cInput)
Escape string for safe use in single-quoted JavaScript string literals.

```ring
escaped = s.escapeJs("hello 'world'" + nl + "newline")
# Returns: "hello \'world\' \n newline"
```

### s.escapeJsDq(cInput)
Escape string for safe use in double-quoted JavaScript string literals.

```ring
escaped = s.escapeJsDq('hello "world"')
# Returns: hello \"world\"
```

### s.escapeUrl(cInput)
URL-encode string for safe embedding in URL query values.

```ring
encoded = s.escapeUrl("hello world&foo=bar")
# Returns: "hello%20world%26foo%3Dbar"
```
