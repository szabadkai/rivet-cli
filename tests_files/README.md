# Rivet Test Suites

This directory contains comprehensive test suites for demonstrating and validating Rivet's API testing capabilities.

## Test Suites

### 1. `api-smoke-test.rivet.yaml`
**Comprehensive API functionality test**
- Tests all HTTP methods (GET, POST, PUT, DELETE)
- JSON and form data handling
- Authentication testing
- Header validation
- Redirect handling
- Various response scenarios

**Usage:**
```bash
rivet run tests/api-smoke-test.rivet.yaml
rivet run tests/api-smoke-test.rivet.yaml --report html,json
```

### 2. `users-api.rivet.yaml`
**Data-driven testing with JSONPlaceholder API**
- Tests user endpoints with real data
- Validates user information from CSV dataset
- Tests related resources (posts, albums)
- Parallel execution example

**Usage:**
```bash
rivet run tests/users-api.rivet.yaml --parallel 3
rivet run tests/users-api.rivet.yaml --data data/users-test-data.csv
```

### 3. `status-codes.rivet.yaml`
**HTTP status code testing**
- Tests various HTTP status codes (200, 400, 404, 500, etc.)
- Data-driven with CSV input
- Parallel execution

**Usage:**
```bash
rivet run tests/status-codes.rivet.yaml --parallel 4
```

### 4. `error-scenarios.rivet.yaml`
**Error handling and edge cases**
- Client errors (4xx status codes)
- Server errors (5xx status codes)
- Timeout scenarios
- Large responses
- Empty responses
- Malformed content

**Usage:**
```bash
rivet run tests/error-scenarios.rivet.yaml --bail
rivet run tests/error-scenarios.rivet.yaml --timeout 10s
```

### 5. `content-types.rivet.yaml`
**Content type and payload testing**
- JSON, XML, form-encoded data
- Plain text and binary content
- Multipart form simulation
- Large payloads
- Custom content types

**Usage:**
```bash
rivet run tests/content-types.rivet.yaml
```

## Test Data Files

### `data/users-test-data.csv`
Real user data from JSONPlaceholder for data-driven testing:
- User IDs, names, and email addresses
- Used with `users-api.rivet.yaml`

### `data/status-codes.csv`
HTTP status codes and expected messages:
- Various status codes for comprehensive testing
- Used with `status-codes.rivet.yaml`

### `data/posts.csv`
Post and user ID combinations for testing relationships

## Environment Configurations

### `envs/test.env`
Test environment settings using httpbin.org and public APIs

### `envs/staging.env`
Staging environment template with placeholders for real APIs

### `envs/production.env`
Production environment template with safety settings

## Running Tests

### Basic Usage
```bash
# Run a single test suite
rivet run tests/api-smoke-test.rivet.yaml

# Run all tests in the directory
rivet run tests

# Run with specific environment
rivet run tests/users-api.rivet.yaml --env staging

# Run with parallel execution
rivet run tests/ --parallel 8
```

### Advanced Usage
```bash
# Generate reports
rivet run tests --report html,junit,json --out reports/

# Filter tests by name
rivet run tests/api-smoke-test.rivet.yaml --grep "POST"

# Stop on first failure
rivet run tests --bail

# CI mode (no animations)
rivet run tests --ci

# Custom timeout
rivet run tests/error-scenarios.rivet.yaml --timeout 30s
```

### Environment Variables
```bash
# Override base URL
BASE_URL=https://api.staging.example.com rivet run tests

# Set environment
RIVET_ENV=production rivet run tests

# Custom timeout
TIMEOUT=60s rivet run tests
```

## Expected Results

All test suites are designed to **pass** when run against their respective APIs:

- `api-smoke-test.rivet.yaml` - Uses httpbin.org, should pass completely
- `users-api.rivet.yaml` - Uses jsonplaceholder.typicode.com, should pass with real data
- `status-codes.rivet.yaml` - Tests various status codes, all should return expected codes
- `error-scenarios.rivet.yaml` - Tests error conditions, expects proper error codes
- `content-types.rivet.yaml` - Tests content handling, should pass with proper parsing

## Test Coverage

These suites cover:
- ✅ All HTTP methods (GET, POST, PUT, DELETE)
- ✅ Request/response headers
- ✅ JSON and form data payloads
- ✅ Query parameters
- ✅ Authentication mechanisms
- ✅ Error handling (4xx, 5xx status codes)
- ✅ Data-driven testing with CSV
- ✅ Parallel execution
- ✅ JSONPath assertions
- ✅ Various content types
- ✅ Large and empty responses
- ✅ Timeout scenarios
- ✅ Setup and teardown phases

This comprehensive test suite provides excellent examples for learning Rivet and validating its functionality across different scenarios.