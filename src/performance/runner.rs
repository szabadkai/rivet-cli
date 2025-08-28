use crate::config::RivetConfig;
use crate::performance::monitor::PerformanceMonitor;
use crate::performance::patterns::LoadController;
use crate::performance::{LoadPattern, PerformanceMetrics, PerformanceResults};
use crate::runner::executor::RequestExecutor;
use crate::runner::parser::load_test_suite;
use crate::runner::variables::VariableContext;
use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct PerformanceTestRunner {
    concurrent_users: u32,
    target_rps: Option<u32>,
    test_duration: Duration,
    warmup_duration: Duration,
    report_interval: Duration,
    load_pattern: LoadPattern,
    executor: RequestExecutor,
}

impl PerformanceTestRunner {
    pub fn new(
        concurrent_users: u32,
        target_rps: Option<u32>,
        test_duration: Duration,
        warmup_duration: Duration,
        report_interval: Duration,
        load_pattern: LoadPattern,
    ) -> Result<Self> {
        // Use a longer timeout for performance tests to avoid timeouts under load
        let timeout = Duration::from_secs(60);
        let executor = RequestExecutor::new(timeout)?;

        Ok(Self {
            concurrent_users,
            target_rps,
            test_duration,
            warmup_duration,
            report_interval,
            load_pattern,
            executor,
        })
    }

    pub async fn run_performance_test(
        &self,
        target: &Path,
        env: Option<&str>,
    ) -> Result<PerformanceResults> {
        // Load test suites
        let test_suites = load_test_suite(target)
            .await
            .context("Failed to load test suite for performance testing")?;

        if test_suites.is_empty() {
            anyhow::bail!("No test suites found in target path");
        }

        // For performance testing, we'll focus on the first test suite
        let (suite_name, config) = &test_suites[0];

        if config.tests.is_empty() {
            anyhow::bail!("Test suite '{}' contains no tests", suite_name);
        }

        println!("🚀 Starting performance test on suite: {}", suite_name);
        println!("   Tests to execute: {}", config.tests.len());
        println!("   Concurrent users: {}", self.concurrent_users);
        if let Some(rps) = self.target_rps {
            println!("   Target RPS: {}", rps);
        }
        println!("   Test duration: {:?}", self.test_duration);
        println!("   Load pattern: {:?}", self.load_pattern);

        // Setup shared metrics and load controller
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
        let load_controller = Arc::new(LoadController::new(
            self.load_pattern.clone(),
            self.target_rps,
            self.concurrent_users,
            self.warmup_duration,
        ));

        // Setup monitoring
        let monitor = PerformanceMonitor::new(self.report_interval, self.load_pattern.clone());
        monitor
            .start_background_monitoring(
                Arc::clone(&metrics),
                self.test_duration,
                Arc::clone(&load_controller),
            )
            .await;

        // Warmup phase
        if self.warmup_duration > Duration::ZERO {
            println!("\n⏳ Warming up for {:?}...", self.warmup_duration);
            sleep(self.warmup_duration).await;
        }

        println!("\n🔥 Starting load generation...");

        // Start the performance test
        let test_start = Instant::now();

        // Run the load generation
        self.generate_load(
            config,
            env,
            Arc::clone(&metrics),
            Arc::clone(&load_controller),
            test_start,
        )
        .await?;

        // Wait for any remaining requests to complete (with timeout)
        sleep(Duration::from_secs(2)).await;

        // Generate final results
        let final_metrics = metrics.lock().await;
        let results = final_metrics.calculate_results();

        // Print final summary
        let monitor = PerformanceMonitor::new(self.report_interval, self.load_pattern.clone());
        monitor.print_final_summary(&final_metrics);

        Ok(results)
    }

    async fn generate_load(
        &self,
        config: &RivetConfig,
        env: Option<&str>,
        metrics: Arc<Mutex<PerformanceMetrics>>,
        load_controller: Arc<LoadController>,
        test_start: Instant,
    ) -> Result<()> {
        let mut futures = FuturesUnordered::new();
        let total_duration = self.test_duration;

        // Spawn worker tasks
        for worker_id in 0..self.concurrent_users {
            let config = config.clone();
            let env = env.map(|s| s.to_string());
            let executor = self.executor.clone();
            let metrics = Arc::clone(&metrics);
            let load_controller = Arc::clone(&load_controller);

            futures.push(tokio::spawn(async move {
                Self::worker_task(
                    worker_id,
                    config,
                    env.as_deref(),
                    executor,
                    metrics,
                    load_controller,
                    test_start,
                    total_duration,
                )
                .await
            }));
        }

        // Wait for all workers to complete or timeout
        let timeout_future = sleep(self.test_duration);
        tokio::pin!(timeout_future);

        loop {
            tokio::select! {
                _ = &mut timeout_future => {
                    println!("\n⏱️  Test duration reached, stopping load generation...");
                    break;
                }
                result = futures.next() => {
                    match result {
                        Some(Ok(Ok(()))) => {
                            // Worker completed successfully
                        }
                        Some(Ok(Err(e))) => {
                            println!("⚠️  Worker error: {}", e);
                        }
                        Some(Err(e)) => {
                            println!("⚠️  Worker task failed: {}", e);
                        }
                        None => {
                            // All workers completed
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn worker_task(
        worker_id: u32,
        config: RivetConfig,
        env: Option<&str>,
        executor: RequestExecutor,
        metrics: Arc<Mutex<PerformanceMetrics>>,
        load_controller: Arc<LoadController>,
        _test_start: Instant,
        total_duration: Duration,
    ) -> Result<()> {
        // Create variable context for this worker
        let mut context = VariableContext::new();

        // Load environment variables if specified
        if let Some(_env_name) = env {
            if let Some(env_vars) = config.vars.as_ref() {
                for (key, value) in env_vars {
                    context.set_variable(key.clone(), value.clone());
                }
            }
        }

        let test_count = config.tests.len();
        let mut current_test_index = 0;

        // Worker runs for the specified duration
        let worker_start = Instant::now();

        while worker_start.elapsed() < total_duration {
            // Get current test to execute (round-robin)
            let test_step = &config.tests[current_test_index];
            current_test_index = (current_test_index + 1) % test_count;

            let _request_start = Instant::now();

            // Execute the request
            let test_result = executor
                .execute_test(
                    &format!("worker_{}_test_{}", worker_id, current_test_index),
                    &test_step.request,
                    test_step.expect.as_ref(),
                    &context,
                )
                .await;

            let response_time = test_result.duration;
            let is_error = !test_result.passed;
            let status_code = test_result.response_status.unwrap_or(0);

            // Estimate request/response size (simplified)
            let bytes_sent = test_step
                .request
                .body
                .as_ref()
                .map(|b| b.len() as u64)
                .unwrap_or(100); // Estimate header size
            let bytes_received = test_result
                .response_body
                .as_ref()
                .map(|b| b.len() as u64)
                .unwrap_or(0);

            // Record metrics
            {
                let mut metrics_guard = metrics.lock().await;
                if is_error && status_code == 0 {
                    // Connection error
                    metrics_guard.record_connection_error();
                } else {
                    metrics_guard.record_request(
                        response_time,
                        status_code,
                        bytes_sent,
                        bytes_received,
                        is_error,
                    );
                }
            }

            // Apply rate limiting if configured
            if let Some(delay) = load_controller.request_delay() {
                sleep(delay).await;
            }

            // Small delay between requests to prevent overwhelming
            sleep(Duration::from_millis(1)).await;
        }

        Ok(())
    }
}
