//! Utility functions for the temporal neural solver

use crate::core::types::*;
use crate::core::errors::*;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Timing utilities
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn new(name: String) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_micros(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1_000_000.0
    }

    pub fn elapsed_nanos(&self) -> u128 {
        self.elapsed().as_nanos()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        println!("⏱️  {} took {:.3}µs", self.name, elapsed.as_secs_f64() * 1_000_000.0);
    }
}

/// Create a timer with RAII semantics
pub fn time_block(name: &str) -> Timer {
    Timer::new(name.to_string())
}

/// Calculate performance metrics from timing data
pub fn calculate_metrics(timings: &[Duration]) -> PerformanceMetrics {
    if timings.is_empty() {
        return PerformanceMetrics {
            min_latency: Duration::ZERO,
            max_latency: Duration::ZERO,
            mean_latency: Duration::ZERO,
            p50_latency: Duration::ZERO,
            p90_latency: Duration::ZERO,
            p99_latency: Duration::ZERO,
            p999_latency: Duration::ZERO,
            throughput_ops_per_sec: 0.0,
            samples: 0,
        };
    }

    let mut sorted_timings = timings.to_vec();
    sorted_timings.sort_unstable();

    let n = sorted_timings.len();
    let sum: Duration = sorted_timings.iter().sum();
    let mean = sum / n as u32;

    let min_latency = sorted_timings[0];
    let max_latency = sorted_timings[n - 1];
    let p50_latency = sorted_timings[n / 2];
    let p90_latency = sorted_timings[(n * 9) / 10];
    let p99_latency = sorted_timings[(n * 99) / 100];
    let p999_latency = sorted_timings[((n * 999) / 1000).min(n - 1)];

    let throughput_ops_per_sec = if p50_latency > Duration::ZERO {
        1.0 / p50_latency.as_secs_f64()
    } else {
        0.0
    };

    PerformanceMetrics {
        min_latency,
        max_latency,
        mean_latency: mean,
        p50_latency,
        p90_latency,
        p99_latency,
        p999_latency,
        throughput_ops_per_sec,
        samples: n,
    }
}

/// Format duration in human-readable form
pub fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();

    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.1}µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.1}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.1}s", nanos as f64 / 1_000_000_000.0)
    }
}

/// Validate input vector dimensions
pub fn validate_input(input: &[f32], expected_size: usize) -> Result<()> {
    if input.len() != expected_size {
        return Err(TemporalSolverError::DimensionMismatch {
            expected: expected_size,
            got: input.len(),
        });
    }

    // Check for NaN or infinite values
    for (i, &value) in input.iter().enumerate() {
        if !value.is_finite() {
            return Err(TemporalSolverError::NumericalError(
                format!("Invalid value {} at index {}", value, i)
            ));
        }
    }

    Ok(())
}

/// Generate deterministic test data
pub fn generate_test_data(size: usize, seed: u64) -> Vec<f32> {
    let mut data = Vec::with_capacity(size);

    for i in 0..size {
        // Use simple but deterministic pattern
        let value = ((i as f64 * 0.01 + seed as f64 * 0.001).sin() + 1.0) * 0.5;
        data.push(value as f32);
    }

    data
}

/// Generate test input vector
pub fn generate_test_input(seed: u64) -> InputVector {
    let data = generate_test_data(128, seed);
    let mut input = [0.0f32; 128];
    input.copy_from_slice(&data);
    input
}

/// Compare two output vectors with tolerance
pub fn compare_outputs(a: &OutputVector, b: &OutputVector, tolerance: f32) -> bool {
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = (x - y).abs();
        let rel_error = if x.abs() > tolerance {
            diff / x.abs()
        } else {
            diff
        };

        if rel_error > tolerance {
            return false;
        }
    }

    true
}

/// Calculate speedup ratio
pub fn calculate_speedup(baseline_duration: Duration, optimized_duration: Duration) -> f64 {
    if optimized_duration > Duration::ZERO {
        baseline_duration.as_secs_f64() / optimized_duration.as_secs_f64()
    } else {
        0.0
    }
}

