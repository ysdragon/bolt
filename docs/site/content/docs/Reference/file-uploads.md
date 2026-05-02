---
title: "File Uploads"
weight: 10
summary: "Handle uploaded files, save to disk, and inspect file metadata"
---

### $bolt.filesCount()
Get number of uploaded files.

```ring
count = $bolt.filesCount()
```

### $bolt.file(nIndex)
Get uploaded file by index (1-based). Returns a list with `:name`, `:field`, `:type`, `:size`.

```ring
f = $bolt.file(1)
? f[:name]   # "photo.jpg"
? f[:field]  # "avatar"
? f[:type]   # "image/jpeg"
? f[:size]   # 102400
```

### $bolt.files()
Get all uploaded files as a list of file lists.

```ring
aFiles = $bolt.files()
for f in aFiles
    ? f[:name]
next
```

### $bolt.fileByField(cName)
Get first file matching a form field name.

```ring
f = $bolt.fileByField("avatar")
? f[:name]
```

### $bolt.fileSave(nIndex, cPath)
Save uploaded file to disk.

```ring
f = $bolt.file(1)
$bolt.fileSave(1, "./uploads/" + f[:name])
```

**Example: Handle file upload**
```ring
@post("/upload", func {
    if $bolt.filesCount() > 0
        for i = 1 to $bolt.filesCount()
            f = $bolt.file(i)
            $bolt.fileSave(i, "./uploads/" + f[:name])
        next
        $bolt.json([:uploaded = $bolt.filesCount()])
    else
        $bolt.badRequest("No files uploaded")
    ok
})
```
