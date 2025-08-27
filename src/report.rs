use crate::runner::test_runner::TestSuiteResult;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub timestamp: DateTime<Utc>,
    pub summary: TestSummary,
    pub suites: Vec<TestSuiteReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteReport {
    pub name: String,
    pub duration: Duration,
    pub summary: TestSummary,
    pub tests: Vec<TestResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub error: Option<String>,
    pub response_status: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn generate_reports(
        results: &[TestSuiteResult],
        formats: &str,
        output_dir: &Path,
        template: &str,
    ) -> Result<Vec<PathBuf>> {
        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        let report = Self::build_report(results);
        let mut generated_files = Vec::new();

        for format in formats.split(',') {
            let format = format.trim().to_lowercase();
            match format.as_str() {
                "json" => {
                    let path = Self::generate_json_report(&report, output_dir)?;
                    generated_files.push(path);
                }
                "html" => {
                    let path = Self::generate_html_report(&report, output_dir, template)?;
                    generated_files.push(path);
                }
                "junit" => {
                    let path = Self::generate_junit_report(&report, output_dir)?;
                    generated_files.push(path);
                }
                _ => {
                    eprintln!("Warning: Unknown report format '{}'", format);
                }
            }
        }

        Ok(generated_files)
    }

    fn build_report(results: &[TestSuiteResult]) -> TestReport {
        let mut suites = Vec::new();
        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_duration = Duration::ZERO;

        for suite_result in results {
            let suite_tests: Vec<TestResult> = suite_result
                .results
                .iter()
                .map(|test| TestResult {
                    name: test.name.clone(),
                    status: if test.passed {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    },
                    duration: test.duration,
                    error: test.error.clone(),
                    response_status: test.response_status,
                })
                .collect();

            let suite_summary = TestSummary {
                total: suite_result.passed + suite_result.failed,
                passed: suite_result.passed,
                failed: suite_result.failed,
                duration: suite_result.duration,
                success_rate: if suite_result.passed + suite_result.failed > 0 {
                    (suite_result.passed as f64
                        / (suite_result.passed + suite_result.failed) as f64)
                        * 100.0
                } else {
                    0.0
                },
            };

            suites.push(TestSuiteReport {
                name: suite_result.name.clone(),
                duration: suite_result.duration,
                summary: suite_summary,
                tests: suite_tests,
            });

            total_tests += suite_result.passed + suite_result.failed;
            total_passed += suite_result.passed;
            total_failed += suite_result.failed;
            total_duration += suite_result.duration;
        }

        TestReport {
            timestamp: Utc::now(),
            summary: TestSummary {
                total: total_tests,
                passed: total_passed,
                failed: total_failed,
                duration: total_duration,
                success_rate: if total_tests > 0 {
                    (total_passed as f64 / total_tests as f64) * 100.0
                } else {
                    0.0
                },
            },
            suites,
        }
    }

    fn generate_json_report(report: &TestReport, output_dir: &Path) -> Result<PathBuf> {
        let timestamp = report.timestamp.format("%Y%m%d_%H%M%S");
        let filename = format!("rivet_report_{}.json", timestamp);
        let path = output_dir.join(&filename);

        let json = serde_json::to_string_pretty(report)?;
        fs::write(&path, json)?;

        Ok(path)
    }

    fn generate_html_report(
        report: &TestReport,
        output_dir: &Path,
        template_name: &str,
    ) -> Result<PathBuf> {
        use tera::{Context, Tera};

        let timestamp = report.timestamp.format("%Y%m%d_%H%M%S");
        let filename = format!("rivet_report_{}.html", timestamp);
        let path = output_dir.join(&filename);

        // Create a minimal Tera instance with our HTML template
        let mut tera = Tera::default();

        // Select template based on template_name
        let template_content = match template_name {
            "simple" | "minimal" => include_str!("../templates/report_simple.html"),
            "chatty" => include_str!("../templates/report_chatty.html"),
            "compact" => include_str!("../templates/report_compact.html"),
            "detailed" => include_str!("../templates/report_detailed.html"),
            _ => include_str!("../templates/report_detailed.html"),
        };

        tera.add_raw_template("report.html", template_content)
            .map_err(|e| anyhow::anyhow!("Failed to add template: {}", e))?;

        let mut context = Context::new();
        context.insert("report", report);
        context.insert(
            "timestamp",
            &report.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        );

        // Add duration as seconds for template use
        context.insert(
            "total_duration_secs",
            &format!("{:.2}", report.summary.duration.as_secs_f64()),
        );

        // Pre-process suite and test data for template
        let mut enhanced_suites = Vec::new();
        for suite in &report.suites {
            let mut enhanced_tests = Vec::new();
            for test in &suite.tests {
                enhanced_tests.push(serde_json::json!({
                    "name": test.name,
                    "status": test.status,
                    "duration_ms": (test.duration.as_secs_f64() * 1000.0) as u64,
                    "error": test.error,
                    "response_status": test.response_status
                }));
            }
            enhanced_suites.push(serde_json::json!({
                "name": suite.name,
                "duration_secs": format!("{:.2}", suite.duration.as_secs_f64()),
                "summary": suite.summary,
                "tests": enhanced_tests
            }));
        }
        context.insert("enhanced_suites", &enhanced_suites);
        context.insert(
            "success_rate_rounded",
            &format!("{:.1}", report.summary.success_rate),
        );

        let html = tera.render("report.html", &context)?;
        fs::write(&path, html)?;

        Ok(path)
    }

    fn generate_junit_report(report: &TestReport, output_dir: &Path) -> Result<PathBuf> {
        let timestamp = report.timestamp.format("%Y%m%d_%H%M%S");
        let filename = format!("rivet_junit_{}.xml", timestamp);
        let path = output_dir.join(&filename);

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuites name=\"rivet\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
            report.summary.total,
            report.summary.failed,
            report.summary.duration.as_secs_f64()
        ));

        for suite in &report.suites {
            xml.push_str(&format!(
                "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
                suite.name,
                suite.summary.total,
                suite.summary.failed,
                suite.duration.as_secs_f64()
            ));

            for test in &suite.tests {
                xml.push_str(&format!(
                    "    <testcase name=\"{}\" time=\"{:.3}\"",
                    test.name,
                    test.duration.as_secs_f64()
                ));

                match test.status {
                    TestStatus::Failed => {
                        xml.push_str(">\n");
                        xml.push_str(&format!(
                            "      <failure message=\"{}\">{}</failure>\n",
                            test.error.as_deref().unwrap_or("Test failed"),
                            test.error.as_deref().unwrap_or("Test failed")
                        ));
                        xml.push_str("    </testcase>\n");
                    }
                    TestStatus::Skipped => {
                        xml.push_str(">\n");
                        xml.push_str("      <skipped/>\n");
                        xml.push_str("    </testcase>\n");
                    }
                    TestStatus::Passed => {
                        xml.push_str("/>\n");
                    }
                }
            }

            xml.push_str("  </testsuite>\n");
        }

        xml.push_str("</testsuites>\n");

        fs::write(&path, xml)?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{executor::TestResult as ExecutorTestResult, test_runner::TestSuiteResult};
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_sample_test_suite_result() -> TestSuiteResult {
        TestSuiteResult {
            name: "API Test Suite".to_string(),
            results: vec![
                ExecutorTestResult {
                    name: "Test GET users".to_string(),
                    passed: true,
                    duration: Duration::from_millis(150),
                    error: None,
                    response_status: Some(200),
                    response_body: Some(r#"{"users": []}"#.to_string()),
                },
                ExecutorTestResult {
                    name: "Test POST user".to_string(),
                    passed: false,
                    duration: Duration::from_millis(300),
                    error: Some("Status code mismatch: expected 201, got 400".to_string()),
                    response_status: Some(400),
                    response_body: Some(r#"{"error": "Invalid data"}"#.to_string()),
                },
                ExecutorTestResult {
                    name: "Test DELETE user".to_string(),
                    passed: true,
                    duration: Duration::from_millis(100),
                    error: None,
                    response_status: Some(204),
                    response_body: None,
                },
            ],
            duration: Duration::from_millis(550),
            passed: 2,
            failed: 1,
        }
    }

    #[test]
    fn test_build_report() {
        let suite_results = vec![create_sample_test_suite_result()];
        let report = ReportGenerator::build_report(&suite_results);

        // Check overall summary
        assert_eq!(report.summary.total, 3);
        assert_eq!(report.summary.passed, 2);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.summary.duration, Duration::from_millis(550));
        assert!((report.summary.success_rate - 66.67).abs() < 0.1);

        // Check suite details
        assert_eq!(report.suites.len(), 1);
        let suite = &report.suites[0];
        assert_eq!(suite.name, "API Test Suite");
        assert_eq!(suite.duration, Duration::from_millis(550));
        assert_eq!(suite.summary.total, 3);
        assert_eq!(suite.summary.passed, 2);
        assert_eq!(suite.summary.failed, 1);

        // Check test details
        assert_eq!(suite.tests.len(), 3);
        assert!(matches!(suite.tests[0].status, TestStatus::Passed));
        assert!(matches!(suite.tests[1].status, TestStatus::Failed));
        assert!(matches!(suite.tests[2].status, TestStatus::Passed));
        assert_eq!(
            suite.tests[1].error,
            Some("Status code mismatch: expected 201, got 400".to_string())
        );
    }

    #[test]
    fn test_build_report_empty() {
        let suite_results = vec![];
        let report = ReportGenerator::build_report(&suite_results);

        assert_eq!(report.summary.total, 0);
        assert_eq!(report.summary.passed, 0);
        assert_eq!(report.summary.failed, 0);
        assert_eq!(report.summary.success_rate, 0.0);
        assert!(report.suites.is_empty());
    }

    #[test]
    fn test_build_report_all_passed() {
        let suite_result = TestSuiteResult {
            name: "All Pass Suite".to_string(),
            results: vec![
                ExecutorTestResult {
                    name: "Test 1".to_string(),
                    passed: true,
                    duration: Duration::from_millis(100),
                    error: None,
                    response_status: Some(200),
                    response_body: None,
                },
                ExecutorTestResult {
                    name: "Test 2".to_string(),
                    passed: true,
                    duration: Duration::from_millis(200),
                    error: None,
                    response_status: Some(200),
                    response_body: None,
                },
            ],
            duration: Duration::from_millis(300),
            passed: 2,
            failed: 0,
        };

        let report = ReportGenerator::build_report(&vec![suite_result]);
        assert_eq!(report.summary.success_rate, 100.0);
    }

    #[test]
    fn test_build_report_all_failed() {
        let suite_result = TestSuiteResult {
            name: "All Fail Suite".to_string(),
            results: vec![
                ExecutorTestResult {
                    name: "Test 1".to_string(),
                    passed: false,
                    duration: Duration::from_millis(100),
                    error: Some("Error 1".to_string()),
                    response_status: Some(500),
                    response_body: None,
                },
                ExecutorTestResult {
                    name: "Test 2".to_string(),
                    passed: false,
                    duration: Duration::from_millis(200),
                    error: Some("Error 2".to_string()),
                    response_status: Some(404),
                    response_body: None,
                },
            ],
            duration: Duration::from_millis(300),
            passed: 0,
            failed: 2,
        };

        let report = ReportGenerator::build_report(&vec![suite_result]);
        assert_eq!(report.summary.success_rate, 0.0);
    }

    #[test]
    fn test_build_report_multiple_suites() {
        let suite1 = TestSuiteResult {
            name: "Suite 1".to_string(),
            results: vec![ExecutorTestResult {
                name: "Test 1".to_string(),
                passed: true,
                duration: Duration::from_millis(100),
                error: None,
                response_status: Some(200),
                response_body: None,
            }],
            duration: Duration::from_millis(100),
            passed: 1,
            failed: 0,
        };

        let suite2 = TestSuiteResult {
            name: "Suite 2".to_string(),
            results: vec![
                ExecutorTestResult {
                    name: "Test 2".to_string(),
                    passed: true,
                    duration: Duration::from_millis(150),
                    error: None,
                    response_status: Some(200),
                    response_body: None,
                },
                ExecutorTestResult {
                    name: "Test 3".to_string(),
                    passed: false,
                    duration: Duration::from_millis(200),
                    error: Some("Failed".to_string()),
                    response_status: Some(500),
                    response_body: None,
                },
            ],
            duration: Duration::from_millis(350),
            passed: 1,
            failed: 1,
        };

        let report = ReportGenerator::build_report(&vec![suite1, suite2]);

        // Overall summary should aggregate both suites
        assert_eq!(report.summary.total, 3);
        assert_eq!(report.summary.passed, 2);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.summary.duration, Duration::from_millis(450));
        assert!((report.summary.success_rate - 66.67).abs() < 0.1);
        assert_eq!(report.suites.len(), 2);
    }

    #[test]
    fn test_generate_json_report() {
        let temp_dir = TempDir::new().unwrap();
        let suite_results = vec![create_sample_test_suite_result()];
        let report = ReportGenerator::build_report(&suite_results);

        let path = ReportGenerator::generate_json_report(&report, temp_dir.path()).unwrap();

        assert!(path.exists());
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("rivet_report_"));
        assert!(path.extension().unwrap() == "json");

        // Verify the JSON content is valid
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed_report: TestReport = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed_report.summary.total, 3);
        assert_eq!(parsed_report.suites.len(), 1);
    }

    #[test]
    fn test_generate_junit_report() {
        let temp_dir = TempDir::new().unwrap();
        let suite_results = vec![create_sample_test_suite_result()];
        let report = ReportGenerator::build_report(&suite_results);

        let path = ReportGenerator::generate_junit_report(&report, temp_dir.path()).unwrap();

        assert!(path.exists());
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("rivet_junit_"));
        assert!(path.extension().unwrap() == "xml");

        // Verify the XML content contains expected elements
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<?xml version=\"1.0\""));
        assert!(content.contains("<testsuites"));
        assert!(content.contains("<testsuite"));
        assert!(content.contains("<testcase"));
        assert!(content.contains("API Test Suite"));
        assert!(content.contains("Test GET users"));
        assert!(content.contains("<failure"));
        assert!(content.contains("Status code mismatch"));
    }

    #[test]
    fn test_generate_html_report_templates() {
        let temp_dir = TempDir::new().unwrap();
        let suite_results = vec![create_sample_test_suite_result()];
        let report = ReportGenerator::build_report(&suite_results);

        // Test different template names
        let templates = vec!["simple", "detailed", "compact", "chatty", "unknown"];

        for template in templates {
            let path =
                ReportGenerator::generate_html_report(&report, temp_dir.path(), template).unwrap();

            assert!(path.exists());
            assert!(path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("rivet_report_"));
            assert!(path.extension().unwrap() == "html");

            let content = std::fs::read_to_string(&path).unwrap();
            assert!(content.contains("<!DOCTYPE html") || content.contains("<html"));
            
            // Verify it's valid HTML (contains basic structure)
            assert!(content.len() > 100); // Should be a substantial HTML file
        }
    }

    #[test]
    fn test_generate_reports_multiple_formats() {
        let temp_dir = TempDir::new().unwrap();
        let suite_results = vec![create_sample_test_suite_result()];

        let generated_files = ReportGenerator::generate_reports(
            &suite_results,
            "json,html,junit",
            temp_dir.path(),
            "compact",
        )
        .unwrap();

        assert_eq!(generated_files.len(), 3);

        // Verify all files were created
        for path in &generated_files {
            assert!(path.exists());
        }

        // Check file extensions
        let extensions: Vec<String> = generated_files
            .iter()
            .map(|p| p.extension().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(extensions.contains(&"json".to_string()));
        assert!(extensions.contains(&"html".to_string()));
        assert!(extensions.contains(&"xml".to_string()));
    }

    #[test]
    fn test_generate_reports_unknown_format() {
        let temp_dir = TempDir::new().unwrap();
        let suite_results = vec![create_sample_test_suite_result()];

        // This should not panic, just skip unknown formats
        let generated_files = ReportGenerator::generate_reports(
            &suite_results,
            "json,unknown_format,html",
            temp_dir.path(),
            "compact",
        )
        .unwrap();

        assert_eq!(generated_files.len(), 2); // Only json and html should be generated
    }

    #[test]
    fn test_test_status_serialization() {
        // Test that TestStatus can be serialized/deserialized
        let passed = TestStatus::Passed;
        let failed = TestStatus::Failed;
        let skipped = TestStatus::Skipped;

        let passed_json = serde_json::to_string(&passed).unwrap();
        let failed_json = serde_json::to_string(&failed).unwrap();
        let skipped_json = serde_json::to_string(&skipped).unwrap();

        assert_eq!(passed_json, "\"Passed\"");
        assert_eq!(failed_json, "\"Failed\"");
        assert_eq!(skipped_json, "\"Skipped\"");

        let passed_back: TestStatus = serde_json::from_str(&passed_json).unwrap();
        let failed_back: TestStatus = serde_json::from_str(&failed_json).unwrap();
        let skipped_back: TestStatus = serde_json::from_str(&skipped_json).unwrap();

        assert!(matches!(passed_back, TestStatus::Passed));
        assert!(matches!(failed_back, TestStatus::Failed));
        assert!(matches!(skipped_back, TestStatus::Skipped));
    }
}
