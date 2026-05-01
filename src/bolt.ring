// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

// ========================================
// Global State
// ========================================

$bolt = NULL
$bolt_handler_counter = 0
$bolt_route_prefix = ""
$bolt_cookie_secret = ""
$bolt_last_route = ""

/// @class Bolt
/// @brief Main HTTP server class for the Bolt framework.
/// @details Provides routing, middleware, WebSocket, SSE, sessions, caching,
///          JWT, CORS, compression, TLS, and more. Use brace syntax to configure
///          and start the server. Inside route handlers, access the server via `$bolt.`:
///          @code
///          new Bolt() {
///              @get("/", func {
///                  $bolt.send("Hello")
///              })
///          }
///          @endcode
class Bolt {

    pHandle = NULL
    nPort = 3000
    cHost = "0.0.0.0"
    aRoutes = []

    /// @brief Initializes a new Bolt server instance.
    /// @return Self for chaining.
    func init() {
        pHandle = bolt_new()
        $bolt = self
        return self
    }

    /// @brief Called automatically when brace block ends. Starts the server.
    func braceEnd() {
        startServer()
    }

    // ========================================
    // Setters
    // ========================================

    /// @brief Sets the server listening port.
    /// @param nValue Port number (default: 3000).
    func setPort(nValue) {
        nPort = nValue
    }

    /// @brief Sets the server bind host address.
    /// @param cValue Host address (default: "0.0.0.0").
    func setHost(cValue) {
        cHost = cValue
        bolt_set_host(pHandle, cValue)
    }

    // ========================================
    // Routing
    // ========================================

    /// @brief Registers a GET route handler.
    /// @param cPath URL path pattern (e.g., "/users/:id").
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @get(cPath, fHandler) {
        addRoute("GET", cPath, fHandler)
    }

