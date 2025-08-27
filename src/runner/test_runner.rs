use crate::config::{RivetConfig, TestStep};
use crate::runner::{
    data::load_csv_data,
    executor::{RequestExecutor, TestResult},
    parser::load_test_suite,
    variables::VariableContext,
};
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct TestRunner {
    executor: RequestExecutor,
    parallel_workers: usize,
    bail_on_failure: bool,
    filter_pattern: Option<String>,
    ci_mode: bool,
}

pub struct TestSuiteResult {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub results: Vec<TestResult>,
    pub duration: Duration,
    pub passed: usize,
    pub failed: usize,
}

impl TestRunner {
    pub fn new(
        timeout: Duration,
        parallel_workers: usize,
        bail_on_failure: bool,
        filter_pattern: Option<String>,
        ci_mode: bool,
    ) -> Result<Self> {
        let executor = RequestExecutor::new(timeout)?;
        
        Ok(Self {
            executor,
            parallel_workers,
            bail_on_failure,
            filter_pattern,
            ci_mode,
        })
    }
    
    pub async fn run_tests(&self, target: &Path, env: Option<&str>) -> Result<Vec<TestSuiteResult>> {
        let test_suites = load_test_suite(target).await?;
        
        if test_suites.len() <= 1 || self.parallel_workers <= 1 {
            // Sequential execution for single suite or when parallel is disabled
            self.run_suites_sequential(test_suites, env).await
        } else {
            // Parallel execution for multiple suites
            self.run_suites_parallel(test_suites, env).await
        }
    }
    
    async fn run_suites_sequential(&self, test_suites: Vec<(String, crate::config::RivetConfig)>, env: Option<&str>) -> Result<Vec<TestSuiteResult>> {
        let mut all_results = Vec::new();
        
        for (suite_name, config) in test_suites {
            if self.ci_mode {
                println!("RUN {}", suite_name);
            } else {
                println!("\n{} {}", "RUN".cyan().bold(), suite_name.bright_white());
            }
            
            let suite_start = Instant::now();
            let results = self.run_single_suite(&config, env).await?;
            let duration = suite_start.elapsed();
            
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = results.len() - passed;
            
            self.print_suite_summary(&suite_name, passed, failed, duration);
            
            all_results.push(TestSuiteResult {
                name: suite_name,
                results,
                duration,
                passed,
                failed,
            });
            
            if self.bail_on_failure && failed > 0 {
                break;
            }
        }
        
        Ok(all_results)
    }
    
    async fn run_suites_parallel(&self, test_suites: Vec<(String, crate::config::RivetConfig)>, env: Option<&str>) -> Result<Vec<TestSuiteResult>> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use std::sync::Arc;
        
        let env = Arc::new(env);
        let mut all_results = Vec::new();
        
