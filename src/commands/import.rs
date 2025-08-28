use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Expectation, Request, RivetConfig, StatusExpectation, TestStep};

pub async fn handle_import(tool: String, file: PathBuf, out: PathBuf) -> Result<()> {
    println!(
        "{} Importing from {}: {}",
        "→".cyan(),
        tool,
        file.display().to_string().bright_white()
    );
    println!("{} Output directory: {}", "→".cyan(), out.display());

    match tool.to_lowercase().as_str() {
        "postman" => {
            import_postman_collection(file, out).await?;
        }
        "insomnia" => {
            println!("{} Insomnia importer not yet implemented", "✔".yellow());
        }
        "bruno" => {
            println!("{} Bruno importer not yet implemented", "✔".yellow());
        }
        "curl" => {
            println!("{} cURL importer not yet implemented", "✔".yellow());
        }
        _ => {
            anyhow::bail!("Unsupported import tool: {}", tool);
        }
    }

    Ok(())
}

// Postman Collection v2.1 data structures
#[derive(Debug, Deserialize)]
struct PostmanCollection {
    info: PostmanInfo,
    item: Vec<PostmanItem>,
    #[serde(default)]
    variables: Option<Vec<PostmanVariable>>, // Some use 'variables' instead of 'variable'
    #[serde(default)]
    variable: Option<Vec<PostmanVariable>>,
}

