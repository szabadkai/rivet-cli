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
    if let Some(num_str) = timeout_str.strip_suffix("ms") {
        if num_str.is_empty() {
            return Err(anyhow!("Invalid timeout format: {}", timeout_str));
        }
        let millis: u64 = num_str.parse()?;
        Ok(Duration::from_millis(millis))
    } else if let Some(num_str) = timeout_str.strip_suffix('s') {
        if num_str.is_empty() {
            return Err(anyhow!("Invalid timeout format: {}", timeout_str));
        }
        let seconds: u64 = num_str.parse()?;
        Ok(Duration::from_secs(seconds))
    } else {
        let seconds: u64 = timeout_str.parse()?;
        Ok(Duration::from_secs(seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_headers_valid() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "Authorization: Bearer token123".to_string(),
            "Accept: application/json".to_string(),
        ];

        let result = parse_headers(&headers).unwrap();

        assert_eq!(
            result.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            result.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(result.get("Accept"), Some(&"application/json".to_string()));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_headers_with_spaces() {
        let headers = vec!["  Content-Type  :   application/json  ".to_string()];
        let result = parse_headers(&headers).unwrap();
        assert_eq!(
            result.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_parse_headers_invalid_format() {
        let headers = vec!["InvalidHeader".to_string()];
        let result = parse_headers(&headers);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid header format"));
    }

    #[test]
    fn test_parse_headers_empty() {
        let headers = vec![];
        let result = parse_headers(&headers).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_headers_colon_in_value() {
        let headers = vec!["X-Custom: value:with:colons".to_string()];
        let result = parse_headers(&headers).unwrap();
        assert_eq!(
            result.get("X-Custom"),
            Some(&"value:with:colons".to_string())
        );
    }

    #[test]
    fn test_parse_timeout_seconds() {
        assert_eq!(parse_timeout("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_timeout("5s").unwrap(), Duration::from_secs(5));
    }

    #[test]
    fn test_parse_timeout_milliseconds() {
        assert_eq!(parse_timeout("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(
            parse_timeout("1000ms").unwrap(),
            Duration::from_millis(1000)
        );
    }

    #[test]
    fn test_parse_timeout_plain_number() {
        assert_eq!(parse_timeout("15").unwrap(), Duration::from_secs(15));
        assert_eq!(parse_timeout("0").unwrap(), Duration::from_secs(0));
    }

    #[test]
    fn test_parse_timeout_invalid() {
        assert!(parse_timeout("invalid").is_err());
        assert!(parse_timeout("10x").is_err());
        assert!(parse_timeout("").is_err());
    }

    #[test]
    fn test_parse_timeout_edge_cases() {
        assert_eq!(parse_timeout("0s").unwrap(), Duration::from_secs(0));
        assert_eq!(parse_timeout("0ms").unwrap(), Duration::from_millis(0));
        
        // Test invalid edge cases
        assert!(parse_timeout("s").is_err());
        assert!(parse_timeout("ms").is_err());
    }
}