    /// @brief Registers a POST route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @post(cPath, fHandler) {
        addRoute("POST", cPath, fHandler)
    }

    /// @brief Registers a PUT route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @put(cPath, fHandler) {
        addRoute("PUT", cPath, fHandler)
    }

    /// @brief Registers a DELETE route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @delete(cPath, fHandler) {
        addRoute("DELETE", cPath, fHandler)
    }

    /// @brief Registers a PATCH route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @patch(cPath, fHandler) {
        addRoute("PATCH", cPath, fHandler)
    }

    /// @brief Registers a HEAD route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @head(cPath, fHandler) {
        addRoute("HEAD", cPath, fHandler)
    }

    /// @brief Registers an OPTIONS route handler.
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @options(cPath, fHandler) {
        addRoute("OPTIONS", cPath, fHandler)
    }

    /// @brief Registers a route handler for any HTTP method.
    /// @param cMethod HTTP method string (e.g., "GET", "POST").
    /// @param cPath URL path pattern.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @route(cMethod, cPath, fHandler) {
        addRoute(upper(cMethod), cPath, fHandler)
    }

    /// @brief Registers a route with the server.
    /// @param cMethod HTTP method string.
    /// @param cPath URL path pattern (e.g., "/users/:id").
    /// @param fHandler Function reference or handler name string.
    /// @return Self for chaining.
    func addRoute(cMethod, cPath, fHandler) {
        // Use provided name if string, else create unique name
        if (isString(fHandler)) {
            cHandlerName = fHandler
        else
            $bolt_handler_counter++
            cHandlerName = "bolt_handler_" + $bolt_handler_counter
        }

        // Apply route prefix if set
        cFullPath = $bolt_route_prefix + cPath

        // Register route with Rust
        aRoutes + [
            :method = cMethod,
            :path = cFullPath,
            :handlerName = cHandlerName
        ]
        bolt_route(pHandle, cMethod, cFullPath, cHandlerName)

        // Store last route for .where() chaining
        $bolt_last_route = cHandlerName
        return self
    }

    /// @brief Adds a before middleware to the last registered route.
    /// @param cMiddlewareName Name of the middleware handler.
    /// @return Self for chaining.
    /// @code
    /// @get("/api/data", func { ... }).before("authMiddleware")
    /// @endcode
    func before(cMiddlewareName) {
        if (!isNull($bolt_last_route)) {
            bolt_route_before(pHandle, $bolt_last_route, cMiddlewareName)
        }
        return self
    }

    /// @brief Adds an after middleware to the last registered route.
    /// @param cMiddlewareName Name of the middleware handler.
    /// @return Self for chaining.
    /// @code
    /// @get("/api/data", func { ... }).after("logMiddleware")
    /// @endcode
    func after(cMiddlewareName) {
        if (!isNull($bolt_last_route)) {
            bolt_route_after(pHandle, $bolt_last_route, cMiddlewareName)
        }
        return self
    }

    /// @brief Adds rate limiting to the last registered route.
    /// @param nMax Maximum number of requests allowed.
    /// @param nWindow Time window in seconds.
    /// @return Self for chaining.
    /// @code
    /// @get("/api/data", func { ... }).routeRateLimit(100, 60)
    /// @endcode
    func routeRateLimit(nMax, nWindow) {
        if (!isNull($bolt_last_route)) {
            bolt_route_rate_limit(pHandle, $bolt_last_route, nMax, nWindow)
        }
        return self
    }

    /// @brief Adds a regex constraint to a route parameter.
    /// @param cParamName Name of the route parameter.
    /// @param cPattern Regex pattern to match.
    /// @return Self for chaining.
    /// @code
    /// @get("/users/:id", func { ... }).where("id", "[0-9]+")
    /// @endcode
    func where(cParamName, cPattern) {
        if (!isNull($bolt_last_route)) {
            bolt_add_constraint(pHandle, $bolt_last_route, cParamName, cPattern)
        }
        return self
    }

    /// @brief Adds multiple regex constraints to route parameters at once.
    /// @param aConstraints List of [paramName, pattern] pairs.
    /// @return Self for chaining.
    /// @code
    /// @get("/posts/:id/:slug", func { ... }).whereAll([["id", "[0-9]+"], ["slug", "[a-z-]+"]])
    /// @endcode
    func whereAll(aConstraints) {
        for aConstraint in aConstraints {
            where(aConstraint[1], aConstraint[2])
        }
        return self
    }

    /// @brief Adds a description to the last registered route for OpenAPI docs.
    /// @param cDescription Description text.
    /// @return Self for chaining.
    /// @code
    /// @get("/users", func { ... }).describe("Get all users")
    /// @endcode
    func describe(cDescription) {
        if (!isNull($bolt_last_route) && len(aRoutes) > 0) {
            aLastRoute = aRoutes[len(aRoutes)]
            bolt_route_describe(pHandle, aLastRoute[:method], aLastRoute[:path], cDescription)
        }
        return self
    }

    /// @brief Adds a tag to the last registered route for OpenAPI docs grouping.
    /// @param cTag Tag name.
    /// @return Self for chaining.
    /// @code
    /// @get("/users", func { ... }).tag("Users")
    /// @endcode
    func tag(cTag) {
        if (!isNull($bolt_last_route) && len(aRoutes) > 0) {
            aLastRoute = aRoutes[len(aRoutes)]
            bolt_route_tag(pHandle, aLastRoute[:method], aLastRoute[:path], cTag)
        }
        return self
    }

    /// @brief Sets a prefix for all subsequent routes.
    /// @param cPrefix URL prefix string (e.g., "/api/v1").
    /// @return Self for chaining.
    func prefix(cPrefix) {
        $bolt_route_prefix = cPrefix
        return self
    }

    /// @brief Clears the current route prefix.
    /// @return Self for chaining.
    func endPrefix() {
        $bolt_route_prefix = ""
        return self
    }

    /// @brief Registers a custom error handler.
    /// @param fHandler Function or handler name string.
    /// @return Self for chaining.
    func @error(fHandler) {
        if (isString(fHandler) && !isNull(fHandler)) {
            cHandlerName = fHandler
        else
            $bolt_handler_counter++
            cHandlerName = "bolt_error_handler_" + $bolt_handler_counter
        }
        bolt_set_error_handler(pHandle, cHandlerName)
        return self
    }

    // ========================================
    // Middleware
    // ========================================

    /// @brief Registers a global middleware.
    /// @param cMiddlewareName Name of the middleware handler.
    func @use(cMiddlewareName) {
        bolt_use(pHandle, cMiddlewareName)
    }

    // ========================================
    // Static Files
    // ========================================

    /// @brief Serves static files from a directory.
    /// @param cUrlPath URL path prefix (e.g., "/static").
    /// @param cDirPath Directory path to serve files from.
    func @static(cUrlPath, cDirPath) {
        bolt_static(pHandle, cUrlPath, cDirPath)
    }

    // ========================================
    // CORS
    // ========================================

    /// @brief Enables Cross-Origin Resource Sharing (CORS).
    func enableCors() {
        bolt_cors(pHandle, 1)
    }

    /// @brief Disables Cross-Origin Resource Sharing (CORS).
    func disableCors() {
        bolt_cors(pHandle, 0)
    }

    /// @brief Sets the allowed CORS origin.
    /// @param cOrigin Allowed origin URL (e.g., "https://example.com").
    func corsOrigin(cOrigin) {
        bolt_cors_origin(pHandle, cOrigin)
    }

    // ========================================
    // Compression
    // ========================================

    /// @brief Enables response compression (gzip/deflate).
    func enableCompression() {
        bolt_compression(pHandle, 1)
    }

    /// @brief Disables response compression.
    func disableCompression() {
        bolt_compression(pHandle, 0)
    }

    // ========================================
    // TLS/HTTPS
    // ========================================

    /// @brief Enables TLS/HTTPS with certificate and key files.
    /// @param cCertPath Path to the TLS certificate file.
    /// @param cKeyPath Path to the TLS private key file.
    func enableTls(cCertPath, cKeyPath) {
        bolt_tls(pHandle, cCertPath, cKeyPath)
    }

    // ========================================
    // WebSocket
    // ========================================

    /// @brief Registers a WebSocket route with event callbacks.
    /// @param cPath URL path for the WebSocket endpoint.
    /// @param fOnConnect Function or handler name for connection events. Pass "" to skip.
    /// @param fOnMessage Function or handler name for message events. Pass "" to skip.
    /// @param fOnDisconnect Function or handler name for disconnection events. Pass "" to skip.
    /// @return Self for chaining.
    /// @code
    /// @ws("/chat", onConnectFunc, onMessageFunc, onDisconnectFunc)
    /// @endcode
    func @ws(cPath, fOnConnect, fOnMessage, fOnDisconnect) {
        cConnect = ""
        cMessage = ""
        cDisconnect = ""

        if (isString(fOnConnect) && !isNull(fOnConnect)) {
            cConnect = fOnConnect
        else
            if (!isString(fOnConnect)) {
                $bolt_handler_counter++
                cConnect = "ws_on_connect_" + $bolt_handler_counter
            }
        }

        if (isString(fOnMessage) && !isNull(fOnMessage)) {
            cMessage = fOnMessage
        else
            if (!isString(fOnMessage)) {
                $bolt_handler_counter++
                cMessage = "ws_on_message_" + $bolt_handler_counter
            }
        }

        if (isString(fOnDisconnect) && !isNull(fOnDisconnect)) {
            cDisconnect = fOnDisconnect
        else
            if (!isString(fOnDisconnect)) {
                $bolt_handler_counter++
                cDisconnect = "ws_on_disconnect_" + $bolt_handler_counter
            }
        }

        bolt_ws_route(pHandle, cPath, cConnect, cMessage, cDisconnect)
        return self
    }

    // ========================================
    // WebSocket Event Context (use inside callbacks)
    // ========================================

    /// @brief Gets the client ID of the current WebSocket event.
    /// @return Client ID string.
    func wsClientId() {
        return bolt_ws_client_id(pHandle)
    }

    /// @brief Gets the event type of the current WebSocket event.
    /// @return Event type string (e.g., "connect", "message", "disconnect").
    func wsEventType() {
        return bolt_ws_event_type(pHandle)
    }

    /// @brief Gets the message content of the current WebSocket event.
    /// @return Message string.
    func wsEventMessage() {
        return bolt_ws_event_message(pHandle)
    }

    /// @brief Checks if the current WebSocket message is binary.
    /// @return True (1) if binary, false (0) otherwise.
    func wsEventIsBinary() {
        return bolt_ws_event_is_binary(pHandle)
    }

    /// @brief Gets the binary data of the current WebSocket event as base64.
    /// @return Base64-encoded binary data string.
    func wsEventBinary() {
        return bolt_ws_event_binary(pHandle)
    }

    /// @brief Gets the path of the current WebSocket event.
    /// @return Path string.
    func wsEventPath() {
        return bolt_ws_event_path(pHandle)
    }

    /// @brief Gets a route parameter from the WebSocket event.
    /// @param cName Parameter name.
    /// @return Parameter value string.
    func wsParam(cName) {
        return bolt_ws_param(pHandle, cName)
    }

    // ========================================
    // WebSocket Per-Client Send
    // ========================================

    /// @brief Sends a text message to a specific WebSocket client.
    /// @param cClientId Target client ID.
    /// @param cMessage Message to send.
    /// @return Success status.
    func wsSendTo(cClientId, cMessage) {
        return bolt_ws_send_to(pHandle, cClientId, cMessage)
    }

    /// @brief Sends binary data to a specific WebSocket client.
    /// @param cClientId Target client ID.
    /// @param cBase64Data Base64-encoded binary data.
    /// @return Success status.
    func wsSendBinaryTo(cClientId, cBase64Data) {
        return bolt_ws_send_binary_to(pHandle, cClientId, cBase64Data)
    }

    /// @brief Closes a specific WebSocket client connection.
    /// @param cClientId Target client ID.
    /// @return Success status.
    func wsCloseClient(cClientId) {
        return bolt_ws_close_client(pHandle, cClientId)
    }

    /// @brief Gets a list of all connected WebSocket clients.
    /// @return List of client IDs.
    func wsClientList() {
        cJson = bolt_ws_client_list(pHandle)
        if (cJson = "[]") { return [] }
        return bolt_json_decode(cJson)
    }

    // ========================================
    // WebSocket Rooms
    // ========================================

    /// @brief Joins a client to a WebSocket room.
    /// @param cRoom Room name.
    /// @param cClientId Client ID to join.
    /// @return Success status.
    func wsRoomJoin(cRoom, cClientId) {
        return bolt_ws_room_join(pHandle, cRoom, cClientId)
    }

    /// @brief Removes a client from a WebSocket room.
    /// @param cRoom Room name.
    /// @param cClientId Client ID to remove.
    /// @return Success status.
    func wsRoomLeave(cRoom, cClientId) {
        return bolt_ws_room_leave(pHandle, cRoom, cClientId)
    }

    /// @brief Broadcasts a text message to all clients in a room.
    /// @param cRoom Room name.
    /// @param cMessage Message to broadcast.
    /// @return Number of clients that received the message.
    func wsRoomBroadcast(cRoom, cMessage) {
        return bolt_ws_room_broadcast(pHandle, cRoom, cMessage)
    }

    /// @brief Broadcasts binary data to all clients in a room.
    /// @param cRoom Room name.
    /// @param cBase64Data Base64-encoded binary data.
    /// @return Number of clients that received the data.
    func wsRoomBroadcastBinary(cRoom, cBase64Data) {
        return bolt_ws_room_broadcast_binary(pHandle, cRoom, cBase64Data)
    }

    /// @brief Gets a list of all clients in a room.
    /// @param cRoom Room name.
    /// @return List of client IDs.
    func wsRoomMembers(cRoom) {
        cJson = bolt_ws_room_members(pHandle, cRoom)
        if (cJson = "[]") { return [] }
        return bolt_json_decode(cJson)
    }

    /// @brief Gets the number of clients in a room.
    /// @param cRoom Room name.
    /// @return Number of clients.
    func wsRoomCount(cRoom) {
        return bolt_ws_room_count(pHandle, cRoom)
    }

    // ========================================
    // Request Context Getters
    // ========================================

    /// @brief Gets the HTTP method of the current request.
    /// @return Method string (e.g., "GET", "POST").
    func method() {
        return bolt_req_method(pHandle)
    }

    /// @brief Gets the path of the current request.
    /// @return Path string.
    func path() {
        return bolt_req_path(pHandle)
    }

    /// @brief Gets a route parameter value.
    /// @param cName Parameter name.
    /// @return Parameter value string.
    func param(cName) {
        return bolt_req_param(pHandle, cName)
    }

    /// @brief Gets a query string parameter value.
    /// @param cName Parameter name.
    /// @return Parameter value string.
    func query(cName) {
        return bolt_req_query(pHandle, cName)
    }

    /// @brief Gets a request header value.
    /// @param cName Header name.
    /// @return Header value string.
    func header(cName) {
        return bolt_req_header(pHandle, cName)
    }

    /// @brief Gets the raw request body as a string.
    /// @return Body string.
    func body() {
        return bolt_req_body(pHandle)
    }

    /// @brief Gets the request body parsed as JSON.
    /// @return Parsed JSON data or empty list if invalid.
    func jsonBody() {
        cBody = bolt_req_body(pHandle)
        if (isNull(cBody)) { return [] }
        cResult = bolt_json_decode(cBody)
        if (isNull(cResult)) { return NULL }
        return cResult
    }

    // ========================================
    // Cookies
    // ========================================

    /// @brief Gets a cookie value by name.
    /// @param cName Cookie name.
    /// @return Cookie value string.
    func cookie(cName) {
        return bolt_req_cookie(pHandle, cName)
    }

    /// @brief Sets a cookie with default path ("/").
    /// @param cName Cookie name.
    /// @param cValue Cookie value.
    func setCookie(cName, cValue) {
        bolt_set_cookie(pHandle, cName, cValue, "Path=/")
    }

    /// @brief Sets a cookie with custom options.
    /// @param cName Cookie name.
    /// @param cValue Cookie value.
    /// @param cOptions Cookie options string (e.g., "Path=/; Max-Age=3600").
    func setCookieEx(cName, cValue, cOptions) {
        bolt_set_cookie(pHandle, cName, cValue, cOptions)
    }

    /// @brief Deletes a cookie by name.
    /// @param cName Cookie name to delete.
    func deleteCookie(cName) {
        bolt_set_cookie(pHandle, cName, "", "Path=/; Max-Age=0")
    }

    // ========================================
    // Sessions
    // ========================================

    /// @brief Sets a session value.
    /// @param cKey Session key.
    /// @param cValue Session value.
    func setSession(cKey, cValue) {
        bolt_session_set(pHandle, cKey, cValue)
    }

    /// @brief Gets a session value by key.
    /// @param cKey Session key.
    /// @return Session value string.
    func getSession(cKey) {
        return bolt_session_get(pHandle, cKey)
    }

    /// @brief Deletes a session value by key.
    /// @param cKey Session key to delete.
    func deleteSession(cKey) {
        bolt_session_delete(pHandle, cKey)
    }

    /// @brief Clears all session data.
    func clearSession() {
        bolt_session_clear(pHandle)
    }

    // ========================================
    // File Uploads
    // ========================================

    /// @brief Gets the number of uploaded files in the request.
    /// @return File count.
    func filesCount() {
        return bolt_req_files_count(pHandle)
    }

    /// @brief Gets an uploaded file by index.
    /// @param nIndex File index (1-based).
    /// @return File data.
    func file(nIndex) {
        return bolt_req_file(pHandle, nIndex)
    }

    /// @brief Gets all uploaded files as a list.
    /// @return List of file data.
    func files() {
        return bolt_req_files(pHandle)
    }

    /// @brief Gets an uploaded file by form field name.
    /// @param cName Form field name.
    /// @return File data.
    func fileByField(cName) {
        return bolt_req_file_by_field(pHandle, cName)
    }

    /// @brief Saves an uploaded file to disk.
    /// @param nIndex File index (1-based).
    /// @param cPath Destination file path.
    /// @return Success status.
    func fileSave(nIndex, cPath) {
        return bolt_req_file_save(pHandle, nIndex, cPath)
    }

    // ========================================
    // Response Methods
    // ========================================

    /// @brief Sends a text response with 200 status.
    /// @param cContent Response content.
    func send(cContent) {
        bolt_respond(pHandle, 200, cContent)
    }

    /// @brief Sends a response with only a status code.
    /// @param nStatus HTTP status code.
    func sendStatus(nStatus) {
        bolt_respond_status(pHandle, nStatus)
    }

    /// @brief Sends a text response with a custom status code.
    /// @param nStatus HTTP status code.
    /// @param cContent Response content.
    func sendWithStatus(nStatus, cContent) {
        bolt_respond(pHandle, nStatus, cContent)
    }

    /// @brief Sends a JSON response with 200 status.
    /// @param aData Data to encode as JSON.
    func json(aData) {
        cJson = bolt_json_encode(aData)
        bolt_respond_json(pHandle, 200, cJson)
    }

    /// @brief Sends a JSON response with a custom status code.
    /// @param nStatus HTTP status code.
    /// @param aData Data to encode as JSON.
    func jsonWithStatus(nStatus, aData) {
        cJson = bolt_json_encode(aData)
        bolt_respond_json(pHandle, nStatus, cJson)
    }

    /// @brief Sends a file as the response.
    /// @param cFilePath Path to the file to send.
    func sendFile(cFilePath) {
        bolt_respond_file(pHandle, cFilePath)
    }

    /// @brief Sends a file with a custom content type.
    /// @param cFilePath Path to the file to send.
    /// @param cContentType MIME type string.
    func sendFileAs(cFilePath, cContentType) {
        bolt_respond_file(pHandle, cFilePath, cContentType)
    }

    /// @brief Sends binary data as the response.
    /// @param cBase64Data Base64-encoded binary data.
    func sendBinary(cBase64Data) {
        bolt_respond_binary(pHandle, cBase64Data)
    }

    /// @brief Sends binary data with a custom content type.
    /// @param cBase64Data Base64-encoded binary data.
    /// @param cContentType MIME type string.
    func sendBinaryAs(cBase64Data, cContentType) {
        bolt_respond_binary(pHandle, cBase64Data, cContentType)
    }

    /// @brief Sends a temporary (302) redirect response.
    /// @param cUrl Redirect URL.
    func redirect(cUrl) {
        bolt_respond_redirect(pHandle, cUrl, 0)
    }

    /// @brief Sends a permanent (301) redirect response.
    /// @param cUrl Redirect URL.
    func redirectPermanent(cUrl) {
        bolt_respond_redirect(pHandle, cUrl, 1)
    }

    /// @brief Sends a 404 Not Found response.
    func notFound() {
        bolt_respond(pHandle, 404, "Not Found")
    }

    /// @brief Sends a 400 Bad Request response.
    /// @param cMessage Optional error message.
    func badRequest(cMessage) {
        if (isNull(cMessage)) { cMessage = "Bad Request" }
        bolt_respond(pHandle, 400, cMessage)
    }

    /// @brief Sends a 500 Internal Server Error response.
    /// @param cMessage Optional error message.
    func serverError(cMessage) {
        if (isNull(cMessage)) { cMessage = "Internal Server Error" }
        bolt_respond(pHandle, 500, cMessage)
    }

    /// @brief Sends a 401 Unauthorized response.
    func unauthorized() {
        bolt_respond(pHandle, 401, "Unauthorized")
    }

    /// @brief Sends a 403 Forbidden response.
    func forbidden() {
        bolt_respond(pHandle, 403, "Forbidden")
    }

    // ========================================
    // Custom Headers
    // ========================================

    /// @brief Sets a response header.
    /// @param cName Header name.
    /// @param cValue Header value.
    func setHeader(cName, cValue) {
        bolt_set_header(pHandle, cName, cValue)
    }

    // ========================================
    // Template Engine
    // ========================================

    /// @brief Renders a template string with data and sends as response.
    /// @param cTemplate Template string.
    /// @param aData Data to pass to the template.
    func render(cTemplate, aData) {
        cResult = bolt_render_template(pHandle, cTemplate, aData)
        send(cResult)
    }

    /// @brief Renders a template file with data and sends as response.
    /// @param cFilepath Path to the template file.
    /// @param aData Data to pass to the template.
    func renderFile(cFilepath, aData) {
        cResult = bolt_render_file(pHandle, cFilepath, aData)
        send(cResult)
    }

    /// @brief Renders a template string with data and returns the result.
    /// @param cTemplate Template string.
    /// @param aData Data to pass to the template.
    /// @return Rendered string.
    func renderTemplate(cTemplate, aData) {
        return bolt_render_template(pHandle, cTemplate, aData)
    }

    // ========================================
    // Utilities (JSON, URL, etc.)
    // ========================================

    /// @brief Encodes a Ring list/value to JSON string.
    /// @param aList Data to encode.
    /// @return JSON string.
    func jsonEncode(aList) {
        return bolt_json_encode(aList)
    }

    /// @brief Decodes a JSON string to Ring data.
    /// @param cJson JSON string to decode.
    /// @return Decoded data.
    func jsonDecode(cJson) {
        return bolt_json_decode(cJson)
    }

    /// @brief Encodes a Ring list/value to pretty-printed JSON.
    /// @param aList Data to encode.
    /// @return Pretty-printed JSON string.
    func jsonPretty(aList) {
        return bolt_json_pretty(aList)
    }

    /// @brief URL-encodes a string.
    /// @param cStr String to encode.
    /// @return Encoded string.
    func urlEncode(cStr) {
        return bolt_urlencode(cStr)
    }

    /// @brief URL-decodes a string.
    /// @param cStr String to decode.
    /// @return Decoded string.
    func urlDecode(cStr) {
        return bolt_urldecode(cStr)
    }

    // ========================================
    // Caching (In-Memory with TTL)
    // ========================================

    /// @brief Sets a cache value (no expiry).
    /// @param cKey Cache key.
    /// @param cValue Cache value.
    func cacheSet(cKey, cValue) {
        bolt_cache_set(pHandle, cKey, cValue, 0)
    }
    
    /// @brief Sets a cache value with a TTL.
    /// @param cKey Cache key.
    /// @param cValue Cache value.
    /// @param nTTL Time-to-live in seconds.
    func cacheSetTTL(cKey, cValue, nTTL) {
        bolt_cache_set(pHandle, cKey, cValue, nTTL)
    }

    /// @brief Gets a cached value by key.
    /// @param cKey Cache key.
    /// @return Cached value or empty string if not found.
    func cacheGet(cKey) {
        return bolt_cache_get(pHandle, cKey)
    }

    /// @brief Deletes a cached value by key.
    /// @param cKey Cache key to delete.
    func cacheDelete(cKey) {
        bolt_cache_delete(pHandle, cKey)
    }

    /// @brief Clears all cached values.
    func cacheClear() {
        bolt_cache_clear(pHandle)
    }

    // ========================================
    // Server Configuration
    // ========================================

    /// @brief Sets the request timeout in milliseconds.
    /// @param nMs Timeout in milliseconds.
    func setTimeout(nMs) {
        bolt_set_timeout(pHandle, nMs)
    }

    /// @brief Sets the maximum request body size.
    /// @param nBytes Maximum body size in bytes.
    func setBodyLimit(nBytes) {
        bolt_set_body_limit(pHandle, nBytes)
    }

    /// @brief Sets the maximum number of session entries.
    /// @param nMaxEntries Maximum session count.
    func setSessionCapacity(nMaxEntries) {
        bolt_set_session_capacity(pHandle, nMaxEntries)
    }

    /// @brief Sets the session time-to-live.
    /// @param nSeconds Session TTL in seconds.
    func setSessionTTL(nSeconds) {
        bolt_set_session_ttl(pHandle, nSeconds)
    }

    /// @brief Sets the maximum number of cache entries.
    /// @param nCapacity Maximum cache count.
    func setCacheCapacity(nCapacity) {
        bolt_set_cache_capacity(pHandle, nCapacity)
    }

    /// @brief Sets the cache time-to-live.
    /// @param nSeconds Cache TTL in seconds.
    func setCacheTTL(nSeconds) {
        bolt_set_cache_ttl(pHandle, nSeconds)
    }

    /// @brief Adds an IP address to the whitelist.
    /// @param cIp IP address to whitelist.
    func ipWhitelist(cIp) {
        bolt_ip_whitelist(pHandle, cIp)
    }

    /// @brief Adds an IP address to the blacklist.
    /// @param cIp IP address to blacklist.
    func ipBlacklist(cIp) {
        bolt_ip_blacklist(pHandle, cIp)
    }

    /// @brief Adds an IP address to the proxy whitelist.
    /// @param cIp IP address to whitelist.
    func proxyWhitelist(cIp) {
        bolt_proxy_whitelist(pHandle, cIp)
    }

    // ========================================
    // CSRF Protection
    // ========================================

    /// @brief Enables CSRF protection with a secret key.
    /// @param cSecret CSRF secret key.
    func enableCsrf(cSecret) {
        bolt_enable_csrf(pHandle, cSecret)
    }

    // ========================================
    // Health
    // ========================================

    /// @brief Gets the server health status.
    /// @return Health status data.
    func healthCheck() {
        return bolt_health_status(pHandle)
    }

    // ========================================
    // JSON Schema Validation
    // ========================================

    /// @brief Validates JSON data against a schema.
    /// @param cJson JSON string to validate.
    /// @param cSchema JSON schema string.
    /// @return True (1) if valid, false (0) otherwise.
    func validateJson(cJson, cSchema) {
        return bolt_validate_json(cJson, cSchema)
    }

    /// @brief Validates JSON data and returns validation errors.
    /// @param cJson JSON string to validate.
    /// @param cSchema JSON schema string.
    /// @return List of validation errors.
    func validateJsonErrors(cJson, cSchema) {
        cErrors = bolt_validate_json_errors(cJson, cSchema)
        return bolt_json_decode(cErrors)
    }

    // ========================================
    // WebSocket Broadcast
    // ========================================

    /// @brief Broadcasts a message to all connected WebSocket clients.
    /// @param cMessage Message to broadcast.
    /// @return Number of clients that received the message, or -1 on error.
    func wsBroadcast(cMessage) {
        return bolt_ws_broadcast(pHandle, cMessage)
    }

    /// @brief Gets the total number of WebSocket connections.
    /// @return Connection count.
    func wsConnectionCount() {
        return bolt_ws_connection_count(pHandle)
    }

    // ========================================
    // ETag
    // ========================================

    /// @brief Generates an ETag hash for content.
    /// @param cContent Content to hash.
    /// @return ETag string.
    func etag(cContent) {
        return bolt_etag(cContent)
    }

    // ========================================
    // SSE (Server-Sent Events) - Broadcast Pattern
    // Clients subscribe to @sse endpoints, use sseBroadcast() from any route to push
    // ========================================

    /// @brief Registers a Server-Sent Events (SSE) endpoint.
    /// @param cPath URL path for the SSE endpoint.
    func @sse(cPath) {
        bolt_sse_route(pHandle, cPath, "")
    }

    /// @brief Broadcasts data to all SSE clients on a path.
    /// @param cPath SSE endpoint path.
    /// @param cData Data to send.
    /// @return Number of clients notified, or -1 on error.
    func sseBroadcast(cPath, cData) {
        return bolt_sse_broadcast(pHandle, cPath, cData)
    }

    /// @brief Broadcasts a named event to all SSE clients on a path.
    /// @param cPath SSE endpoint path.
    /// @param cEventName Event name.
    /// @param cData Event data.
    /// @return Number of clients notified, or -1 on error.
    func sseBroadcastEvent(cPath, cEventName, cData) {
        return bolt_sse_broadcast_event(pHandle, cPath, cEventName, cData)
    }

    // ========================================
    // Middleware (@before / @after)
    // ========================================

    /// @brief Registers a global before middleware.
    /// @param fHandler Function or handler name string.
    func @before(fHandler) {
        if (isString(fHandler) && !isNull(fHandler)) {
            cHandlerName = fHandler
        else
            $bolt_handler_counter++
            cHandlerName = "bolt_before_" + $bolt_handler_counter
        }
        bolt_before(pHandle, cHandlerName)
    }

    /// @brief Registers a global after middleware.
    /// @param fHandler Function or handler name string.
    func @after(fHandler) {
        if (isString(fHandler) && !isNull(fHandler)) {
            cHandlerName = fHandler
        else
            $bolt_handler_counter++
            cHandlerName = "bolt_after_" + $bolt_handler_counter
        }
        bolt_after(pHandle, cHandlerName)
    }

    // ========================================
    // OpenAPI / Swagger Documentation
    // ========================================

    /// @brief Enables the auto-generated OpenAPI spec and /docs endpoint.
    func enableDocs() {
        bolt_openapi_route(pHandle)
    }

    /// @brief Sets API documentation metadata.
    /// @param cTitle API title.
    /// @param cVersion API version.
    /// @param cDescription API description.
    func setDocsInfo(cTitle, cVersion, cDescription) {
        bolt_openapi_info(pHandle, cTitle, cVersion, cDescription)
    }

    /// @brief Sets a custom OpenAPI spec JSON.
    /// @param cSpecJson OpenAPI spec as JSON string.
    func setOpenApiSpec(cSpecJson) {
        bolt_openapi_spec(pHandle, cSpecJson)
    }

    // ========================================
    // Additional Utility Methods
    // ========================================

    /// @brief Gets a form field value from multipart form data.
    /// @param cName Field name.
    /// @return Field value string.
    func formField(cName) {
        return bolt_req_form_field(pHandle, cName)
    }

    /// @brief Gets the unique request ID.
    /// @return Request ID string.
    func requestId() {
        return bolt_req_request_id(pHandle)
    }

    /// @brief Gets the client IP address.
    /// @return IP address string.
    func clientIp() {
        return bolt_req_client_ip(pHandle)
    }

    /// @brief Encodes data as a JWT token.
    /// @param aData Payload data (Ring list).
    /// @param cSecret Secret key for signing.
    /// @return JWT token string.
    func jwtEncode(aData, cSecret) {
        cJson = bolt_json_encode(aData)
        return bolt_jwt_encode(cJson, cSecret)
    }

    /// @brief Encodes data as a JWT token with expiration.
    /// @param aData Payload data (Ring list).
    /// @param cSecret Secret key for signing.
    /// @param nExpires Expiration time in seconds.
    /// @return JWT token string.
    func jwtEncodeExp(aData, cSecret, nExpires) {
        cJson = bolt_json_encode(aData)
        return bolt_jwt_encode(cJson, cSecret, nExpires)
    }

    /// @brief Decodes a JWT token and returns the payload.
    /// @param cToken JWT token string.
    /// @param cSecret Secret key for verification.
    /// @return Decoded payload data or NULL if invalid.
    func jwtDecode(cToken, cSecret) {
        cJson = bolt_jwt_decode(cToken, cSecret)
        if (isNull(cJson)) { return NULL }
        return bolt_json_decode(cJson)
    }

    /// @brief Verifies a JWT token's signature.
    /// @param cToken JWT token string.
    /// @param cSecret Secret key for verification.
    /// @return True (1) if valid, false (0) otherwise.
    func jwtVerify(cToken, cSecret) {
        return bolt_jwt_verify(cToken, cSecret)
    }

    /// @brief Decodes a Basic Auth header.
    /// @param cHeader Authorization header value.
    /// @return Decoded credentials or NULL if invalid.
    func basicAuthDecode(cHeader) {
        cJson = bolt_basic_auth_decode(cHeader)
        if (isNull(cJson)) { return NULL }
        return bolt_json_decode(cJson)
    }

    /// @brief Encodes credentials as a Basic Auth header value.
    /// @param cUsername Username.
    /// @param cPassword Password.
    /// @return Full Authorization header value (e.g., "Basic dXNlcjpwYXNz").
    func basicAuthEncode(cUsername, cPassword) {
        return bolt_basic_auth_encode(cUsername, cPassword)
    }

    /// @brief Computes SHA-256 hash of data.
    /// @param cData Data to hash.
    /// @return Hex-encoded hash string.
    func sha256(cData) {
        return bolt_hash_sha256(cData)
    }

    /// @brief Generates a UUID v4 string.
    /// @return UUID string.
    func uuid() {
        return bolt_uuid()
    }

    /// @brief Gets the current Unix timestamp in seconds.
    /// @return Timestamp as number.
    func unixtime() {
        return bolt_unixtime()
    }

    /// @brief Gets the current Unix timestamp in milliseconds.
    /// @return Timestamp as number.
    func unixtimeMs() {
        return bolt_unixtime_ms()
    }

    /// @brief Enables request logging.
    func enableLogging() {
        bolt_logging(1)
    }

    /// @brief Disables request logging.
    func disableLogging() {
        bolt_logging(0)
    }

    /// @brief Logs a message at the default level.
    /// @param cMessage Message to log.
    func log(cMessage) {
        if (isString(cMessage)) {
            bolt_log(cMessage)
        }
    }

    /// @brief Logs a message at a specific level.
    /// @param cMessage Message to log.
    /// @param cLevel Log level (e.g., "info", "warn", "error").
    func logWithLevel(cMessage, cLevel) {
        bolt_log(cMessage, cLevel)
    }

    /// @brief Sets the minimum log level.
    /// @param cLevel Log level (e.g., "info", "warn", "error").
    func setLogLevel(cLevel) {
        bolt_set_log_level(cLevel)
    }

    /// @brief Sets global rate limiting.
    /// @param nMax Maximum requests allowed.
    /// @param nWindow Time window in seconds.
    func rateLimit(nMax, nWindow) {
        bolt_rate_limit(nMax, nWindow)
    }

    /// @brief Checks if the current request exceeds the rate limit.
    /// @return True (1) if allowed, false (0) if limited.
    func checkRateLimit() {
        return bolt_check_rate_limit()
    }

    /// @brief Validates a route parameter against a regex pattern.
    /// @param cParamName Parameter name.
    /// @param cPattern Regex pattern.
    /// @return True (1) if valid, false (0) otherwise.
    func validateParam(cParamName, cPattern) {
        return bolt_validate_param(pHandle, cParamName, cPattern)
    }

    /// @brief Tests a value against a regex pattern.
    /// @param cValue Value to test.
    /// @param cPattern Regex pattern.
    /// @return True (1) if matches, false (0) otherwise.
    func matchRegex(cValue, cPattern) {
        return bolt_validate_regex(cValue, cPattern)
    }

    /// @brief Sets the secret key for signing cookies.
    /// @param cSecret Secret key string.
    func setCookieSecret(cSecret) {
        $bolt_cookie_secret = cSecret
    }

    /// @brief Sets a signed cookie.
    /// @param cName Cookie name.
    /// @param cValue Cookie value to sign and set.
    /// @note Requires setCookieSecret() to be called first.
    func setSignedCookie(cName, cValue) {
        if (isNull($bolt_cookie_secret)) {
            raise("Cookie secret not set. Call setCookieSecret() first.")
        }
        cSigned = bolt_sign_cookie(cValue, $bolt_cookie_secret)
        setCookie(cName, cSigned)
    }

    /// @brief Gets and verifies a signed cookie value.
    /// @param cName Cookie name.
    /// @return Verified cookie value or empty string if invalid.
    func getSignedCookie(cName) {
        if (isNull($bolt_cookie_secret)) {
            return ""
        }
        cRaw = cookie(cName)
        return bolt_verify_cookie(cRaw, $bolt_cookie_secret)
    }

    /// @brief Sets a flash message (one-read session message).
    /// @param cKey Flash message key.
    /// @param cValue Flash message value.
    func setFlash(cKey, cValue) {
        setSession("_flash_" + cKey, cValue)
    }

    /// @brief Gets and deletes a flash message.
    /// @param cKey Flash message key.
    /// @return Flash message value or empty string if not found.
    func getFlash(cKey) {
        cValue = getSession("_flash_" + cKey)
        if (!isNull(cValue)) {
            deleteSession("_flash_" + cKey)
        }
        return cValue
    }

    /// @brief Checks if a flash message exists.
    /// @param cKey Flash message key.
    /// @return True if exists, false otherwise.
    func hasFlash(cKey) {
        return getSession("_flash_" + cKey) != NULL
    }

    /// @brief Generates a CSRF token.
    /// @return CSRF token string.
    func csrfToken() {
        return bolt_csrf_token()
    }

    /// @brief Verifies a CSRF token.
    /// @param cToken Token to verify.
    /// @param cExpected Expected token value.
    /// @return True (1) if valid, false (0) otherwise.
    func verifyCsrf(cToken, cExpected) {
        return bolt_verify_csrf(cToken, cExpected)
    }

    // ========================================
    // Server Control
    // ========================================

    /// @brief Starts the HTTP server. Blocks until server stops.
    func startServer() {
        bolt_set_host(pHandle, cHost)
        bolt_listen(pHandle, nPort)
    }

    /// @brief Stops the HTTP server.
    func stop() {
        bolt_stop(pHandle)
    }

}

