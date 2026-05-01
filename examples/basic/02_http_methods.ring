// HTTP Methods - GET, POST, PUT, DELETE, PATCH
// Run: ring 02_http_methods.ring
// Test: See curl commands below

load "bolt.ring"

new Bolt() {
	// GET - Retrieve data
	// curl http://localhost:3000/users
	@get("/users", func {
		$bolt.json([
			[:id = 1, :name = "Alice"],
			[:id = 2, :name = "Bob"]
		])
	})
	
	// POST - Create new resource
	// curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Charlie"}'
	@post("/users", func {
		cBody = $bolt.body()
				
		$bolt.json([
			:message = "User created",
			:data = cBody
		])
	})
	
	// PUT - Update entire resource
	// curl -X PUT http://localhost:3000/users/1 -H "Content-Type: application/json" -d '{"name":"Alice Updated"}'
	@put("/users/:id", func {
		cId = $bolt.param("id")
		cBody = $bolt.body()
		
		$bolt.json([
			:message = "User " + cId + " updated",
			:data = cBody
		])
	})
	
	// PATCH - Partial update
	// curl -X PATCH http://localhost:3000/users/1 -H "Content-Type: application/json" -d '{"name":"Alice"}'
	@patch("/users/:id", func {
		cId = $bolt.param("id")
		cBody = $bolt.body()
		
		$bolt.json([
			:message = "User " + cId + " patched",
			:data = cBody
		])
	})
	
	// DELETE - Remove resource
	// curl -X DELETE http://localhost:3000/users/1
	@delete("/users/:id", func {
		cId = $bolt.param("id")
		
		$bolt.json([
			:message = "User " + cId + " deleted"
		])
	})
	
	// Info page
	@get("/", func {
		$bolt.send(`<h1>HTTP Methods Example</h1>
<p>Try these curl commands:</p>
<pre>
GET:    curl http://localhost:3000/users
POST:   curl -X POST http://localhost:3000/users -d '{"name":"Charlie"}'
PUT:    curl -X PUT http://localhost:3000/users/1 -d '{"name":"Updated"}'
PATCH:  curl -X PATCH http://localhost:3000/users/1 -d '{"name":"Patched"}'
DELETE: curl -X DELETE http://localhost:3000/users/1
</pre>`)
	})
}
