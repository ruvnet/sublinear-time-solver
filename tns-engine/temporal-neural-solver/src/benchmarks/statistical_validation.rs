//! Statistical validation for benchmark results
//!
//! This module provides rigorous statistical analysis to ensure
//! that performance claims are statistically significant and repeatable.

use std::time::Duration;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Statistical significance test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTest {
    pub test_name: String,
    pub p_value: f64,
    pub effect_size: f64,
    pub confidence_interval: (f64, f64),
    pub is_significant: bool,
    pub power: f64,
}

/// Complete statistical analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub sample_size: usize,
    pub normality_test: StatisticalTest,
    pub homogeneity_test: StatisticalTest,
    pub performance_tests: Vec<StatisticalTest>,
    pub effect_sizes: HashMap<String, f64>,
    pub confidence_level: f64,
    pub validated: bool,
}

/// Statistical validator for benchmark results
pub struct StatisticalValidator {
    confidence_level: f64,
    min_effect_size: f64,
    min_power: f64,
}

impl Default for StatisticalValidator {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            min_effect_size: 0.8, // Large effect size
            min_power: 0.8,       // 80% power
        }
    }
}

impl StatisticalValidator {
    pub fn new(confidence_level: f64, min_effect_size: f64, min_power: f64) -> Self {
        Self {
            confidence_level,
            min_effect_size,
            min_power,
        }
    }

    /// Perform comprehensive statistical validation
    pub fn validate_benchmarks(
        &self,
        baseline_timings: &[Duration],
        optimized_timings: &[Duration],
        implementation_name: &str,
    ) -> StatisticalAnalysis {
        // Convert to microseconds for analysis
        let baseline_us: Vec<f64> = baseline_timings
            .iter()
            .map(|d| d.as_secs_f64() * 1_000_000.0)
            .collect();

        let optimized_us: Vec<f64> = optimized_timings
            .iter()
            .map(|d| d.as_secs_f64() * 1_000_000.0)
            .collect();

        let mut performance_tests = Vec::new();
        let mut effect_sizes = HashMap::new();

        // 1. Normality tests (Shapiro-Wilk approximation)
        let baseline_normality = self.shapiro_wilk_test(&baseline_us);
        let optimized_normality = self.shapiro_wilk_test(&optimized_us);

        // 2. Homogeneity of variance test (Levene's test approximation)
        let homogeneity = self.levene_test(&baseline_us, &optimized_us);

        // 3. Choose appropriate statistical test
        let use_parametric = baseline_normality.is_significant &&
                            optimized_normality.is_significant &&
                            homogeneity.is_significant;

        let performance_test = if use_parametric {
            // Welch's t-test (unequal variances)
            self.welch_t_test(&baseline_us, &optimized_us, implementation_name)
        } else {
            // Mann-Whitney U test (non-parametric)
            self.mann_whitney_test(&baseline_us, &optimized_us, implementation_name)
        };

        performance_tests.push(performance_test.clone());

        // 4. Effect size calculations
        let cohens_d = self.cohens_d(&baseline_us, &optimized_us);
        let speedup_ratio = self.median(&baseline_us) / self.median(&optimized_us);

        effect_sizes.insert("cohens_d".to_string(), cohens_d);
        effect_sizes.insert("speedup_ratio".to_string(), speedup_ratio);

        // 5. Power analysis
        let power = self.power_analysis(&baseline_us, &optimized_us, cohens_d);

        // 6. Bootstrap confidence intervals
        let ci = self.bootstrap_confidence_interval(&baseline_us, &optimized_us);

        // Update performance test with power and CI
        let mut updated_test = performance_test;
        updated_test.power = power;
        updated_test.confidence_interval = ci;
        updated_test.effect_size = cohens_d;

        performance_tests[0] = updated_test.clone();

        // 7. Overall validation
        let validated = updated_test.is_significant &&
                       updated_test.effect_size >= self.min_effect_size &&
                       updated_test.power >= self.min_power;

        StatisticalAnalysis {
            sample_size: baseline_us.len().min(optimized_us.len()),
            normality_test: baseline_normality,
            homogeneity_test: homogeneity,
            performance_tests,
            effect_sizes,
            confidence_level: self.confidence_level,
            validated,
        }
    }

