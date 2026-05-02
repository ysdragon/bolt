aPackageInfo = [
	:name = "Bolt",
	:description = "Blazing-fast web framework for the Ring Programming Language.",
	:folder = "bolt",
	:developer = "ysdragon",
	:email = "youssefelkholey@gmail.com",
	:license = "MIT",
	:version = "1.0.0",
	:ringversion = "1.25",
	:versions = 	[
		[
			:version = "1.0.0",
			:branch = "master"
		]
	],
	:libs = 	[
		[
			:name = "",
			:version = "",
			:providerusername = ""
		]
	],
	:files = 	[
		// Root
		"lib.ring",
		"main.ring",
		"package.ring",
		"README.md",
		"LICENSE",

		// Docs
		"docs/API.md",
		"docs/USAGE.md",

		// Assets
		"assets/logo.png",

		// Source
		"src/bolt.ring",

		// Utils
		"src/utils/color.ring",
		"src/utils/install.ring",
		"src/utils/uninstall.ring",

		// Examples
		"examples/basic/01_hello.ring",
		"examples/basic/02_http_methods.ring",
		"examples/basic/03_route_params.ring",
		"examples/basic/04_request_response.ring",
		"examples/basic/05_json_api.ring",
		"examples/basic/06_static_files.ring",
		"examples/basic/static/index.html",

		// Rust - Config
		"src/rust_src/Cargo.toml",
		"src/rust_src/src/lib.rs",

		// Rust - Modules
		"src/rust_src/src/modules/mod.rs",
		"src/rust_src/src/modules/base64.rs",
		"src/rust_src/src/modules/crypto.rs",
		"src/rust_src/src/modules/datetime.rs",
		"src/rust_src/src/modules/env.rs",
		"src/rust_src/src/modules/hash.rs",
		"src/rust_src/src/modules/json.rs",
		"src/rust_src/src/modules/sanitize.rs",
		"src/rust_src/src/modules/validate.rs",

		// Rust - Server
		"src/rust_src/src/server/mod.rs",
		"src/rust_src/src/server/auth.rs",
		"src/rust_src/src/server/cache.rs",
		"src/rust_src/src/server/logging.rs",
		"src/rust_src/src/server/middleware.rs",
		"src/rust_src/src/server/openapi.rs",
		"src/rust_src/src/server/rate_limit.rs",
		"src/rust_src/src/server/response.rs",
		"src/rust_src/src/server/sessions.rs",
		"src/rust_src/src/server/sse.rs",
		"src/rust_src/src/server/templates.rs",
		"src/rust_src/src/server/uploads.rs",
		"src/rust_src/src/server/websocket.rs"
	],
	:ringfolderfiles = 	[

	],
	:windowsfiles = 	[
		"lib/windows/amd64/ring_bolt.dll",
		"lib/windows/i386/ring_bolt.dll",
		"lib/windows/arm64/ring_bolt.dll"
	],
	:linuxfiles = 	[
		"lib/linux/amd64/libring_bolt.so",
		"lib/linux/arm64/libring_bolt.so",
		"lib/linux/musl/amd64/libring_bolt.so",
		"lib/linux/musl/arm64/libring_bolt.so"
	],
	:ubuntufiles = 	[

	],
	:fedorafiles = 	[

	],
	:macosfiles = 	[
		"lib/macos/amd64/libring_bolt.dylib",
		"lib/macos/arm64/libring_bolt.dylib"
	],
	:freebsdfiles = 	[
		"lib/freebsd/amd64/libring_bolt.so"
	],
	:windowsringfolderfiles = 	[

	],
	:linuxringfolderfiles = 	[

	],
	:ubunturingfolderfiles = 	[

	],
	:fedoraringfolderfiles = 	[

	],
	:freebsdringfolderfiles = 	[

	],
	:macosringfolderfiles = 	[

	],
	:run = "ring main.ring",
	:windowsrun = "",
	:linuxrun = "",
	:macosrun = "",
	:ubunturun = "",
	:fedorarun = "",
	:setup = "ring src/utils/install.ring",
	:windowssetup = "",
	:linuxsetup = "",
	:macossetup = "",
	:ubuntusetup = "",
	:fedorasetup = "",
	:remove = "ring src/utils/uninstall.ring",
	:windowsremove = "",
	:linuxremove = "",
	:macosremove = "",
	:ubunturemove = "",
	:fedoraremove = "",
    :remotefolder = "bolt",
    :branch = "master",
    :providerusername = "ysdragon",
    :providerwebsite = "github.com"
]
