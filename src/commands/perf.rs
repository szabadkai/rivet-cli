use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Duration;

use crate::performance::PerformanceTestRunner;
use crate::utils::parse_timeout;

pub struct PerfOptions {
    pub target: PathBuf,
    pub duration: String,
    pub rps: Option<u32>,
    pub concurrent: u32,
    pub warmup: String,
    pub report_interval: String,
    pub output: Option<PathBuf>,
    pub pattern: String,
    pub env: Option<String>,
}

pub async fn handle_perf(options: PerfOptions) -> Result<()> {
    println!("{} Starting performance test", "→".cyan());
    println!(
        "Target: {}",
        options.target.display().to_string().bright_white()
    );
    println!("Duration: {}", options.duration.bright_white());

    if let Some(rps) = options.rps {
        println!("Target RPS: {}", rps.to_string().bright_white());
    }

    println!(
        "Concurrent users: {}",
        options.concurrent.to_string().bright_white()
    );
    println!("Pattern: {}", options.pattern.bright_white());

    // Parse time strings to Durations
    let test_duration = parse_timeout(&options.duration)?;
    let warmup_duration = parse_timeout(&options.warmup)?;
    let report_interval = parse_timeout(&options.report_interval)?;

    // Validate load pattern
    let load_pattern = match options.pattern.as_str() {
        "constant" => crate::performance::LoadPattern::Constant,
        "ramp-up" => crate::performance::LoadPattern::RampUp,
        "spike" => crate::performance::LoadPattern::Spike,
        _ => {
            anyhow::bail!(
                "Invalid load pattern '{}'. Use: constant, ramp-up, spike",
                options.pattern
            );
        }
    };

    // Create performance test runner
    let runner = PerformanceTestRunner::new(
        options.concurrent,
        options.rps,
        test_duration,
        warmup_duration,
        report_interval,
        load_pattern,
    )?;

    // Run performance test
    let results = runner
        .run_performance_test(&options.target, options.env.as_deref())
        .await?;

    // Print final summary
    println!();
    println!("{} Performance test completed", "✔".green().bold());

    let avg_response_time = results.average_response_time;
    let total_requests = results.total_requests;
    let success_rate = results.success_rate * 100.0;
    let actual_rps = results.requests_per_second;

    println!(
        "Total requests: {}",
        total_requests.to_string().bright_white()
    );
    println!(
        "Success rate: {:.1}%",
        success_rate.to_string().bright_white()
    );
    println!(
        "Average response time: {:.2}ms",
        avg_response_time.as_millis().to_string().bright_white()
    );
    println!("Actual RPS: {:.1}", actual_rps.to_string().bright_white());

    if results.p99_response_time > Duration::from_millis(1000) {
        println!(
            "{} P99 response time is high: {:.2}ms",
            "⚠".yellow(),
            results
                .p99_response_time
                .as_millis()
                .to_string()
                .bright_white()
        );
    }

    // Save performance report if requested
    if let Some(output_path) = options.output {
        println!();
        println!("Saving performance report to: {}", output_path.display());
        results.save_report(&output_path)?;
        println!("{} Performance report saved", "✔".green());
    }

    // Exit with error code if test failed performance criteria
    if success_rate < 95.0 {
        anyhow::bail!(
            "Performance test failed: Success rate {:.1}% is below 95%",
            success_rate
        );
    }

    Ok(())
}
