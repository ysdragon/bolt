---
title: "Benchmarks"
weight: 0
summary: "Performance comparison with other frameworks"
---

Hello-world endpoint tested with `wrk -t8 -c100 -d10s` (5s warmup) on a Ryzen 9 7950x VM (12 vCPUs).

<div class="benchmark-table-wrapper">
<table class="benchmark-table">
  <colgroup>
    <col><col><col><col><col><col>
  </colgroup>
  <thead>
    <tr>
      <th>Framework</th>
      <th>Language</th>
      <th class="num">Requests/sec</th>
      <th class="num">Avg Latency</th>
      <th class="num">vs Bolt</th>
      <th class="bar-col">Throughput</th>
    </tr>
  </thead>
  <tbody>
    <tr class="bm-faster">
      <td><strong>Actix-web</strong></td>
      <td>Rust</td>
      <td class="num">785,403</td>
      <td class="num">270.21us</td>
      <td class="num"><span class="badge badge-faster">2.3x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:100%"></div></td>
    </tr>
    <tr class="bm-faster">
      <td><strong>ASP.NET</strong></td>
      <td>.NET</td>
      <td class="num">529,987</td>
      <td class="num">376.44us</td>
      <td class="num"><span class="badge badge-faster">1.5x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:67%"></div></td>
    </tr>
    <tr class="bm-faster">
      <td><strong>Fiber</strong></td>
      <td>Go</td>
      <td class="num">500,740</td>
      <td class="num">341.55us</td>
      <td class="num"><span class="badge badge-faster">1.5x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:64%"></div></td>
    </tr>
    <tr class="bm-faster">
      <td><strong>Java Virtual Threads</strong></td>
      <td>Java</td>
      <td class="num">487,340</td>
      <td class="num">218.52us</td>
      <td class="num"><span class="badge badge-faster">1.4x faster</span></td>
      <td class="bar-col"><div class="bar" style="width:62%"></div></td>
    </tr>
    <tr class="bm-bolt">
      <td><strong>⚡ Bolt</strong></td>
      <td><strong>Ring/Rust</strong></td>
      <td class="num"><strong>342,610</strong></td>
      <td class="num"><strong>272.48us</strong></td>
      <td class="num"><span class="badge badge-bolt">—</span></td>
      <td class="bar-col"><div class="bar bar-bolt" style="width:44%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Gin</td>
      <td>Go</td>
      <td class="num">296,610</td>
      <td class="num">448.94us</td>
      <td class="num"><span class="badge badge-slower">1.2x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:38%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Bun</td>
      <td>JS</td>
      <td class="num">153,550</td>
      <td class="num">620.98us</td>
      <td class="num"><span class="badge badge-slower">2.2x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:20%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Elysia</td>
      <td>Bun</td>
      <td class="num">152,201</td>
      <td class="num">627.12us</td>
      <td class="num"><span class="badge badge-slower">2.3x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:19%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>NestJS+Fastify/Node</td>
      <td>JS</td>
      <td class="num">67,382</td>
      <td class="num">1.63ms</td>
      <td class="num"><span class="badge badge-slower">5.1x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:9%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Express/Bun</td>
      <td>JS</td>
      <td class="num">58,397</td>
      <td class="num">1.64ms</td>
      <td class="num"><span class="badge badge-slower">5.9x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:7%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Flask</td>
      <td>Python</td>
      <td class="num">48,851</td>
      <td class="num">1.76ms</td>
      <td class="num"><span class="badge badge-slower">7.0x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:6%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>Express/Node</td>
      <td>JS</td>
      <td class="num">39,571</td>
      <td class="num">2.85ms</td>
      <td class="num"><span class="badge badge-slower">8.7x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:5%"></div></td>
    </tr>
    <tr class="bm-slower">
      <td>FastAPI</td>
      <td>Python</td>
      <td class="num">2,153</td>
      <td class="num">44.26ms</td>
      <td class="num"><span class="badge badge-slower">159x slower</span></td>
      <td class="bar-col"><div class="bar" style="width:0.3%"></div></td>
    </tr>
  </tbody>
</table>
</div>