/// Detect CPU features at runtime
pub fn detect_cpu_features() -> HardwareFeatures {
    #[cfg(target_arch = "x86_64")]
    {
        HardwareFeatures {
            has_avx: is_x86_feature_detected!("avx"),
            has_avx2: is_x86_feature_detected!("avx2"),
            has_avx512: is_x86_feature_detected!("avx512f"),
            has_fma: is_x86_feature_detected!("fma"),
            has_sse4_2: is_x86_feature_detected!("sse4.2"),
            cpu_cores: num_cpus::get_physical(),
            cache_line_size: 64, // Typical x86_64 cache line size
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        HardwareFeatures {
            has_avx: false,
            has_avx2: false,
            has_avx512: false,
            has_fma: false,
            has_sse4_2: false,
            cpu_cores: num_cpus::get_physical(),
            cache_line_size: 64,
        }
    }
}

/// System information gathering
pub fn get_system_info() -> HashMap<String, String> {
    let mut info = HashMap::new();

    info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    info.insert("os".to_string(), std::env::consts::OS.to_string());
    info.insert("family".to_string(), std::env::consts::FAMILY.to_string());

    let features = detect_cpu_features();
    info.insert("cpu_cores".to_string(), features.cpu_cores.to_string());
    info.insert("has_avx2".to_string(), features.has_avx2.to_string());
    info.insert("has_avx512".to_string(), features.has_avx512.to_string());

    // Add environment variables
    if let Ok(rust_version) = std::env::var("CARGO_PKG_RUST_VERSION") {
        info.insert("rust_version".to_string(), rust_version);
    }

    if let Ok(profile) = std::env::var("PROFILE") {
        info.insert("profile".to_string(), profile);
    }

    if let Ok(target) = std::env::var("TARGET") {
        info.insert("target".to_string(), target);
    }

    info
}

/// Memory alignment utilities
pub fn is_aligned(ptr: *const u8, alignment: usize) -> bool {
    (ptr as usize) % alignment == 0
}

pub fn check_simd_alignment(data: &[f32]) -> bool {
    let ptr = data.as_ptr() as *const u8;
    is_aligned(ptr, 32) // AVX2 requires 32-byte alignment
}

/// Statistical helper functions
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let m = mean(data);
    let sum_sq_diff: f64 = data.iter().map(|x| (x - m).powi(2)).sum();
    sum_sq_diff / (data.len() - 1) as f64
}

pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

pub fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = (p * (sorted.len() - 1) as f64).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

/// Progress tracking utilities
pub struct ProgressBar {
    total: usize,
    current: usize,
    start_time: Instant,
    last_update: Instant,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        let now = Instant::now();
        Self {
            total,
            current: 0,
            start_time: now,
            last_update: now,
        }
    }

    pub fn update(&mut self, current: usize) {
        self.current = current;
        let now = Instant::now();

        // Update every 100ms
        if now.duration_since(self.last_update) > Duration::from_millis(100) {
            self.display();
            self.last_update = now;
        }
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.display();
        println!(); // New line after completion
    }

    fn display(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.start_time.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            self.current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let eta = if rate > 0.0 && self.current < self.total {
            Duration::from_secs_f64((self.total - self.current) as f64 / rate)
        } else {
            Duration::ZERO
        };

        print!("\r🔄 Progress: {:.1}% ({}/{}) | {:.1} it/s | ETA: {}",
            percentage, self.current, self.total, rate, format_duration(eta));
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics() {
        let timings = vec![
            Duration::from_micros(100),
            Duration::from_micros(200),
            Duration::from_micros(150),
            Duration::from_micros(120),
            Duration::from_micros(180),
        ];

        let metrics = calculate_metrics(&timings);
        assert_eq!(metrics.samples, 5);
        assert!(metrics.throughput_ops_per_sec > 0.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_nanos(500)), "500ns");
        assert_eq!(format_duration(Duration::from_micros(1500)), "1.5ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.5s");
    }

    #[test]
    fn test_validate_input() {
        let input = vec![1.0, 2.0, 3.0];
        assert!(validate_input(&input, 3).is_ok());
        assert!(validate_input(&input, 4).is_err());

        let invalid_input = vec![1.0, f32::NAN, 3.0];
        assert!(validate_input(&invalid_input, 3).is_err());
    }

    #[test]
    fn test_generate_test_data() {
        let data1 = generate_test_data(10, 42);
        let data2 = generate_test_data(10, 42);
        assert_eq!(data1, data2); // Should be deterministic

        let data3 = generate_test_data(10, 43);
        assert_ne!(data1, data3); // Different seed should give different data
    }

    #[test]
    fn test_compare_outputs() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [1.01, 1.99, 3.01, 3.99];

        assert!(compare_outputs(&a, &b, 0.02)); // Within tolerance
        assert!(!compare_outputs(&a, &b, 0.005)); // Outside tolerance
    }

    #[test]
    fn test_calculate_speedup() {
        let baseline = Duration::from_micros(100);
        let optimized = Duration::from_micros(20);

        let speedup = calculate_speedup(baseline, optimized);
        assert!((speedup - 5.0).abs() < 0.001);
    }
}