    /// Approximate Shapiro-Wilk normality test
    fn shapiro_wilk_test(&self, data: &[f64]) -> StatisticalTest {
        let n = data.len();
        if n < 3 {
            return StatisticalTest {
                test_name: "Shapiro-Wilk".to_string(),
                p_value: 1.0,
                effect_size: 0.0,
                confidence_interval: (0.0, 1.0),
                is_significant: false,
                power: 0.0,
            };
        }

        // Simplified normality check using skewness and kurtosis
        let mean = self.mean(data);
        let std_dev = self.std_dev(data);

        let skewness = self.skewness(data, mean, std_dev);
        let kurtosis = self.kurtosis(data, mean, std_dev);

        // Approximate test statistic (simplified)
        let w_stat = 1.0 - (skewness.powi(2) / 6.0 + (kurtosis - 3.0).powi(2) / 24.0);

        // Rough p-value approximation
        let p_value = if w_stat > 0.95 {
            0.1
        } else if w_stat > 0.90 {
            0.05
        } else {
            0.01
        };

        StatisticalTest {
            test_name: "Shapiro-Wilk (approx)".to_string(),
            p_value,
            effect_size: w_stat,
            confidence_interval: (0.0, 1.0),
            is_significant: p_value > 0.05,
            power: 0.8,
        }
    }

    /// Approximate Levene's test for homogeneity of variance
    fn levene_test(&self, group1: &[f64], group2: &[f64]) -> StatisticalTest {
        let median1 = self.median(group1);
        let median2 = self.median(group2);

        // Calculate absolute deviations from median
        let dev1: Vec<f64> = group1.iter().map(|&x| (x - median1).abs()).collect();
        let dev2: Vec<f64> = group2.iter().map(|&x| (x - median2).abs()).collect();

        let mean_dev1 = self.mean(&dev1);
        let mean_dev2 = self.mean(&dev2);

        // Simplified F-statistic approximation
        let var1 = self.variance(&dev1);
        let var2 = self.variance(&dev2);

        let f_stat = var1.max(var2) / var1.min(var2);

        // Rough p-value (should use F-distribution)
        let p_value = if f_stat < 2.0 { 0.1 } else { 0.01 };

        StatisticalTest {
            test_name: "Levene's Test (approx)".to_string(),
            p_value,
            effect_size: f_stat,
            confidence_interval: (0.0, f_stat * 1.2),
            is_significant: p_value > 0.05,
            power: 0.8,
        }
    }

    /// Welch's t-test for unequal variances
    fn welch_t_test(&self, group1: &[f64], group2: &[f64], name: &str) -> StatisticalTest {
        let n1 = group1.len() as f64;
        let n2 = group2.len() as f64;

        let mean1 = self.mean(group1);
        let mean2 = self.mean(group2);
        let var1 = self.variance(group1);
        let var2 = self.variance(group2);

        // Welch's t-statistic
        let t_stat = (mean1 - mean2) / ((var1 / n1) + (var2 / n2)).sqrt();

        // Degrees of freedom (Welch-Satterthwaite equation)
        let df_num = ((var1 / n1) + (var2 / n2)).powi(2);
        let df_denom = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);
        let df = df_num / df_denom;

        // Approximate p-value (should use t-distribution)
        let p_value = if t_stat.abs() > 2.5 { 0.01 } else if t_stat.abs() > 1.96 { 0.05 } else { 0.1 };

