#!/bin/bash

# Install Git hooks for Rivet CLI development
# This script sets up pre-commit hooks to ensure code quality

set -e

REPO_ROOT=$(git rev-parse --show-toplevel)
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "🔧 Installing Git hooks for Rivet CLI..."

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash

# Pre-commit hook for Rivet CLI
# Ensures code formatting and basic quality checks

set -e

echo "🔍 Running pre-commit checks..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Format check
echo "📝 Checking code formatting..."
if ! cargo fmt -- --check; then
    echo "❌ Code formatting issues found."
    echo "💡 Run 'cargo fmt' to fix formatting issues."
    exit 1
fi

# Clippy check
echo "🔍 Running clippy checks..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy found issues."
    echo "💡 Fix the clippy warnings above."
    exit 1
fi

# Test check (only if there are Rust file changes)
if git diff --cached --name-only | grep -E '\.(rs)$' > /dev/null; then
    echo "🧪 Running tests..."
    if ! cargo test; then
        echo "❌ Tests failed."
        echo "💡 Fix failing tests before committing."
        exit 1
    fi
fi

echo "✅ Pre-commit checks passed!"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

# Create commit-msg hook for conventional commits (optional)
cat > "$HOOKS_DIR/commit-msg" << 'EOF'
#!/bin/bash

# Commit message hook for conventional commits
# Validates commit message format

commit_regex='^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .{1,50}'

error_msg="❌ Invalid commit message format.
Expected format: type(scope): description

Examples:
  feat: add compact HTML template
  fix(cli): resolve config parsing issue
  docs: update installation instructions
  chore(deps): update dependencies

Valid types: feat, fix, docs, style, refactor, test, chore"

if ! grep -qE "$commit_regex" "$1"; then
    echo "$error_msg"
    exit 1
fi
EOF

chmod +x "$HOOKS_DIR/commit-msg"

echo "✅ Git hooks installed successfully!"
echo ""
echo "📋 Installed hooks:"
echo "  • pre-commit: Runs formatting, clippy, and tests"
echo "  • commit-msg: Validates conventional commit format"
echo ""
echo "🔧 To bypass hooks (emergency only):"
echo "  git commit --no-verify"
echo ""
echo "💡 For better integration, consider using pre-commit:"
echo "  pip install pre-commit"
echo "  pre-commit install"