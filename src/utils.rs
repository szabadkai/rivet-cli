use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::Duration;

pub fn parse_headers(headers: &[String]) -> Result<HashMap<String, String>> {
    let mut parsed = HashMap::new();
    
    for header in headers {
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid header format: {}", header));
        }
        
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();
        parsed.insert(key, value);
    }
    
    Ok(parsed)
}

pub fn parse_timeout(timeout_str: &str) -> Result<Duration> {
    if timeout_str.ends_with('s') {
        let seconds: u64 = timeout_str[..timeout_str.len()-1].parse()?;
        Ok(Duration::from_secs(seconds))
    } else if timeout_str.ends_with("ms") {
        let millis: u64 = timeout_str[..timeout_str.len()-2].parse()?;
        Ok(Duration::from_millis(millis))
    } else {
        let seconds: u64 = timeout_str.parse()?;
        Ok(Duration::from_secs(seconds))
    }
}