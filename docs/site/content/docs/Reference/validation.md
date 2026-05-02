---
title: "Validation"
weight: 20
summary: "JSON Schema validation, parameter validation, and regex matching"
---

### $bolt.validateJson(cJson, cSchema)
Validate JSON against JSON Schema (returns 1 if valid).

```ring
schema = '{
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0}
    },
    "required": ["name"]
}'

if $bolt.validateJson($bolt.body(), schema)
    # Valid
ok
```

### $bolt.validateJsonErrors(cJson, cSchema)
Get validation errors as a decoded list.

```ring
errors = $bolt.validateJsonErrors($bolt.body(), schema)
if errors != ""
    $bolt.badRequest(errors)
ok
```

### $bolt.validateParam(cParamName, cPattern)
Validate URL parameter against regex.

```ring
if !$bolt.validateParam("id", "[0-9]+")
    $bolt.badRequest("Invalid ID format")
ok
```

### $bolt.matchRegex(cValue, cPattern)
Match value against regex (returns 1 or 0).

```ring
if $bolt.matchRegex(email, "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
    # Valid email
ok
```
