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

```bash
# Build from source (requires Rust)
git clone https://github.com/yourusername/rivet-cli
cd rivet-cli
cargo build --release
```

## Quick Start

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
rivet run tests/example.rivet.yaml --report html,json --parallel 8
```

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