use anyhow::{anyhow, Result};
use jsonpath_rust::JsonPathFinder;
use prost::Message;
use prost_types::FileDescriptorSet;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
// tonic imports are available for potential future use
use walkdir::WalkDir;

pub struct GrpcClient {
    channel: Channel,
    descriptors: FileDescriptorSet,
    services: HashMap<String, ServiceInfo>,
}

#[derive(Debug, Clone)]
struct ServiceInfo {
    service_name: String,
    methods: HashMap<String, MethodInfo>,
}

#[derive(Debug, Clone)]
struct MethodInfo {
    method_name: String,
    input_type: String,
    output_type: String,
}

impl GrpcClient {
    pub async fn new(proto_path: &Path, endpoint: &str) -> Result<Self> {
        // Load and compile protobuf descriptors
        let descriptors = Self::compile_protos(proto_path)?;
        let services = Self::parse_services(&descriptors)?;
        
        // Create gRPC channel - fail if server is unreachable
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to gRPC server at {}: {}", endpoint, e))?;

        Ok(Self {
            channel,
            descriptors,
            services,
        })
    }

    fn compile_protos(proto_path: &Path) -> Result<FileDescriptorSet> {
        if !proto_path.exists() {
            return Err(anyhow!("Proto directory does not exist: {}", proto_path.display()));
        }

        // Find all .proto files
        let proto_files: Vec<_> = WalkDir::new(proto_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("proto"))
            .map(|e| e.path().to_path_buf())
            .collect();

        if proto_files.is_empty() {
            return Err(anyhow!("No .proto files found in directory: {}", proto_path.display()));
        }

        // Create temporary directory for compilation
        let temp_dir = tempfile::tempdir()?;
        let descriptor_path = temp_dir.path().join("descriptors.pb");

        // Use protoc to compile proto files to FileDescriptorSet
        let mut cmd = Command::new("protoc");
        cmd.arg("--descriptor_set_out")
           .arg(&descriptor_path)
           .arg("--include_imports")
           .arg("--include_source_info");

        // Add proto path
        cmd.arg(format!("--proto_path={}", proto_path.display()));

        // Add proto files
        for proto_file in &proto_files {
            cmd.arg(proto_file);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to compile proto files: {}", stderr));
        }

        // Read the compiled descriptor set
        let descriptor_bytes = fs::read(&descriptor_path)?;
        let descriptor_set = FileDescriptorSet::decode(&descriptor_bytes[..])?;

        Ok(descriptor_set)
    }

    fn parse_services(descriptors: &FileDescriptorSet) -> Result<HashMap<String, ServiceInfo>> {
        let mut services = HashMap::new();

        for file_desc in &descriptors.file {
            for service_desc in &file_desc.service {
                let service_name = service_desc.name.clone().unwrap_or_default();
                let mut methods = HashMap::new();

                for method_desc in &service_desc.method {
                    let method_name = method_desc.name.clone().unwrap_or_default();
                    let input_type = method_desc.input_type.clone().unwrap_or_default();
                    let output_type = method_desc.output_type.clone().unwrap_or_default();

                    methods.insert(method_name.clone(), MethodInfo {
                        method_name,
                        input_type,
                        output_type,
                    });
                }

                services.insert(service_name.clone(), ServiceInfo {
                    service_name,
                    methods,
                });
            }
        }

        Ok(services)
    }

    pub async fn call(
        &mut self,
        service_method: &str,
        request_data: Option<&str>,
        timeout: Duration,
    ) -> Result<Value> {
        // Validate service/method format
        let parts: Vec<&str> = service_method.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(anyhow!("Invalid service/method format. Expected 'Service/Method'"));
        }
        let (service_name, method_name) = (parts[0], parts[1]);

        // Check if service exists
        let service = self.services.get(service_name)
            .ok_or_else(|| anyhow!("Service '{}' not found in proto files", service_name))?;

        // Check if method exists
        let _method = service.methods.get(method_name)
            .ok_or_else(|| anyhow!("Method '{}' not found in service '{}'", method_name, service_name))?;

