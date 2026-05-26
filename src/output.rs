
use serde::Serialize;

use crate::cli::Args;
use crate::stats::{FinalStats, Snapshot};

fn fmt_us(us: u64) -> String {
    if us < 1_000 {
        format!("{us}µs")
    } else if us < 1_000_000 {
        format!("{:.2}ms", us as f64 / 1_000.0)
    } else {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    }
}

fn fmt_bytes(b: u64) -> String {
    if b < 1_024 {
        format!("{b}B")
    } else if b < 1_048_576 {
        format!("{:.1}KB", b as f64 / 1_024.0)
    } else {
        format!("{:.2}MB", b as f64 / 1_048_576.0)
    }
}

pub fn print_header(args: &Args) {
    if args.json { return; }
    println!();
    println!("  httpmark v{}", env!("CARGO_PKG_VERSION"));
    println!("  URL          : {}", args.url);
    println!("  Connections  : {}", args.connections);
    println!("  Duration     : {}s", args.duration);
    if args.qps > 0 {
        println!("  Target QPS   : {}", args.qps);
    }
    if args.ramp > 0 {
        println!("  Ramp         : {}s", args.ramp);
    }
    println!("  HTTP/2       : {}", if args.http2 { "yes" } else { "no" });
    println!();
    println!("  {:<8}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}",
        "t(s)", "req/s", "2xx/s", "errors/s", "MB/s", "~latency");
    println!("  {}", "─".repeat(72));
}

pub fn print_interval(snap: &Snapshot, elapsed: f64, interval: f64) {
    if interval <= 0.0 { return; }
    let rps       = snap.interval_reqs as f64 / interval;
    let ok_rps    = snap.interval_2xx as f64 / interval;
    let err_s     = snap.interval_errors as f64 / interval;
    let mbs       = snap.interval_bytes as f64 / interval / 1_048_576.0;

    println!("  {:<8.1}  {:>10.0}  {:>10.0}  {:>10.1}  {:>10.2}",
        elapsed, rps, ok_rps, err_s, mbs);
}

pub fn print_summary(args: &Args, s: &FinalStats, duration: f64) {
    if args.json {
        print_json(args, s, duration);
        return;
    }

    println!();
    println!("  Results");
    println!("  {}", "─".repeat(48));
    println!("  Requests      : {}", s.requests);
    println!("  Throughput    : {:.0} req/s", s.requests as f64 / duration);
    println!("  Bandwidth     : {}/s", fmt_bytes((s.bytes_recv as f64 / duration) as u64));
    println!("  Data recv     : {}", fmt_bytes(s.bytes_recv));
    println!();
    println!("  Status codes");
    if s.status_2xx > 0 { println!("    2xx : {}", s.status_2xx); }
    if s.status_3xx > 0 { println!("    3xx : {}", s.status_3xx); }
    if s.status_4xx > 0 { println!("    4xx : {}", s.status_4xx); }
    if s.status_5xx > 0 { println!("    5xx : {}", s.status_5xx); }
    if s.errors     > 0 { println!("  Errors  : {}", s.errors); }
    println!();
    println!("  Latency percentiles");
    println!("    p50   : {}", fmt_us(s.p50_us));
    println!("    p90   : {}", fmt_us(s.p90_us));
    println!("    p95   : {}", fmt_us(s.p95_us));
    println!("    p99   : {}", fmt_us(s.p99_us));
    println!("    p99.9 : {}", fmt_us(s.p999_us));
    println!("    max   : {}", fmt_us(s.max_us));
    println!("    mean  : {}", fmt_us(s.mean_us as u64));
    println!();
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    url:          &'a str,
    connections:  usize,
    duration_s:   f64,
    requests:     u64,
    rps:          f64,
    bandwidth_bps: f64,
    bytes_recv:   u64,
    errors:       u64,
    status: StatusBreakdown,
    latency: Latency,
}

#[derive(Serialize)]
struct StatusBreakdown {
    s2xx: u64,
    s3xx: u64,
    s4xx: u64,
    s5xx: u64,
}

#[derive(Serialize)]
struct Latency {
    p50_us:  u64,
    p90_us:  u64,
    p95_us:  u64,
    p99_us:  u64,
    p999_us: u64,
    max_us:  u64,
    mean_us: f64,
}

fn print_json(args: &Args, s: &FinalStats, duration: f64) {
    let out = JsonOutput {
        url:           &args.url,
        connections:   args.connections,
        duration_s:    duration,
        requests:      s.requests,
        rps:           s.requests as f64 / duration,
        bandwidth_bps: s.bytes_recv as f64 / duration,
        bytes_recv:    s.bytes_recv,
        errors:        s.errors,
        status: StatusBreakdown {
            s2xx: s.status_2xx,
            s3xx: s.status_3xx,
            s4xx: s.status_4xx,
            s5xx: s.status_5xx,
        },
        latency: Latency {
            p50_us:  s.p50_us,
            p90_us:  s.p90_us,
            p95_us:  s.p95_us,
            p99_us:  s.p99_us,
            p999_us: s.p999_us,
            max_us:  s.max_us,
            mean_us: s.mean_us,
        },
    };
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
