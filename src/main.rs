use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell as CompShell};
use owo_colors::OwoColorize;
use std::path::PathBuf;

mod commands;
mod config;
mod grpc;
mod performance;
mod report;
mod runner;
mod ui;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "rivet")]
#[command(version = "0.1.0")]
#[command(about = "API testing that lives in git")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a single HTTP request
    Send {
        /// HTTP method (GET, POST, etc.)
        method: String,
        /// URL to request
        url: String,
        /// Headers in format "Key: Value"
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// Request body data
        #[arg(short = 'd', long = "data")]
        data: Option<String>,
        /// Save request to file
        #[arg(long = "save")]
        save: Option<PathBuf>,
        /// Skip SSL verification
        #[arg(long = "insecure")]
        insecure: bool,
        /// Request timeout
        #[arg(long = "timeout", default_value = "30s")]
        timeout: String,
    },
    /// Run test suites
    Run {
        /// File or directory to run
        target: PathBuf,
        /// Environment to use
        #[arg(long = "env")]
        env: Option<String>,
        /// Data file for data-driven tests
        #[arg(long = "data")]
        data: Option<PathBuf>,
        /// Number of parallel workers
        #[arg(long = "parallel", default_value = "1")]
        parallel: usize,
        /// Filter tests by name pattern
        #[arg(long = "grep")]
        grep: Option<String>,
        /// Stop on first failure
        #[arg(long = "bail")]
        bail: bool,
        /// Report formats (comma-separated)
        #[arg(long = "report")]
        report: Option<String>,
        /// HTML template to use (detailed, simple, chatty, compact)
        #[arg(long = "template")]
        template: Option<String>,
        /// Auto-open HTML report in browser
        #[arg(long = "open", conflicts_with = "no_open")]
        open: bool,
        /// Disable auto-opening HTML report in browser
        #[arg(long = "no-open", conflicts_with = "open")]
        no_open: bool,
        /// CI mode (no animations)
        #[arg(long = "ci")]
        ci: bool,
    },
    /// Generate test files from OpenAPI spec
    Gen {
        /// OpenAPI specification file
        #[arg(long = "spec")]
        spec: PathBuf,
        /// Output directory
        #[arg(long = "out", default_value = "tests/")]
        out: PathBuf,
    },
    /// Generate coverage report
    Coverage {
        /// OpenAPI specification file
        #[arg(long = "spec")]
        spec: PathBuf,
        /// Report files to analyze
        #[arg(long = "from")]
        from: Vec<PathBuf>,
        /// Output file for coverage report
        #[arg(long = "out")]
        out: Option<PathBuf>,
    },
    /// Import from other tools
    Import {
        /// Tool to import from (postman, insomnia, bruno, curl)
        tool: String,
        /// File to import
        file: PathBuf,
        /// Output directory
        #[arg(long = "out", default_value = "tests/")]
        out: PathBuf,
    },
    /// Make gRPC calls
    Grpc {
        /// gRPC server address (e.g., http://localhost:50051)
        #[arg(long = "server")]
        server: String,
        /// Proto files directory
        #[arg(long = "proto")]
        proto: PathBuf,
        /// Service call (e.g., svc.Users/GetUser)
        #[arg(long = "call")]
        call: String,
        /// Request data (JSON)
        #[arg(long = "data")]
        data: Option<String>,
        /// JSONPath expectations
        #[arg(long = "expect-jsonpath")]
        expect_jsonpath: Vec<String>,
        /// Request timeout
        #[arg(long = "timeout", default_value = "30s")]
        timeout: String,
    },
    /// Run performance tests
    Perf {
        /// File or directory to performance test
        target: PathBuf,
        /// Test duration (e.g., "30s", "5m")
        #[arg(long = "duration", default_value = "30s")]
        duration: String,
        /// Target requests per second
        #[arg(long = "rps")]
        rps: Option<u32>,
        /// Number of concurrent connections
        #[arg(long = "concurrent", default_value = "10")]
        concurrent: u32,
        /// Warmup period before measuring
        #[arg(long = "warmup", default_value = "5s")]
        warmup: String,
        /// Report interval during test
        #[arg(long = "report-interval", default_value = "5s")]
        report_interval: String,
        /// Performance report output file
        #[arg(long = "output")]
        output: Option<PathBuf>,
        /// Load pattern (constant, ramp-up, spike)
        #[arg(long = "pattern", default_value = "constant")]
        pattern: String,
        /// Environment to use
        #[arg(long = "env")]
        env: Option<String>,
    },
    /// Generate shell completions (internal)
    #[command(hide = true)]
    Completions {
        /// Shell: bash, zsh, fish
        shell: String,
    },
    /// Generate man page (internal)
    #[command(hide = true)]
    Man,
}

