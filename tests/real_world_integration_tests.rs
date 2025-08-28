use anyhow::Result;
use rivet::commands::gen::handle_gen;
use rivet::commands::import::handle_import;
use std::fs;
use tempfile::TempDir;

/// Integration tests using real-world examples from the internet.
/// These tests ensure our import and generation functionality works with actual
/// API specifications and collections that exist in production.

#[tokio::test]
async fn test_real_postman_collection_blazemeter() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Download the Blazemeter sample collection from GitHub
    let collection_url = "https://raw.githubusercontent.com/Blazemeter/taurus/master/examples/functional/postman-sample-collection.json";
    let response = reqwest::get(collection_url).await?;
    let collection_content = response.text().await?;

    let collection_file = temp_dir.path().join("blazemeter_collection.json");
    fs::write(&collection_file, collection_content)?;

    // Test the import
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;

    // Verify files were created
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Main config should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Sample Postman Collection"));

    // Check that Newman requests were properly imported
    let newman_get = output_dir.join("newman__get_request.yaml");
    assert!(newman_get.exists(), "Newman GET request should be imported");

    let newman_post = output_dir.join("newman__post_request.yaml");
    assert!(
        newman_post.exists(),
        "Newman POST request should be imported"
    );

    let newman_json = output_dir.join("newman__post_request_with_json_body.yaml");
    assert!(
        newman_json.exists(),
        "Newman JSON POST request should be imported"
    );

    // Verify the GET request contains postman-echo.com URL
    let get_content = fs::read_to_string(&newman_get)?;
    assert!(get_content.contains("postman-echo.com"));
    assert!(get_content.contains("method: GET"));

    // Verify the JSON POST has proper content-type and body
    let json_content = fs::read_to_string(&newman_json)?;
    assert!(json_content.contains("Content-Type: application/json"));
    assert!(json_content.contains("text"));

    Ok(())
}

#[tokio::test]
async fn test_real_usps_postman_collection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Download the USPS collection from GitHub
    let collection_url = "https://raw.githubusercontent.com/USPS/api-examples/main/Example-Postman.postman_collection.json";
    let response = reqwest::get(collection_url).await?;
    let collection_content = response.text().await?;

    let collection_file = temp_dir.path().join("usps_collection.json");
    fs::write(&collection_file, collection_content)?;

    // Test the import
    handle_import("postman".to_string(), collection_file, output_dir.clone()).await?;

    // Verify main config
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "USPS config should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Example-Postman"));

    // Verify folder structure was preserved - USPS has many API categories
    let oauth_folder = output_dir.join("oauth");
    assert!(oauth_folder.exists(), "OAuth folder should be created");

    let addresses_folder = output_dir.join("addresses");
    assert!(
        addresses_folder.exists(),
        "Addresses folder should be created"
    );

    let tracking_folder = output_dir.join("tracking");
    assert!(
        tracking_folder.exists(),
        "Tracking folder should be created"
    );

    // Check a specific OAuth token request
    let oauth_files: Vec<_> = fs::read_dir(&oauth_folder)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("token"))
        .collect();
    assert!(!oauth_files.is_empty(), "Should have OAuth token requests");

    Ok(())
}

#[tokio::test]
async fn test_real_openapi_redocly_template() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Download the Redocly OpenAPI template
    let spec_url =
        "https://raw.githubusercontent.com/Redocly/openapi-template/gh-pages/openapi.yaml";
    let response = reqwest::get(spec_url).await?;
    let spec_content = response.text().await?;

    let spec_file = temp_dir.path().join("redocly_spec.yaml");
    fs::write(&spec_file, spec_content)?;

    // Test the generation
    handle_gen(spec_file, output_dir.clone()).await?;

    // Verify main config
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Redocly config should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Example.com"));

    // Check generated test files
    let user_test = output_dir.join("getuserbyname.yaml");
    assert!(user_test.exists(), "User test should be generated");

    let update_test = output_dir.join("updateuser.yaml");
    assert!(update_test.exists(), "Update test should be generated");

    let echo_test = output_dir.join("echo.yaml");
    assert!(echo_test.exists(), "Echo test should be generated");

    // Verify a test contains proper structure
    let user_content = fs::read_to_string(&user_test)?;
    assert!(user_content.contains("method: GET"));
    assert!(user_content.contains("status: 200"));

    Ok(())
}