#[derive(Debug, Deserialize)]
struct PostmanInfo {
    name: String,
    description: Option<String>,
    #[allow(dead_code)] // Used for validation but not processing
    schema: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostmanItem {
    // Try folder first since it has the 'item' field that's distinctive
    Folder(PostmanFolderItem),
    Request(Box<PostmanRequestItem>),
}

#[derive(Debug, Deserialize)]
struct PostmanRequestItem {
    name: String,
    request: PostmanRequest,
    response: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    #[allow(dead_code)] // Reserved for future test script processing
    event: Option<Vec<serde_json::Value>>, // For test scripts
}

#[derive(Debug, Deserialize)]
struct PostmanFolderItem {
    name: String,
    #[allow(dead_code)] // Reserved for future folder description processing
    description: Option<String>,
    item: Vec<PostmanItem>,
}

#[derive(Debug, Deserialize)]
struct PostmanRequest {
    method: String,
    #[serde(default)]
    header: Option<Vec<PostmanHeader>>,
    url: PostmanUrl,
    #[serde(default)]
    body: Option<PostmanBody>,
    #[serde(default)]
    #[allow(dead_code)] // Reserved for future request description processing
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PostmanHeader {
    key: String,
    value: String,
    #[serde(default)]
    disabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostmanUrl {
    String(String),
    Object(PostmanUrlObject),
}

#[derive(Debug, Deserialize)]
struct PostmanUrlObject {
    raw: Option<String>,
    protocol: Option<String>,
    host: Option<Vec<String>>,
    path: Option<Vec<String>>,
    query: Option<Vec<PostmanQuery>>,
}

#[derive(Debug, Deserialize)]
struct PostmanQuery {
    key: String,
    value: String,
    disabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PostmanBody {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    raw: Option<String>,
    #[serde(default)]
    formdata: Option<Vec<PostmanFormData>>,
    #[serde(default)]
    urlencoded: Option<Vec<PostmanFormData>>,
}

#[derive(Debug, Deserialize)]
struct PostmanFormData {
    key: String,
    value: Option<String>, // Can be null
    #[serde(default)]
    disabled: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)] // Reserved for future form data type processing
    r#type: Option<String>, // "text", "file", etc.
}

#[derive(Debug, Deserialize)]
struct PostmanVariable {
    key: String,
    value: String,
}

async fn import_postman_collection(file: PathBuf, out: PathBuf) -> Result<()> {
    println!("{} Reading Postman collection...", "→".cyan());

    if !file.exists() {
        return Err(anyhow!(
            "Postman collection file does not exist: {}",
            file.display()
        ));
    }

    let contents = fs::read_to_string(&file)?;
    let collection: PostmanCollection = serde_json::from_str(&contents)
        .map_err(|e| anyhow!("Failed to parse Postman collection: {}", e))?;

    println!(
        "{} Parsed collection: {}",
        "✓".green(),
        collection.info.name.bright_white()
    );

    // Create output directory if it doesn't exist
    fs::create_dir_all(&out)?;

    let mut test_count = 0;
    let mut folder_count = 0;

    // Convert collection variables to rivet variables
    let mut variables = HashMap::new();
    // Check both 'variable' and 'variables' fields
    let vars_to_process = collection
        .variable
        .as_ref()
        .or(collection.variables.as_ref());
    if let Some(vars) = vars_to_process {
        for var in vars {
            variables.insert(var.key.clone(), var.value.clone());
        }
    }

    // Process all items (requests and folders)
    process_postman_items(
        &collection.item,
        &out,
        "",
        &mut test_count,
        &mut folder_count,
        &variables,
    )?;

    // Create a main rivet config file
    let main_config = RivetConfig {
        name: collection.info.name.clone(),
        description: collection.info.description.clone(),
        env: None,
        vars: if variables.is_empty() {
            None
        } else {
            Some(variables)
        },
        setup: None,
        tests: vec![], // Individual test files will be referenced
        dataset: None,
        teardown: None,
    };

    let config_path = out.join("rivet.yaml");
    let config_yaml = serde_yaml::to_string(&main_config)?;
    fs::write(&config_path, config_yaml)?;

    println!("\n{} Import completed successfully!", "✓".green().bold());
    println!(
        "  {} tests created in {} folders",
        test_count.to_string().bright_white(),
        folder_count.to_string().bright_white()
    );
    println!(
        "  Main config: {}",
        config_path.display().to_string().bright_blue()
    );

    Ok(())
}

fn process_postman_items(
    items: &[PostmanItem],
    base_path: &Path,
    folder_prefix: &str,
    test_count: &mut usize,
    folder_count: &mut usize,
    variables: &HashMap<String, String>,
) -> Result<()> {
    for item in items {
        match item {
            PostmanItem::Request(request_item) => {
                // Convert postman request synchronously
                let rivet_request =
                    convert_postman_request_to_rivet(&request_item.request, variables)?;
                let expectation = create_expectation_from_responses(&request_item.response);

                let request_name = sanitize_filename(&request_item.name);
                let filename = if folder_prefix.is_empty() {
                    format!("{}.yaml", request_name)
                } else {
                    format!("{}_{}.yaml", folder_prefix, request_name)
                };

                let test_step = TestStep {
                    name: request_item.name.clone(),
                    description: None,
                    request: rivet_request,
                    expect: expectation,
                };

                let test_config = RivetConfig {
                    name: request_item.name.clone(),
                    description: None,
                    env: None,
                    vars: None,
                    setup: None,
                    tests: vec![test_step],
                    dataset: None,
                    teardown: None,
                };

                let test_path = base_path.join(filename);
                let test_yaml = serde_yaml::to_string(&test_config)?;
                std::fs::write(&test_path, test_yaml)?;

                println!("  {} Created: {}", "✓".green(), test_path.display());
                *test_count += 1;
            }
            PostmanItem::Folder(folder_item) => {
                let folder_name = sanitize_filename(&folder_item.name);
                let folder_path = base_path.join(&folder_name);
                std::fs::create_dir_all(&folder_path)?;
                *folder_count += 1;

                let new_prefix = if folder_prefix.is_empty() {
                    folder_name
                } else {
                    format!("{}_{}", folder_prefix, folder_name)
                };

                process_postman_items(
                    &folder_item.item,
                    &folder_path,
                    &new_prefix,
                    test_count,
                    folder_count,
                    variables,
                )?;
            }
        }
    }
    Ok(())
}

fn convert_postman_request_to_rivet(
    postman_request: &PostmanRequest,
    _variables: &HashMap<String, String>,
) -> Result<Request> {
    // Convert URL
    let url = match &postman_request.url {
        PostmanUrl::String(url_str) => url_str.clone(),
        PostmanUrl::Object(url_obj) => {
            if let Some(raw) = &url_obj.raw {
                raw.clone()
            } else {
                // Reconstruct URL from parts
                let protocol = url_obj.protocol.as_deref().unwrap_or("https");
                let host = url_obj
                    .host
                    .as_ref()
                    .map(|h| h.join("."))
                    .unwrap_or_else(|| "localhost".to_string());
                let path = url_obj
                    .path
                    .as_ref()
                    .map(|p| "/".to_string() + &p.join("/"))
                    .unwrap_or_else(|| "/".to_string());

                let mut url = format!("{}://{}{}", protocol, host, path);

                // Add query parameters
                if let Some(query) = &url_obj.query {
                    let params: Vec<String> = query
                        .iter()
                        .filter(|q| !q.disabled.unwrap_or(false))
                        .map(|q| format!("{}={}", q.key, q.value))
                        .collect();
                    if !params.is_empty() {
                        url.push('?');
                        url.push_str(&params.join("&"));
                    }
                }
                url
            }
        }
    };

    // Convert headers
    let headers = if let Some(postman_headers) = &postman_request.header {
        let mut header_map = HashMap::new();
        for header in postman_headers {
            if !header.disabled.unwrap_or(false) {
                header_map.insert(header.key.clone(), header.value.clone());
            }
        }
        if header_map.is_empty() {
            None
        } else {
            Some(header_map)
        }
    } else {
        None
    };

    // Convert body
    let body = if let Some(postman_body) = &postman_request.body {
        match postman_body.mode.as_deref() {
            Some("raw") => postman_body.raw.clone(),
            Some("formdata") => {
                // Convert form data to form-encoded string
                if let Some(form_data) = &postman_body.formdata {
                    let params: Vec<String> = form_data
                        .iter()
                        .filter(|f| !f.disabled.unwrap_or(false))
                        .map(|f| format!("{}={}", f.key, f.value.as_deref().unwrap_or("")))
                        .collect();
                    if params.is_empty() {
                        None
                    } else {
                        Some(params.join("&"))
                    }
                } else {
                    None
                }
            }
            Some("urlencoded") => {
                // Convert URL encoded data
                if let Some(form_data) = &postman_body.urlencoded {
                    let params: Vec<String> = form_data
                        .iter()
                        .filter(|f| !f.disabled.unwrap_or(false))
                        .map(|f| format!("{}={}", f.key, f.value.as_deref().unwrap_or("")))
                        .collect();
                    if params.is_empty() {
                        None
                    } else {
                        Some(params.join("&"))
                    }
                } else {
                    None
                }
            }
            _ => {
                // If no mode or unknown mode, try to use raw if available
                postman_body.raw.clone()
            }
        }
    } else {
        None
    };

    Ok(Request {
        method: postman_request.method.to_uppercase(),
        url,
        headers,
        params: None, // Query params are included in URL
        body,
    })
}

fn create_expectation_from_responses(
    responses: &Option<Vec<serde_json::Value>>,
) -> Option<Expectation> {
    if let Some(response_examples) = responses {
        if let Some(first_response) = response_examples.first() {
            // Try to extract status code from response example
            if let Some(code) = first_response.get("code").and_then(|c| c.as_u64()) {
                return Some(Expectation {
                    status: Some(StatusExpectation::Number(code as u16)),
                    schema: None,
                    jsonpath: None,
                    headers: None,
                });
            }
        }
    }

    // Default expectation for successful requests
    Some(Expectation {
        status: Some(StatusExpectation::Number(200)),
        schema: None,
        jsonpath: None,
        headers: None,
    })
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            ' ' => '_',
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase()
}
