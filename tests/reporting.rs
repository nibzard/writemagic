//! Test reporting utilities

use crate::{TestSuiteResults, TestResult};

/// Generate HTML test report
pub fn generate_html_report(results: &TestSuiteResults) -> String {
    format!(
        r#"<html><head><title>Test Results</title></head><body>
        <h1>WriteMagic Test Results</h1>
        <p>Total: {}, Passed: {}, Failed: {}</p>
        </body></html>"#,
        results.total_tests, results.passed, results.failed
    )
}

/// Generate JSON test report
pub fn generate_json_report(results: &TestSuiteResults) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}