---
title: "DateTime Class"
weight: 30
summary: "Timestamp operations, date formatting, parsing, and arithmetic"
---

The `DateTime` class provides timestamp operations, formatting, and arithmetic.

```ring
dt = new DateTime
```

### dt.now()
Get the current local datetime string in ISO format.

```ring
cNow = dt.now()  # "2026-05-02T14:30:00"
```

### dt.nowUtc()
Get the current UTC datetime string in ISO format.

```ring
cUtc = dt.nowUtc()
```

### dt.timestamp()
Get the current Unix timestamp in seconds.

```ring
ts = dt.timestamp()
```

### dt.timestampMs()
Get the current Unix timestamp in milliseconds.

```ring
tsMs = dt.timestampMs()
```

### dt.formatDate(nTimestamp, cFormat)
Format a Unix timestamp to a string.

```ring
cDate = dt.formatDate(ts, "%Y-%m-%d %H:%M:%S")  # "2026-05-02 14:30:00"
```

### dt.parseDate(cDateStr, cFormat)
Parse a datetime string to a Unix timestamp.

```ring
ts = dt.parseDate("2026-05-02 14:30:00", "%Y-%m-%d %H:%M:%S")
```

### dt.diff(nTs1, nTs2)
Calculate the difference between two timestamps in seconds.

```ring
diff = dt.diff(ts1, ts2)
```

### dt.addDays(nTimestamp, nDays)
Add days to a timestamp.

```ring
future = dt.addDays(ts, 7)  # 7 days from now
```

### dt.addHours(nTimestamp, nHours)
Add hours to a timestamp.

```ring
future = dt.addHours(ts, 24)  # 24 hours from now
```