// ========================================
// Env Class
// ========================================

/// @class Env
/// @brief Environment variable management class.
/// @details Loads .env files and manages environment variables:
///          @code
///          env = new Env()
///          env.loadEnv()
///          cDbUrl = env.getVar("DATABASE_URL")
///          @endcode
class Env {

    /// @brief Initializes the Env class and loads .env from current directory.
    func init() {
        bolt_env_load()
    }

    /// @brief Loads .env file from the current directory.
    func loadEnv() {
        bolt_env_load()
    }

    /// @brief Loads environment variables from a specific file.
    /// @param cPath Path to the env file.
    func loadFile(cPath) {
        bolt_env_load_file(cPath)
    }

    /// @brief Gets an environment variable value.
    /// @param cKey Variable name.
    /// @return Variable value or empty string if not found.
    func getVar(cKey) {
        return bolt_env_get(cKey)
    }

    /// @brief Gets an environment variable with a default fallback.
    /// @param cKey Variable name.
    /// @param cDefault Default value if not found.
    /// @return Variable value or default.
    func getOr(cKey, cDefault) {
        return bolt_env_get_or(cKey, cDefault)
    }

    /// @brief Sets an environment variable.
    /// @param cKey Variable name.
    /// @param cValue Variable value.
    func setVar(cKey, cValue) {
        bolt_env_set(cKey, cValue)
    }
}

