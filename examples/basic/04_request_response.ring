// Request & Response - Headers, Status Codes, Body
// Run: ring 04_request_response.ring

load "bolt.ring"

new Bolt() {
	// Reading request headers
	// curl http://localhost:3000/headers -H "X-Custom-Header: MyValue" -H "User-Agent: TestClient"
	@get("/headers", func {
		cUserAgent = $bolt.header("User-Agent")
		cCustom = $bolt.header("X-Custom-Header")
		
		$bolt.json([
			:userAgent = cUserAgent,
			:custom = cCustom,
			:requestId = $bolt.requestId()
		])
	})
	
	// Setting response headers
	// curl -i http://localhost:3000/custom-headers
	@get("/custom-headers", func {
		$bolt.setHeader("X-Powered-By", "Bolt")
		$bolt.setHeader("X-Version", "1.0")
		$bolt.setHeader("X-Request-Time", "" + $bolt.unixtime())
		
		$bolt.send("Check the response headers!")
	})
	
	// Status codes
	// curl -i http://localhost:3000/status/404
	@get("/status/:code", func {
		cCode = $bolt.param("code")
		nCode = 0 + cCode
		
		$bolt.jsonWithStatus(nCode, [
			:statusCode = nCode,
			:message = "Status code set"
		])
	})
	
	// Reading request body
	// curl -X POST http://localhost:3000/echo -d "Hello, Bolt!"
	@post("/echo", func {
		cBody = $bolt.body()
		
		$bolt.setHeader("Content-Type", "text/plain")
		$bolt.send("You sent: " + cBody)
	})
	
	// JSON request body
	// curl -X POST http://localhost:3000/json -H "Content-Type: application/json" -d '{"name":"Bolt","version":1}'
	@post("/json", func {
		cBody = $bolt.body()
		? "Received JSON: " + cBody
		
		$bolt.json([
			:received = cBody,
			:processed = true
		])
	})
	
	// Redirect
	// curl -L http://localhost:3000/old-url
	@get("/old-url", func {
		$bolt.redirect("/new-url")
	})
	
	@get("/new-url", func {
		$bolt.send("You were redirected here!")
	})
	
	@get("/", func {
		$bolt.send(`<h1>Request & Response Example</h1>
<h3>Test these endpoints:</h3>
<pre>
// Headers
curl http://localhost:3000/headers -H 'X-Custom-Header: MyValue'

// Custom response headers
curl -i http://localhost:3000/custom-headers

// Status codes
curl -i http://localhost:3000/status/200
curl -i http://localhost:3000/status/404
curl -i http://localhost:3000/status/500

// Redirect
curl -L http://localhost:3000/old-url
</pre>`)
	})
}
