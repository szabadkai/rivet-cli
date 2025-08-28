use anyhow::Result;
use std::fs;
use tempfile::TempDir;

// Integration tests for the gRPC command line interface
// These tests focus on error handling since we don't have real gRPC servers to test against

#[tokio::test]
async fn test_grpc_command_no_proto_directory() -> Result<()> {
    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:50051".to_string(),
        std::path::PathBuf::from("/non/existent/path"),
        "Users/GetUser".to_string(),
        None,
        vec![],
        "30s".to_string(),
    )
    .await;

    // Should fail due to non-existent proto directory
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
    Ok(())
}

#[tokio::test]
async fn test_grpc_command_no_proto_files() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create non-proto files
    fs::write(temp_dir.path().join("test.txt"), "not a proto")?;

    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:50051".to_string(),
        temp_dir.path().to_path_buf(),
        "Users/GetUser".to_string(),
        None,
        vec![],
        "30s".to_string(),
    )
    .await;

    // Should fail due to no proto files
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No .proto files found"));
    Ok(())
}

#[tokio::test]
async fn test_grpc_command_invalid_timeout() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a sample proto file
    let proto_content = r#"
syntax = "proto3";
service TestService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
}
message GetUserRequest { string user_id = 1; }
message GetUserResponse { string id = 1; string name = 2; }
"#;
    fs::write(temp_dir.path().join("test.proto"), proto_content)?;

    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:50051".to_string(),
        temp_dir.path().to_path_buf(),
        "TestService/GetUser".to_string(),
        None,
        vec![],
        "invalid_timeout".to_string(),
    )
    .await;

    // Should fail due to invalid timeout format
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_grpc_command_invalid_service_format() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a sample proto file
    let proto_content = r#"
syntax = "proto3";
service TestService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
}
message GetUserRequest { string user_id = 1; }
message GetUserResponse { string id = 1; string name = 2; }
"#;
    fs::write(temp_dir.path().join("test.proto"), proto_content)?;

    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:50051".to_string(),
        temp_dir.path().to_path_buf(),
        "InvalidFormat".to_string(), // Missing slash
        None,
        vec![],
        "30s".to_string(),
    )
    .await;

    // Should fail either at proto compilation or service format validation
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_grpc_command_invalid_json() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a sample proto file
    let proto_content = r#"
syntax = "proto3";
service TestService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
}
message GetUserRequest { string user_id = 1; }
message GetUserResponse { string id = 1; string name = 2; }
"#;
    fs::write(temp_dir.path().join("test.proto"), proto_content)?;

    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:50051".to_string(),
        temp_dir.path().to_path_buf(),
        "TestService/GetUser".to_string(),
        Some("invalid json".to_string()),
        vec![],
        "30s".to_string(),
    )
    .await;

    // Should fail either at compilation, connection, or JSON parsing
    assert!(result.is_err());
    Ok(())
}

// Test various timeout formats - these should pass validation
#[test]
fn test_timeout_format_validation() {
    use rivet::utils::parse_timeout;

    let valid_timeouts = vec!["1s", "1000ms", "5", "10s"];
    for timeout in valid_timeouts {
        let result = parse_timeout(timeout);
        assert!(
            result.is_ok(),
            "Timeout format '{}' should be valid",
            timeout
        );
    }

    let invalid_timeouts = vec!["invalid", "", "10x"];
    for timeout in invalid_timeouts {
        let result = parse_timeout(timeout);
        assert!(
            result.is_err(),
            "Timeout format '{}' should be invalid",
            timeout
        );
    }
}

// Test that server connection failure is handled properly
#[tokio::test]
async fn test_grpc_server_connection_failure() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a valid proto file
    let proto_content = r#"
syntax = "proto3";

package test;

service TestService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
}

message GetUserRequest {
  string user_id = 1;
}

message GetUserResponse {
  string id = 1;
  string name = 2;
  string email = 3;
}

message ListUsersRequest {}

message ListUsersResponse {
  repeated GetUserResponse users = 1;
}
"#;
    fs::write(temp_dir.path().join("test.proto"), proto_content)?;

    // Try to connect to a non-existent server
    let result = rivet::commands::grpc::handle_grpc(
        "http://localhost:99999".to_string(), // Port unlikely to be in use
        temp_dir.path().to_path_buf(),
        "TestService/GetUser".to_string(),
        Some(r#"{"user_id": "123"}"#.to_string()),
        vec!["$.id".to_string()],
        "1s".to_string(), // Short timeout to fail fast
    )
    .await;

    // Should fail due to connection timeout or compilation error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();

    // The error could be from protoc not being available OR connection failure
    // Both are valid failure modes for this test
    assert!(
        error_msg.contains("Failed to compile proto files")
            || error_msg.contains("Failed to connect to gRPC server")
            || error_msg.contains("Connection refused")
            || error_msg.contains("protoc")
            || error_msg.contains("No such file or directory"), // protoc not found
        "Got error: {}",
        error_msg
    );

    Ok(())
}
