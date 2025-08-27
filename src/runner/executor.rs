use crate::config::{Request, Expectation, StatusExpectation};
use crate::runner::variables::VariableContext;
use anyhow::{Context, Result};
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::time::{Duration, Instant};
use url::Url;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
    #[allow(dead_code)]
    pub response_status: Option<u16>,
    #[allow(dead_code)]
    pub response_body: Option<String>,
}

pub struct RequestExecutor {
    client: Client,
}

impl RequestExecutor {
    pub fn new(timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self { client })
    }
    
    pub async fn execute_test(
        &self,
        name: &str,
        request: &Request,
        expectation: Option<&Expectation>,
        context: &VariableContext,
    ) -> TestResult {
        let start_time = Instant::now();
        
        match self.execute_request(request, context).await {
            Ok(response) => {
                let duration = start_time.elapsed();
                let status = response.status().as_u16();
                
                match response.text().await {
                    Ok(body) => {
                        if let Some(expect) = expectation {
                            match self.validate_response(status, &body, expect, context) {
                                Ok(()) => TestResult {
                                    name: name.to_string(),
                                    passed: true,
                                    duration,
                                    error: None,
                                    response_status: Some(status),
                                    response_body: Some(body.clone()),
                                },
                                Err(e) => TestResult {
                                    name: name.to_string(),
                                    passed: false,
                                    duration,
                                    error: Some(e.to_string()),
                                    response_status: Some(status),
                                    response_body: Some(body.clone()),
                                },
                            }
                        } else {
                            // No expectations, just check if request succeeded
                            TestResult {
                                name: name.to_string(),
                                passed: status < 400,
                                duration,
                                error: if status >= 400 {
                                    Some(format!("HTTP {}", status))
                                } else {
                                    None
                                },
                                response_status: Some(status),
                                response_body: Some(body),
                            }
                        }
                    }
                    Err(e) => TestResult {
                        name: name.to_string(),
                        passed: false,
                        duration: start_time.elapsed(),
                        error: Some(format!("Failed to read response body: {}", e)),
                        response_status: Some(status),
                        response_body: None,
                    },
                }
            }
            Err(e) => TestResult {
                name: name.to_string(),
                passed: false,
                duration: start_time.elapsed(),
                error: Some(e.to_string()),
                response_status: None,
                response_body: None,
            },
        }
    }
    
    async fn execute_request(
        &self,
        request: &Request,
        context: &VariableContext,
    ) -> Result<Response> {
        // Substitute variables in URL
        let url_str = context.substitute_variables(&request.url);
        let mut url = Url::parse(&url_str)
            .with_context(|| format!("Invalid URL: {}", url_str))?;
        
        // Add query parameters
        if let Some(params) = &request.params {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                let key = context.substitute_variables(key);
                let value = context.substitute_variables(value);
                query_pairs.append_pair(&key, &value);
            }
        }
        
        // Parse method
        let method = request.method.parse::<Method>()
            .with_context(|| format!("Invalid HTTP method: {}", request.method))?;
        
        // Build request
        let mut req_builder = self.client.request(method, url);
        
        // Add headers
        if let Some(headers) = &request.headers {
            for (key, value) in headers {
                let key = context.substitute_variables(key);
                let value = context.substitute_variables(value);
                req_builder = req_builder.header(key, value);
            }
        }
        
        // Add body
        if let Some(body) = &request.body {
            let body = context.substitute_variables(body);
            req_builder = req_builder.body(body);
        }
        
        // Execute request
        let response = req_builder.send().await
            .context("Failed to send HTTP request")?;
        
        Ok(response)
    }
    
    fn validate_response(
        &self,
        status: u16,
        body: &str,
        expectation: &Expectation,
        context: &VariableContext,
    ) -> Result<()> {
        // Validate status code
        if let Some(expected_status) = &expectation.status {
            let expected_code = match expected_status {
                StatusExpectation::Number(code) => *code,
                StatusExpectation::String(code_str) => {
                    let substituted = context.substitute_variables(code_str);
                    substituted.parse::<u16>()
                        .with_context(|| format!("Invalid status code: {}", substituted))?
                }
            };
            
            if status != expected_code {
                anyhow::bail!(
                    "Expected status {} but got {}",
                    expected_code,
                    status
                );
            }
        }
        
        // Validate headers (we'd need response headers for this - skipping for now)
        // if let Some(expected_headers) = &expectation.headers {
        //     // TODO: Validate headers
        // }
        
        // Validate JSON path assertions
        if let Some(jsonpath_assertions) = &expectation.jsonpath {
            // Try to parse as JSON
            let json_value: Value = serde_json::from_str(body)
                .context("Response body is not valid JSON")?;
            
            for (path, expected_value) in jsonpath_assertions {
                self.validate_jsonpath(&json_value, path, expected_value, context)?;
            }
        }
        
        Ok(())
    }
    
    fn validate_jsonpath(
        &self,
        json: &Value,
        path: &str,
        expected: &Value,
        context: &VariableContext,
    ) -> Result<()> {
        // Simple JSONPath implementation for basic cases
        let actual_value = self.extract_jsonpath_value(json, path)?;
        
        // Handle variable substitution in expected values
        let expected_value = match expected {
            Value::String(s) => {
                let substituted = context.substitute_variables(s);
                // Try to parse as number or boolean if it looks like one
                if let Ok(num) = substituted.parse::<i64>() {
                    Value::Number(num.into())
                } else if let Ok(b) = substituted.parse::<bool>() {
                    Value::Bool(b)
                } else {
                    Value::String(substituted)
                }
            }
            other => other.clone(),
        };
        
        if actual_value != expected_value {
            anyhow::bail!(
                "JSONPath assertion failed for '{}': expected {:?} but got {:?}",
                path,
                expected_value,
                actual_value
            );
        }
        
        Ok(())
    }
    
    fn extract_jsonpath_value(&self, json: &Value, path: &str) -> Result<Value> {
        // Simple JSONPath implementation supporting basic syntax
        let mut current = json;
        
        // Handle root array access like "$[0].userId"
        if path.starts_with("$[") {
            let remaining = path.strip_prefix("$").unwrap();
            if let Some(bracket_end) = remaining.find(']') {
                let index_str = &remaining[1..bracket_end];
                let index: usize = index_str.parse()
                    .with_context(|| format!("Invalid array index: {}", index_str))?;
                
                current = current.get(index).ok_or_else(|| {
                    anyhow::anyhow!("Array index {} not found", index)
                })?;
                
                let remaining_path = &remaining[bracket_end + 1..];
                if remaining_path.starts_with('.') {
                    return self.extract_jsonpath_value(current, remaining_path.strip_prefix('.').unwrap());
                } else {
                    return Ok(current.clone());
                }
            }
        }
        
        let parts = path.strip_prefix("$.").unwrap_or(path).split('.');
        
        for part in parts {
            if part.is_empty() {
                continue;
            }
            
            // Handle array indexing like "items[0]"
            if part.contains('[') && part.ends_with(']') {
                let (field, index_str) = part.split_once('[').unwrap();
                let index_str = index_str.strip_suffix(']').unwrap();
                
                if !field.is_empty() {
                    current = current.get(field).ok_or_else(|| {
                        anyhow::anyhow!("Field '{}' not found in JSON", field)
                    })?;
                }
                
                let index: usize = index_str.parse()
                    .with_context(|| format!("Invalid array index: {}", index_str))?;
                
                current = current.get(index).ok_or_else(|| {
                    anyhow::anyhow!("Array index {} not found", index)
                })?;
            } else {
                current = current.get(part).ok_or_else(|| {
                    anyhow::anyhow!("Field '{}' not found in JSON", part)
                })?;
            }
        }
        
        Ok(current.clone())
    }
}