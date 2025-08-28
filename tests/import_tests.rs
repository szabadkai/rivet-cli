use anyhow::Result;
use rivet::commands::import::handle_import;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_postman_collection_import() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Create a sample Postman collection
    let collection = r#"{
        "info": {
            "name": "Test Collection",
            "description": "A test collection",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "variable": [
            {
                "key": "baseUrl",
                "value": "https://api.test.com"
            }
        ],
        "item": [
            {
                "name": "Get Test",
                "request": {
                    "method": "GET",
                    "url": "{{baseUrl}}/test",
                    "header": [
                        {
                            "key": "Accept",
                            "value": "application/json"
                        }
                    ]
                },
                "response": [
                    {
                        "code": 200
                    }
                ]
            },
            {
                "name": "Test Folder",
                "item": [
                    {
                        "name": "Nested Test",
                        "request": {
                            "method": "POST",
                            "url": "{{baseUrl}}/nested",
                            "body": {
                                "mode": "raw",
                                "raw": "{\"test\": \"data\"}"
                            }
                        }
                    }
                ]
            }
        ]
    }"#;

    let collection_file = temp_dir.path().join("test_collection.json");
    fs::write(&collection_file, collection)?;

    // Test the import
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;

    // Verify main config file was created
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Main config file should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Test Collection"));
    assert!(config_content.contains("baseUrl: https://api.test.com"));

    // Verify individual test files were created
    let get_test_file = output_dir.join("get_test.yaml");
    assert!(get_test_file.exists(), "GET test file should be created");

    let get_test_content = fs::read_to_string(&get_test_file)?;
    assert!(get_test_content.contains("method: GET"));
    assert!(get_test_content.contains("url: '{{baseUrl}}/test'"));
    assert!(get_test_content.contains("Accept: application/json"));
    assert!(get_test_content.contains("status: 200"));

    // Verify folder structure was preserved
    let folder_dir = output_dir.join("test_folder");
    assert!(folder_dir.exists(), "Folder should be created");

    let nested_test_file = folder_dir.join("test_folder_nested_test.yaml");
    assert!(
        nested_test_file.exists(),
        "Nested test file should be created"
    );

    let nested_content = fs::read_to_string(&nested_test_file)?;
    assert!(nested_content.contains("method: POST"));
    assert!(nested_content.contains("body: '{\"test\": \"data\"}'"));

    Ok(())
}

#[tokio::test]
async fn test_postman_collection_with_variables() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Create a collection with multiple variables
    let collection = r#"{
        "info": {
            "name": "Variable Test",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "variable": [
            {
                "key": "host",
                "value": "api.example.com"
            },
            {
                "key": "version",
                "value": "v1"
            }
        ],
        "item": [
            {
                "name": "Test Request",
                "request": {
                    "method": "GET",
                    "url": "https://{{host}}/{{version}}/test"
                }
            }
        ]
    }"#;

    let collection_file = temp_dir.path().join("variables.json");
    fs::write(&collection_file, collection)?;

    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;

    let main_config = output_dir.join("rivet.yaml");
    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("host: api.example.com"));
    assert!(config_content.contains("version: v1"));

    Ok(())
}

#[tokio::test]
async fn test_postman_different_request_body_modes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    let collection = r#"{
        "info": {
            "name": "Body Modes Test",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": [
            {
                "name": "Raw Body",
                "request": {
                    "method": "POST",
                    "url": "https://api.test.com/raw",
                    "body": {
                        "mode": "raw",
                        "raw": "{\"key\": \"value\"}"
                    }
                }
            },
            {
                "name": "Form Data",
                "request": {
                    "method": "POST",
                    "url": "https://api.test.com/form",
                    "body": {
                        "mode": "formdata",
                        "formdata": [
                            {
                                "key": "field1",
                                "value": "value1"
                            },
                            {
                                "key": "field2",
                                "value": "value2"
                            }
                        ]
                    }
                }
            }
        ]
    }"#;

    let collection_file = temp_dir.path().join("body_modes.json");
    fs::write(&collection_file, collection)?;

    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;

    // Check raw body handling
    let raw_file = output_dir.join("raw_body.yaml");
    let raw_content = fs::read_to_string(&raw_file)?;
    assert!(raw_content.contains("body: '{\"key\": \"value\"}'"));

    // Check form data handling
    let form_file = output_dir.join("form_data.yaml");
    let form_content = fs::read_to_string(&form_file)?;
    assert!(form_content.contains("body: field1=value1&field2=value2"));

    Ok(())
}

#[tokio::test]
async fn test_unsupported_import_tool() {
    let temp_dir = TempDir::new().unwrap();
    let dummy_file = temp_dir.path().join("test.json");
    fs::write(&dummy_file, "{}").unwrap();

    let result = handle_import(
        "unsupported".to_string(),
        dummy_file,
        temp_dir.path().join("output"),
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported import tool"));
}

#[tokio::test]
async fn test_nonexistent_collection_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent.json");

    let result = handle_import(
        "postman".to_string(),
        nonexistent,
        temp_dir.path().join("output"),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_invalid_postman_collection() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_file = temp_dir.path().join("invalid.json");
    fs::write(&invalid_file, "invalid json content").unwrap();

    let result = handle_import(
        "postman".to_string(),
        invalid_file,
        temp_dir.path().join("output"),
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse Postman collection"));
}
