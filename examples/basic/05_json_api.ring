// JSON API - Building RESTful APIs
// Run: ring 05_json_api.ring

load "bolt.ring"

// Users list (Mock DB)
aUsers = [
	[:id = 1, :name = "Alice", :email = "alice@example.com"],
	[:id = 2, :name = "Bob", :email = "bob@example.com"]
]
nNextId = 3

new Bolt() {
	// List all users
	@get("/api/users", func {
		$bolt.json([
			:success = true,
			:data = aUsers,
			:count = len(aUsers)
		])
	})
	
	// Get single user
	@get("/api/users/:id", func {
		cId = $bolt.param("id")
		nId = 0 + cId
		
		for aUser in aUsers {
			if (aUser[:id] = nId) {
				$bolt.json([
					:success = true,
					:data = aUser
				])
				return
			}
		}
		
		$bolt.jsonWithStatus(404, [
			:success = false,
			:error = "User not found"
		])
	})
	
	// Create user
	@post("/api/users", func {
		cBody = $bolt.body()
		
		aNewUser = [
			:id = nNextId,
			:name = "User " + nNextId,
			:email = "user" + nNextId + "@example.com"
		]
		
		add(aUsers, aNewUser)
		nNextId++
		
		$bolt.jsonWithStatus(201, [
			:success = true,
			:message = "User created",
			:data = aNewUser
		])
	})
	
	// Update user
	@put("/api/users/:id", func {
		cId = $bolt.param("id")
		nId = 0 + cId
		
		for i = 1 to len(aUsers) {
			if (aUsers[i][:id] = nId) {
				aUsers[i][:name] = "Updated User " + cId
				aUsers[i][:email] = "updated" + cId + "@example.com"
				
				$bolt.json([
					:success = true,
					:message = "User updated",
					:data = aUsers[i]
				])
				return
			}
		}
		
		$bolt.jsonWithStatus(404, [
			:success = false,
			:error = "User not found"
		])
	})
	
	// Delete user
	@delete("/api/users/:id", func {
		cId = $bolt.param("id")
		nId = 0 + cId
		
		for i = 1 to len(aUsers) {
			if (aUsers[i][:id] = nId) {
				del(aUsers, i)
				
				$bolt.json([
					:success = true,
					:message = "User deleted"
				])
				return
			}
		}
		
		$bolt.jsonWithStatus(404, [
			:success = false,
			:error = "User not found"
		])
	})
	
	// API info
	@get("/api", func {
		$bolt.json([
			:name = "Bolt JSON API Example",
			:version = "1.0",
			:endpoints = [
				"GET /api/users - List all users",
				"GET /api/users/:id - Get user by ID",
				"POST /api/users - Create user",
				"PUT /api/users/:id - Update user",
				"DELETE /api/users/:id - Delete user"
			]
		])
	})
	
	@get("/", func {
		$bolt.redirect("/api")
	})
}
