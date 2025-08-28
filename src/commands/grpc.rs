use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::grpc::GrpcClient;
use crate::utils::parse_timeout;

pub async fn handle_grpc(
    server: String,
    proto: PathBuf,
    call: String,
    data: Option<String>,
    expect_jsonpath: Vec<String>,
    timeout: String,
) -> Result<()> {
    println!("{} Making gRPC call: {}", "→".cyan(), call.bright_white());
    println!("{} gRPC server: {}", "→".cyan(), server.bright_blue());
    println!("{} Proto directory: {}", "→".cyan(), proto.display());
    println!("{} Timeout: {}", "→".cyan(), timeout);

    if let Some(ref request_data) = data {
        println!("{} Request data: {}", "→".cyan(), request_data);
    }

    if !expect_jsonpath.is_empty() {
        println!("{} JSONPath expectations:", "→".cyan());
        for expectation in &expect_jsonpath {
            println!("  - {}", expectation.bright_yellow());
        }
    }

    // Parse timeout
    let timeout_duration = parse_timeout(&timeout)?;

    // Create gRPC client - this will now fail if server is unreachable
    println!("{} Compiling proto files...", "→".cyan());
    let mut client = GrpcClient::new(&proto, &server).await?;

    // Display available services and methods
    let services = client.list_services();
    if !services.is_empty() {
        println!("{} Available services: {}", "→".cyan(), services.join(", ").bright_green());
    }

    // Make the gRPC call
    println!("{} Calling {}...", "→".cyan(), call.bright_white());
    let response = client.call(&call, data.as_deref(), timeout_duration).await?;

    // Display response
    println!("\n{} Response:", "✓".green().bold());
    println!("{}", serde_json::to_string_pretty(&response)?.bright_white());

    // Validate expectations
    if !expect_jsonpath.is_empty() {
        println!("\n{} Validating expectations:", "→".cyan());
        let validation_results = client.validate_expectations(&response, &expect_jsonpath)?;
        for result in validation_results {
            println!("  {}", result);
        }
    }

    println!("\n{} gRPC call completed", "✓".green().bold());

    Ok(())
}
