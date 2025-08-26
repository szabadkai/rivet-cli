use anyhow::Result;
use owo_colors::OwoColorize;
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;

use crate::ui::response_box;
use crate::utils::{parse_headers, parse_timeout};

pub async fn handle_send(
    method: String,
    url: String,
    headers: Vec<String>,
    data: Option<String>,
    save: Option<PathBuf>,
    insecure: bool,
    timeout: String,
) -> Result<()> {
    let start = Instant::now();
    
    // Build HTTP client
    let mut client_builder = Client::builder()
        .timeout(parse_timeout(&timeout)?);
    
    if insecure {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    
    let client = client_builder.build()?;
    
    // Parse headers
    let parsed_headers = parse_headers(&headers)?;
    
    // Build request
    let mut request = client.request(
        method.parse()?,
        &url,
    );
    
    for (key, value) in parsed_headers {
        request = request.header(key, value);
    }
    
    if let Some(ref body) = data {
        request = request.body(body.clone());
    }
    
    // Show request box
    println!("╭─ Request ──────────────────────────────╮");
    println!("│ {} {}   │", method.bright_green(), url);
    for header_str in &headers {
        let display_header = if header_str.to_lowercase().contains("authorization") {
            let parts: Vec<&str> = header_str.splitn(2, ':').collect();
            if parts.len() == 2 {
                format!("{}: ****", parts[0].trim())
            } else {
                header_str.clone()
            }
        } else {
            header_str.clone()
        };
        println!("│ {}             │", display_header);
    }
    println!("╰────────────────────────────────────────╯");
    println!();
    
    // Send request
    let response = request.send().await?;
    let duration = start.elapsed();
    
    // Get response info
    let status = response.status();
    let status_text = format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
    let response_headers = response.headers().clone();
    let body_bytes = response.bytes().await?;
    let body_size = body_bytes.len();
    
    // Show response box
    response_box::print_response_box(&status_text, duration, body_size, &response_headers);
    
    // Pretty print JSON if possible
    if let Ok(json_value) = serde_json::from_slice::<Value>(&body_bytes) {
        let pretty_json = serde_json::to_string_pretty(&json_value)?;
        for (i, line) in pretty_json.lines().enumerate() {
            println!("{:>3}  {}", (i + 1).to_string().dimmed(), line);
        }
    } else {
        // Print as text
        if let Ok(text) = String::from_utf8(body_bytes.to_vec()) {
            println!("{}", text);
        } else {
            println!("{}", "[Binary data]".dimmed());
        }
    }
    
    // Save request file if requested
    if let Some(save_path) = save {
        save_request_file(&save_path, &method, &url, &headers, &data).await?;
        println!("\n{} Saved request to {}", "✓".green(), save_path.display());
    }
    
    Ok(())
}

async fn save_request_file(
    path: &PathBuf,
    method: &str,
    url: &str,
    headers: &[String],
    data: &Option<String>,
) -> Result<()> {
    let mut request_yaml = format!(
        "request:\n  method: {}\n  url: \"{}\"\n",
        method, url
    );
    
    if !headers.is_empty() {
        request_yaml.push_str("  headers:\n");
        for header in headers {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                request_yaml.push_str(&format!(
                    "    {}: \"{}\"\n",
                    parts[0].trim(),
                    parts[1].trim()
                ));
            }
        }
    }
    
    if let Some(body) = data {
        request_yaml.push_str(&format!("  body: \"{}\"\n", body));
    }
    
    tokio::fs::write(path, request_yaml).await?;
    Ok(())
}