use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use std::path::PathBuf;

mod commands;
mod config;
mod http;
mod grpc;
mod report;
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
}

fn print_banner() {
    let banner = r#"
██████╗ ██╗██╗   ██╗███████╗████████╗
██╔══██╗██║╚██╗ ██╔╝██╔════╝╚══██╔══╝   rivet v0.1.0
██████╔╝██║ ╚████╔╝ ███████╗   ██║      API testing that lives in git.
██╔═══╝ ██║  ╚██╔╝  ╚════██║   ██║
██║     ██║   ██║   ███████║   ██║      https://rivet.dev
╚═╝     ╚═╝   ╚═╝   ╚══════╝   ╚═╝
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
    
    // Print banner for most commands
    if !matches!(cli.command, Commands::Send { .. }) {
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
            ci,
        } => {
            run::handle_run(target, env, data, parallel, grep, bail, report, ci).await?;
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
            proto,
            call,
            data,
            expect_jsonpath,
            timeout,
        } => {
            commands::grpc::handle_grpc(proto, call, data, expect_jsonpath, timeout).await?;
        }
    }

    Ok(())
}