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
    pub fn generate_reports(results: &[TestSuiteResult], formats: &str, output_dir: &Path, template: &str) -> Result<Vec<PathBuf>> {
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
            let suite_tests: Vec<TestResult> = suite_result.results.iter().map(|test| {
                TestResult {
                    name: test.name.clone(),
                    status: if test.passed { TestStatus::Passed } else { TestStatus::Failed },
                    duration: test.duration,
                    error: test.error.clone(),
                    response_status: test.response_status,
                }
            }).collect();
            
            let suite_summary = TestSummary {
                total: suite_result.passed + suite_result.failed,
                passed: suite_result.passed,
                failed: suite_result.failed,
                duration: suite_result.duration,
                success_rate: if suite_result.passed + suite_result.failed > 0 {
                    (suite_result.passed as f64 / (suite_result.passed + suite_result.failed) as f64) * 100.0
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
    
    fn generate_html_report(report: &TestReport, output_dir: &Path, template_name: &str) -> Result<PathBuf> {
        use tera::{Tera, Context};
        
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
            "detailed" | _ => include_str!("../templates/report_detailed.html"),
        };
        
        tera.add_raw_template("report.html", template_content)
            .map_err(|e| anyhow::anyhow!("Failed to add template: {}", e))?;
        
        let mut context = Context::new();
        context.insert("report", report);
        context.insert("timestamp", &report.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        
        // Add duration as seconds for template use
        context.insert("total_duration_secs", &format!("{:.2}", report.summary.duration.as_secs_f64()));
        
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
        context.insert("success_rate_rounded", &format!("{:.1}", report.summary.success_rate));
        
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