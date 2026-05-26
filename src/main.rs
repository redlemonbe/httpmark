mod cli;
mod worker;
mod stats;
mod output;

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::Semaphore;
use tokio::time::sleep;

use cli::Args;
use stats::Stats;
use worker::run_worker;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(Args::parse_and_validate()?);

    rustls::crypto::ring::default_provider().install_default().ok();

    if let Some(compare_url) = args.compare.clone() {
        // Compare mode — two parallel benchmarks, no interval output
        let mut cmp = (*args).clone();
        cmp.url = compare_url;
        cmp.compare = None;
        let cmp_args = Arc::new(cmp);

        output::print_compare_header(&args, &cmp_args);

        let (stats_a, stats_b) = tokio::join!(
            run_benchmark(Arc::clone(&args)),
            run_benchmark(Arc::clone(&cmp_args)),
        );

        let fa = stats_a.final_snapshot();
        let fb = stats_b.final_snapshot();
        output::print_comparison(&args, &fa, &cmp_args, &fb, args.duration as f64);
    } else {
        // Single mode — with per-second interval output
        output::print_header(&args);

        let stats = Arc::new(Stats::new());
        let sem   = Arc::new(Semaphore::new(args.connections));
        let start = Instant::now();
        let duration = Duration::from_secs(args.duration);
        let ramp     = Duration::from_secs(args.ramp);

        let stats_reporter = Arc::clone(&stats);
        let json = args.json;
        let interval_handle = if !json && args.csv_interval == 0 {
            Some(tokio::spawn(async move {
                let mut last = Instant::now();
                loop {
                    sleep(Duration::from_secs(1)).await;
                    let elapsed = last.elapsed().as_secs_f64();
                    last = Instant::now();
                    let snap = stats_reporter.snapshot();
                    output::print_interval(&snap, start.elapsed().as_secs_f64(), elapsed);
                }
            }))
        } else {
            None
        };

        let mut handles = Vec::with_capacity(args.connections);
        for _ in 0..args.connections {
            let a = Arc::clone(&args);
            let s = Arc::clone(&stats);
            let p = Arc::clone(&sem);
            handles.push(tokio::spawn(async move {
                let _permit = p.acquire().await.ok();
                run_worker(a, s, start, duration, ramp).await;
            }));
        }

        sleep(duration).await;
        stats.stop();
        if let Some(h) = interval_handle { h.abort(); }
        for h in handles { let _ = tokio::time::timeout(Duration::from_secs(2), h).await; }

        let final_stats = stats.final_snapshot();
        output::print_summary(&args, &final_stats, duration.as_secs_f64());
    }

    Ok(())
}

async fn run_benchmark(args: Arc<Args>) -> Arc<Stats> {
    let stats = Arc::new(Stats::new());
    let sem   = Arc::new(Semaphore::new(args.connections));
    let start = Instant::now();
    let duration = Duration::from_secs(args.duration);
    let ramp     = Duration::from_secs(args.ramp);

    let mut handles = Vec::with_capacity(args.connections);
    for _ in 0..args.connections {
        let a = Arc::clone(&args);
        let s = Arc::clone(&stats);
        let p = Arc::clone(&sem);
        handles.push(tokio::spawn(async move {
            let _permit = p.acquire().await.ok();
            run_worker(a, s, start, duration, ramp).await;
        }));
    }

    sleep(duration).await;
    stats.stop();
    for h in handles { let _ = tokio::time::timeout(Duration::from_secs(2), h).await; }
    stats
}
