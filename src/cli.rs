use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(
    name = "httpmark",
    version,
    about = "Fast HTTP benchmark tool — wrk/ab replacement",
    long_about = "Measures latency percentiles, throughput and error rates for HTTP/1.1 and HTTP/2 servers.\n\nACCEPTABLE USE: Only benchmark servers you own or have explicit permission to test."
)]
pub struct Args {
    /// Target URL
    pub url: String,

    /// Concurrent connections
    #[arg(short = 'c', long, default_value = "10")]
    pub connections: usize,

    /// Test duration in seconds
    #[arg(short = 'd', long, default_value = "10")]
    pub duration: u64,

    /// Target QPS (0 = unlimited)
    #[arg(short = 'q', long = "qps", default_value = "0")]
    pub qps: u64,

    /// Ramp-up duration in seconds (linearly increase QPS from 0)
    #[arg(long, default_value = "0")]
    pub ramp: u64,

    /// Worker threads (default: number of logical CPUs)
    #[arg(short = 't', long)]
    pub threads: Option<usize>,

    /// Additional request headers (format: "Name: Value"), repeatable
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// HTTP method [GET, POST, PUT, DELETE, HEAD, OPTIONS]
    #[arg(short = 'm', long, default_value = "GET")]
    pub method: String,

    /// Request body (string or @file path)
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Content-Type header for request body
    #[arg(long, default_value = "application/json")]
    pub content_type: String,

    /// Skip TLS certificate verification (dangerous, use only for testing)
    #[arg(long)]
    pub tls_skip_verify: bool,

    /// Force HTTP/2 (requires TLS or h2c)
    #[arg(short = '2', long)]
    pub http2: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Print per-interval CSV line every N seconds (0 = disable)
    #[arg(long, default_value = "0")]
    pub csv_interval: u64,

    /// Compare mode: second URL to benchmark in parallel
    #[arg(long)]
    pub compare: Option<String>,
}

impl Args {
    pub fn parse_and_validate() -> Result<Self> {
        let args = Self::parse();

        if args.connections == 0 {
            bail!("--connections must be >= 1");
        }
        if args.duration == 0 {
            bail!("--duration must be >= 1");
        }
        if !matches!(args.method.to_uppercase().as_str(),
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH") {
            bail!("unsupported method: {}", args.method);
        }

        Ok(args)
    }

    pub fn method_http(&self) -> http::Method {
        self.method.to_uppercase().parse().unwrap_or(http::Method::GET)
    }

    pub fn parsed_headers(&self) -> Vec<(String, String)> {
        self.headers.iter().filter_map(|h| {
            let mut parts = h.splitn(2, ':');
            let name = parts.next()?.trim().to_owned();
            let value = parts.next()?.trim().to_owned();
            Some((name, value))
        }).collect()
    }
}
