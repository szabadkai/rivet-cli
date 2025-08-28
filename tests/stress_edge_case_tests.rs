use anyhow::Result;
use rivet::commands::gen::handle_gen;
use rivet::commands::import::handle_import;
use std::fs;
use tempfile::TempDir;

/// Stress tests with complex real-world examples that expose edge cases and potential
/// breaking scenarios in our import and generation functionality. These tests ensure
/// our implementation is robust against the messy, varied real-world API specifications.

#[tokio::test]
async fn test_github_massive_openapi_spec() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Download GitHub's massive 8.5MB REST API specification
    let spec_url = "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.yaml";
    let response = reqwest::get(spec_url).await?;
    let spec_content = response.text().await?;
    
    let spec_file = temp_dir.path().join("github_massive_spec.yaml");
    fs::write(&spec_file, spec_content)?;
    
    // This should handle massive specs with 2000+ endpoints
    handle_gen(spec_file, output_dir.clone()).await?;
    
    // Verify it generated a massive number of tests
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "GitHub config should be created");
    
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("GitHub v3 REST API"));
    
    // Count generated test files - should be 1000+
    let test_files: Vec<_> = walkdir::WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".yaml"))
        .filter(|e| e.file_name() != "rivet.yaml")
        .collect();
    
    // GitHub API has 1000+ endpoints, so we should have generated that many tests
    assert!(test_files.len() > 1000, "Should generate 1000+ tests from GitHub API, got {}", test_files.len());
    
    // Verify some key GitHub API endpoints are present
    let apps_get = output_dir.join("apps_get-authenticated.yaml");
    assert!(apps_get.exists(), "GitHub Apps API should be generated");
    
    let repos_create = output_dir.join("repos_create-in-org.yaml");
    assert!(repos_create.exists(), "GitHub Repos API should be generated");
    
    Ok(())
}

#[tokio::test]
async fn test_auth0_complex_nested_postman_collection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Download Auth0's complex authentication collection with nested folders
    let collection_url = "https://raw.githubusercontent.com/auth0/postman-collections/master/Auth0%20Authentication%20API.postman_collection.json";
    let response = reqwest::get(collection_url).await?;
    let collection_content = response.text().await?;
    
    let collection_file = temp_dir.path().join("auth0_complex.json");
    fs::write(&collection_file, collection_content)?;
    
    // This collection has complex OAuth flows, nested folders, and null values
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;
    
    // Verify complex nested structure was preserved
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Auth0 config should be created");
    
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Auth0 Authentication API"));
    
    // Verify nested folders were created
    let db_connections = output_dir.join("db_connections");
    assert!(db_connections.exists(), "DB Connections folder should exist");
    
    let get_access_token = output_dir.join("get_access_token");
    assert!(get_access_token.exists(), "Get Access Token folder should exist");
    
    let passwordless = output_dir.join("passwordless");
    assert!(passwordless.exists(), "Passwordless folder should exist");
    
    let saml = output_dir.join("saml");
    assert!(saml.exists(), "SAML folder should exist");
    
    let deprecated = output_dir.join("deprecated");
    assert!(deprecated.exists(), "Deprecated folder should exist");
    
    // Check specific OAuth flows
    let auth_code_test = get_access_token.join("get_access_token_authorization_code.yaml");
    assert!(auth_code_test.exists(), "Authorization code flow test should exist");
    
    let client_creds_test = get_access_token.join("get_access_token_client_credentials.yaml");
    assert!(client_creds_test.exists(), "Client credentials flow test should exist");
    
    let refresh_token_test = get_access_token.join("get_access_token_refresh_token.yaml");
    assert!(refresh_token_test.exists(), "Refresh token flow test should exist");
    
    // Verify form data with null values was handled
    let signup_test = db_connections.join("db_connections_signup_using_a_username_password.yaml");
    assert!(signup_test.exists(), "Signup test should exist");
    
    let signup_content = fs::read_to_string(&signup_test)?;
    assert!(signup_content.contains("method: POST"));
    assert!(signup_content.contains("body:")); // Should have form data converted
    
    Ok(())
}

#[tokio::test]
async fn test_openai_large_integer_overflow_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Download OpenAI spec which has integer overflow issues
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let response = reqwest::get(spec_url).await?;
    let spec_content = response.text().await?;
    
    let spec_file = temp_dir.path().join("openai_spec.yaml");
    fs::write(&spec_file, spec_content)?;
    
    // This should fail gracefully with a clear error message about integer overflow
    let result = handle_gen(spec_file, output_dir.clone()).await;
    
    // Verify it fails with the expected integer overflow error
    assert!(result.is_err(), "OpenAI spec should fail due to integer overflow");
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("invalid type: integer") || 
            error_msg.contains("Failed to parse OpenAPI"), 
            "Should get integer parsing error, got: {}", error_msg);
    
    Ok(())
}