// ========================================
// Hash Class
// ========================================

/// @class Hash
/// @brief Password hashing class supporting Argon2, bcrypt, and scrypt.
/// @details Provides secure password hashing and verification:
///          @code
///          hash = new Hash
///          cHashed = hash.argon2("mypassword")
///          bValid = hash.verifyArgon2("mypassword", cHashed)
///          @endcode
class Hash {

    /// @brief Hashes a password using Argon2id.
    /// @param cPassword Plain text password.
    /// @return PHC-formatted hash string.
    func argon2(cPassword) {
        return bolt_hash_argon2(cPassword)
    }

    /// @brief Verifies a password against an Argon2 hash.
    /// @param cPassword Plain text password.
    /// @param cHash PHC-formatted hash string.
    /// @return True (1) if valid, false (0) otherwise.
    func verifyArgon2(cPassword, cHash) {
        return bolt_verify_argon2(cPassword, cHash)
    }

    /// @brief Hashes a password using bcrypt.
    /// @param cPassword Plain text password.
    /// @return Bcrypt hash string.
    func bcrypt(cPassword) {
        return bolt_hash_bcrypt(cPassword)
    }

    /// @brief Verifies a password against a bcrypt hash.
    /// @param cPassword Plain text password.
    /// @param cHash Bcrypt hash string.
    /// @return True (1) if valid, false (0) otherwise.
    func verifyBcrypt(cPassword, cHash) {
        return bolt_verify_bcrypt(cPassword, cHash)
    }

