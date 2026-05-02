---
title: "Benchmarks"
weight: 0
summary: "Performance comparison with other frameworks"
---

Hello-world endpoint tested with `wrk -t4 -c100 -d10s` on a Ryzen 9 7950x VM.

<div class="benchmark-table-wrapper">
<table class="benchmark-table">
  <thead>
    <tr>
      <th>Framework</th>
      <th>Language</th>
      <th class="num">Requests/sec</th>
      <th class="num">vs Bolt</th>
      <th class="bar-col">Throughput</th>
    </tr>
  </thead>
  <tbody>
    <tr class="bm-faster">
      <td><strong>Actix-web</strong></td>
      <td>Rust</td>
      <td class="num">490,787</td>
      <td class="num"><span class="badge badge-faster">1.6x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:100%"></div></td>
    </tr>
    <tr class="bm-faster">
      <td><strong>Fiber</strong></td>
      <td>Go</td>
      <td class="num">320,953</td>
      <td class="num"><span class="badge badge-faster">1.1x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:65%"></div></td>
    </tr>
    <tr class="bm-bolt">
      <td><strong>⚡ Bolt</strong></td>
      <td><strong>Ring/Rust</strong></td>
      <td class="num"><strong>298,189</strong></td>
      <td class="num"><span class="badge badge-bolt">—</span></td>
      <td class="bar-col"><div class="bar bar-bolt" style="width:61%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Axum</td>
      <td>Rust</td>
      <td class="num">267,995</td>
      <td class="num"><span class="badge badge-slower">1.1x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:55%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Gin</td>
      <td>Go</td>
      <td class="num">227,923</td>
      <td class="num"><span class="badge badge-slower">1.3x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:46%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Bun</td>
      <td>JS</td>
      <td class="num">158,280</td>
      <td class="num"><span class="badge badge-slower">1.9x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:32%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Express/Bun</td>
      <td>JS</td>
      <td class="num">65,388</td>
      <td class="num"><span class="badge badge-slower">4.6x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:13%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Express/Node</td>
      <td>JS</td>
      <td class="num">38,506</td>
      <td class="num"><span class="badge badge-slower">7.7x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:8%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>FastAPI</td>
      <td>Python</td>
      <td class="num">10,219</td>
      <td class="num"><span class="badge badge-slower">29x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:2%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Flask</td>
      <td>Python</td>
      <td class="num">2,338</td>
      <td class="num"><span class="badge badge-slower">128x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:0.5%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Ring HTTPLib</td>
      <td>Ring/C++</td>
      <td class="num">221</td>
      <td class="num"><span class="badge badge-slower">1349x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:0.05%"></div></td>
    </tr>
  </tbody>
</table>
</div>