        // Process suites in chunks to limit concurrency
        for chunk in test_suites.chunks(self.parallel_workers) {
            let mut futures = FuturesUnordered::new();
            
            for (suite_name, config) in chunk {
                let suite_name = suite_name.clone();
                let config = config.clone();
                let env = Arc::clone(&env);
                let executor = self.executor.clone();
                let ci_mode = self.ci_mode;
                let bail_on_failure = self.bail_on_failure;
                let filter_pattern = self.filter_pattern.clone();
                
                // Announce start
                if ci_mode {
                    println!("RUN {}", suite_name);
                } else {
                    println!("\n{} {}", "RUN".cyan().bold(), suite_name.bright_white());
                }
                
                futures.push(async move {
                    let suite_start = Instant::now();
                    
                    // Create a temporary runner for this suite
                    let temp_runner = TestRunner {
                        executor,
                        parallel_workers: 1, // Use sequential within each suite for parallel suite execution
                        bail_on_failure,
                        filter_pattern,
                        ci_mode,
                    };
                    
                    let results = temp_runner.run_single_suite(&config, env.as_deref()).await;
                    let duration = suite_start.elapsed();
                    
                    (suite_name, results, duration)
                });
            }
            
            // Collect results from this chunk
            while let Some((suite_name, results, duration)) = futures.next().await {
                match results {
                    Ok(results) => {
                        let passed = results.iter().filter(|r| r.passed).count();
                        let failed = results.len() - passed;
                        
                        self.print_suite_summary(&suite_name, passed, failed, duration);
                        
                        all_results.push(TestSuiteResult {
                            name: suite_name,
                            results,
                            duration,
                            passed,
                            failed,
                        });
                        
                        if self.bail_on_failure && failed > 0 {
                            return Ok(all_results);
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(all_results)
    }
    
    fn print_suite_summary(&self, _suite_name: &str, passed: usize, failed: usize, duration: Duration) {
        if failed == 0 {
            if self.ci_mode {
                println!("  PASS {} tests in {:?}", passed, duration);
            } else {
                println!("  {} {} tests passed in {:?}", 
                    "✔".green().bold(), 
                    passed, 
                    duration
                );
            }
        } else {
            if self.ci_mode {
                println!("  FAIL {} passed, {} failed in {:?}", passed, failed, duration);
            } else {
                println!("  {} {} passed, {} failed in {:?}", 
                    if failed > 0 { "✖".red().bold().to_string() } else { "✔".green().bold().to_string() },
                    passed, 
                    failed, 
                    duration
                );
            }
        }
    }
    
    async fn run_single_suite(&self, config: &RivetConfig, env: Option<&str>) -> Result<Vec<TestResult>> {
        // Create variable context
        let mut context = VariableContext::new()
            .with_env_vars()
            .with_config_vars(config.vars.as_ref());
        
        if let Some(env_name) = env {
            context.set("RIVET_ENV".to_string(), env_name.to_string());
        }
        
        let mut all_results = Vec::new();
        
        // Run setup steps
        if let Some(setup_steps) = &config.setup {
            for step in setup_steps {
                if self.should_run_test(&step.name) {
                    let result = self.executor.execute_test(
                        &format!("Setup: {}", step.name),
                        &step.request,
                        step.expect.as_ref(),
                        &context,
                    ).await;
                    
                    self.print_test_result(&result);
                    all_results.push(result);
                }
            }
        }
        
        // Run main tests
        if let Some(dataset) = &config.dataset {
            // Data-driven testing
            let data_file = PathBuf::from(&dataset.file);
            let data_rows = load_csv_data(&data_file).await
                .with_context(|| format!("Failed to load dataset: {}", dataset.file))?;
            
            let parallel = dataset.parallel.unwrap_or(self.parallel_workers);
            
            for data_row in data_rows {
                let row_context = context.clone().with_data_row(&data_row);
                let test_results = self.run_test_steps(&config.tests, &row_context, parallel).await;
                all_results.extend(test_results);
            }
        } else {
            // Regular testing
            let test_results = self.run_test_steps(&config.tests, &context, self.parallel_workers).await;
            all_results.extend(test_results);
        }
        
        // Run teardown steps
        if let Some(teardown_steps) = &config.teardown {
            for step in teardown_steps {
                if self.should_run_test(&step.name) {
                    let result = self.executor.execute_test(
                        &format!("Teardown: {}", step.name),
                        &step.request,
                        step.expect.as_ref(),
                        &context,
                    ).await;
                    
                    self.print_test_result(&result);
                    all_results.push(result);
                }
            }
        }
        
        Ok(all_results)
    }
    
    async fn run_test_steps(
        &self,
        steps: &[TestStep],
        context: &VariableContext,
        parallel: usize,
    ) -> Vec<TestResult> {
        let filtered_steps: Vec<_> = steps
            .iter()
            .filter(|step| self.should_run_test(&step.name))
            .collect();
        
        if parallel <= 1 {
            // Sequential execution
            let mut results = Vec::new();
            for step in filtered_steps {
                let result = self.executor.execute_test(
                    &step.name,
                    &step.request,
                    step.expect.as_ref(),
                    context,
                ).await;
                
                self.print_test_result(&result);
                let passed = result.passed;
                results.push(result);
                
                if self.bail_on_failure && !passed {
                    break;
                }
            }
            results
        } else {
            // Parallel execution
            use futures::stream::{FuturesUnordered, StreamExt};
            use std::sync::Arc;
            
            let executor = Arc::new(&self.executor);
            let context = Arc::new(context);
            let mut results = Vec::new();
            
            // Process in chunks to limit concurrency
            for chunk in filtered_steps.chunks(parallel) {
                let mut futures = FuturesUnordered::new();
                
                for step in chunk {
                    let executor = Arc::clone(&executor);
                    let context = Arc::clone(&context);
                    let step_name = step.name.clone();
                    let step_request = step.request.clone();
                    let step_expect = step.expect.clone();
                    
                    futures.push(async move {
                        executor.execute_test(
                            &step_name,
                            &step_request,
                            step_expect.as_ref(),
                            &context,
                        ).await
                    });
                }
                
                // Collect results from this chunk
                while let Some(result) = futures.next().await {
                    self.print_test_result(&result);
                    let passed = result.passed;
                    results.push(result);
                    
                    if self.bail_on_failure && !passed {
                        return results;
                    }
                }
            }
            
            results
        }
    }
    
    fn should_run_test(&self, test_name: &str) -> bool {
        if let Some(pattern) = &self.filter_pattern {
            test_name.contains(pattern)
        } else {
            true
        }
    }
    
    fn print_test_result(&self, result: &TestResult) {
        if self.ci_mode {
            // CI mode: plain text, no colors or fancy symbols
            if result.passed {
                println!("  PASS {} ({:?})", result.name, result.duration);
            } else {
                println!("  FAIL {} ({:?})", result.name, result.duration);
                if let Some(error) = &result.error {
                    println!("    Error: {}", error);
                }
            }
        } else {
            // Interactive mode: colors and symbols
            if result.passed {
                println!("  {} {} ({:?})", 
                    "✔".green(), 
                    result.name, 
                    result.duration
                );
            } else {
                println!("  {} {} ({:?})", 
                    "✖".red(), 
                    result.name, 
                    result.duration
                );
                
                if let Some(error) = &result.error {
                    println!("    {}: {}", "Error".red().bold(), error);
                }
            }
        }
    }
}

// Implement Clone for VariableContext in the variables module instead