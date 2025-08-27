use anyhow::Result;
use crate::config::UserConfig;
use crate::report::ReportGenerator;
use crate::runner::TestRunner;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::time::Duration;

fn open_in_browser(file_path: &PathBuf) -> Result<()> {
    let path = file_path.canonicalize()?;
    let url = format!("file://{}", path.display());
    
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&url).spawn()?;
    
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&url).spawn()?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", &url])
        .spawn()?;
    
    Ok(())
}

pub async fn handle_run(
    target: PathBuf,
    env: Option<String>,
    _data: Option<PathBuf>,
    parallel: usize,
    grep: Option<String>,
    bail: bool,
    report: Option<String>,
    template: Option<String>,
    open: bool,
    no_open: bool,
    ci: bool,
) -> Result<()> {
    // Load user config
    let user_config = UserConfig::load().unwrap_or_default();
    
    // Determine final template to use (CLI overrides config)
    let final_template = template.as_ref().unwrap_or(&user_config.reports.default_template);
    
    // Determine final auto-open behavior (CLI flags override config)
    let should_open = if open {
        true  // Explicit --open flag
    } else if no_open {
        false // Explicit --no-open flag
    } else {
        user_config.reports.auto_open_browser // Use config default
    };
    
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
    
    // Generate reports if requested
    if let Some(report_formats) = &report {
        let reports_dir = PathBuf::from("./reports");
        match ReportGenerator::generate_reports(&results, report_formats, &reports_dir, final_template) {
            Ok(generated_files) => {
                println!();
                println!("Reports generated:");
                for file in &generated_files {
                    println!("  📊 {}", file.display());
                    
                    // Auto-open HTML reports based on config/flags
                    if should_open && file.extension().map_or(false, |ext| ext == "html") {
                        if let Err(e) = open_in_browser(file) {
                            eprintln!("Warning: Failed to open browser: {}", e);
                        } else {
                            println!("  🌐 Opened in browser");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to generate reports: {}", e);
            }
        }
    }
    
    // Print overall summary
    let total_passed: usize = results.iter().map(|r| r.passed).sum();
    let total_failed: usize = results.iter().map(|r| r.failed).sum();
    let total_tests = total_passed + total_failed;
    let total_duration: Duration = results.iter().map(|r| r.duration).sum();
    
    println!();
    if total_failed == 0 {
        if ci {
            println!("PASS {} tests in {:?}", total_tests, total_duration);
        } else {
            println!("{} {} tests passed in {:?}", 
                "✔".green().bold(), 
                total_tests, 
                total_duration
            );
            
            if total_tests > 0 {
                // Add some celebration for successful runs (only in interactive mode)
                println!("      .       .  *     .     *");
                println!("   *    .   *   .  *      .        *");
            }
        }
    } else {
        if ci {
            println!("FAIL {} passed, {} failed in {:?}", total_passed, total_failed, total_duration);
        } else {
            println!("{} {} passed, {} failed in {:?}", 
                "✖".red().bold(),
                total_passed, 
                total_failed, 
                total_duration
            );
        }
        
        // Exit with error code if any tests failed
        std::process::exit(1);
    }
    
    Ok(())
}