    /// @brief Hashes a password using scrypt.
    /// @param cPassword Plain text password.
    /// @return Scrypt hash string.
    func scrypt(cPassword) {
        return bolt_hash_scrypt(cPassword)
    }

    /// @brief Verifies a password against a scrypt hash.
    /// @param cPassword Plain text password.
    /// @param cHash Scrypt hash string.
    /// @return True (1) if valid, false (0) otherwise.
    func verifyScrypt(cPassword, cHash) {
        return bolt_verify_scrypt(cPassword, cHash)
    }
}

// ========================================
// Validate Class
// ========================================

/// @class Validate
/// @brief Input validation utilities.
/// @details Provides validation for common data types:
///          @code
///          v = new Validate
///          if v.email("test@example.com") { ... }
///          @endcode
class Validate {

    /// @brief Validates an email address format.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func email(cStr) {
        return bolt_validate_email(cStr)
    }

    /// @brief Validates a URL format.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func url(cStr) {
        return bolt_validate_url(cStr)
    }

    /// @brief Validates an IP address (v4 or v6).
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func ip(cStr) {
        return bolt_validate_ip(cStr)
    }

    /// @brief Validates an IPv4 address.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func ipv4(cStr) {
        return bolt_validate_ipv4(cStr)
    }

    /// @brief Validates an IPv6 address.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func ipv6(cStr) {
        return bolt_validate_ipv6(cStr)
    }

    /// @brief Validates a UUID format.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func uuid(cStr) {
        return bolt_validate_uuid(cStr)
    }

    /// @brief Validates a JSON string.
    /// @param cStr String to validate.
    /// @return True (1) if valid JSON, false (0) otherwise.
    func jsonString(cStr) {
        return bolt_validate_json_string(cStr)
    }

    /// @brief Validates string length within a range.
    /// @param cStr String to validate.
    /// @param nMin Minimum length.
    /// @param nMax Maximum length.
    /// @return True (1) if valid, false (0) otherwise.
    func length(cStr, nMin, nMax) {
        return bolt_validate_length(cStr, nMin, nMax)
    }

    /// @brief Validates a number within a range.
    /// @param nNum Number to validate.
    /// @param nMin Minimum value.
    /// @param nMax Maximum value.
    /// @return True (1) if valid, false (0) otherwise.
    func range(nNum, nMin, nMax) {
        return bolt_validate_range(nNum, nMin, nMax)
    }

    /// @brief Validates that a string contains only alphabetic characters.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func alpha(cStr) {
        return bolt_validate_alpha(cStr)
    }

    /// @brief Validates that a string contains only alphanumeric characters.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func alphanumeric(cStr) {
        return bolt_validate_alphanumeric(cStr)
    }

    /// @brief Validates that a string contains only numeric characters.
    /// @param cStr String to validate.
    /// @return True (1) if valid, false (0) otherwise.
    func numeric(cStr) {
        return bolt_validate_numeric(cStr)
    }
}

