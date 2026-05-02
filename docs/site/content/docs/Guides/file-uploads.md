---
title: "File Uploads"
weight: 9
summary: "Handling single and multiple file uploads with multipart forms"
---

### Single File Upload

```ring
@post("/upload", func {
    if $bolt.filesCount() < 1
        $bolt.badRequest("No file uploaded")
        return
    ok
    
    # Get file info (1-indexed!)
    f = $bolt.file(1)
    name = f[:name]
    field = f[:field]
    type = f[:type]
    size = f[:size]
    
    # Validate
    if size > 5 * 1024 * 1024
        $bolt.badRequest("File too large (max 5MB)")
        return
    ok
    
    # Save file
    savePath = "./uploads/" + name
    $bolt.fileSave(1, savePath)
    
    $bolt.json([
        :success = true,
        :file = [
            :name = name,
            :type = type,
            :size = size
        ]
    ])
})
```

### Multiple File Upload

```ring
@post("/upload-multiple", func {
    count = $bolt.filesCount()
    
    if count < 1
        $bolt.badRequest("No files uploaded")
        return
    ok
    
    uploaded = []
    
    for i = 1 to count
        f = $bolt.file(i)
        name = f[:name]
        size = f[:size]
        
        # Save each file
        $bolt.fileSave(i, "./uploads/" + name)
        
        add(uploaded, [:name = name, :size = size])
    next
    
    $bolt.json([:uploaded = uploaded, :count = count])
})
```

### HTML Form

```html
<form action="/upload" method="POST" enctype="multipart/form-data">
    <input type="file" name="file" />
    <button type="submit">Upload</button>
</form>

<!-- Multiple files -->
<form action="/upload-multiple" method="POST" enctype="multipart/form-data">
    <input type="file" name="files" multiple />
    <button type="submit">Upload</button>
</form>
```
