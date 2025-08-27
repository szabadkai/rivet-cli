use anyhow::Result;
use crate::runner::TestRunner;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Duration;

pub async fn handle_run(
    target: PathBuf,
    env: Option<String>,
    _data: Option<PathBuf>,
    parallel: usize,
    grep: Option<String>,
    bail: bool,
    _report: Option<String>,
    ci: bool,
) -> Result<()> {
    println!("Running tests from: {}", target.display());
    println!("Environment: {}", env.as_deref().unwrap_or("default"));
    println!("Parallel workers: {}", parallel);
    
    if let Some(pattern) = &grep {
        println!("Filter pattern: {}", pattern);
    }
    
    // Create test runner
    let timeout = Duration::from_secs(30); // Default timeout
    let runner = TestRunner::new(
        timeout,
        parallel,
        bail,
        grep,
        ci,
    )?;
    
    // Run tests
    let results = runner.run_tests(&target, env.as_deref()).await?;
    
    // Print overall summary
    let total_passed: usize = results.iter().map(|r| r.passed).sum();
    let total_failed: usize = results.iter().map(|r| r.failed).sum();
    let total_tests = total_passed + total_failed;
    let total_duration: Duration = results.iter().map(|r| r.duration).sum();
    
    println!();
    if total_failed == 0 {
        println!("{} {} tests passed in {:?}", 
            "✔".green().bold(), 
            total_tests, 
            total_duration
        );
        
        if !ci && total_tests > 0 {
            // Add some celebration for successful runs
            println!("      .       .  *     .     *");
            println!("   *    .   *   .  *      .        *");
        }
        
        // Don't show banner at end anymore
    } else {
        println!("{} {} passed, {} failed in {:?}", 
            "✖".red().bold(),
            total_passed, 
            total_failed, 
            total_duration
        );
        
        // Don't show banner at end anymore
        
        // Exit with error code if any tests failed
        std::process::exit(1);
    }
    
    Ok(())
}