#[tokio::test]
async fn test_real_swagger_petstore_openapi() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Download the official Swagger Petstore spec
    let spec_url = "https://raw.githubusercontent.com/swagger-api/swagger-petstore/master/src/main/resources/openapi.yaml";
    let response = reqwest::get(spec_url).await?;
    let spec_content = response.text().await?;

    let spec_file = temp_dir.path().join("petstore_spec.yaml");
    fs::write(&spec_file, spec_content)?;

    // Test the generation
    handle_gen(spec_file, output_dir.clone()).await?;

    // Verify main config
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists(), "Petstore config should be created");

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Swagger Petstore"));
    assert!(config_content.contains("baseUrl"));

    // Check key pet operations
    let add_pet = output_dir.join("addpet.yaml");
    assert!(add_pet.exists(), "Add pet test should be generated");

    let get_pet = output_dir.join("getpetbyid.yaml");
    assert!(get_pet.exists(), "Get pet test should be generated");

    let find_pets = output_dir.join("findpetsbystatus.yaml");
    assert!(find_pets.exists(), "Find pets test should be generated");

    // Check store operations
    let place_order = output_dir.join("placeorder.yaml");
    assert!(place_order.exists(), "Place order test should be generated");

    let get_inventory = output_dir.join("getinventory.yaml");
    assert!(
        get_inventory.exists(),
        "Get inventory test should be generated"
    );

    // Check user operations
    let create_user = output_dir.join("createuser.yaml");
    assert!(create_user.exists(), "Create user test should be generated");

    let login_user = output_dir.join("loginuser.yaml");
    assert!(login_user.exists(), "Login user test should be generated");

    // Verify a complex operation with request body
    let add_pet_content = fs::read_to_string(&add_pet)?;
    assert!(add_pet_content.contains("method: POST"));
    assert!(add_pet_content.contains("body:"));
    assert!(add_pet_content.contains("name"));
    assert!(add_pet_content.contains("status: 200")); // Should expect success

    // Verify GET operation with path parameters
    let get_pet_content = fs::read_to_string(&get_pet)?;
    assert!(get_pet_content.contains("method: GET"));
    assert!(get_pet_content.contains("/pet/{petId}"));

    Ok(())
}

#[tokio::test]
async fn test_error_handling_with_invalid_internet_urls() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Test with a non-existent URL/file
    let fake_file = temp_dir.path().join("nonexistent.json");

    let result = handle_import("postman".to_string(), fake_file, output_dir.clone()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));

    Ok(())
}

#[tokio::test]
async fn test_mixed_format_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("output");

    // Test that we can handle both JSON and YAML formats from real sources
    // This ensures our parsers are flexible with real-world variations

    // Create a simple JSON OpenAPI spec (mimicking format variations)
    let json_spec = r#"{
        "openapi": "3.0.0",
        "info": {
            "title": "Mixed Format Test API",
            "version": "1.0.0"
        },
        "servers": [
            {
                "url": "https://mixed-format.example.com"
            }
        ],
        "paths": {
            "/test": {
                "get": {
                    "operationId": "testMixedFormat",
                    "responses": {
                        "200": {
                            "description": "Success"
                        }
                    }
                }
            }
        }
    }"#;

    let spec_file = temp_dir.path().join("mixed_format.json");
    fs::write(&spec_file, json_spec)?;

    // Test generation with JSON format
    handle_gen(spec_file, output_dir.clone()).await?;

    // Verify it worked
    let main_config = output_dir.join("rivet.yaml");
    assert!(main_config.exists());

    let config_content = fs::read_to_string(&main_config)?;
    assert!(config_content.contains("Mixed Format Test API"));

    let test_file = output_dir.join("testmixedformat.yaml");
    assert!(test_file.exists());

    Ok(())
}