// ========================================
// Crypto Class
// ========================================

/// @class Crypto
/// @brief Encryption and HMAC utilities.
/// @details Provides AES-256-GCM encryption and HMAC-SHA256:
///          @code
///          crypto = new Crypto
///          cEncrypted = crypto.aesEncrypt("secret", "0123456789abcdef0123456789abcdef")
///          cDecrypted = crypto.aesDecrypt(cEncrypted, "0123456789abcdef0123456789abcdef")
///          @endcode
class Crypto {

    /// @brief Encrypts plaintext using AES-256-GCM.
    /// @param cPlaintext Data to encrypt.
    /// @param cKey 32-byte encryption key (hex or string).
    /// @return Base64-encoded ciphertext with IV and tag.
    func aesEncrypt(cPlaintext, cKey) {
        return bolt_aes_encrypt(cPlaintext, cKey)
    }

    /// @brief Decrypts AES-256-GCM ciphertext.
    /// @param cCiphertext Base64-encoded ciphertext.
    /// @param cKey 32-byte decryption key (must match encryption key).
    /// @return Decrypted plaintext string.
    func aesDecrypt(cCiphertext, cKey) {
        return bolt_aes_decrypt(cCiphertext, cKey)
    }

    /// @brief Computes HMAC-SHA256 signature.
    /// @param cMessage Message to sign.
    /// @param cKey Signing key.
    /// @return Hex-encoded HMAC signature.
    func hmacSha256(cMessage, cKey) {
        return bolt_hmac_sha256(cMessage, cKey)
    }

