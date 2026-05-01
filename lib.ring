// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

if (isWindows()) {
	loadlib("ring_bolt.dll")
elseif (isUnix() && !(isMacOSX() || isAndroid()))
	loadlib("libring_bolt.so")
elseif (isMacOSX())
	loadlib("libring_bolt.dylib")
else
	raise("Unsupported OS! You need to build the library for your OS.")
}

load "src/bolt.ring"
