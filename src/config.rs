use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct RivetConfig {
    pub name: String,
    pub env: Option<String>,
    pub vars: Option<HashMap<String, String>>,
    pub setup: Option<Vec<TestStep>>,
    pub tests: Vec<TestStep>,
    pub dataset: Option<Dataset>,
    pub teardown: Option<Vec<TestStep>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestStep {
    pub name: String,
    pub request: Request,
    pub expect: Option<Expectation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Expectation {
    pub status: Option<u16>,
    pub schema: Option<String>,
    pub jsonpath: Option<HashMap<String, serde_json::Value>>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub file: String,
    pub parallel: Option<usize>,
}