    /// @brief Verifies an HMAC-SHA256 signature.
    /// @param cMessage Original message.
    /// @param cKey Signing key.
    /// @param cSignature Hex-encoded HMAC signature to verify.
    /// @return True (1) if valid, false (0) otherwise.
    func hmacVerify(cMessage, cKey, cSignature) {
        return bolt_hmac_verify(cMessage, cKey, cSignature)
    }
}

// ========================================
// DateTime Class
// ========================================

/// @class DateTime
/// @brief Date and time utilities.
/// @details Provides timestamp operations, formatting, and arithmetic:
///          @code
///          dt = new DateTime
///          nTs = dt.timestamp()
///          cFormatted = dt.formatDate(nTs, "%Y-%m-%d %H:%M:%S")
///          @endcode
class DateTime {

    /// @brief Gets the current local datetime string.
    /// @return Datetime string in ISO format.
    func now() {
        return bolt_datetime_now()
    }

    /// @brief Gets the current UTC datetime string.
    /// @return Datetime string in ISO format.
    func nowUtc() {
        return bolt_datetime_now_utc()
    }

    /// @brief Gets the current Unix timestamp in seconds.
    /// @return Timestamp as number.
    func timestamp() {
        return bolt_datetime_timestamp()
    }

    /// @brief Gets the current Unix timestamp in milliseconds.
    /// @return Timestamp as number.
    func timestampMs() {
        return bolt_datetime_timestamp_ms()
    }

