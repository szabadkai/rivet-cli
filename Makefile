.PHONY: check fmt clippy test build clean install help

# Default target
help:
	@echo "Available commands:"
	@echo "  make check    - Run all code quality checks (fmt, clippy, test, build)"
	@echo "  make fmt      - Format code with cargo fmt"
	@echo "  make clippy   - Run clippy linter"
	@echo "  make test     - Run all tests"
	@echo "  make build    - Build in release mode"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install  - Install rivet binary locally"

# Run all checks (same as pre-push hook)
check:
	@echo "🔍 Running all development checks..."
	@echo "📝 Formatting code..."
	cargo fmt
	@echo "🔧 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "🧪 Running tests..."
	cargo test
	@echo "🏗️  Building release..."
	cargo build --release
	@echo "🎉 All checks passed!"

# Format code
fmt:
	@echo "📝 Formatting code with cargo fmt..."
	cargo fmt

# Run clippy
clippy:
	@echo "🔧 Running cargo clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Build release
build:
	@echo "🏗️  Building in release mode..."
	cargo build --release

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Install binary locally
install:
	@echo "📦 Installing rivet locally..."
	cargo install --path .