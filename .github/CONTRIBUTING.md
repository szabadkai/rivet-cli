# Contributing to Rivet CLI

Thank you for your interest in contributing to rivet-cli!

## Development Setup

The repository is set up with automatic code quality checks:

### Pre-push Hooks (Automatic)

A pre-push git hook is already configured that will:
- ✅ Check code formatting with `cargo fmt --check`
- ✅ Run library tests to ensure nothing is broken
- ⚠️ Show clippy warnings (informational only)

This means **you can't push unformatted code or failing tests** - the hook will prevent it automatically.

### Development Commands

```bash
# Run all checks (recommended before committing)
make check

# Individual tasks
make fmt      # Format code
make clippy   # Show linter warnings
make test     # Run all tests
make build    # Build in release mode
```

### Quick Development Workflow

1. Make your changes
2. Run `make check` to verify everything works
3. Commit and push - hooks will run automatically
4. If hooks fail, fix the issues and try again

## Code Quality Standards

- **Formatting**: All code must be formatted with `cargo fmt` (enforced by hooks)
- **Tests**: All tests must pass (enforced by hooks)
- **Clippy**: Address clippy warnings when practical (shown but not blocking)
- **Documentation**: Update relevant docs for new features

## Performance Testing

If working on the performance testing module:
- Tests are in `tests/performance_tests.rs`
- Implementation is in `src/performance/`
- Run performance tests specifically: `cargo test --test performance_tests`

## Questions?

See [DEVELOPMENT.md](../DEVELOPMENT.md) for more detailed development information.