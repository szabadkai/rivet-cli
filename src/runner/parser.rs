use crate::config::RivetConfig;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub async fn load_test_suite(path: &Path) -> Result<Vec<(String, RivetConfig)>> {
    if path.is_file() {
        let config = load_single_file(path).await?;
        let file_name = path.file_name()
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
    let content = fs::read_to_string(path).await
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
        anyhow::bail!("No .rivet.yaml files found in directory: {}", path.display());
    }
    
    // Sort by filename for consistent execution order
    configs.sort_by(|a, b| a.0.cmp(&b.0));
    
    Ok(configs)
}