pub fn print_banner() {
    let banner = r#"
    ██████╗ ██╗██╗   ██╗███████╗████████╗
    ██╔══██╗██║██║   ██║██╔════╝╚══██╔══╝   rivet v0.1.0
    ██████╔╝██║██║   ██║█████╗     ██║      API testing that lives in git
    ██╔══██╗██║╚██╗ ██╔╝██╔══╝     ██║      
    ██║  ██║██║ ╚████╔╝ ███████╗   ██║      https://github.com/szabadkai/rivet-cli
    ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝   ╚═╝
"#;

    if atty::is(atty::Stream::Stdout) {
        println!("{}", banner.cyan());
    } else {
        println!("rivet v0.1.0 — API testing that lives in git");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Print banner for user-facing commands only
    if !matches!(
        cli.command,
        Commands::Send { .. } | Commands::Completions { .. } | Commands::Man
    ) {
        print_banner();
    }

    match cli.command {
        Commands::Send {
            method,
            url,
            headers,
            data,
            save,
            insecure,
            timeout,
        } => {
            send::handle_send(method, url, headers, data, save, insecure, timeout).await?;
        }
        Commands::Run {
            target,
            env,
            data,
            parallel,
            grep,
            bail,
            report,
            template,
            open,
            no_open,
            ci,
        } => {
            run::handle_run(run::RunOptions {
                target,
                env,
                _data: data,
                parallel,
                grep,
                bail,
                report,
                template,
                open,
                no_open,
                ci,
            })
            .await?;
        }
        Commands::Gen { spec, out } => {
            gen::handle_gen(spec, out).await?;
        }
        Commands::Coverage { spec, from, out } => {
            coverage::handle_coverage(spec, from, out).await?;
        }
        Commands::Import { tool, file, out } => {
            import::handle_import(tool, file, out).await?;
        }
        Commands::Grpc {
            server,
            proto,
            call,
            data,
            expect_jsonpath,
            timeout,
        } => {
            commands::grpc::handle_grpc(server, proto, call, data, expect_jsonpath, timeout)
                .await?;
        }
        Commands::Perf {
            target,
            duration,
            rps,
            concurrent,
            warmup,
            report_interval,
            output,
            pattern,
            env,
        } => {
            commands::perf::handle_perf(commands::perf::PerfOptions {
                target,
                duration,
                rps,
                concurrent,
                warmup,
                report_interval,
                output,
                pattern,
                env,
            })
            .await?;
        }
        Commands::Completions { shell } => {
            // Generate completions to stdout for the requested shell
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let sh = match shell.as_str() {
                "bash" => CompShell::Bash,
                "zsh" => CompShell::Zsh,
                "fish" => CompShell::Fish,
                "powershell" | "pwsh" => CompShell::PowerShell,
                "elvish" => CompShell::Elvish,
                other => {
                    eprintln!(
                        "Unsupported shell: {} (use bash|zsh|fish|powershell|elvish)",
                        other
                    );
                    std::process::exit(2);
                }
            };
            generate(sh, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Man => {
            // Generate a man page to stdout using clap_mangen
            let cmd = Cli::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut std::io::stdout())?;
        }
    }

    Ok(())
}
