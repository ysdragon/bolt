// Route Parameters & Query Strings
// Run: ring 03_route_params.ring

load "bolt.ring"

new Bolt() {
	// Route parameters (part of URL path)
	// curl http://localhost:3000/users/123
	@get("/users/:id", func {
		cId = $bolt.param("id")
		
		$bolt.json([
			:message = "Fetching user",
			:userId = cId
		])
	})
	
	// Multiple route parameters
	// curl http://localhost:3000/posts/5/comments/42
	@get("/posts/:postId/comments/:commentId", func {
		cPostId = $bolt.param("postId")
		cCommentId = $bolt.param("commentId")
		
		$bolt.json([
			:post = cPostId,
			:comment = cCommentId
		])
	})
	
	// Query strings (after ?)
	// curl "http://localhost:3000/search?q=bolt&limit=10&page=2"
	@get("/search", func {
		cQuery = $bolt.query("q")
		cLimit = $bolt.query("limit")
		cPage = $bolt.query("page")
		
		$bolt.json([
			:query =  cQuery,
			:limit = cLimit,
			:page = cPage
		])
	})
	
	// Combining route params + query strings
	// curl "http://localhost:3000/users/123/posts?status=published&sort=date"
	@get("/users/:userId/posts", func {
		cUserId = $bolt.param("userId")
		cStatus = $bolt.query("status")
		cSort = $bolt.query("sort")
		
		$bolt.json([
			:userId = cUserId,
			:filters = [
				:status = cStatus,
				:sort = cSort
			]
		])
	})
	
	@get("/", func {
		$bolt.send("
<h1>Route Parameters & Query Strings</h1>
<h3>Try these:</h3>
<ul>
	<li><a href='/users/123'>GET /users/123</a></li>
	<li><a href='/posts/5/comments/42'>GET /posts/5/comments/42</a></li>
	<li><a href='/search?q=bolt&limit=10&page=2'>GET /search?q=bolt&limit=10&page=2</a></li>
	<li><a href='/users/123/posts?status=published&sort=date'>GET /users/123/posts?status=published&sort=date</a></li>
</ul>
		")
	})
}