        // Parse request data
        let request_json: Value = match request_data {
            Some(data) => serde_json::from_str(data)
                .map_err(|e| anyhow!("Invalid JSON request data: {}", e))?,
            None => Value::Object(serde_json::Map::new()),
        };

        // Make the actual gRPC call with timeout
        tokio::time::timeout(timeout, self.make_grpc_call(service_name, method_name, &request_json))
            .await
            .map_err(|_| anyhow!("gRPC call timed out after {:?}", timeout))?
    }

    async fn make_grpc_call(
        &mut self,
        service_name: &str,
        method_name: &str,
        _request_data: &Value,
    ) -> Result<Value> {
        // For now, this is a placeholder that demonstrates the structure
        // In a full implementation, you would:
        // 1. Convert JSON to protobuf message using the type information
        // 2. Make the actual gRPC call using tonic
        // 3. Convert the protobuf response back to JSON
        
        // This requires dynamic protobuf handling which is complex
        // For demonstration, return an error indicating the service is not mocked
        Err(anyhow!(
            "Real gRPC call to {}/{} not yet implemented. This requires dynamic protobuf message handling.",
            service_name,
            method_name
        ))
    }

    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    pub fn list_methods(&self, service_name: &str) -> Vec<String> {
        self.services.get(service_name)
            .map(|service| service.methods.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn validate_expectations(
        &self,
        response: &Value,
        expectations: &[String],
    ) -> Result<Vec<String>> {
        let mut results = Vec::new();

        for expectation in expectations {
            let response_str = serde_json::to_string(response)?;
            let finder = JsonPathFinder::from_str(&response_str, expectation)
                .map_err(|e| anyhow!("JSONPath error: {}", e))?;
            let found = finder.find();
            
            match found {
                Value::Null => {
                    results.push(format!("❌ JSONPath '{}' not found", expectation));
                }
                _ => {
                    results.push(format!("✅ JSONPath '{}' found: {}", expectation, found));
                }
            }
        }

        Ok(results)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_proto_compilation_no_directory() {
        let non_existent_path = std::path::Path::new("/non/existent/path");
        let result = GrpcClient::compile_protos(non_existent_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_proto_compilation_no_proto_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some non-proto files
        fs::write(temp_dir.path().join("test.txt"), "not a proto file").unwrap();
        
        let result = GrpcClient::compile_protos(temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No .proto files found"));
    }

    #[test]
    fn test_validate_service_method_format() {
        let invalid_formats = vec![
            "InvalidFormat",
            "Too/Many/Slashes",
            "/MissingService", 
            "MissingMethod/",
            "",
        ];

        // Since we can't easily create a real GrpcClient in tests without protoc,
        // we'll test the validation logic by creating a simple validation function
        fn validate_format(service_method: &str) -> bool {
            let parts: Vec<&str> = service_method.split('/').collect();
            parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
        }

        for format in invalid_formats {
            assert!(!validate_format(format), "Expected '{}' to be invalid", format);
        }

        // Valid formats
        let valid_formats = vec![
            "Users/GetUser",
            "Products/ListProducts", 
            "Auth/Login",
        ];

        for format in valid_formats {
            assert!(validate_format(format), "Expected '{}' to be valid", format);
        }
    }

    #[test]
    fn test_validate_expectations() {
        // Test JSONPath validation without needing a full client
        use jsonpath_rust::JsonPathFinder;
        
        let response = serde_json::json!({
            "user": {
                "id": "123",
                "name": "John Doe"
            },
            "status": "success"
        });

        let response_str = serde_json::to_string(&response).unwrap();

        // Test successful path
        let finder = JsonPathFinder::from_str(&response_str, "$.user.id").unwrap();
        let found = finder.find();
        assert_ne!(found, serde_json::Value::Null);

        // Test failed path  
        let finder = JsonPathFinder::from_str(&response_str, "$.nonexistent").unwrap();
        let found = finder.find();
        assert_eq!(found, serde_json::Value::Null);
    }
}
