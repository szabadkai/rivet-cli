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
        _ci_mode: bool,
    ) -> Result<Self> {
        let executor = RequestExecutor::new(timeout)?;
        
        Ok(Self {
            executor,
            parallel_workers,
            bail_on_failure,
            filter_pattern,
        })
    }
    
    pub async fn run_tests(&self, target: &Path, env: Option<&str>) -> Result<Vec<TestSuiteResult>> {
        let test_suites = load_test_suite(target).await?;
        let mut all_results = Vec::new();
        
        for (suite_name, config) in test_suites {
            println!("\n{} {}", "RUN".cyan().bold(), suite_name.bright_white());
            
            let suite_start = Instant::now();
            let results = self.run_single_suite(&config, env).await?;
            let duration = suite_start.elapsed();
            
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = results.len() - passed;
            
            // Print summary
            if failed == 0 {
                println!("  {} {} tests passed in {:?}", 
                    "✔".green().bold(), 
                    passed, 
                    duration
                );
            } else {
                println!("  {} {} passed, {} failed in {:?}", 
                    if failed > 0 { "✖".red().bold().to_string() } else { "✔".green().bold().to_string() },
                    passed, 
                    failed, 
                    duration
                );
            }
            
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
            // For now, use sequential execution even when parallel is requested
            // TODO: Implement proper parallel execution
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

// Implement Clone for VariableContext in the variables module instead