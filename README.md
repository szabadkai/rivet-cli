# Rivet CLI

**API testing that lives in git.**

Rivet is a modern API testing tool designed for command-line workflows. It allows you to send HTTP requests, run test suites, import from Postman/Insomnia, generate coverage reports, and make gRPC calls - all with beautiful terminal output.

## Features

- 🚀 **Send Requests**: Send individual HTTP requests with pretty output and JSON syntax highlighting
- 🧪 **Run Test Suites**: Execute test suites with parallel execution, data-driven testing, and rich reporting
- 📊 **Generate Reports**: Export results in JSON, JUnit, and HTML formats
- 📝 **OpenAPI Integration**: Generate tests from OpenAPI specs and track endpoint coverage
- 📥 **Import Support**: Import collections from Postman, Insomnia, Bruno, and cURL
- 🔗 **gRPC Support**: Make gRPC unary calls with metadata and field assertions
- 🎨 **Beautiful Terminal UI**: Spinners, progress bars, and colored output that works great in CI/CD

## Installation

### Pre-built Binaries

Download the latest release for your platform:

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/szabadkai/rivet-cli/releases/latest/download/rivet-macos-arm64 -o rivet
chmod +x rivet
sudo mv rivet /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/szabadkai/rivet-cli/releases/latest/download/rivet-macos-x86_64 -o rivet
chmod +x rivet
sudo mv rivet /usr/local/bin/
```

**Linux:**
```bash
curl -L https://github.com/szabadkai/rivet-cli/releases/latest/download/rivet-linux-x86_64 -o rivet
chmod +x rivet
sudo mv rivet /usr/local/bin/
```

**Windows:**
Download `rivet-windows-x86_64.exe` from the [latest release](https://github.com/szabadkai/rivet-cli/releases/latest) and add to your PATH.

### Build from Source

```bash
# Requires Rust 1.70+
git clone https://github.com/szabadkai/rivet-cli
cd rivet-cli
cargo build --release
```

## Quick Start

### Development (no Make required)

Common dev tasks are available via Cargo aliases/xtask:

```bash
# One-time setup (git hooks)
cargo dev-setup

# Build / Test / Lints
cargo build
cargo test
cargo fmt
cargo fmt-check
cargo clippy

# Run all checks like CI
cargo ci

# Start a release (maintainers)
cargo release patch   # or minor/major
```

### Send a simple HTTP request

```bash
rivet send GET https://httpbin.org/json
```

### Send a POST request with headers

```bash
rivet send POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "test"}'
```

### Run a test suite

```bash
# Basic test run
rivet run tests/example.rivet.yaml

# With HTML report (auto-opens in browser)
rivet run tests/example.rivet.yaml --report html

# With custom template and multiple formats
rivet run tests/example.rivet.yaml --report html,json --template compact --parallel 8
```

### HTML Report Templates

Rivet includes several beautiful HTML report templates:

- **`detailed`** - Professional, comprehensive business reports
- **`compact`** - Interactive, filterable, space-efficient with expandable test details
- **`chatty`** - Friendly, conversational with storytelling elements
- **`simple`** - Clean, minimal reports with basic metrics

```bash
rivet run tests/ --report html --template compact --open
```

### Configuration

Rivet supports user configuration via `~/.rivet/config.json`:

```json
{
  "reports": {
    "auto_open_browser": true,
    "default_template": "compact",
    "default_formats": ["html"]
  }
}
```

**Default Settings:**
- Auto-opens HTML reports in browser
- Uses the interactive `compact` template by default
- Generates HTML reports by default

This file is automatically created on first run with sensible defaults.

### Generate tests from OpenAPI spec

```bash
rivet gen --spec api-spec.yaml --out tests/
```

### Import from Postman

```bash
rivet import postman collection.json --out tests/
```

### Make a gRPC call

```bash
rivet grpc --proto ./protos --call svc.Users/GetUser --data '{"id": 42}'
```

## Test File Format

Rivet uses YAML files for test definitions:

```yaml
name: User API Tests
env: ${RIVET_ENV:dev}

vars:
  baseUrl: ${BASE_URL:https://api.example.com}
  token: ${TOKEN}

tests:
  - name: Get user
    request:
      method: GET
      url: "{{baseUrl}}/users/{{userId}}"
      headers:
        Authorization: "Bearer {{token}}"
    expect:
      status: 200
      jsonpath:
        "$.id": "{{userId}}"

dataset:
  file: data/users.csv
  parallel: 4
```

## Commands

- `rivet send <METHOD> <URL>` - Send a single HTTP request
- `rivet run <file|dir>` - Run test suites
- `rivet gen --spec <openapi.yaml>` - Generate tests from OpenAPI spec
- `rivet coverage --spec <openapi.yaml> --from <reports>` - Generate coverage report
- `rivet import <tool> <file>` - Import from other tools
- `rivet grpc --proto <dir> --call <service/method>` - Make gRPC calls

## Project Structure

```
rivet-cli/
├── src/
│   ├── commands/          # Command implementations
│   ├── ui/               # Terminal UI components
│   ├── config.rs         # Configuration structures
│   ├── http.rs           # HTTP client utilities
│   ├── grpc.rs           # gRPC client utilities
│   ├── report.rs         # Report generation
│   └── utils.rs          # Common utilities
├── tests/                # Example test files
├── examples/             # Example configurations
├── data/                 # Test data files
└── reports/              # Generated reports
```

## License

MIT License - see LICENSE file for details.
