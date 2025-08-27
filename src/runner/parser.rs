use crate::config::RivetConfig;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub async fn load_test_suite(path: &Path) -> Result<Vec<(String, RivetConfig)>> {
    if path.is_file() {
        let config = load_single_file(path).await?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(vec![(file_name, config)])
    } else if path.is_dir() {
        load_directory(path).await
    } else {
        anyhow::bail!("Path does not exist: {}", path.display());
    }
}

async fn load_single_file(path: &Path) -> Result<RivetConfig> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let config: RivetConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML in file: {}", path.display()))?;

    Ok(config)
}

async fn load_directory(path: &Path) -> Result<Vec<(String, RivetConfig)>> {
    let mut configs = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains(".rivet."))
                .unwrap_or(false)
        })
    {
        let config = load_single_file(entry.path()).await?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        configs.push((file_name, config));
    }

    if configs.is_empty() {
        anyhow::bail!(
            "No .rivet.yaml files found in directory: {}",
            path.display()
        );
    }

    // Sort by filename for consistent execution order
    configs.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::{NamedTempFile, TempDir};
    use tokio::fs::write;

    fn create_sample_rivet_config() -> RivetConfig {
        let mut vars = HashMap::new();
        vars.insert("baseUrl".to_string(), "https://api.example.com".to_string());

        RivetConfig {
            name: "Test Suite".to_string(),
            description: Some("Sample test suite".to_string()),
            env: Some("test".to_string()),
            vars: Some(vars),
            setup: None,
            tests: vec![crate::config::TestStep {
                name: "Test GET request".to_string(),
                description: Some("Test a simple GET request".to_string()),
                request: crate::config::Request {
                    method: "GET".to_string(),
                    url: "{{baseUrl}}/users".to_string(),
                    headers: None,
                    params: None,
                    body: None,
                },
                expect: Some(crate::config::Expectation {
                    status: Some(crate::config::StatusExpectation::Number(200)),
                    schema: None,
                    jsonpath: None,
                    headers: None,
                }),
            }],
            dataset: None,
            teardown: None,
        }
    }

    #[tokio::test]
    async fn test_load_single_file_valid() {
        let config = create_sample_rivet_config();
        let yaml_content = serde_yaml::to_string(&config).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), yaml_content).await.unwrap();

        let loaded_config = load_single_file(temp_file.path()).await.unwrap();

        assert_eq!(loaded_config.name, "Test Suite");
        assert_eq!(
            loaded_config.description,
            Some("Sample test suite".to_string())
        );
        assert_eq!(loaded_config.env, Some("test".to_string()));
        assert_eq!(loaded_config.tests.len(), 1);
    }

    #[tokio::test]
    async fn test_load_single_file_invalid_yaml() {
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), "invalid: yaml: content: [")
            .await
            .unwrap();

        let result = load_single_file(temp_file.path()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse YAML"));
    }

    #[tokio::test]
    async fn test_load_single_file_nonexistent() {
        let result = load_single_file(Path::new("/nonexistent/file.yaml")).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_load_test_suite_single_file() {
        let config = create_sample_rivet_config();
        let yaml_content = serde_yaml::to_string(&config).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), yaml_content).await.unwrap();

        let result = load_test_suite(temp_file.path()).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.name, "Test Suite");
    }

    #[tokio::test]
    async fn test_load_directory_with_rivet_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid rivet file
        let config1 = create_sample_rivet_config();
        let yaml_content1 = serde_yaml::to_string(&config1).unwrap();
        let rivet_file1 = temp_dir.path().join("test1.rivet.yaml");
        write(&rivet_file1, yaml_content1).await.unwrap();

        // Create another rivet file
        let mut config2 = create_sample_rivet_config();
        config2.name = "Second Test Suite".to_string();
        let yaml_content2 = serde_yaml::to_string(&config2).unwrap();
        let rivet_file2 = temp_dir.path().join("test2.rivet.yml");
        write(&rivet_file2, yaml_content2).await.unwrap();

        // Create a non-rivet YAML file (should be ignored)
        let non_rivet_file = temp_dir.path().join("config.yaml");
        write(&non_rivet_file, "some: config").await.unwrap();

        let result = load_directory(temp_dir.path()).await.unwrap();

        assert_eq!(result.len(), 2);
        // Results should be sorted by filename
        assert_eq!(result[0].0, "test1.rivet.yaml");
        assert_eq!(result[1].0, "test2.rivet.yml");
        assert_eq!(result[0].1.name, "Test Suite");
        assert_eq!(result[1].1.name, "Second Test Suite");
    }

    #[tokio::test]
    async fn test_load_directory_no_rivet_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a non-rivet file
        let non_rivet_file = temp_dir.path().join("config.yaml");
        write(&non_rivet_file, "some: config").await.unwrap();

        let result = load_directory(temp_dir.path()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No .rivet.yaml files found"));
    }

    #[tokio::test]
    async fn test_load_test_suite_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create a rivet file in the directory
        let config = create_sample_rivet_config();
        let yaml_content = serde_yaml::to_string(&config).unwrap();
        let rivet_file = temp_dir.path().join("api.rivet.yaml");
        write(&rivet_file, yaml_content).await.unwrap();

        let result = load_test_suite(temp_dir.path()).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "api.rivet.yaml");
        assert_eq!(result[0].1.name, "Test Suite");
    }

    #[tokio::test]
    async fn test_load_test_suite_nonexistent_path() {
        let result = load_test_suite(Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path does not exist"));
    }

    #[tokio::test]
    async fn test_rivet_file_filtering() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with different extensions and naming patterns
        let files_to_create = vec![
            ("test.rivet.yaml", true), // Should be included
            ("test.rivet.yml", true),  // Should be included
            ("config.yaml", false),    // Should be ignored (no .rivet.)
            ("test.yaml", false),      // Should be ignored (no .rivet.)
            ("rivet.yaml", false),     // Should be ignored (no .rivet.)
            ("test.rivet.txt", false), // Should be ignored (wrong extension)
        ];

        let config = create_sample_rivet_config();
        let yaml_content = serde_yaml::to_string(&config).unwrap();

        for (filename, _) in &files_to_create {
            let file_path = temp_dir.path().join(filename);
            write(&file_path, &yaml_content).await.unwrap();
        }

        let result = load_directory(temp_dir.path()).await.unwrap();

        // Should only include the 2 valid rivet files
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|(name, _)| name == "test.rivet.yaml"));
        assert!(result.iter().any(|(name, _)| name == "test.rivet.yml"));
    }
}
