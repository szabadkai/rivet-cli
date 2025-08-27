# Contributing to Rivet CLI

Thank you for your interest in contributing to Rivet CLI! This document provides guidelines and information for contributors.

## 🚀 Development Setup

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - For version control

### Quick Start

```bash
# Clone the repository
git clone https://github.com/szabadkai/rivet-cli
cd rivet-cli

# Set up development environment (installs git hooks)
cargo dev-setup

# Build the project
cargo build

# Run tests
cargo test
```

## 🔧 Development Workflow

### Code Quality Standards

We maintain high code quality standards with automated checks:

**Formatting:**
```bash
# Format code (required before committing)
cargo fmt

# Check formatting
cargo fmt-check
```

**Linting:**
```bash
# Run clippy lints
cargo clippy
```

**Testing:**
```bash
# Run all tests
cargo test
```

**All Checks:**
```bash
# Run all quality checks (CI simulation)
cargo ci
```

### Git Hooks

We provide git hooks to ensure code quality:

```bash
# Install git hooks (automatic formatting, linting, testing)
cargo install-hooks

# Manual installation
./scripts/install-git-hooks.sh
```

The hooks will:
- ✅ Check code formatting with `cargo fmt`
- ✅ Run clippy lints with zero warnings
- ✅ Run tests on Rust file changes
- ✅ Validate commit message format

### Commit Message Format

We use [Conventional Commits](https://conventionalcommits.org/):

```
type(scope): description

# Examples:
feat: add compact HTML template with interactive filtering
fix(cli): resolve config parsing issue with user directories  
docs: update installation instructions for macOS
chore(deps): update serde to v1.0.200
```

**Types:**
- `feat` - New features
- `fix` - Bug fixes
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Build process, dependencies, etc.

## 📋 Pull Request Process

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/rivet-cli
   cd rivet-cli
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow the coding standards
   - Add tests for new functionality
   - Update documentation as needed

4. **Run Quality Checks**
```bash
cargo ci  # Runs formatting, clippy, tests, and build
```

5. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Pull Request Guidelines**
   - Provide clear description of changes
   - Link to related issues
   - Ensure CI checks pass
   - Request review from maintainers

## 🧪 Testing

### Running Tests

```bash
# All tests
make test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Adding Tests

- **Unit tests** - In the same file as the code
- **Integration tests** - In `tests/` directory
- **Example tests** - Test actual rivet files in `tests/` directory

### Test Guidelines

- Test both success and error cases
- Use descriptive test names
- Keep tests focused and independent
- Mock external dependencies when possible

## 📝 Documentation

### Code Documentation

- Add rustdoc comments for public APIs
- Include examples in documentation
- Document complex algorithms or business logic

### User Documentation

- Update `README.md` for new features
- Update `CHANGELOG.md` for all changes
- Add examples to demonstrate usage

## 🏗️ Architecture Overview

```
rivet-cli/
├── src/
│   ├── commands/          # CLI command implementations
│   │   ├── run.rs         # Test runner command
│   │   ├── send.rs        # HTTP request command
│   │   └── ...
│   ├── ui/               # Terminal UI components
│   ├── runner/           # Test execution engine
│   ├── report.rs         # Report generation
│   ├── config.rs         # Configuration management
│   └── main.rs          # CLI entry point
├── templates/            # HTML report templates
├── .github/             # GitHub Actions workflows
└── scripts/             # Development scripts
```

## 🔀 Release Process

Releases are automated via GitHub Actions:

```bash
# Create release (maintainers only)
cargo release patch  # 0.1.0 -> 0.1.1
cargo release minor  # 0.1.0 -> 0.2.0
cargo release major  # 0.1.0 -> 1.0.0
```

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment**: OS, Rust version, Rivet version
2. **Steps to reproduce**: Minimal example
3. **Expected behavior**: What should happen
4. **Actual behavior**: What actually happens
5. **Error messages**: Full error output
6. **Additional context**: Logs, screenshots, etc.

## 💡 Feature Requests

When requesting features:

1. **Use case**: Describe the problem you're solving
2. **Proposed solution**: How should it work?
3. **Alternatives**: Other solutions you considered
4. **Breaking changes**: Would this break existing functionality?

## 📞 Getting Help

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - Questions and general discussion
- **Code Review** - Request feedback on your PR

## 🎉 Recognition

Contributors are recognized in:
- `CHANGELOG.md` - Credit for significant contributions
- GitHub contributors page
- Release notes for major features

Thank you for contributing to Rivet CLI! 🚀
