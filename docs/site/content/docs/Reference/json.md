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

### JSON Booleans

Ring has no native boolean type. Use `jsonTrue()` / `jsonFalse()` to create
values that encode as JSON `true` / `false` instead of numbers.

```ring
# Creating JSON booleans
data = [
    :active   = $bolt.jsonTrue(),
    :disabled = $bolt.jsonFalse(),
    :count    = 1   # regular number, NOT a boolean
]
? $bolt.jsonEncode(data)
# {"active":true,"count":1,"disabled":false}
```

Decoded JSON booleans are sentinel strings. Test them with `jsonIsTrue()` /
`jsonIsFalse()`, or convert to a number with `jsonToBool()`.

```ring
data = $bolt.jsonDecode('{"enabled": true, "muted": false, "score": 1}')
? $bolt.jsonIsTrue(data[:enabled])   # 1
? $bolt.jsonIsFalse(data[:muted])    # 1
? $bolt.jsonToBool(data[:enabled])   # 1
? $bolt.jsonToBool(data[:muted])     # 0
```

Booleans round-trip correctly through encode/decode:

```ring
original = '{"active": true, "disabled": false, "count": 1}'
restored = $bolt.jsonEncode($bolt.jsonDecode(original))
# {"active":true,"count":1,"disabled":false}
```

### $bolt.jsonTrue()
Returns a JSON `true` value.

### $bolt.jsonFalse()
Returns a JSON `false` value.

### $bolt.jsonIsTrue(xValue)
Returns `1` if the value is JSON `true`, `0` otherwise.

### $bolt.jsonIsFalse(xValue)
Returns `1` if the value is JSON `false`, `0` otherwise.

### $bolt.jsonToBool(xValue)
Converts a JSON boolean to a Ring number (`1` or `0`). Errors if the value is not a boolean.
