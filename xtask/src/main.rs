use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn repo_root() -> PathBuf {
    // xtask's manifest dir is `<repo>/xtask` — go up one level.
    let xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.parent().unwrap().to_path_buf()
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let mut c = Command::new(cmd);
    c.args(args).current_dir(repo_root());
    let status = c
        .status()
        .map_err(|e| format!("failed to spawn {cmd}: {e}"))?;
    if !status.success() {
        return Err(format!("command failed: {} {}", cmd, args.join(" ")));
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "xtask usage:\n  cargo xtask ci\n  cargo xtask install-hooks\n  cargo xtask dev-setup\n  cargo xtask release <patch|minor|major>\n  cargo xtask help"
    );
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1); // skip program name
    let cmd = args.next().unwrap_or_else(|| "help".to_string());

    let res = match cmd.as_str() {
        "ci" => task_ci(),
        "install-hooks" => task_install_hooks(),
        "dev-setup" => task_dev_setup(),
        "release" => {
            let level = args.next().unwrap_or_else(|| "patch".to_string());
            task_release(&level)
        }
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("unknown subcommand: {other}\n");
            print_usage();
            Err("unknown subcommand".into())
        }
    };

    match res {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn task_ci() -> Result<(), String> {
    println!("🤖 Running CI checks (fmt, clippy, test, build)...");
    run("cargo", &["fmt", "--", "--check"]) ?;
    run("cargo", &["clippy", "--", "-D", "warnings"]) ?;
    run("cargo", &["test"]) ?;
    run("cargo", &["build"]) ?;
    println!("✅ CI checks passed");
    Ok(())
}

fn task_install_hooks() -> Result<(), String> {
    println!("🔧 Installing git hooks...");
    run("bash", &["scripts/install-git-hooks.sh"]) ?;
    println!("✅ Hooks installed");
    Ok(())
}

fn task_dev_setup() -> Result<(), String> {
    println!("🛠️  Setting up development environment...");
    run("bash", &["scripts/install-git-hooks.sh"]) ?;
    println!("✅ Development environment ready!");
    Ok(())
}

fn task_release(level: &str) -> Result<(), String> {
    match level {
        "patch" | "minor" | "major" => {
            println!("🚀 Starting {level} release via scripts/release.sh...");
            run("bash", &["scripts/release.sh", level]) ?;
            Ok(())
        }
        _ => {
            eprintln!("invalid release level: {level}");
            print_usage();
            Err("invalid release level".into())
        }
    }
}