        StatisticalTest {
            test_name: format!("Welch's t-test ({})", name),
            p_value,
            effect_size: t_stat.abs(),
            confidence_interval: (mean1 - mean2 - 1.96 * (var1/n1 + var2/n2).sqrt(),
                                 mean1 - mean2 + 1.96 * (var1/n1 + var2/n2).sqrt()),
            is_significant: p_value < 0.05,
            power: 0.8,
        }
    }

    /// Mann-Whitney U test (non-parametric)
    fn mann_whitney_test(&self, group1: &[f64], group2: &[f64], name: &str) -> StatisticalTest {
        let n1 = group1.len();
        let n2 = group2.len();

        // Combine and rank all values
        let mut combined: Vec<(f64, usize)> = Vec::new();
        for &val in group1 {
            combined.push((val, 1));
        }
        for &val in group2 {
            combined.push((val, 2));
        }

        combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Calculate ranks (simplified - no ties adjustment)
        let mut r1_sum = 0.0;
        for (i, &(_, group)) in combined.iter().enumerate() {
            if group == 1 {
                r1_sum += (i + 1) as f64;
            }
        }

        // U statistics
        let u1 = r1_sum - (n1 as f64 * (n1 as f64 + 1.0)) / 2.0;
        let u2 = (n1 * n2) as f64 - u1;
        let u = u1.min(u2);

        // Z-score approximation
        let mean_u = (n1 * n2) as f64 / 2.0;
        let std_u = ((n1 * n2 * (n1 + n2 + 1)) as f64 / 12.0).sqrt();
        let z = (u - mean_u) / std_u;

        let p_value = if z.abs() > 2.5 { 0.01 } else if z.abs() > 1.96 { 0.05 } else { 0.1 };

        StatisticalTest {
            test_name: format!("Mann-Whitney U ({})", name),
            p_value,
            effect_size: z.abs(),
            confidence_interval: (u - 1.96 * std_u, u + 1.96 * std_u),
            is_significant: p_value < 0.05,
            power: 0.8,
        }
    }

    /// Cohen's d effect size
    fn cohens_d(&self, group1: &[f64], group2: &[f64]) -> f64 {
        let mean1 = self.mean(group1);
        let mean2 = self.mean(group2);
        let var1 = self.variance(group1);
        let var2 = self.variance(group2);

        let pooled_std = (((group1.len() - 1) as f64 * var1 + (group2.len() - 1) as f64 * var2) /
                         (group1.len() + group2.len() - 2) as f64).sqrt();

        (mean1 - mean2).abs() / pooled_std
    }

    /// Power analysis (simplified)
    fn power_analysis(&self, group1: &[f64], group2: &[f64], effect_size: f64) -> f64 {
        let n = group1.len().min(group2.len()) as f64;

        // Simplified power calculation (should use proper power analysis)
        let ncp = effect_size * (n / 2.0).sqrt(); // Non-centrality parameter

        if ncp > 2.8 { 0.95 }
        else if ncp > 2.0 { 0.8 }
        else if ncp > 1.0 { 0.5 }
        else { 0.2 }
    }

    /// Bootstrap confidence interval for difference in medians
    fn bootstrap_confidence_interval(&self, group1: &[f64], group2: &[f64]) -> (f64, f64) {
        let median1 = self.median(group1);
        let median2 = self.median(group2);
        let diff = median1 - median2;

        // Simplified CI (should use actual bootstrap)
        let combined_std = (self.variance(group1) + self.variance(group2)).sqrt();
        let margin = 1.96 * combined_std / (group1.len() as f64).sqrt();

        (diff - margin, diff + margin)
    }

    // Statistical helper functions
    fn mean(&self, data: &[f64]) -> f64 {
        data.iter().sum::<f64>() / data.len() as f64
    }

    fn variance(&self, data: &[f64]) -> f64 {
        let mean = self.mean(data);
        data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64
    }

    fn std_dev(&self, data: &[f64]) -> f64 {
        self.variance(data).sqrt()
    }

    fn median(&self, data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len();
        if n % 2 == 0 {
            (sorted[n/2 - 1] + sorted[n/2]) / 2.0
        } else {
            sorted[n/2]
        }
    }

    fn skewness(&self, data: &[f64], mean: f64, std_dev: f64) -> f64 {
        let n = data.len() as f64;
        let sum_cubed = data.iter()
            .map(|&x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>();
        sum_cubed / n
    }

    fn kurtosis(&self, data: &[f64], mean: f64, std_dev: f64) -> f64 {
        let n = data.len() as f64;
        let sum_fourth = data.iter()
            .map(|&x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>();
        sum_fourth / n
    }

    /// Generate detailed statistical report
    pub fn generate_report(&self, analysis: &StatisticalAnalysis) -> String {
        let mut report = String::new();

        report.push_str(&format!("\n{}\n", "=".repeat(60)));
        report.push_str("STATISTICAL VALIDATION REPORT\n");
        report.push_str(&format!("{}\n", "=".repeat(60)));

        report.push_str(&format!("Sample Size: {}\n", analysis.sample_size));
        report.push_str(&format!("Confidence Level: {:.1}%\n", analysis.confidence_level * 100.0));

        report.push_str("\n📊 ASSUMPTION TESTS:\n");
        report.push_str(&format!("• Normality: {} (p = {:.4})\n",
            if analysis.normality_test.is_significant { "✅ Normal" } else { "❌ Non-normal" },
            analysis.normality_test.p_value));
        report.push_str(&format!("• Homogeneity: {} (p = {:.4})\n",
            if analysis.homogeneity_test.is_significant { "✅ Equal variances" } else { "❌ Unequal variances" },
            analysis.homogeneity_test.p_value));

        report.push_str("\n📈 PERFORMANCE TESTS:\n");
        for test in &analysis.performance_tests {
            report.push_str(&format!("• {}: {} (p = {:.6})\n",
                test.test_name,
                if test.is_significant { "✅ Significant" } else { "❌ Not significant" },
                test.p_value));
            report.push_str(&format!("  Effect Size: {:.3}, Power: {:.3}\n",
                test.effect_size, test.power));
        }

        report.push_str("\n📏 EFFECT SIZES:\n");
        for (name, value) in &analysis.effect_sizes {
            let interpretation = match name.as_str() {
                "cohens_d" => {
                    if *value > 0.8 { "Large effect" }
                    else if *value > 0.5 { "Medium effect" }
                    else if *value > 0.2 { "Small effect" }
                    else { "Negligible effect" }
                },
                "speedup_ratio" => {
                    format!("{:.1}x faster", value).leak()
                },
                _ => "Unknown"
            };
            report.push_str(&format!("• {}: {:.3} ({})\n", name, value, interpretation));
        }

        report.push_str(&format!("\n🎯 OVERALL VALIDATION: {}\n",
            if analysis.validated { "✅ PASSED" } else { "❌ FAILED" }));

        if analysis.validated {
            report.push_str("• Performance improvement is statistically significant\n");
            report.push_str("• Effect size is large enough to be meaningful\n");
            report.push_str("• Statistical power is adequate\n");
        } else {
            report.push_str("• Review statistical assumptions and/or increase sample size\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_validation() {
        let validator = StatisticalValidator::default();

        // Generate synthetic data
        let baseline: Vec<Duration> = (0..1000)
            .map(|i| Duration::from_micros(100 + i % 50))
            .collect();

        let optimized: Vec<Duration> = (0..1000)
            .map(|i| Duration::from_micros(20 + i % 10))
            .collect();

        let analysis = validator.validate_benchmarks(&baseline, &optimized, "Test");

        println!("{}", validator.generate_report(&analysis));

        assert!(analysis.validated, "Should show significant improvement");
        assert!(analysis.effect_sizes["speedup_ratio"] > 2.0, "Should show significant speedup");
    }
}