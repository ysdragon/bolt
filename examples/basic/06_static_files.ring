// Static File Serving Test
// Run: ring 03_static.ring
// Test: curl http://localhost:3000/static/index.html

load "bolt.ring"

new Bolt() {
	// API route
	@get("/", func {
		$bolt.send("<h1>Bolt</h1><p>API + Static files!</p>")
	})
	
	@get("/api/hello", func {
		$bolt.json([
			:message = "Hello from API!"
		])
	})

	// Static files
	@static("/static", "./static")
}