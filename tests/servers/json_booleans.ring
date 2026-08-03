load "bolt.ring"


new Bolt() {
    cPort = sysget("BOLT_TEST_PORT")
    if cPort = "" { cPort = "3000" }
    port = number(cPort)

    @get("/health", func {
        $bolt.json([:status = "ok"])
    })

    @get("/flags", func {
        data = [
            :active   = $bolt.jsonTrue(),
            :disabled = $bolt.jsonFalse(),
            :count    = 1
        ]
        $bolt.json(data)
    })

    @get("/decode", func {
        data = $bolt.jsonDecode('{"enabled": true, "muted": false, "score": 1}')

        $bolt.json([
            :enabled        = $bolt.jsonIsTrue(data[:enabled]),
            :muted          = $bolt.jsonIsFalse(data[:muted]),
            :enabledIsFalse = $bolt.jsonIsFalse(data[:enabled]),
            :mutedIsTrue    = $bolt.jsonIsTrue(data[:muted]),
            :score          = data[:score],
            :asBool         = $bolt.jsonToBool(data[:enabled]),
            :asBoolFalse    = $bolt.jsonToBool(data[:muted])
        ])
    })

    @get("/roundtrip", func {
        original = '{"active": true, "disabled": false, "count": 1}'
        restored = $bolt.jsonEncode($bolt.jsonDecode(original))
        $bolt.json([:original = original, :restored = restored])
    })
}
