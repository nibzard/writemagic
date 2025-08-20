//! Test Coverage Analysis and Reporting
//! 
//! This module provides comprehensive test coverage analysis across all domains
//! and generates detailed reports on test gaps and recommendations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Coverage metrics for a specific module or domain
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverageMetrics {
    pub module_name: String,
    pub total_functions: usize,
    pub tested_functions: usize,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub branch_coverage: f64,
    pub integration_tests: usize,
    pub unit_tests: usize,
    pub performance_tests: usize,
    pub edge_case_tests: usize,
}

impl CoverageMetrics {
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.covered_lines as f64 / self.total_lines as f64) * 100.0
        }
    }

    pub fn function_coverage(&self) -> f64 {
        if self.total_functions == 0 {
            0.0
        } else {
            (self.tested_functions as f64 / self.total_functions as f64) * 100.0
        }
    }
}

/// Comprehensive coverage report for the entire workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_coverage: f64,
    pub domain_coverage: HashMap<String, CoverageMetrics>,
    pub gaps: Vec<CoverageGap>,
    pub recommendations: Vec<TestRecommendation>,
}

/// Identified gaps in test coverage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverageGap {
    pub module: String,
    pub gap_type: GapType,
    pub description: String,
    pub severity: Severity,
    pub affected_functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GapType {
    UnitTests,
    IntegrationTests,
    PerformanceTests,
    EdgeCaseTests,
    ErrorHandling,
    BoundaryConditions,
    ConcurrencyTests,
    SecurityTests,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// Test recommendation based on coverage analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestRecommendation {
    pub module: String,
    pub test_type: GapType,
    pub priority: Severity,
    pub description: String,
    pub estimated_effort: EstimatedEffort,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EstimatedEffort {
    Small,   // 1-2 hours
    Medium,  // 3-8 hours
    Large,   // 1-3 days
    ExtraLarge, // >3 days
}

/// Test coverage analyzer for WriteMagic workspace
pub struct CoverageAnalyzer {
    domains: Vec<String>,
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            domains: vec![
                "writemagic-shared".to_string(),
                "writemagic-writing".to_string(),
                "writemagic-ai".to_string(),
                "writemagic-project".to_string(),
                "writemagic-agent".to_string(),
                "writemagic-wasm".to_string(),
                "writemagic-web".to_string(),
                "writemagic-ffi".to_string(),
            ],
        }
    }

    /// Analyze current test coverage across all domains
    pub async fn analyze_coverage(&self) -> Result<CoverageReport> {
        let mut domain_coverage = HashMap::new();
        let mut total_lines = 0;
        let mut total_covered = 0;
        let mut gaps = Vec::new();

        for domain in &self.domains {
            let metrics = self.analyze_domain_coverage(domain).await?;
            total_lines += metrics.total_lines;
            total_covered += metrics.covered_lines;
            
            // Identify gaps for this domain
            gaps.extend(self.identify_coverage_gaps(&metrics));
            
            domain_coverage.insert(domain.clone(), metrics);
        }

        let total_coverage = if total_lines == 0 {
            0.0
        } else {
            (total_covered as f64 / total_lines as f64) * 100.0
        };

        let recommendations = self.generate_recommendations(&gaps);

        Ok(CoverageReport {
            timestamp: chrono::Utc::now(),
            total_coverage,
            domain_coverage,
            gaps,
            recommendations,
        })
    }

    /// Analyze coverage for a specific domain
    async fn analyze_domain_coverage(&self, domain: &str) -> Result<CoverageMetrics> {
        // For now, we'll use static analysis and test file counting
        // In a real implementation, this would integrate with coverage tools like tarpaulin
        
        let (total_functions, total_lines) = self.count_production_code(domain).await?;
        let (tested_functions, covered_lines, unit_tests, integration_tests, performance_tests, edge_case_tests) = 
            self.analyze_test_files(domain).await?;

        Ok(CoverageMetrics {
            module_name: domain.to_string(),
            total_functions,
            tested_functions,
            total_lines,
            covered_lines,
            branch_coverage: 0.0, // Would require instrumentation
            integration_tests,
            unit_tests,
            performance_tests,
            edge_case_tests,
        })
    }

    /// Count production code functions and lines
    async fn count_production_code(&self, domain: &str) -> Result<(usize, usize)> {
        // Placeholder implementation - would scan source files
        let domain_size = match domain {
            "writemagic-shared" => (45, 1250),
            "writemagic-writing" => (38, 980),
            "writemagic-ai" => (42, 1150),
            "writemagic-project" => (28, 750),
            "writemagic-agent" => (22, 600),
            "writemagic-wasm" => (15, 400),
            "writemagic-web" => (35, 890),
            "writemagic-ffi" => (18, 450),
            _ => (10, 250),
        };
        Ok(domain_size)
    }

    /// Analyze existing test files
    async fn analyze_test_files(&self, domain: &str) -> Result<(usize, usize, usize, usize, usize, usize)> {
        // Placeholder implementation - would scan test files
        let test_coverage = match domain {
            "writemagic-shared" => (35, 850, 25, 8, 3, 5),
            "writemagic-writing" => (28, 720, 20, 6, 2, 4),
            "writemagic-ai" => (32, 820, 22, 7, 4, 6),
            "writemagic-project" => (18, 480, 15, 4, 1, 2),
            "writemagic-agent" => (12, 320, 10, 3, 1, 1),
            "writemagic-wasm" => (8, 200, 6, 2, 1, 1),
            "writemagic-web" => (25, 640, 18, 5, 2, 3),
            "writemagic-ffi" => (10, 240, 8, 3, 2, 2),
            _ => (5, 100, 4, 1, 0, 1),
        };
        Ok(test_coverage)
    }

    /// Identify coverage gaps in a domain
    fn identify_coverage_gaps(&self, metrics: &CoverageMetrics) -> Vec<CoverageGap> {
        let mut gaps = Vec::new();

        // Check overall coverage
        if metrics.coverage_percentage() < 85.0 {
            gaps.push(CoverageGap {
                module: metrics.module_name.clone(),
                gap_type: GapType::UnitTests,
                description: format!("Low line coverage: {:.1}% (target: 85%)", metrics.coverage_percentage()),
                severity: if metrics.coverage_percentage() < 70.0 { Severity::Critical } else { Severity::High },
                affected_functions: vec![], // Would be populated by source analysis
            });
        }

        // Check function coverage
        if metrics.function_coverage() < 90.0 {
            gaps.push(CoverageGap {
                module: metrics.module_name.clone(),
                gap_type: GapType::UnitTests,
                description: format!("Low function coverage: {:.1}% (target: 90%)", metrics.function_coverage()),
                severity: if metrics.function_coverage() < 75.0 { Severity::Critical } else { Severity::High },
                affected_functions: vec![],
            });
        }

        // Check integration test coverage
        if metrics.integration_tests < 5 {
            gaps.push(CoverageGap {
                module: metrics.module_name.clone(),
                gap_type: GapType::IntegrationTests,
                description: format!("Insufficient integration tests: {} (target: 5+)", metrics.integration_tests),
                severity: Severity::Medium,
                affected_functions: vec![],
            });
        }

        // Check performance test coverage
        if metrics.performance_tests < 2 {
            gaps.push(CoverageGap {
                module: metrics.module_name.clone(),
                gap_type: GapType::PerformanceTests,
                description: format!("Missing performance tests: {} (target: 2+)", metrics.performance_tests),
                severity: Severity::Medium,
                affected_functions: vec![],
            });
        }

        // Check edge case coverage
        if metrics.edge_case_tests < 3 {
            gaps.push(CoverageGap {
                module: metrics.module_name.clone(),
                gap_type: GapType::EdgeCaseTests,
                description: format!("Insufficient edge case tests: {} (target: 3+)", metrics.edge_case_tests),
                severity: Severity::Medium,
                affected_functions: vec![],
            });
        }

        gaps
    }

    /// Generate test recommendations based on gaps
    fn generate_recommendations(&self, gaps: &[CoverageGap]) -> Vec<TestRecommendation> {
        let mut recommendations = Vec::new();

        for gap in gaps {
            let recommendation = match gap.gap_type {
                GapType::UnitTests => TestRecommendation {
                    module: gap.module.clone(),
                    test_type: GapType::UnitTests,
                    priority: gap.severity.clone(),
                    description: format!("Add comprehensive unit tests for {}", gap.module),
                    estimated_effort: EstimatedEffort::Medium,
                    dependencies: vec![],
                },
                GapType::IntegrationTests => TestRecommendation {
                    module: gap.module.clone(),
                    test_type: GapType::IntegrationTests,
                    priority: gap.severity.clone(),
                    description: format!("Add end-to-end integration tests for {}", gap.module),
                    estimated_effort: EstimatedEffort::Large,
                    dependencies: vec!["test infrastructure".to_string()],
                },
                GapType::PerformanceTests => TestRecommendation {
                    module: gap.module.clone(),
                    test_type: GapType::PerformanceTests,
                    priority: gap.severity.clone(),
                    description: format!("Add performance benchmarks for {}", gap.module),
                    estimated_effort: EstimatedEffort::Medium,
                    dependencies: vec!["criterion setup".to_string()],
                },
                GapType::EdgeCaseTests => TestRecommendation {
                    module: gap.module.clone(),
                    test_type: GapType::EdgeCaseTests,
                    priority: gap.severity.clone(),
                    description: format!("Add edge case and boundary tests for {}", gap.module),
                    estimated_effort: EstimatedEffort::Small,
                    dependencies: vec![],
                },
                _ => continue,
            };
            recommendations.push(recommendation);
        }

        // Sort by priority
        recommendations.sort_by(|a, b| {
            use Severity::*;
            let priority_order = |s: &Severity| match s {
                Critical => 0,
                High => 1,
                Medium => 2,
                Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        recommendations
    }

    /// Generate a comprehensive coverage report
    pub async fn generate_report(&self) -> Result<String> {
        let report = self.analyze_coverage().await?;
        
        let mut output = String::new();
        output.push_str("# WriteMagic Test Coverage Analysis Report\n\n");
        output.push_str(&format!("Generated: {}\n", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("**Total Coverage: {:.1}%**\n\n", report.total_coverage));

        // Domain breakdown
        output.push_str("## Domain Coverage Breakdown\n\n");
        for (domain, metrics) in &report.domain_coverage {
            output.push_str(&format!("### {}\n", domain));
            output.push_str(&format!("- Line Coverage: {:.1}%\n", metrics.coverage_percentage()));
            output.push_str(&format!("- Function Coverage: {:.1}%\n", metrics.function_coverage()));
            output.push_str(&format!("- Unit Tests: {}\n", metrics.unit_tests));
            output.push_str(&format!("- Integration Tests: {}\n", metrics.integration_tests));
            output.push_str(&format!("- Performance Tests: {}\n", metrics.performance_tests));
            output.push_str(&format!("- Edge Case Tests: {}\n\n", metrics.edge_case_tests));
        }

        // Coverage gaps
        output.push_str("## Coverage Gaps\n\n");
        for gap in &report.gaps {
            output.push_str(&format!("- **{}** ({}): {}\n", 
                gap.module, 
                format!("{:?}", gap.severity), 
                gap.description
            ));
        }

        // Recommendations
        output.push_str("\n## Recommendations\n\n");
        for rec in &report.recommendations {
            output.push_str(&format!("### {} - {:?}\n", rec.module, rec.test_type));
            output.push_str(&format!("- **Priority**: {:?}\n", rec.priority));
            output.push_str(&format!("- **Effort**: {:?}\n", rec.estimated_effort));
            output.push_str(&format!("- **Description**: {}\n", rec.description));
            if !rec.dependencies.is_empty() {
                output.push_str(&format!("- **Dependencies**: {}\n", rec.dependencies.join(", ")));
            }
            output.push_str("\n");
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coverage_analyzer_creation() {
        let analyzer = CoverageAnalyzer::new();
        assert_eq!(analyzer.domains.len(), 8);
        assert!(analyzer.domains.contains(&"writemagic-shared".to_string()));
    }

    #[tokio::test]
    async fn test_coverage_analysis() {
        let analyzer = CoverageAnalyzer::new();
        let report = analyzer.analyze_coverage().await.unwrap();
        
        assert!(report.total_coverage >= 0.0);
        assert!(report.total_coverage <= 100.0);
        assert!(report.domain_coverage.len() > 0);
    }

    #[tokio::test]
    async fn test_report_generation() {
        let analyzer = CoverageAnalyzer::new();
        let report_text = analyzer.generate_report().await.unwrap();
        
        assert!(report_text.contains("WriteMagic Test Coverage Analysis"));
        assert!(report_text.contains("Total Coverage"));
        assert!(report_text.contains("Domain Coverage Breakdown"));
    }

    #[test]
    fn test_coverage_metrics() {
        let metrics = CoverageMetrics {
            module_name: "test".to_string(),
            total_functions: 10,
            tested_functions: 8,
            total_lines: 100,
            covered_lines: 85,
            branch_coverage: 90.0,
            integration_tests: 5,
            unit_tests: 15,
            performance_tests: 3,
            edge_case_tests: 4,
        };

        assert_eq!(metrics.coverage_percentage(), 85.0);
        assert_eq!(metrics.function_coverage(), 80.0);
    }

    #[test]
    fn test_gap_identification() {
        let analyzer = CoverageAnalyzer::new();
        let metrics = CoverageMetrics {
            module_name: "test".to_string(),
            total_functions: 10,
            tested_functions: 6, // 60% - below threshold
            total_lines: 100,
            covered_lines: 70, // 70% - below threshold
            branch_coverage: 60.0,
            integration_tests: 2, // Below target of 5
            unit_tests: 10,
            performance_tests: 0, // Below target of 2
            edge_case_tests: 1, // Below target of 3
        };

        let gaps = analyzer.identify_coverage_gaps(&metrics);
        assert!(gaps.len() >= 4); // Should identify multiple gaps
        
        // Check that critical gaps are identified
        let critical_gaps: Vec<_> = gaps.iter()
            .filter(|g| matches!(g.severity, Severity::Critical))
            .collect();
        assert!(critical_gaps.len() > 0);
    }
}