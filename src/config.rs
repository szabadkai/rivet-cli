use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserConfig {
    pub reports: ReportConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportConfig {
    pub auto_open_browser: bool,
    pub default_template: String,
    pub default_formats: Vec<String>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            auto_open_browser: true,  // Default to true as requested
            default_template: "chatty".to_string(), // Default to chatty since user likes it
            default_formats: vec!["html".to_string()],
        }
    }
}

impl UserConfig {
    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.json");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: UserConfig = serde_json::from_str(&content)
                .unwrap_or_default(); // Fall back to default if parsing fails
            Ok(config)
        } else {
            // Create default config file
            let default_config = UserConfig::default();
            default_config.save()?;
            Ok(default_config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    fn get_config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Unable to find home directory"))?;
        
        Ok(PathBuf::from(home).join(".rivet"))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RivetConfig {
    pub name: String,
    pub description: Option<String>,
    pub env: Option<String>,
    pub vars: Option<HashMap<String, String>>,
    pub setup: Option<Vec<TestStep>>,
    pub tests: Vec<TestStep>,
    pub dataset: Option<Dataset>,
    pub teardown: Option<Vec<TestStep>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestStep {
    pub name: String,
    pub description: Option<String>,
    pub request: Request,
    pub expect: Option<Expectation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub params: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expectation {
    pub status: Option<StatusExpectation>,
    pub schema: Option<String>,
    pub jsonpath: Option<HashMap<String, serde_json::Value>>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StatusExpectation {
    Number(u16),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dataset {
    pub file: String,
    pub parallel: Option<usize>,
}