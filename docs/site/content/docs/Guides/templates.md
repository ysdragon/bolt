---
title: "Templates"
weight: 10
summary: "Rendering HTML templates with MiniJinja"
---

### MiniJinja Syntax

Bolt uses MiniJinja, a Jinja2-compatible template engine.

**templates/layout.html:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <header>
        <h1>{{ title }}</h1>
    </header>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        <p>&copy; {{ year }}</p>
    </footer>
</body>
</html>
```

**templates/home.html:**
```html
{% extends "layout.html" %}

{% block content %}
    <p>Welcome, {{ user.name }}!</p>
    
    <h2>Your Items</h2>
    <ul>
    {% for item in items %}
        <li>{{ item.name }} - ${{ item.price }}</li>
    {% endfor %}
    </ul>
    
    {% if user.admin %}
        <a href="/admin">Admin Panel</a>
    {% endif %}
{% endblock %}
```

### Rendering Templates

```ring
@get("/", func {
    html = $bolt.renderFile("templates/home.html", [
        :title = "Welcome",
        :year = 2024,
        :user = [
            :name = "John",
            :admin = true
        ],
        :items = [
            [:name = "Widget", :price = 9.99],
            [:name = "Gadget", :price = 19.99]
        ]
    ])
    $bolt.send(html)
})
```

### Inline Templates

```ring
@get("/greeting/:name", func {
    html = $bolt.render("<h1>Hello, {{ name }}!</h1>", [
        :name = $bolt.param("name")
    ])
    $bolt.send(html)
})
```
