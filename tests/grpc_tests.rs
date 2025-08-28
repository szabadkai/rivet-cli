use rivet::utils::parse_timeout;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

// Test utility functions without requiring a full gRPC client

#[test]
fn test_parse_timeout_formats() {
    assert_eq!(parse_timeout("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(
        parse_timeout("5000ms").unwrap(),
        Duration::from_millis(5000)
    );
    assert_eq!(parse_timeout("10").unwrap(), Duration::from_secs(10));
    assert_eq!(parse_timeout("0s").unwrap(), Duration::from_secs(0));
    assert_eq!(parse_timeout("1ms").unwrap(), Duration::from_millis(1));
}

#[test]
fn test_parse_timeout_invalid() {
    assert!(parse_timeout("invalid").is_err());
    assert!(parse_timeout("").is_err());
    assert!(parse_timeout("30x").is_err());
}

#[test]
fn test_service_method_format_validation() {
    fn validate_format(service_method: &str) -> bool {
        let parts: Vec<&str> = service_method.split('/').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
    }

    // Valid formats
    let valid_calls = vec![
        "Users/GetUser",
        "Products/ListProducts",
        "Auth/Login",
        "Service.Name/Method_Name",
    ];

    for call in valid_calls {
        assert!(validate_format(call), "Expected '{}' to be valid", call);
    }

    // Invalid formats
    let invalid_calls = vec![
        "InvalidFormat",
        "Too/Many/Slashes",
        "/MissingService",
        "MissingMethod/",
        "",
    ];

    for call in invalid_calls {
        assert!(!validate_format(call), "Expected '{}' to be invalid", call);
    }
}

#[test]
fn test_jsonpath_expectations() {
    use jsonpath_rust::JsonPathFinder;
    use serde_json::json;

    let response = json!({
        "user": {
            "id": "123",
            "name": "John Doe",
            "profile": {
                "email": "john@example.com"
            }
        },
        "status": "success",
        "count": 42
    });

    let response_str = serde_json::to_string(&response).unwrap();

    // Test successful paths
    let success_paths = vec!["$.user.id", "$.user.name", "$.status", "$.count"];
    for path in success_paths {
        let finder = JsonPathFinder::from_str(&response_str, path).unwrap();
        let found = finder.find();
        assert_ne!(
            found,
            serde_json::Value::Null,
            "Expected to find path: {}",
            path
        );
    }

    // Test failed paths
    let fail_paths = vec!["$.user.nonexistent", "$.missing.field"];
    for path in fail_paths {
        let finder = JsonPathFinder::from_str(&response_str, path).unwrap();
        let found = finder.find();
        assert_eq!(
            found,
            serde_json::Value::Null,
            "Expected path to fail: {}",
            path
        );
    }
}

#[test]
fn test_proto_directory_validation() {
    // Test non-existent directory
    let non_existent = std::path::Path::new("/non/existent/path");
    assert!(!non_existent.exists());

    // Test directory with no proto files
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "not a proto file").unwrap();

    // Find proto files (should be none)
    let proto_files: Vec<_> = walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|s| s.to_str()) == Some("proto")
        })
        .collect();

    assert!(proto_files.is_empty(), "Should find no proto files");
}

#[test]
fn test_create_sample_proto_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a sample proto file
    let proto_content = r#"
syntax = "proto3";

package test;

service TestService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
}

message GetUserRequest {
  string user_id = 1;
}

message GetUserResponse {
  string id = 1;
  string name = 2;
  string email = 3;
}
"#;

    let proto_file = temp_dir.path().join("test.proto");
    fs::write(&proto_file, proto_content).unwrap();

    assert!(proto_file.exists());
    let content = fs::read_to_string(&proto_file).unwrap();
    assert!(content.contains("service TestService"));
    assert!(content.contains("rpc GetUser"));
}

// Test that protoc is available (if installed)
#[test]
fn test_protoc_availability() {
    use std::process::Command;

    let output = Command::new("protoc").arg("--version").output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("protoc is available: {}", version.trim());
            } else {
                println!("protoc command failed");
            }
        }
        Err(_) => {
            println!("protoc is not available - this is expected in CI/test environments");
        }
    }

    // This test always passes - it's just informational
    assert!(true);
}