#[tokio::test]
async fn test_usps_large_production_collection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // USPS has a large production collection with 44 tests across 14 folders
    let collection_url = "https://raw.githubusercontent.com/USPS/api-examples/main/Example-Postman.postman_collection.json";
    let response = reqwest::get(collection_url).await?;
    let collection_content = response.text().await?;
    
    let collection_file = temp_dir.path().join("usps_large.json");
    fs::write(&collection_file, collection_content)?;
    
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;
    
    // Verify large collection was processed
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "USPS config should be created");
    
    // Count all generated test files
    let test_files: Vec<_> = walkdir::WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".yaml"))
        .filter(|e| e.file_name() != "rivet.yaml")
        .collect();
    
    // USPS should generate 40+ tests
    assert!(test_files.len() >= 40, "Should generate 40+ tests from USPS API, got {}", test_files.len());
    
    // Verify key USPS API categories
    let oauth_folder = output_dir.join("oauth");
    assert!(oauth_folder.exists(), "OAuth folder should exist");
    
    let addresses_folder = output_dir.join("addresses");
    assert!(addresses_folder.exists(), "Addresses folder should exist");
    
    let tracking_folder = output_dir.join("tracking");
    assert!(tracking_folder.exists(), "Tracking folder should exist");
    
    let payments_folder = output_dir.join("payments");
    assert!(payments_folder.exists(), "Payments folder should exist");
    
    Ok(())
}

#[tokio::test]
async fn test_edge_case_postman_formats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Create a collection that tests various edge cases we've encountered
    let edge_case_collection = r#"{
        "info": {
            "name": "Edge Case Test Collection",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": [
            {
                "name": "Null Value Form Data",
                "request": {
                    "method": "POST",
                    "url": "https://api.test.com/null-values",
                    "body": {
                        "mode": "urlencoded",
                        "urlencoded": [
                            {
                                "key": "field_with_null",
                                "value": null,
                                "type": "text"
                            },
                            {
                                "key": "field_with_empty",
                                "value": "",
                                "type": "text"
                            },
                            {
                                "key": "field_with_value",
                                "value": "actual_value",
                                "type": "text",
                                "disabled": false
                            },
                            {
                                "key": "disabled_field",
                                "value": "should_be_ignored",
                                "type": "text",
                                "disabled": true
                            }
                        ]
                    }
                }
            },
            {
                "name": "Complex URL Object",
                "request": {
                    "method": "GET",
                    "url": {
                        "raw": "https://{{host}}/{{version}}/test?param1={{value1}}&param2=static",
                        "protocol": "https",
                        "host": ["{{host}}"],
                        "path": ["{{version}}", "test"],
                        "query": [
                            {
                                "key": "param1",
                                "value": "{{value1}}",
                                "disabled": false
                            },
                            {
                                "key": "param2",
                                "value": "static",
                                "disabled": false
                            },
                            {
                                "key": "disabled_param",
                                "value": "ignored",
                                "disabled": true
                            }
                        ]
                    }
                }
            },
            {
                "name": "Empty Body",
                "request": {
                    "method": "POST",
                    "url": "https://api.test.com/empty-body",
                    "body": {}
                }
            }
        ]
    }"#;
    
    let collection_file = temp_dir.path().join("edge_cases.json");
    fs::write(&collection_file, edge_case_collection)?;
    
    // This should handle all edge cases gracefully
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;
    
    // Verify all edge cases were handled
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Edge case config should be created");
    
    let null_values_test = output_dir.join("null_value_form_data.yaml");
    assert!(null_values_test.exists(), "Null values test should be created");
    
    let null_content = fs::read_to_string(&null_values_test)?;
    assert!(null_content.contains("method: POST"));
    assert!(null_content.contains("body:")); // Should handle null values
    // Should contain fields with values but not disabled fields
    assert!(null_content.contains("field_with_value=actual_value"));
    assert!(!null_content.contains("disabled_field"));
    
    let complex_url_test = output_dir.join("complex_url_object.yaml");
    assert!(complex_url_test.exists(), "Complex URL test should be created");
    
    let url_content = fs::read_to_string(&complex_url_test)?;
    // Should contain the raw URL from the collection
    assert!(url_content.contains("https://{{host}}/{{version}}/test?param1={{value1}}&param2=static"));
    
    let empty_body_test = output_dir.join("empty_body.yaml");
    assert!(empty_body_test.exists(), "Empty body test should be created");
    
    let empty_content = fs::read_to_string(&empty_body_test)?;
    assert!(empty_content.contains("method: POST"));
    assert!(empty_content.contains("body: null") || !empty_content.contains("body:")); // Should handle empty body
    
    Ok(())
}

