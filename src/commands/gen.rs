use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Expectation, Request, RivetConfig, StatusExpectation, TestStep};

pub async fn handle_gen(spec: PathBuf, out: PathBuf) -> Result<()> {
    println!(
        "{} Generating tests from OpenAPI spec: {}",
        "→".cyan(),
        spec.display().to_string().bright_white()
    );
    println!("{} Output directory: {}", "→".cyan(), out.display());

    generate_openapi_tests(spec, out).await?;

    Ok(())
}

async fn generate_openapi_tests(spec_path: PathBuf, out: PathBuf) -> Result<()> {
    println!("{} Reading OpenAPI specification...", "→".cyan());

    if !spec_path.exists() {
        return Err(anyhow!(
            "OpenAPI spec file does not exist: {}",
            spec_path.display()
        ));
    }

    let spec_content = fs::read_to_string(&spec_path)?;

    // Try to parse as YAML first, then JSON
    let spec: openapiv3::OpenAPI = if spec_path.extension().and_then(|s| s.to_str()) == Some("json")
    {
        serde_json::from_str(&spec_content)
            .map_err(|e| anyhow!("Failed to parse OpenAPI JSON: {}", e))?
    } else {
        serde_yaml::from_str(&spec_content)
            .map_err(|e| anyhow!("Failed to parse OpenAPI YAML: {}", e))?
    };

    println!(
        "{} Parsed OpenAPI spec: {} v{}",
        "✓".green(),
        spec.info.title.bright_white(),
        spec.info.version.bright_white()
    );

    // Create output directory
    fs::create_dir_all(&out)?;

    let mut test_count = 0;
    let mut endpoint_count = 0;

    // Extract base URL from servers
    let base_url = if let Some(server) = spec.servers.first() {
        server.url.clone()
    } else {
        "https://api.example.com".to_string()
    };

    // Generate tests for each path and operation
    for (path, path_item) in &spec.paths.paths {
        if let openapiv3::ReferenceOr::Item(path_item) = path_item {
            endpoint_count += 1;

            // Generate test for each HTTP method
            let operations = [
                ("GET", &path_item.get),
                ("POST", &path_item.post),
                ("PUT", &path_item.put),
                ("DELETE", &path_item.delete),
                ("PATCH", &path_item.patch),
                ("HEAD", &path_item.head),
                ("OPTIONS", &path_item.options),
                ("TRACE", &path_item.trace),
            ];

            for (method, operation_ref) in operations {
                if let Some(operation) = operation_ref {
                    generate_test_for_operation(
                        method,
                        path,
                        operation,
                        &base_url,
                        &out,
                        &mut test_count,
                    )
                    .await?;
                }
            }
        }
    }

    // Create main rivet config file
    let main_config = RivetConfig {
        name: format!("{} Tests", spec.info.title),
        description: spec.info.description.clone(),
        env: None,
        vars: Some({
            let mut vars = HashMap::new();
            vars.insert("baseUrl".to_string(), base_url);
            vars
        }),
        setup: None,
        tests: vec![], // Individual test files will be loaded
        dataset: None,
        teardown: None,
    };

    let config_path = out.join("rivet.yaml");
    let config_yaml = serde_yaml::to_string(&main_config)?;
    fs::write(&config_path, config_yaml)?;

    println!(
        "\n{} Test generation completed successfully!",
        "✓".green().bold()
    );
    println!(
        "  {} tests generated from {} endpoints",
        test_count.to_string().bright_white(),
        endpoint_count.to_string().bright_white()
    );
    println!(
        "  Main config: {}",
        config_path.display().to_string().bright_blue()
    );

    Ok(())
}

async fn generate_test_for_operation(
    method: &str,
    path: &str,
    operation: &openapiv3::Operation,
    base_url: &str,
    out_dir: &Path,
    test_count: &mut usize,
) -> Result<()> {
    let operation_id = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| format!("{}_{}", method.to_lowercase(), sanitize_path(path)));

    let summary = operation
        .summary
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    // Construct full URL
    let full_url = if path.starts_with('/') {
        format!("{}{}", base_url, path)
    } else {
        format!("{}/{}", base_url, path)
    };

    // Extract headers from parameters
    let mut headers = HashMap::new();
    let mut query_params = HashMap::new();

    for param_ref in &operation.parameters {
        if let openapiv3::ReferenceOr::Item(param) = param_ref {
            let param_name = &param.clone().parameter_data().name;
            match param {
                openapiv3::Parameter::Header { .. } => {
                    headers.insert(param_name.clone(), "{{headerValue}}".to_string());
                }
                openapiv3::Parameter::Query { .. } => {
                    query_params.insert(param_name.clone(), "{{queryValue}}".to_string());
                }
                openapiv3::Parameter::Path { .. } => {
                    // Path parameters are handled in URL substitution
                }
                openapiv3::Parameter::Cookie { .. } => {
                    // Handle cookies if needed
                }
            }
        }
    }

    // Generate request body for POST/PUT/PATCH
    let body = if matches!(method, "POST" | "PUT" | "PATCH") {
        if let Some(openapiv3::ReferenceOr::Item(body)) = &operation.request_body {
            // Convert IndexMap to HashMap
            let content_map: HashMap<String, openapiv3::MediaType> = body
                .content
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            generate_example_body(&content_map).await
        } else {
            None
        }
    } else {
        None
    };

    // Create rivet request
    let rivet_request = Request {
        method: method.to_string(),
        url: full_url,
        headers: if headers.is_empty() {
            None
        } else {
            Some(headers)
        },
        params: if query_params.is_empty() {
            None
        } else {
            Some(query_params)
        },
        body,
    };

    // Generate expectations based on responses
    let expectation = generate_expectation_from_responses(&operation.responses).await;

    let test_step = TestStep {
        name: summary.clone(),
        description: operation.description.clone(),
        request: rivet_request,
        expect: expectation,
    };

    let test_config = RivetConfig {
        name: summary,
        description: operation.description.clone(),
        env: None,
        vars: None,
        setup: None,
        tests: vec![test_step],
        dataset: None,
        teardown: None,
    };

    // Write test file
    let filename = format!("{}.yaml", sanitize_filename(&operation_id));
    let test_path = out_dir.join(filename);
    let test_yaml = serde_yaml::to_string(&test_config)?;
    fs::write(&test_path, test_yaml)?;

    println!("  {} Created: {}", "✓".green(), test_path.display());
    *test_count += 1;

    Ok(())
}

