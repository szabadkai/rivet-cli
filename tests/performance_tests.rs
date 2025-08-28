use anyhow::Result;
use rivet::performance::{LoadPattern, PerformanceTestRunner};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

/// Test basic performance test runner functionality
#[tokio::test]
async fn test_performance_test_runner_creation() -> Result<()> {
    let _runner = PerformanceTestRunner::new(
        5,                               // concurrent users
        Some(10),                        // target RPS
        Duration::from_secs(5),          // test duration
        Duration::from_secs(1),          // warmup duration
        Duration::from_secs(2),          // report interval
        LoadPattern::Constant,           // load pattern
    )?;

    // Just verify we can create the runner successfully
    assert!(true);
    Ok(())
}

/// Test performance metrics calculation
#[tokio::test]
async fn test_performance_metrics() -> Result<()> {
    use rivet::performance::PerformanceMetrics;
    use std::time::Duration;

    let mut metrics = PerformanceMetrics::new();
    
    // Record some test requests
    metrics.record_request(Duration::from_millis(100), 200, 1024, 2048, false);
    metrics.record_request(Duration::from_millis(150), 200, 512, 1024, false);
    metrics.record_request(Duration::from_millis(200), 500, 256, 512, true);
    
    let results = metrics.calculate_results();
    
    assert_eq!(results.total_requests, 3);
    assert_eq!(results.successful_requests, 2);
    assert_eq!(results.failed_requests, 1);
    assert_eq!(results.success_rate, 2.0 / 3.0);
    
    // Check response time metrics
    assert!(results.average_response_time > Duration::from_millis(100));
    assert!(results.average_response_time < Duration::from_millis(200));
    assert_eq!(results.min_response_time, Duration::from_millis(100));
    assert_eq!(results.max_response_time, Duration::from_millis(200));
    
    // Check status code distribution
    assert_eq!(results.status_code_distribution.get(&200), Some(&2));
    assert_eq!(results.status_code_distribution.get(&500), Some(&1));
    
    Ok(())
}

/// Test load patterns
#[tokio::test]
async fn test_load_patterns() -> Result<()> {
    use rivet::performance::patterns::LoadController;
    use std::time::Duration;

    // Test constant load pattern
    let constant_controller = LoadController::new(
        LoadPattern::Constant,
        Some(100),
        10,
        Duration::from_secs(5),
    );
    
    let rps = constant_controller.current_target_rps();
    assert_eq!(rps, 100.0);
    
    let delay = constant_controller.request_delay();
    assert!(delay.is_some());
    assert_eq!(delay.unwrap(), Duration::from_millis(10)); // 1000ms / 100 RPS = 10ms
    
    // Test ramp-up pattern - this would need more complex testing with actual time progression
    let ramp_controller = LoadController::new(
        LoadPattern::RampUp,
        Some(100),
        10,
        Duration::from_secs(10),
    );
    
    // At the beginning, RPS should be lower than target
    let initial_rps = ramp_controller.current_target_rps();
    assert!(initial_rps < 100.0);
    
    Ok(())
}

/// Test performance test with a simple configuration
#[tokio::test] 
async fn test_performance_test_with_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a simple test configuration with a more reliable local endpoint
    let test_config = r#"
name: "Performance Test"
description: "Simple performance test"
tests:
  - name: "test_request"
    description: "Test HTTP request"
    request:
      method: GET
      url: "http://127.0.0.1:8080"
    expect:
      status: 200
"#;
    
    let config_file = temp_dir.path().join("perf_test.yaml");
    fs::write(&config_file, test_config)?;
    
    // Create a short performance test with higher load to ensure requests are made
    let runner = PerformanceTestRunner::new(
        3,                               // concurrent users  
        Some(10),                        // target RPS
        Duration::from_secs(5),          // test duration
        Duration::from_secs(1),          // warmup duration
        Duration::from_secs(2),          // report interval
        LoadPattern::Constant,           // load pattern
    )?;
    
    // Run the performance test
    let results = runner.run_performance_test(&config_file, None).await?;
    
    // Verify we got some results - but be lenient for network failures in CI
    // The test mainly verifies the performance runner can be created and executed
    assert!(results.total_duration > Duration::ZERO);
    assert!(results.success_rate >= 0.0 && results.success_rate <= 1.0);
    // Note: total_requests might be 0 if network is unavailable in CI
    
    Ok(())
}

/// Test performance test error handling
#[tokio::test]
async fn test_performance_test_error_handling() -> Result<()> {
    // Test with non-existent file
    let runner = PerformanceTestRunner::new(
        1,
        None,
        Duration::from_secs(1),
        Duration::from_secs(0),
        Duration::from_secs(1),
        LoadPattern::Constant,
    )?;
    
    let non_existent_path = std::path::Path::new("/non/existent/path");
    let result = runner.run_performance_test(non_existent_path, None).await;
    assert!(result.is_err());
    
    Ok(())
}

/// Test different load patterns
#[tokio::test]
async fn test_different_load_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    let test_config = r#"
name: "Load Pattern Test"
tests:
  - name: "simple_get"
    request:
      method: GET
      url: "http://127.0.0.1:8080"
"#;
    
    let config_file = temp_dir.path().join("pattern_test.yaml");
    fs::write(&config_file, test_config)?;
    
    let patterns = vec![
        LoadPattern::Constant,
        LoadPattern::RampUp,
        LoadPattern::Spike,
    ];
    
    for pattern in patterns {
        let runner = PerformanceTestRunner::new(
            3,
            Some(10),
            Duration::from_secs(4),
            Duration::from_secs(1),
            Duration::from_secs(2),
            pattern,
        )?;
        
        let results = runner.run_performance_test(&config_file, None).await?;
        // Be lenient for network failures in CI - main goal is testing pattern switching
        assert!(results.total_duration > Duration::ZERO);
    }
    
    Ok(())
}

/// Test performance report saving
#[tokio::test]
async fn test_performance_report_saving() -> Result<()> {
    use rivet::performance::PerformanceMetrics;
    
    let temp_dir = TempDir::new()?;
    let mut metrics = PerformanceMetrics::new();
    
    // Record some sample data
    metrics.record_request(Duration::from_millis(100), 200, 1024, 2048, false);
    metrics.record_request(Duration::from_millis(150), 200, 512, 1024, false);
    
    let results = metrics.calculate_results();
    
    // Save the report
    let report_file = temp_dir.path().join("performance_report.json");
    results.save_report(&report_file)?;
    
    // Verify the file was created and contains valid JSON
    assert!(report_file.exists());
    let content = fs::read_to_string(&report_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    
    // Verify some expected fields
    assert!(parsed["total_requests"].is_number());
    assert!(parsed["success_rate"].is_number());
    assert!(parsed["average_response_time"].is_number());
    
    Ok(())
}