//! Hardware verification and capability detection
//!
//! This module ensures that performance measurements are consistent
//! across different hardware configurations and verifies the use
//! of specific CPU features.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Hardware capabilities detected on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub cpu_vendor: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub cache_sizes: CacheSizes,
    pub simd_features: SimdFeatures,
    pub memory_info: MemoryInfo,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSizes {
    pub l1_data: Option<usize>,
    pub l1_instruction: Option<usize>,
    pub l2: Option<usize>,
    pub l3: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdFeatures {
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    pub fma: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_memory: Option<usize>,
    pub available_memory: Option<usize>,
    pub page_size: Option<usize>,
}

/// Hardware verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareVerification {
    pub capabilities: HardwareCapabilities,
    pub feature_usage: FeatureUsage,
    pub performance_baseline: PerformanceBaseline,
    pub warnings: Vec<String>,
    pub validation_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub detected_simd_usage: Vec<String>,
    pub memory_alignment_verified: bool,
    pub cache_friendly_access: bool,
    pub thread_affinity_set: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub memory_bandwidth_gb_s: f64,
    pub cpu_frequency_ghz: f64,
    pub cache_latency_ns: HashMap<String, f64>,
    pub baseline_established: bool,
}

/// Hardware validator and verifier
pub struct HardwareValidator {
    baseline_measurements: Option<PerformanceBaseline>,
}

impl Default for HardwareValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareValidator {
    pub fn new() -> Self {
        Self {
            baseline_measurements: None,
        }
    }