async fn generate_example_body(content: &HashMap<String, openapiv3::MediaType>) -> Option<String> {
    // Look for JSON content type first
    if let Some(json_content) = content.get("application/json") {
        if let Some(example) = &json_content.example {
            return Some(serde_json::to_string_pretty(example).unwrap_or_default());
        }

        // Generate example from schema if available
        if let Some(openapiv3::ReferenceOr::Item(schema)) = &json_content.schema {
            return generate_example_from_schema(schema).await;
        }
    }

    // Fallback to any content type
    for (content_type, media_type) in content {
        if let Some(example) = &media_type.example {
            if content_type.contains("json") {
                return Some(serde_json::to_string_pretty(example).unwrap_or_default());
            } else {
                return Some(example.to_string());
            }
        }
    }

    None
}

async fn generate_example_from_schema(schema: &openapiv3::Schema) -> Option<String> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(openapiv3::Type::Object(obj)) => {
            let mut example = serde_json::Map::new();

            for (prop_name, prop_schema) in &obj.properties {
                if let openapiv3::ReferenceOr::Item(prop_schema) = prop_schema {
                    if let Some(prop_example) = generate_simple_example_value(prop_schema).await {
                        example.insert(prop_name.clone(), prop_example);
                    }
                }
            }

            Some(
                serde_json::to_string_pretty(&serde_json::Value::Object(example))
                    .unwrap_or_default(),
            )
        }
        _ => generate_simple_example_value(schema)
            .await
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default()),
    }
}

async fn generate_simple_example_value(schema: &openapiv3::Schema) -> Option<serde_json::Value> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(schema_type) => match schema_type {
            openapiv3::Type::String(_) => Some(serde_json::Value::String("example".to_string())),
            openapiv3::Type::Number(_) => {
                Some(serde_json::Value::Number(serde_json::Number::from(42)))
            }
            openapiv3::Type::Integer(_) => {
                Some(serde_json::Value::Number(serde_json::Number::from(42)))
            }
            openapiv3::Type::Boolean(_) => Some(serde_json::Value::Bool(true)),
            openapiv3::Type::Array(_) => {
                Some(serde_json::Value::Array(vec![serde_json::Value::String(
                    "example".to_string(),
                )]))
            }
            openapiv3::Type::Object(_) => Some(serde_json::Value::Object(serde_json::Map::new())),
        },
        _ => None,
    }
}

async fn generate_expectation_from_responses(
    responses: &openapiv3::Responses,
) -> Option<Expectation> {
    // Look for 2xx responses first
    for (status_code, _response) in &responses.responses {
        match status_code {
            openapiv3::StatusCode::Code(code) => {
                if *code >= 200 && *code < 300 {
                    return Some(Expectation {
                        status: Some(StatusExpectation::Number(*code)),
                        schema: None,
                        jsonpath: None,
                        headers: None,
                    });
                }
            }
            openapiv3::StatusCode::Range(range) => {
                // Handle range like "2XX"
                if range == &2 {
                    return Some(Expectation {
                        status: Some(StatusExpectation::Number(200)),
                        schema: None,
                        jsonpath: None,
                        headers: None,
                    });
                }
            }
        }
    }

    // Check for default response
    if responses.default.is_some() {
        return Some(Expectation {
            status: Some(StatusExpectation::Number(200)),
            schema: None,
            jsonpath: None,
            headers: None,
        });
    }

    // Default to 200
    Some(Expectation {
        status: Some(StatusExpectation::Number(200)),
        schema: None,
        jsonpath: None,
        headers: None,
    })
}

fn sanitize_path(path: &str) -> String {
    path.replace('/', "_")
        .replace(['{', '}'], "")
        .trim_matches('_')
        .to_lowercase()
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
