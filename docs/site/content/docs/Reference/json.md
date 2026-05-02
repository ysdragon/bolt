---
title: "JSON"
weight: 11
summary: "Encode, decode, and pretty-print JSON data"
---

### $bolt.jsonEncode(aList)
Encode list/object to JSON string.

```ring
str = $bolt.jsonEncode([:name = "John"])  # '{"name":"John"}'
```

### $bolt.jsonDecode(cJson)
Decode JSON string to list/object.

```ring
data = $bolt.jsonDecode('{"name":"John"}')
? data[:name]  # John
```

### $bolt.jsonPretty(aList)
Encode to pretty-printed JSON.

```ring
str = $bolt.jsonPretty([:name = "John"])
```
