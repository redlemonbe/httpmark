# httpmark

Fast HTTP/HTTPS benchmark tool.  
Latency percentiles. QPS rate control. Compare mode. HTTP/2. JSON output. Static binary.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/redlemonbe/httpmark)](https://github.com/redlemonbe/httpmark/releases/latest)

> **Authorized testing only.**  
> Only use httpmark against servers you own or have explicit written authorization to test.  
> Read [ACCEPTABLE_USE.md](ACCEPTABLE_USE.md) before use.

---

## Install

```bash
# x86_64
curl -Lo httpmark https://github.com/redlemonbe/httpmark/releases/latest/download/httpmark-x86_64-linux-gnu
chmod +x httpmark && sudo mv httpmark /usr/local/bin/
```

> Run httpmark on a **separate machine** from the server under test.

---

## Quick start

```bash
# Basic benchmark — 10 connections, 10 seconds
httpmark https://api.example.com/health

# 50 connections, 30 seconds
httpmark https://api.example.com/health -c 50 -d 30

# Cap at 500 QPS with 10s ramp-up
httpmark https://api.example.com/ -c 20 -d 60 --qps 500 --ramp 10

# Force HTTP/2
httpmark https://api.example.com/ -c 20 -d 10 --http2

# POST request with JSON body
httpmark https://api.example.com/items -m POST -b '{"name":"test"}' -H "X-API-Key: secret"

# JSON output (CI/CD, dashboards)
httpmark https://api.example.com/health -c 50 -d 30 --json

# Compare two endpoints side by side
httpmark https://api.example.com/v1/health --compare https://api.example.com/v2/health -c 20 -d 30
```

---

## Compare mode

```bash
httpmark https://api.example.com/v1/items --compare https://api.example.com/v2/items -c 20 -d 30
```

Both targets run in parallel with the same connection count and duration. Results are shown side by side with deltas.

```
  httpmark v0.1.0  —  COMPARE MODE
  A : https://api.example.com/v1/items
  B : https://api.example.com/v2/items
  Connections per target : 20
  Duration               : 30s

  Running both benchmarks in parallel...

  Metric                           A               B  Delta B vs A
  ──────────────────────────────────────────────────────────────
  Requests                    87 432          92 100        +5.4%
  Throughput                2 914/s         3 070/s        +5.4%
  Bandwidth               1.27MB/s        1.34MB/s        +5.5%
  2xx                         87 432          92 100        +5.4%

  Latency                          A               B  Delta B vs A
  ──────────────────────────────────────────────────────────────
  p50                        16.23ms         14.81ms        -8.7%
  p90                        24.11ms         22.33ms        -7.4%
  p95                        29.44ms         27.12ms        -7.9%
  p99                        48.72ms         44.55ms        -8.6%
  p99.9                      89.15ms         81.22ms        -8.9%
  max                       143.22ms        131.44ms        -8.2%
  mean                       17.14ms         15.68ms        -8.5%
```

---

## Output

```
  httpmark v0.1.0
  URL          : https://api.example.com/health
  Connections  : 50
  Duration     : 30s
  HTTP/2       : no

  t(s)           req/s       2xx/s    errors/s        MB/s    ~latency
  ────────────────────────────────────────────────────────────────────────
  1.0             2847        2847         0.0        1.24
  2.0             2931        2931         0.0        1.28
  ...

  Results
  ────────────────────────────────────────────────
  Requests      : 87 432
  Throughput    : 2 914 req/s
  Bandwidth     : 1.27 MB/s
  Data recv     : 38.1 MB

  Status codes
    2xx : 87 432

  Latency percentiles
    p50   : 16.23ms
    p90   : 24.11ms
    p95   : 29.44ms
    p99   : 48.72ms
    p99.9 : 89.15ms
    max   : 143.22ms
    mean  : 17.14ms
```

---

## JSON output (`--json`)

```json
{
  "url": "https://api.example.com/health",
  "connections": 50,
  "duration_s": 30.0,
  "requests": 87432,
  "rps": 2914.4,
  "bandwidth_bps": 1331246.0,
  "bytes_recv": 39937380,
  "errors": 0,
  "status": { "s2xx": 87432, "s3xx": 0, "s4xx": 0, "s5xx": 0 },
  "latency": {
    "p50_us": 16230,
    "p90_us": 24110,
    "p95_us": 29440,
    "p99_us": 48720,
    "p999_us": 89150,
    "max_us": 143220,
    "mean_us": 17140.0
  }
}
```

All latency values are in **microseconds** (`_us` suffix).

---

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `<URL>` | — | Target URL (required) |
| `-c, --connections <N>` | `10` | Concurrent connections |
| `-d, --duration <secs>` | `10` | Test duration |
| `-q, --qps <N>` | `0` (unlimited) | Target QPS cap |
| `--ramp <secs>` | `0` | Linear ramp-up from 0 to --qps |
| `-t, --threads <N>` | CPUs | Tokio worker threads |
| `-H, --header <N: V>` | — | Extra header (repeatable) |
| `-m, --method <M>` | `GET` | HTTP method |
| `-b, --body <DATA>` | — | Request body (string or `@file`) |
| `--content-type <TYPE>` | `application/json` | Content-Type for body |
| `--tls-skip-verify` | — | Skip TLS certificate verification |
| `-2, --http2` | — | Force HTTP/2 (ALPN) |
| `--json` | — | JSON output |
| `--csv-interval <secs>` | `0` | Per-interval CSV (0 = off) |
| `--compare <URL>` | — | Compare mode: second URL to benchmark in parallel |

---

## Build from source

```bash
git clone https://github.com/redlemonbe/httpmark
cd httpmark
cargo build --release
# binary: target/release/httpmark
```

Requires Rust 1.75+.

---

## Contributing

`cargo clippy --all-targets` — zero warnings  
`cargo test` — all tests must pass

---

## Support

[![Sponsor](https://img.shields.io/github/sponsors/redlemonbe?style=flat&logo=github&label=Sponsor)](https://github.com/sponsors/redlemonbe)

**Bitcoin** — `3FP8hkkiu4kwCD1PDFgAv2oq1ZTyXwy3yy`  
**Ethereum** — `0xB5eEAf89edA4204Aa9305B068b37A93439cBb680`

Security issues: redlemonbe@codix.be (private disclosure before opening a public issue)

---

## License

MIT — see [LICENSE](LICENSE)

---

*httpmark is a companion tool for [RunNginx](https://github.com/redlemonbe/RunNginx).*  
Copyright (C) 2026 RedLemonBe
