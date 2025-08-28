# GraphQL Testing Examples

This directory contains example GraphQL test suites demonstrating how to use rivet for GraphQL API testing.

## 🚀 Quick Start

```bash
# Run all GraphQL examples
rivet run examples/graphql/

# Run specific test suite
rivet run examples/graphql/users.rivet.yaml --env dev

# Performance test GraphQL endpoint
rivet perf examples/graphql/users.rivet.yaml --concurrent 10 --duration 30s
```

## 📁 Example Files

### `users.rivet.yaml`
Comprehensive user management API tests demonstrating:
- **Queries**: Fetch users with pagination
- **Mutations**: Create and update users
- **Variables**: Dynamic query parameters
- **Data-driven testing**: CSV dataset integration

### `ecommerce.rivet.yaml`
E-commerce GraphQL API tests showcasing:
- **Complex queries**: Product catalog with filters
- **Nested data**: Orders, inventory, and reviews
- **Authentication**: Admin vs user permissions
- **Search functionality**: Full-text product search

### `error-handling.rivet.yaml`
Error handling patterns for GraphQL APIs:
- **Syntax errors**: Malformed queries
- **Field errors**: Invalid field selections  
- **Validation errors**: Business logic validation
- **Authentication errors**: Unauthorized access
- **Rate limiting**: Handling 429 responses

## 🔧 Configuration

### Environment Variables

Set these environment variables for the examples:

```bash
export BASE_URL="https://api.example.com"
export GRAPHQL_TOKEN="your-graphql-token"
export ADMIN_TOKEN="admin-access-token"
export USER_TOKEN="user-access-token"
```

### Data Files

The examples use CSV files in the `data/` directory:
- `users.csv` - User test data with names, emails, etc.
- `products.csv` - Product test data with categories, quantities, etc.

## 📊 GraphQL Testing Patterns

### 1. Basic Query Structure

```yaml
request:
  method: POST
  url: "{{baseUrl}}/graphql"
  headers:
    Content-Type: application/json
    Authorization: "Bearer {{token}}"
  body: |
    {
      "query": "query GetUsers { users { id name email } }",
      "variables": {}
    }
```

### 2. Variables and Dynamic Queries

```yaml
body: |
  {
    "query": "query GetUser($id: ID!) { user(id: $id) { id name } }",
    "variables": {
      "id": "{{userId}}"
    }
  }
```

### 3. Mutation Testing

```yaml
body: |
  {
    "query": "mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { user { id } errors { field message } } }",
    "variables": {
      "input": {
        "name": "{{userName}}",
        "email": "{{userEmail}}"
      }
    }
  }
```

### 4. Response Validation

```yaml
expect:
  status: 200
  jsonpath:
    "$.data.users": "array"
    "$.data.users[0].id": "exists"
    "$.errors": null  # Ensure no GraphQL errors
```

## 🎯 Best Practices

### ✅ Do:
- Always check for `$.errors` being null in successful responses
- Use specific JSONPath assertions for critical fields
- Test both success and error scenarios
- Use variables for dynamic query parameters
- Include proper Content-Type headers

### ❌ Don't:
- Hardcode sensitive tokens in test files
- Ignore the `errors` field in GraphQL responses
- Test only happy path scenarios
- Use overly complex queries in a single test
- Skip authentication testing

## 🔍 Error Handling

GraphQL APIs can return errors in multiple ways:

1. **HTTP-level errors** (4xx, 5xx status codes)
2. **GraphQL errors** (in the `errors` field)
3. **Field-level errors** (business logic validation)

Always test all three error types:

```yaml
# HTTP error
expect:
  status: 400
  jsonpath:
    "$.errors[0].message": "*syntax*"

# GraphQL error with data
expect:
  status: 200
  jsonpath:
    "$.data.createUser.errors": "array"
    "$.errors": null

# Field-level business error
expect:
  status: 200  
  jsonpath:
    "$.data.user": null
    "$.errors[0].message": "*Unauthorized*"
```

## 🚀 Performance Testing

GraphQL endpoints can be performance tested like any HTTP endpoint:

```bash
# Load test with constant load
rivet perf examples/graphql/users.rivet.yaml \
  --concurrent 20 \
  --duration 60s \
  --pattern constant

# Ramp-up test
rivet perf examples/graphql/ecommerce.rivet.yaml \
  --concurrent 50 \
  --duration 120s \
  --pattern ramp-up \
  --output reports/graphql-perf.json
```

## 📈 Advanced Features

### Subscription Testing
While GraphQL subscriptions require WebSocket connections (not yet supported), you can test subscription-like behavior using polling:

```yaml
# Test subscription setup endpoint
- name: Setup subscription
  request:
    method: POST
    url: "{{baseUrl}}/graphql"
    body: |
      {
        "query": "mutation { subscribeToUpdates(topic: \"users\") { subscriptionId } }"
      }
```

### Schema Validation
You can validate responses against GraphQL schemas using JSON Schema validation:

```yaml
expect:
  status: 200
  schema: "#/components/schemas/GraphQLResponse"
```

## 🤝 Contributing

Have a great GraphQL testing example? Submit a PR with:
- Clear test descriptions
- Realistic use cases  
- Both success and error scenarios
- Documentation for any special setup required