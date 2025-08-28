#!/bin/bash

# Development check script for rivet-cli
# Run this before committing to ensure code quality

set -e

echo "🔍 Running development checks..."

# Format the code
echo "📝 Formatting code with cargo fmt..."
cargo fmt

# Run clippy to catch potential issues (warn only for now)
echo "🔧 Running cargo clippy..."
cargo clippy --all-targets --all-features

# Run all tests
echo "🧪 Running all tests..."
cargo test

# Build in release mode to catch any release-specific issues
echo "🏗️  Building in release mode..."
cargo build --release

echo "🎉 All checks passed! Code is ready for commit."