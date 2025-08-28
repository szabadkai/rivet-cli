use crate::performance::{PerformanceMetrics, LoadPattern};
use owo_colors::OwoColorize;
use std::time::{Duration, Instant};

pub struct PerformanceMonitor {
    start_time: Instant,
    last_report: Instant,
    report_interval: Duration,
    #[allow(dead_code)] // Used for future monitoring enhancements
    load_pattern: LoadPattern,
}

impl PerformanceMonitor {
    pub fn new(report_interval: Duration, load_pattern: LoadPattern) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_report: now,
            report_interval,
            load_pattern,
        }
    }

    /// Check if it's time to generate a progress report
    pub fn should_report(&self) -> bool {
        self.last_report.elapsed() >= self.report_interval
    }

    /// Generate and print a progress report
    pub fn print_progress_report(&mut self, metrics: &PerformanceMetrics, target_duration: Duration, load_controller: &crate::performance::patterns::LoadController) {
        if !self.should_report() {
            return;
        }

        let elapsed = self.start_time.elapsed();
        let remaining = target_duration.saturating_sub(elapsed);
        
        let results = metrics.calculate_results();
        let current_rps = if elapsed.as_secs() > 0 {
            results.total_requests as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        // Calculate progress percentage
        let progress_percent = if target_duration > Duration::ZERO {
            (elapsed.as_secs_f64() / target_duration.as_secs_f64() * 100.0).min(100.0)
        } else {
            0.0
        };

        // Create progress bar
        let bar_width = 20;
        let filled = ((progress_percent / 100.0) * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let progress_bar = format!("[{}{}]", 
            "=".repeat(filled).green(), 
            "-".repeat(empty).dimmed()
        );

        println!();
        println!("{} Performance Test Progress", "📊".bright_white());
        println!("  {} {:.1}% ({:?} / {:?})", 
                 progress_bar, 
                 progress_percent, 
                 elapsed, 
                 target_duration);
        
        println!("  Load Pattern: {}", load_controller.current_phase_description().bright_white());
        
        if results.total_requests > 0 {
            println!("  Current RPS: {:.1}", current_rps.to_string().bright_white());
            println!("  Total Requests: {}", results.total_requests.to_string().bright_white());
            println!("  Success Rate: {:.1}%", (results.success_rate * 100.0).to_string().bright_white());
            
            if !results.average_response_time.is_zero() {
                println!("  Avg Response Time: {:.2}ms", 
                         results.average_response_time.as_millis().to_string().bright_white());
                println!("  P95 Response Time: {:.2}ms", 
                         results.p95_response_time.as_millis().to_string().bright_white());
            }
            
            if results.failed_requests > 0 {
                println!("  {} Errors: {}", 
                         "⚠".yellow(), 
                         results.failed_requests.to_string().bright_white());
            }
        }

        if remaining > Duration::ZERO {
            println!("  Time Remaining: {:?}", remaining.bright_white());
        }

        self.last_report = Instant::now();
    }

    /// Print the final summary when test completes
    pub fn print_final_summary(&self, metrics: &PerformanceMetrics) {
        let results = metrics.calculate_results();
        
        println!();
        println!("{}", "=".repeat(60).dimmed());
        println!("{} Final Performance Results", "🎯".bright_white());
        println!("{}", "=".repeat(60).dimmed());
        
        println!();
        println!("{} Test Summary:", "📋".bright_white());
        println!("  Total Duration: {:?}", results.total_duration.bright_white());
        println!("  Total Requests: {}", results.total_requests.to_string().bright_white());
        println!("  Successful: {}", results.successful_requests.to_string().green());
        println!("  Failed: {}", results.failed_requests.to_string().red());
        println!("  Success Rate: {:.2}%", (results.success_rate * 100.0).to_string().bright_white());
        
        println!();
        println!("{} Performance Metrics:", "⚡".bright_white());
        println!("  Requests/sec: {:.1}", results.requests_per_second.to_string().bright_white());
        println!("  Avg Response: {:.2}ms", results.average_response_time.as_millis().to_string().bright_white());
        println!("  Min Response: {:.2}ms", results.min_response_time.as_millis().to_string().bright_white());
        println!("  Max Response: {:.2}ms", results.max_response_time.as_millis().to_string().bright_white());
        
        println!();
        println!("{} Response Time Percentiles:", "📊".bright_white());
        println!("  P50 (median): {:.2}ms", results.p50_response_time.as_millis().to_string().bright_white());
        println!("  P95: {:.2}ms", results.p95_response_time.as_millis().to_string().bright_white());
        println!("  P99: {:.2}ms", results.p99_response_time.as_millis().to_string().bright_white());
        
        if !results.status_code_distribution.is_empty() {
            println!();
            println!("{} Status Code Distribution:", "🔍".bright_white());
            let mut sorted_codes: Vec<_> = results.status_code_distribution.iter().collect();
            sorted_codes.sort_by_key(|(code, _)| *code);
            
            for (code, count) in sorted_codes {
                let count_str = count.to_string();
                if *code >= 200 && *code < 300 {
                    println!("  {}: {}", code, count_str.green());
                } else if *code >= 400 {
                    println!("  {}: {}", code, count_str.red());
                } else {
                    println!("  {}: {}", code, count_str.yellow());
                }
            }
        }
        
        if results.bytes_per_second_received > 0.0 {
            println!();
            println!("{} Network Traffic:", "🌐".bright_white());
            println!("  Data Sent: {:.2} MB/s", (results.bytes_per_second_sent / 1024.0 / 1024.0).to_string().bright_white());
            println!("  Data Received: {:.2} MB/s", (results.bytes_per_second_received / 1024.0 / 1024.0).to_string().bright_white());
        }
        
        // Performance assessment
        println!();
        self.print_performance_assessment(&results);
        
        println!("{}", "=".repeat(60).dimmed());
    }

    fn print_performance_assessment(&self, results: &crate::performance::PerformanceResults) {
        println!("{} Performance Assessment:", "🔍".bright_white());
        
        let success_rate_percent = results.success_rate * 100.0;
        if success_rate_percent >= 99.0 {
            println!("  Success Rate: {} Excellent (≥99%)", "✅".green());
        } else if success_rate_percent >= 95.0 {
            println!("  Success Rate: {} Good (≥95%)", "✅".green());
        } else if success_rate_percent >= 90.0 {
            println!("  Success Rate: {} Fair (≥90%)", "⚠".yellow());
        } else {
            println!("  Success Rate: {} Poor (<90%)", "❌".red());
        }
        
        let avg_response_ms = results.average_response_time.as_millis();
        if avg_response_ms <= 100 {
            println!("  Avg Response: {} Excellent (≤100ms)", "✅".green());
        } else if avg_response_ms <= 500 {
            println!("  Avg Response: {} Good (≤500ms)", "✅".green());
        } else if avg_response_ms <= 1000 {
            println!("  Avg Response: {} Fair (≤1s)", "⚠".yellow());
        } else {
            println!("  Avg Response: {} Poor (>1s)", "❌".red());
        }
        
        let p95_response_ms = results.p95_response_time.as_millis();
        if p95_response_ms <= 200 {
            println!("  P95 Response: {} Excellent (≤200ms)", "✅".green());
        } else if p95_response_ms <= 1000 {
            println!("  P95 Response: {} Good (≤1s)", "✅".green());
        } else if p95_response_ms <= 2000 {
            println!("  P95 Response: {} Fair (≤2s)", "⚠".yellow());
        } else {
            println!("  P95 Response: {} Poor (>2s)", "❌".red());
        }
        
        if results.requests_per_second >= 100.0 {
            println!("  Throughput: {} High (≥100 RPS)", "✅".green());
        } else if results.requests_per_second >= 50.0 {
            println!("  Throughput: {} Medium (≥50 RPS)", "✅".green());
        } else if results.requests_per_second >= 10.0 {
            println!("  Throughput: {} Low (≥10 RPS)", "⚠".yellow());
        } else {
            println!("  Throughput: {} Very Low (<10 RPS)", "❌".red());
        }
    }

    /// Start a background monitoring task that prints progress reports
    pub async fn start_background_monitoring(
        mut self, 
        metrics: std::sync::Arc<tokio::sync::Mutex<PerformanceMetrics>>,
        target_duration: Duration,
        load_controller: std::sync::Arc<crate::performance::patterns::LoadController>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.report_interval);
            interval.tick().await; // Skip the first tick which fires immediately
            
            loop {
                interval.tick().await;
                
                let metrics_guard = metrics.lock().await;
                self.print_progress_report(&metrics_guard, target_duration, &load_controller);
                
                // Stop monitoring if test duration exceeded
                if self.start_time.elapsed() >= target_duration {
                    break;
                }
            }
        });
    }
}