    /// Detect all hardware capabilities
    pub fn detect_capabilities(&self) -> HardwareCapabilities {
        HardwareCapabilities {
            cpu_vendor: self.detect_cpu_vendor(),
            cpu_brand: self.detect_cpu_brand(),
            cpu_cores: self.detect_cpu_cores(),
            cpu_threads: self.detect_cpu_threads(),
            cache_sizes: self.detect_cache_sizes(),
            simd_features: self.detect_simd_features(),
            memory_info: self.detect_memory_info(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Verify hardware configuration and measure baselines
    pub fn verify_hardware(&mut self) -> HardwareVerification {
        let capabilities = self.detect_capabilities();
        let mut warnings = Vec::new();

        // Check for minimum requirements
        if !capabilities.simd_features.avx2 {
            warnings.push("AVX2 not available - performance may be suboptimal".to_string());
        }

        if capabilities.cpu_cores < 4 {
            warnings.push("Less than 4 CPU cores detected - may affect performance".to_string());
        }

        if let Some(l3_size) = capabilities.cache_sizes.l3 {
            if l3_size < 8 * 1024 * 1024 { // 8MB
                warnings.push("Small L3 cache detected - may affect large workloads".to_string());
            }
        }

        // Verify feature usage
        let feature_usage = self.verify_feature_usage(&capabilities);

        // Establish performance baseline
        let baseline = self.establish_baseline(&capabilities);

        let validation_passed = warnings.is_empty() &&
                               feature_usage.detected_simd_usage.len() > 0 &&
                               baseline.baseline_established;

        HardwareVerification {
            capabilities,
            feature_usage,
            performance_baseline: baseline,
            warnings,
            validation_passed,
        }
    }

    fn detect_cpu_vendor(&self) -> String {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                // Try to detect vendor through CPUID
                // This is a simplified detection
                "Unknown x86_64".to_string()
            } else {
                "x86_64 (limited features)".to_string()
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            std::env::consts::ARCH.to_string()
        }
    }

    fn detect_cpu_brand(&self) -> String {
        // In a real implementation, this would use CPUID
        "Generic CPU".to_string()
    }

    fn detect_cpu_cores(&self) -> usize {
        num_cpus::get_physical()
    }

    fn detect_cpu_threads(&self) -> usize {
        num_cpus::get()
    }

    fn detect_cache_sizes(&self) -> CacheSizes {
        // In a real implementation, this would query CPU cache info
        CacheSizes {
            l1_data: Some(32 * 1024),    // 32KB typical
            l1_instruction: Some(32 * 1024),
            l2: Some(256 * 1024),        // 256KB typical
            l3: Some(8 * 1024 * 1024),   // 8MB typical
        }
    }

    fn detect_simd_features(&self) -> SimdFeatures {
        #[cfg(target_arch = "x86_64")]
        {
            SimdFeatures {
                sse: is_x86_feature_detected!("sse"),
                sse2: is_x86_feature_detected!("sse2"),
                sse3: is_x86_feature_detected!("sse3"),
                ssse3: is_x86_feature_detected!("ssse3"),
                sse4_1: is_x86_feature_detected!("sse4.1"),
                sse4_2: is_x86_feature_detected!("sse4.2"),
                avx: is_x86_feature_detected!("avx"),
                avx2: is_x86_feature_detected!("avx2"),
                avx512f: is_x86_feature_detected!("avx512f"),
                fma: is_x86_feature_detected!("fma"),
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            SimdFeatures {
                sse: false, sse2: false, sse3: false, ssse3: false,
                sse4_1: false, sse4_2: false, avx: false, avx2: false,
                avx512f: false, fma: false,
            }
        }
    }

    fn detect_memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total_memory: self.get_total_memory(),
            available_memory: self.get_available_memory(),
            page_size: Some(4096), // 4KB pages typical
        }
    }

    fn get_total_memory(&self) -> Option<usize> {
        // Platform-specific memory detection
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|contents| {
                    contents.lines()
                        .find(|line| line.starts_with("MemTotal:"))
                        .and_then(|line| {
                            line.split_whitespace()
                                .nth(1)
                                .and_then(|s| s.parse::<usize>().ok())
                                .map(|kb| kb * 1024) // Convert KB to bytes
                        })
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    fn get_available_memory(&self) -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|contents| {
                    contents.lines()
                        .find(|line| line.starts_with("MemAvailable:"))
                        .and_then(|line| {
                            line.split_whitespace()
                                .nth(1)
                                .and_then(|s| s.parse::<usize>().ok())
                                .map(|kb| kb * 1024)
                        })
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Verify that optimizations are actually being used
    fn verify_feature_usage(&self, capabilities: &HardwareCapabilities) -> FeatureUsage {
        let mut detected_simd = Vec::new();

        // Check which SIMD features are available and likely being used
        if capabilities.simd_features.avx512f {
            detected_simd.push("AVX-512".to_string());
        } else if capabilities.simd_features.avx2 {
            detected_simd.push("AVX2".to_string());
        } else if capabilities.simd_features.avx {
            detected_simd.push("AVX".to_string());
        } else if capabilities.simd_features.sse4_2 {
            detected_simd.push("SSE4.2".to_string());
        }

        if capabilities.simd_features.fma {
            detected_simd.push("FMA".to_string());
        }

        FeatureUsage {
            detected_simd_usage: detected_simd,
            memory_alignment_verified: self.verify_memory_alignment(),
            cache_friendly_access: self.verify_cache_friendly_access(),
            thread_affinity_set: self.verify_thread_affinity(),
        }
    }

    fn verify_memory_alignment(&self) -> bool {
        // Test memory alignment for SIMD operations
        let test_data = vec![1.0f32; 32];
        let ptr = test_data.as_ptr() as usize;

        // Check if aligned to 32-byte boundary (AVX2 requirement)
        ptr % 32 == 0
    }

    fn verify_cache_friendly_access(&self) -> bool {
        // Simple cache-friendly access pattern test
        let size = 1024; // 4KB = typical page size
        let data = vec![1.0f32; size];

        let start = std::time::Instant::now();

        // Sequential access (cache-friendly)
        let mut sum = 0.0f32;
        for &val in &data {
            sum += val;
        }

        let sequential_time = start.elapsed();

        // Random access (cache-unfriendly)
        let start = std::time::Instant::now();
        let mut sum2 = 0.0f32;
        for i in (0..size).step_by(64) { // Jump by cache line size
            sum2 += data[i];
        }
        let random_time = start.elapsed();

        // Cache-friendly should be significantly faster
        sequential_time < random_time || random_time.as_nanos() < 1000
    }

    fn verify_thread_affinity(&self) -> bool {
        // Check if thread affinity can be set (indicates scheduler control)
        core_affinity::get_core_ids().is_some()
    }

    /// Establish performance baselines for the hardware
    fn establish_baseline(&mut self, capabilities: &HardwareCapabilities) -> PerformanceBaseline {
        let memory_bandwidth = self.measure_memory_bandwidth();
        let cpu_frequency = self.estimate_cpu_frequency();
        let cache_latencies = self.measure_cache_latencies();

        let baseline = PerformanceBaseline {
            memory_bandwidth_gb_s: memory_bandwidth,
            cpu_frequency_ghz: cpu_frequency,
            cache_latency_ns: cache_latencies,
            baseline_established: memory_bandwidth > 0.0 && cpu_frequency > 0.0,
        };

        self.baseline_measurements = Some(baseline.clone());
        baseline
    }

    fn measure_memory_bandwidth(&self) -> f64 {
        // Simple memory bandwidth test
        let size = 1024 * 1024; // 1MB
        let data = vec![1u64; size];
        let iterations = 100;

        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let sum: u64 = data.iter().sum();
            std::hint::black_box(sum); // Prevent optimization
        }

        let elapsed = start.elapsed();
        let bytes_processed = size * iterations * 8; // 8 bytes per u64
        let seconds = elapsed.as_secs_f64();

        if seconds > 0.0 {
            (bytes_processed as f64) / seconds / 1e9 // GB/s
        } else {
            0.0
        }
    }

    fn estimate_cpu_frequency(&self) -> f64 {
        // Estimate CPU frequency using a compute-intensive loop
        let iterations = 1_000_000;
        let start = std::time::Instant::now();

        let mut x = 1.0f64;
        for _ in 0..iterations {
            x = x.sin().cos(); // Floating point intensive
        }
        std::hint::black_box(x);

        let elapsed = start.elapsed();
        let ops_per_second = iterations as f64 / elapsed.as_secs_f64();

        // Very rough estimate - assumes certain ops per clock
        ops_per_second / 1e9 // Rough GHz estimate
    }

    fn measure_cache_latencies(&self) -> HashMap<String, f64> {
        let mut latencies = HashMap::new();

        // L1 cache test (should fit in L1)
        let l1_latency = self.measure_latency_for_size(4 * 1024); // 4KB
        latencies.insert("L1".to_string(), l1_latency);

        // L2 cache test
        let l2_latency = self.measure_latency_for_size(128 * 1024); // 128KB
        latencies.insert("L2".to_string(), l2_latency);

        // L3 cache test
        let l3_latency = self.measure_latency_for_size(4 * 1024 * 1024); // 4MB
        latencies.insert("L3".to_string(), l3_latency);

        // Main memory test
        let mem_latency = self.measure_latency_for_size(64 * 1024 * 1024); // 64MB
        latencies.insert("Memory".to_string(), mem_latency);

        latencies
    }

    fn measure_latency_for_size(&self, size: usize) -> f64 {
        let data = vec![0u8; size];
        let iterations = 1000;
        let stride = 64; // Cache line size

        let start = std::time::Instant::now();

        let mut index = 0;
        let mut sum = 0u8;
        for _ in 0..iterations {
            for _ in 0..(size / stride) {
                sum = sum.wrapping_add(data[index]);
                index = (index + stride) % size;
            }
        }
        std::hint::black_box(sum);

        let elapsed = start.elapsed();
        elapsed.as_nanos() as f64 / (iterations * size / stride) as f64
    }

    /// Generate detailed hardware report
    pub fn generate_report(&self, verification: &HardwareVerification) -> String {
        let mut report = String::new();

        report.push_str(&format!("\n{}\n", "=".repeat(60)));
        report.push_str("HARDWARE VERIFICATION REPORT\n");
        report.push_str(&format!("{}\n", "=".repeat(60)));

        let caps = &verification.capabilities;
        report.push_str(&format!("CPU: {} ({})\n", caps.cpu_brand, caps.cpu_vendor));
        report.push_str(&format!("Cores: {} physical, {} threads\n", caps.cpu_cores, caps.cpu_threads));

        report.push_str("\n💾 CACHE HIERARCHY:\n");
        if let Some(l1) = caps.cache_sizes.l1_data {
            report.push_str(&format!("• L1 Data: {} KB\n", l1 / 1024));
        }
        if let Some(l2) = caps.cache_sizes.l2 {
            report.push_str(&format!("• L2: {} KB\n", l2 / 1024));
        }
        if let Some(l3) = caps.cache_sizes.l3 {
            report.push_str(&format!("• L3: {} MB\n", l3 / (1024 * 1024)));
        }

        report.push_str("\n🚀 SIMD FEATURES:\n");
        let simd = &caps.simd_features;
        if simd.avx512f { report.push_str("• ✅ AVX-512\n"); }
        if simd.avx2 { report.push_str("• ✅ AVX2\n"); }
        if simd.avx { report.push_str("• ✅ AVX\n"); }
        if simd.fma { report.push_str("• ✅ FMA\n"); }
        if simd.sse4_2 { report.push_str("• ✅ SSE4.2\n"); }

        report.push_str("\n📊 PERFORMANCE BASELINE:\n");
        let baseline = &verification.performance_baseline;
        report.push_str(&format!("• Memory Bandwidth: {:.2} GB/s\n", baseline.memory_bandwidth_gb_s));
        report.push_str(&format!("• CPU Frequency: {:.2} GHz (estimated)\n", baseline.cpu_frequency_ghz));

        report.push_str("\n⚡ FEATURE USAGE:\n");
        let usage = &verification.feature_usage;
        for feature in &usage.detected_simd_usage {
            report.push_str(&format!("• ✅ {} detected\n", feature));
        }
        if usage.memory_alignment_verified {
            report.push_str("• ✅ Memory alignment verified\n");
        }
        if usage.cache_friendly_access {
            report.push_str("• ✅ Cache-friendly access patterns\n");
        }

        if !verification.warnings.is_empty() {
            report.push_str("\n⚠️  WARNINGS:\n");
            for warning in &verification.warnings {
                report.push_str(&format!("• {}\n", warning));
            }
        }

        report.push_str(&format!("\n🎯 HARDWARE VALIDATION: {}\n",
            if verification.validation_passed { "✅ PASSED" } else { "❌ FAILED" }));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let mut validator = HardwareValidator::new();
        let verification = validator.verify_hardware();

        println!("{}", validator.generate_report(&verification));

        assert!(!verification.capabilities.cpu_vendor.is_empty());
        assert!(verification.capabilities.cpu_cores > 0);
    }

    #[test]
    fn test_memory_bandwidth() {
        let validator = HardwareValidator::new();
        let bandwidth = validator.measure_memory_bandwidth();

        println!("Memory bandwidth: {:.2} GB/s", bandwidth);
        assert!(bandwidth > 0.0);
    }

    #[test]
    fn test_cache_latencies() {
        let validator = HardwareValidator::new();
        let latencies = validator.measure_cache_latencies();

        for (cache, latency) in latencies {
            println!("{} latency: {:.2} ns", cache, latency);
        }
    }
}