    /// @brief Formats a Unix timestamp to a string.
    /// @param nTimestamp Unix timestamp.
    /// @param cFormat Format string (e.g., "%Y-%m-%d %H:%M:%S").
    /// @return Formatted datetime string.
    func formatDate(nTimestamp, cFormat) {
        return bolt_datetime_format(nTimestamp, cFormat)
    }

    /// @brief Parses a datetime string to a Unix timestamp.
    /// @param cDateStr Datetime string.
    /// @param cFormat Expected format string (e.g., "%Y-%m-%d %H:%M:%S").
    /// @return Unix timestamp as number.
    func parseDate(cDateStr, cFormat) {
        return bolt_datetime_parse(cDateStr, cFormat)
    }

    /// @brief Calculates the difference between two timestamps.
    /// @param nTs1 First timestamp.
    /// @param nTs2 Second timestamp.
    /// @return Difference in seconds.
    func diff(nTs1, nTs2) {
        return bolt_datetime_diff(nTs1, nTs2)
    }

    /// @brief Adds days to a timestamp.
    /// @param nTimestamp Base timestamp.
    /// @param nDays Number of days to add.
    /// @return New timestamp.
    func addDays(nTimestamp, nDays) {
        return bolt_datetime_add_days(nTimestamp, nDays)
    }

    /// @brief Adds hours to a timestamp.
    /// @param nTimestamp Base timestamp.
    /// @param nHours Number of hours to add.
    /// @return New timestamp.
    func addHours(nTimestamp, nHours) {
        return bolt_datetime_add_hours(nTimestamp, nHours)
    }
}

// ========================================
// Sanitize Class
// ========================================

/// @class Sanitize
/// @brief HTML and XSS sanitization utilities.
/// @details Provides safe HTML handling to prevent XSS attacks:
///          @code
///          s = new Sanitize
///          cSafe = s.html('<script>alert("xss")</script><p>Safe</p>')
///          @endcode
class Sanitize {

    /// @brief Sanitizes HTML by stripping dangerous tags, keeping safe ones.
    /// @param cInput Raw HTML string.
    /// @return Sanitized HTML string.
    func html(cInput) {
        return bolt_sanitize_html(cInput)
    }

    /// @brief Strictly sanitizes HTML by stripping all tags.
    /// @param cInput Raw HTML string.
    /// @return Plain text with all HTML removed.
    func strict(cInput) {
        return bolt_sanitize_strict(cInput)
    }

    /// @brief Escapes HTML special characters to entities.
    /// @param cInput Raw string.
    /// @return Escaped string safe for HTML insertion.
    func escapeHtml(cInput) {
        return bolt_escape_html(cInput)
    }
}