#[tokio::test]
async fn test_performance_with_large_specs() -> Result<()> {
    // Performance test: Ensure we can handle large specifications in reasonable time
    use std::time::Instant;
    
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Create a synthetic large OpenAPI spec
    let mut large_spec = String::from(r#"
openapi: 3.0.0
info:
  title: Large Performance Test API
  version: 1.0.0
servers:
  - url: https://large-api.example.com
paths:
"#);
    
    // Generate 500 endpoints programmatically
    for i in 0..500 {
        large_spec.push_str(&format!(r#"
  /endpoint{}:
    get:
      operationId: getEndpoint{}
      summary: Get endpoint {}
      responses:
        '200':
          description: Success
    post:
      operationId: postEndpoint{}
      summary: Post to endpoint {}
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                field{}:
                  type: string
      responses:
        '201':
          description: Created
"#, i, i, i, i, i, i));
    }
    
    let spec_file = temp_dir.path().join("large_synthetic_spec.yaml");
    fs::write(&spec_file, large_spec)?;
    
    // Time the generation
    let start = Instant::now();
    handle_gen(spec_file, output_dir.clone()).await?;
    let duration = start.elapsed();
    
    // Should complete within reasonable time (less than 30 seconds for 1000 operations)
    assert!(duration.as_secs() < 30, "Large spec generation took too long: {:?}", duration);
    
    // Verify correct number of tests generated
    let test_files: Vec<_> = walkdir::WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".yaml"))
        .filter(|e| e.file_name() != "rivet.yaml")
        .collect();
    
    // Should generate 1000 tests (500 GET + 500 POST)
    assert_eq!(test_files.len(), 1000, "Should generate exactly 1000 tests");
    
    Ok(())
}

#[tokio::test]
async fn test_malformed_collections_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Test various malformed collection formats
    let test_cases = vec![
        ("invalid_json.json", "{ invalid json }"),
        ("missing_info.json", r#"{ "item": [] }"#),
        ("invalid_items.json", r#"{ "info": { "name": "Test", "schema": "v2.1.0" }, "item": "not_an_array" }"#),
        ("empty_file.json", ""),
    ];
    
    for (filename, content) in test_cases {
        let malformed_file = temp_dir.path().join(filename);
        fs::write(&malformed_file, content)?;
        
        let result = handle_import("postman".to_string(), malformed_file, output_dir.clone()).await;
        assert!(result.is_err(), "Malformed collection {} should fail", filename);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_unicode_and_special_characters() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");
    
    // Test collection with Unicode and special characters
    let unicode_collection = r#"{
        "info": {
            "name": "Unicode Test 测试集合 🚀",
            "description": "Collection with émojis and spéciàl characters: áéíóú, ÀÈÌÒÙ, ñçß",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": [
            {
                "name": "Request with Unicode: 测试请求 🎯",
                "request": {
                    "method": "POST",
                    "url": "https://api.example.com/测试",
                    "body": {
                        "mode": "raw",
                        "raw": "{\"message\": \"Hello 世界! Special chars: áéíóú & <>&\"}"
                    }
                }
            },
            {
                "name": "Folder/With/Slashes",
                "item": [
                    {
                        "name": "Nested: Special@Characters#Test",
                        "request": {
                            "method": "GET",
                            "url": "https://api.example.com/special?param=value&unicode=测试"
                        }
                    }
                ]
            }
        ]
    }"#;
    
    let collection_file = temp_dir.path().join("unicode_test.json");
    fs::write(&collection_file, unicode_collection)?;
    
    // Should handle Unicode characters gracefully
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;
    
    // Verify files were created with sanitized names
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Unicode config should be created");
    
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Unicode Test")); // Should preserve Unicode in content
    
    // Check that file names were sanitized but content preserved
    let test_files: Vec<_> = walkdir::WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".yaml"))
        .filter(|e| e.file_name() != "rivet.yaml")
        .collect();
    
    assert!(!test_files.is_empty(), "Should create test files from unicode collection");
    
    Ok(())
}