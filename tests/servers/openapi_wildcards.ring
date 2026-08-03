load "bolt.ring"


new Bolt() {
    cPort = sysget("BOLT_TEST_PORT")
    if cPort = "" { cPort = "3000" }
    port = number(cPort)

    @get("/health", func {
        $bolt.json([:status = "ok"])
    })

    enableDocs()
    setDocsInfo("Wildcard Test API", "1.0.0", "API for testing wildcard routes")

    @get("/files/*path", func {
        $bolt.json([:path = $bolt.param("path")])
    })

    @get("/users/:id/settings/*section", func {
        $bolt.json([
            :id = $bolt.param("id"),
            :section = $bolt.param("section")
        ])
    })
}
