# Development Guide

This guide covers the development workflow and tools for contributing to rivet-cli.

## Pre-push Hooks (Automatic Setup)

The repository includes a pre-push git hook that automatically runs before every push to ensure code quality:

- **Code formatting check**: Ensures all code is properly formatted with `cargo fmt`
- **Tests**: Runs library tests to catch regressions
- **Clippy warnings**: Shows lint warnings (doesn't block pushes)

The hook is located at `.git/hooks/pre-push` and is executable.

## Development Commands

### Using Make (Recommended)

```bash
# Run all development checks (format, clippy, test, build)
make check

# Individual commands
make fmt      # Format code with cargo fmt
make clippy   # Run clippy linter
make test     # Run all tests
make build    # Build in release mode
make clean    # Clean build artifacts
make install  # Install rivet locally
make help     # Show available commands
```

### Using Scripts

```bash
# Run the same checks as make check
./scripts/check.sh
```

### Manual Cargo Commands

```bash
# Basic development workflow
cargo build                                    # Build the project
cargo test                                     # Run all tests
cargo fmt                                      # Format code
cargo clippy --all-targets --all-features     # Run linter

# Performance testing specific
cargo test --test performance_tests           # Run performance tests only
cargo build --release                         # Release build
```

## Code Quality Standards

- **Formatting**: All code must be formatted with `cargo fmt`
- **Tests**: New features should include tests, all tests must pass
- **Clippy**: Address clippy warnings when possible (currently warnings-only)
- **Documentation**: Update README.md for new features

## Performance Testing Module

The performance testing functionality is implemented in:
- `src/performance/` - Core performance testing modules
- `tests/performance_tests.rs` - Comprehensive test suite
- `src/commands/perf.rs` - CLI command handler

Key features:
- Multiple load patterns (constant, ramp-up, spike)
- Real-time monitoring and progress reporting
- JSON report generation for CI/CD
- Comprehensive metrics collection

## Git Workflow

1. Make your changes
2. Run `make check` to verify everything passes
3. Commit your changes
4. Push - the pre-push hook will run automatically
5. If the hook fails, fix the issues and try again

The pre-push hook ensures that:
- Code is properly formatted (`cargo fmt --check`)
- Library tests pass (`cargo test --lib`)
- Build succeeds

This prevents common issues and keeps the main branch clean.