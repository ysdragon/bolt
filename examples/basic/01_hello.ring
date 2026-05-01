// Hello World - Simple HTTP Server
// Run: ring 01_hello.ring
// Test: curl http://localhost:3000/

load "bolt.ring"

new Bolt() {
	@get("/", func {
		$bolt.send("Hello from Bolt ⚡ 🚀")
	})
	
	@get("/json", func {
		$bolt.json([
			:message = "Hello JSON!",
			:status = "ok"
		])
	})
	
	@get("/user/:id", func {
		cUserId = $bolt.param("id")
		$bolt.json([
			:id = cUserId,
			:name = "User " + cUserId
		])
	})
	
	@post("/echo", func {
		cData = $bolt.body()
		$bolt.send("You sent: " + cData)
	})
}
