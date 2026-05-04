---
title: ''
layout: hextra-home
---

<div class="hx:mb-6 hx:flex hx:flex-col hx:gap-4 hx:justify-center hx:items-center hx:w-full hx:mx-auto">
{{< hextra/hero-badge >}}
  <div class="hx:w-2 hx:h-2 hx:rounded-full hx:bg-primary-400"></div>
  <span>Open source, MIT licensed</span>
  {{< icon name="arrow-circle-right" attributes="height=14" >}}
{{< /hextra/hero-badge >}}

{{< hextra/hero-headline >}}
  A <span class="home-highlight">blazing-fast</span> web framework for Ring
{{< /hextra/hero-headline >}}
</div>

<div class="hx:mb-6 hx:flex hx:justify-center hx:items-center hx:w-full">
{{< hextra/hero-subtitle >}}
  Express.js-like DSL with a Rust-powered HTTP engine
{{< /hextra/hero-subtitle >}}
</div>

<div class="hx:mb-6 hx:flex hx:justify-center hx:items-center hx:w-full">

{{< hextra/hero-button text="Get Started" link="/docs/gettingstarted" >}}
{{< hextra/hero-button text="API Reference" link="/docs/reference/server-configuration" >}}
</div>
<div class="hx:mb-6 hx:flex hx:justify-center hx:items-center hx:w-full">
  <p class="hx:text-center hx:text-sm hx:text-gray-600 hx:dark:text-gray-400">
    Available for Windows, Linux, macOS, and FreeBSD
  </p>
</div>

<div class="hx:mt-6"></div>

<div class="hx:w-full hx:mx-auto" style="max-width: 900px;">
{{< hextra/feature-grid cols="3" >}}
  {{< hextra/feature-card
    title="Express-like Routing"
    subtitle="All HTTP methods, URL params, query strings, and regex constraints."
  >}}
  {{< hextra/feature-card
    title="Rust-powered Engine"
    subtitle="~343K req/s backed by Actix-web and Tokio async runtime."
  >}}
  {{< hextra/feature-card
    title="Batteries Included"
    subtitle="WebSocket, SSE, JWT, sessions, caching, templates, OpenAPI, and more."
  >}}
{{< /hextra/feature-grid >}}
</div>
