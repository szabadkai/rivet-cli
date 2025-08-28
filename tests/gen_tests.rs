use anyhow::Result;
use rivet::commands::gen::handle_gen;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_openapi_test_generation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Create a sample OpenAPI spec
    let openapi_spec = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
  description: Test API for generation
servers:
  - url: https://api.test.com/v1
paths:
  /users:
    get:
      operationId: getUsers
      summary: Get all users
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
        - name: Authorization
          in: header
          schema:
            type: string
      responses:
        '200':
          description: List of users
    post:
      operationId: createUser
      summary: Create user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
                email:
                  type: string
      responses:
        '201':
          description: User created
  /users/{id}:
    get:
      operationId: getUserById
      summary: Get user by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: integer
      responses:
        '200':
          description: User details
        '404':
          description: User not found
"#;

    let spec_file = temp_dir.path().join("test_api.yaml");
    fs::write(&spec_file, openapi_spec)?;

    // Test the generation
    handle_gen(spec_file, output_dir.clone()).await?;

    // Verify main config file was created
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Main config file should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Test API Tests"));
    assert!(config_content.contains("baseUrl: https://api.test.com/v1"));

    // Verify individual test files were created
    let get_users_file = output_dir.join("getusers.yaml");
    assert!(get_users_file.exists(), "GET users test should be created");

    let get_users_content = fs::read_to_string(&get_users_file)?;
    assert!(get_users_content.contains("method: GET"));
    assert!(get_users_content.contains("url: https://api.test.com/v1/users"));
    assert!(get_users_content.contains("status: 200"));

    // Test POST endpoint with request body
    let create_user_file = output_dir.join("createuser.yaml");
    assert!(
        create_user_file.exists(),
        "POST user test should be created"
    );

    let create_user_content = fs::read_to_string(&create_user_file)?;
    assert!(create_user_content.contains("method: POST"));
    assert!(create_user_content.contains("email"));
    assert!(create_user_content.contains("name"));
    assert!(create_user_content.contains("status: 201"));

    // Test path parameters
    let get_user_file = output_dir.join("getuserbyid.yaml");
    assert!(
        get_user_file.exists(),
        "GET user by ID test should be created"
    );

    let get_user_content = fs::read_to_string(&get_user_file)?;
    assert!(get_user_content.contains("method: GET"));
    assert!(get_user_content.contains("url: https://api.test.com/v1/users/{id}"));

    Ok(())
}

#[tokio::test]
async fn test_openapi_different_response_codes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    let openapi_spec = r#"
openapi: 3.0.0
info:
  title: Response Codes Test
  version: 1.0.0
servers:
  - url: https://api.test.com
paths:
  /created:
    post:
      operationId: createResource
      summary: Create resource
      responses:
        '201':
          description: Resource created
        '400':
          description: Bad request
  /accepted:
    put:
      operationId: updateResource
      summary: Update resource
      responses:
        '202':
          description: Update accepted
  /range:
    get:
      operationId: getRangeResponse
      summary: Get with range response
      responses:
        '2XX':
          description: Success range
"#;

    let spec_file = temp_dir.path().join("response_codes.yaml");
    fs::write(&spec_file, openapi_spec)?;

    handle_gen(spec_file, output_dir.clone()).await?;

    // Check 201 response
    let create_file = output_dir.join("createresource.yaml");
    let create_content = fs::read_to_string(&create_file)?;
    assert!(create_content.contains("status: 201"));

    // Check 202 response
    let update_file = output_dir.join("updateresource.yaml");
    let update_content = fs::read_to_string(&update_file)?;
    assert!(update_content.contains("status: 202"));

    // Check range response defaults to 200
    let range_file = output_dir.join("getrangeresponse.yaml");
    let range_content = fs::read_to_string(&range_file)?;
    assert!(range_content.contains("status: 200"));

    Ok(())
}

#[tokio::test]
async fn test_openapi_no_servers() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // OpenAPI spec without servers section
    let openapi_spec = r#"
openapi: 3.0.0
info:
  title: No Servers API
  version: 1.0.0
paths:
  /test:
    get:
      operationId: testEndpoint
      summary: Test endpoint
      responses:
        '200':
          description: Success
"#;

    let spec_file = temp_dir.path().join("no_servers.yaml");
    fs::write(&spec_file, openapi_spec)?;

    handle_gen(spec_file, output_dir.clone()).await?;

    // Should default to example.com
    let main_config = output_dir.join("rivet.yaml");
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("baseUrl: https://api.example.com"));

    // Test file should use default URL
    let test_file = output_dir.join("testendpoint.yaml");
    let test_content = fs::read_to_string(&test_file)?;
    assert!(test_content.contains("url: https://api.example.com/test"));

    Ok(())
}

#[tokio::test]
async fn test_openapi_complex_request_body_schema() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    let openapi_spec = r#"
openapi: 3.0.0
info:
  title: Complex Schema API
  version: 1.0.0
servers:
  - url: https://api.test.com
paths:
  /complex:
    post:
      operationId: createComplex
      summary: Create complex object
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
                age:
                  type: integer
                active:
                  type: boolean
                tags:
                  type: array
                  items:
                    type: string
                profile:
                  type: object
                  properties:
                    bio:
                      type: string
      responses:
        '201':
          description: Created
"#;

    let spec_file = temp_dir.path().join("complex.yaml");
    fs::write(&spec_file, openapi_spec)?;

    handle_gen(spec_file, output_dir.clone()).await?;

    let test_file = output_dir.join("createcomplex.yaml");
    let test_content = fs::read_to_string(&test_file)?;

    // Check that the generated body contains example values for different types
    assert!(test_content.contains("name"));
    assert!(test_content.contains("age"));
    assert!(test_content.contains("active"));
    assert!(test_content.contains("tags"));
    assert!(test_content.contains("profile"));

    Ok(())
}

#[tokio::test]
async fn test_nonexistent_openapi_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent.yaml");

    let result = handle_gen(nonexistent, temp_dir.path().join("output")).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_invalid_openapi_spec() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_file = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_file, "invalid: yaml: content: [").unwrap();

    let result = handle_gen(invalid_file, temp_dir.path().join("output")).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse OpenAPI"));
}

#[tokio::test]
async fn test_json_openapi_spec() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // OpenAPI spec in JSON format
    let openapi_spec = r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "JSON API",
    "version": "1.0.0"
  },
  "servers": [
    {
      "url": "https://json-api.test.com"
    }
  ],
  "paths": {
    "/json-test": {
      "get": {
        "operationId": "jsonTest",
        "summary": "JSON test endpoint",
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"#;

    let spec_file = temp_dir.path().join("json_spec.json");
    fs::write(&spec_file, openapi_spec)?;

    handle_gen(spec_file, output_dir.clone()).await?;

    // Verify it parsed the JSON correctly
    let main_config = output_dir.join("rivet.yaml");
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("JSON API Tests"));
    assert!(config_content.contains("baseUrl: https://json-api.test.com"));

    Ok(())
}
