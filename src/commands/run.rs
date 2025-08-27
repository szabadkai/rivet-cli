use crate::config::UserConfig;
use crate::report::ReportGenerator;
use crate::runner::TestRunner;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn open_in_browser(file_path: &Path) -> Result<()> {
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

pub struct RunOptions {
    pub target: PathBuf,
    pub env: Option<String>,
    pub _data: Option<PathBuf>,
    pub parallel: usize,
    pub grep: Option<String>,
    pub bail: bool,
    pub report: Option<String>,
    pub template: Option<String>,
    pub open: bool,
    pub no_open: bool,
    pub ci: bool,
}

pub async fn handle_run(options: RunOptions) -> Result<()> {
    // Load user config
    let user_config = UserConfig::load().unwrap_or_default();

    // Determine final template to use (CLI overrides config)
    let final_template = options
        .template
        .as_ref()
        .unwrap_or(&user_config.reports.default_template);

    // Determine final auto-open behavior (CLI flags override config)
    let should_open = if options.open {
        true // Explicit --open flag
    } else if options.no_open {
        false // Explicit --no-open flag
    } else {
        user_config.reports.auto_open_browser // Use config default
    };

    println!("Running tests from: {}", options.target.display());
    println!(
        "Environment: {}",
        options.env.as_deref().unwrap_or("default")
    );
    println!("Parallel workers: {}", options.parallel);

    if let Some(pattern) = &options.grep {
        println!("Filter pattern: {}", pattern);
    }

    // Create test runner
    let timeout = Duration::from_secs(30); // Default timeout
    let runner = TestRunner::new(
        timeout,
        options.parallel,
        options.bail,
        options.grep,
        options.ci,
    )?;

    // Run tests
    let results = runner
        .run_tests(&options.target, options.env.as_deref())
        .await?;

    // Generate reports if requested
    if let Some(report_formats) = &options.report {
        let reports_dir = PathBuf::from("./reports");
        match ReportGenerator::generate_reports(
            &results,
            report_formats,
            &reports_dir,
            final_template,
        ) {
            Ok(generated_files) => {
                println!();
                println!("Reports generated:");
                for file in &generated_files {
                    println!("  📊 {}", file.display());

                    // Auto-open HTML reports based on config/flags
                    if should_open && file.extension().is_some_and(|ext| ext == "html") {
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
        if options.ci {
            println!("PASS {} tests in {:?}", total_tests, total_duration);
        } else {
            println!(
                "{} {} tests passed in {:?}",
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
        if options.ci {
            println!(
                "FAIL {} passed, {} failed in {:?}",
                total_passed, total_failed, total_duration
            );
        } else {
            println!(
                "{} {} passed, {} failed in {:?}",
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
