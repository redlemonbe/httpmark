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

    // Initialize rustls ring crypto provider
    rustls::crypto::ring::default_provider().install_default().ok();

    output::print_header(&args);

    let stats = Arc::new(Stats::new());
    let sem = Arc::new(Semaphore::new(args.connections));

    let start = Instant::now();
    let duration = Duration::from_secs(args.duration);
    let ramp = Duration::from_secs(args.ramp);

    // Interval reporter
    let stats_clone = Arc::clone(&stats);
    let csv_interval = args.csv_interval;
    let json = args.json;
    let interval_handle = if !json && csv_interval == 0 {
        let start2 = start;
        Some(tokio::spawn(async move {
            let mut last = Instant::now();
            loop {
                sleep(Duration::from_secs(1)).await;
                let elapsed = last.elapsed().as_secs_f64();
                last = Instant::now();
                let snap = stats_clone.snapshot();
                output::print_interval(&snap, start2.elapsed().as_secs_f64(), elapsed);
            }
        }))
    } else {
        None
    };

    // Launch workers
    let mut handles = Vec::with_capacity(args.connections);
    for _ in 0..args.connections {
        let args = Arc::clone(&args);
        let stats = Arc::clone(&stats);
        let sem = Arc::clone(&sem);
        let start = start;

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok();
            run_worker(args, stats, start, duration, ramp).await;
        }));
    }

    sleep(duration).await;

    // Signal stop
    stats.stop();

    if let Some(h) = interval_handle {
        h.abort();
    }

    for h in handles {
        let _ = tokio::time::timeout(Duration::from_secs(2), h).await;
    }

    let final_stats = stats.final_snapshot();
    output::print_summary(&args, &final_stats, duration.as_secs_f64());

    Ok(())
}
