---
title: "Templates"
weight: 12
summary: "Render inline templates, template strings, and template files with MiniJinja"
---

### $bolt.render(cTemplate, aData)
Render inline template string and send as response.

```ring
@get("/", func {
    $bolt.render("<h1>{{ title }}</h1>", [:title = "Hello"])
})
```

### $bolt.renderTemplate(cTemplate, aData)
Render template string and return the result (does not send).

```ring
html = $bolt.renderTemplate("<h1>{{ title }}</h1>", [:title = "Hello"])
```

### $bolt.renderFile(cFilepath, aData)
Render template from file and send as response.

```ring
@get("/", func {
    $bolt.renderFile("templates/home.html", [:user = "John"])
})
```

**Template Syntax (MiniJinja):**
```html
<h1>{{ title }}</h1>

{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}

{% if user %}
    <p>Welcome, {{ user }}!</p>
{% endif %}
```
