---
title: "Testing & Deployment"
weight: 20
summary: "Testing with curl and production deployment with systemd and nginx"
---

### Testing with curl

```bash
# GET request
curl http://localhost:3000/users

# POST JSON
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com"}'

# With authentication
curl http://localhost:3000/profile \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9..."

# File upload
curl -X POST http://localhost:3000/upload \
  -F "file=@photo.jpg"
```

### Production Checklist

1. **Enable HTTPS**
   ```ring
   enableTls("cert.pem", "key.pem")
   ```

2. **Set appropriate limits**
   ```ring
   setBodyLimit(10 * 1024 * 1024)  # Limit body size
   $bolt.rateLimit(100, 60)               # Rate limiting
   ```

3. **Configure CORS properly**
   ```ring
   enableCors()
   corsOrigin("https://yourdomain.com")  # Not "*" in production
   ```

4. **Enable security features**
   ```ring
   enableCsrf("strong-secret")
   setCookieSecret("another-strong-secret")
   ```

5. **Use systemd service**
   ```ini
   [Unit]
   Description=Bolt API Server
   After=network.target

   [Service]
   Type=simple
   User=www-data
   WorkingDirectory=/var/www/myapp
   ExecStart=/usr/bin/ring app.ring
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

6. **Reverse proxy with nginx**
   ```nginx
   server {
       listen 443 ssl;
       server_name api.yourdomain.com;
       
       ssl_certificate /etc/ssl/certs/cert.pem;
       ssl_certificate_key /etc/ssl/private/key.pem;
       
       location / {
           proxy_pass http://127.0.0.1:3000;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection "upgrade";
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```
