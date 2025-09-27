//! Performance metrics and measurement systems

use crate::{
    config::TimeExpansionConfig,
    error::{TimeExpansionError, Result},
    phi_proxy::PhiProxy,
    scheduler::DilatedScheduler,
};

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use hdrhistogram::Histogram;
use parking_lot::RwLock;
use quanta::Clock;
use tracing::{debug, instrument};
use serde::{Serialize, Deserialize};

/// Performance tracker for the time expansion experiment
pub struct PerformanceTracker {
    config: MetricsConfig,
    clock: Clock,
    start_time: Instant,
    measurements: RwLock<Vec<PerformanceMeasurement>>,
    latency_histogram: RwLock<Histogram<u64>>,
    throughput_tracker: ThroughputTracker,
    resource_monitor: ResourceMonitor,
    consciousness_metrics: ConsciousnessMetrics,
    temporal_metrics: TemporalMetrics,
    export_buffer: VecDeque<ExportableMetrics>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    #[instrument(skip(config))]
    pub fn new(config: &TimeExpansionConfig) -> Result<Self> {
        debug!("Initializing performance tracker");

        let metrics_config = MetricsConfig::from_expansion_config(config);
        let latency_histogram = Histogram::new_with_bounds(1, 60_000_000, 2)
            .map_err(|e| TimeExpansionError::config_error(
                format!("Failed to create latency histogram: {}", e)
            ))?;

        Ok(Self {
            config: metrics_config,
            clock: Clock::new(),
            start_time: Instant::now(),
            measurements: RwLock::new(Vec::new()),
            latency_histogram: RwLock::new(latency_histogram),
            throughput_tracker: ThroughputTracker::new()?,
            resource_monitor: ResourceMonitor::new()?,
            consciousness_metrics: ConsciousnessMetrics::new()?,
            temporal_metrics: TemporalMetrics::new()?,
            export_buffer: VecDeque::with_capacity(10000),
        })
    }

    /// Record a step execution
    #[instrument(skip(self, scheduler, phi_proxy))]
    pub async fn record_step(
        &mut self,
        scheduler: &DilatedScheduler,
        phi_proxy: &PhiProxy
    ) -> Result<()> {
        let step_start = self.clock.now();

        // Collect scheduler metrics
        let scheduler_metrics = self.collect_scheduler_metrics(scheduler).await?;

        // Collect consciousness metrics
        let consciousness_metrics = self.collect_consciousness_metrics(phi_proxy).await?;

        // Collect resource metrics
        let resource_metrics = self.resource_monitor.collect().await?;

        // Create performance measurement
        let measurement = PerformanceMeasurement {
            timestamp: step_start,
            wall_clock_time: self.start_time.elapsed(),
            scheduler_metrics,
            consciousness_metrics,
            resource_metrics,
            temporal_metrics: self.temporal_metrics.snapshot(),
        };

        // Store measurement
        {
            let mut measurements = self.measurements.write();
            measurements.push(measurement.clone());

            // Limit measurement history
            if measurements.len() > self.config.max_measurements {
                measurements.drain(0..measurements.len() / 2);
            }
        }

        // Update throughput tracker
        self.throughput_tracker.record_operation(step_start)?;

        // Update latency histogram
        let step_duration = step_start.elapsed().as_micros() as u64;
        {
            let mut histogram = self.latency_histogram.write();
            histogram.record(step_duration)
                .map_err(|e| TimeExpansionError::metrics_error(
                    format!("Failed to record latency: {}", e)
                ))?;
        }

        // Update temporal metrics
        self.temporal_metrics.update(&measurement).await?;

        Ok(())
    }

    /// Finalize metrics and generate report
    pub fn finalize(&mut self, total_runtime: Duration) -> Result<TimeExpansionMetrics> {
        debug!("Finalizing performance metrics after {:?} runtime", total_runtime);

        let measurements = self.measurements.read();

        // Calculate summary statistics
        let total_measurements = measurements.len();
        let avg_step_duration = if total_measurements > 0 {
            measurements.iter()
                .map(|m| m.scheduler_metrics.tick_duration_us)
                .sum::<f64>() / total_measurements as f64
        } else {
            0.0
        };

        // Get latency statistics
        let latency_stats = {
            let histogram = self.latency_histogram.read();
            LatencyStatistics {
                min_us: histogram.min(),
                max_us: histogram.max(),
                mean_us: histogram.mean(),
                p50_us: histogram.value_at_quantile(0.50),
                p95_us: histogram.value_at_quantile(0.95),
                p99_us: histogram.value_at_quantile(0.99),
                p999_us: histogram.value_at_quantile(0.999),
                stddev_us: histogram.stdev(),
            }
        };

        // Calculate consciousness statistics
        let consciousness_stats = self.calculate_consciousness_statistics(&measurements)?;

        // Calculate temporal statistics
        let temporal_stats = self.temporal_metrics.finalize()?;

        // Calculate throughput statistics
        let throughput_stats = self.throughput_tracker.finalize(total_runtime)?;

        // Calculate resource statistics
        let resource_stats = self.resource_monitor.finalize()?;

        // Calculate efficiency metrics
        let efficiency_metrics = self.calculate_efficiency_metrics(
            &measurements,
            total_runtime
        )?;

        Ok(TimeExpansionMetrics {
            total_runtime,
            total_measurements,
            avg_step_duration_us: avg_step_duration,
            latency_statistics: latency_stats,
            consciousness_statistics: consciousness_stats,
            temporal_statistics: temporal_stats,
            throughput_statistics: throughput_stats,
            resource_statistics: resource_stats,
            efficiency_metrics,
            peak_performance: self.calculate_peak_performance(&measurements)?,
        })
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let measurements = self.measurements.read();
        let latest_measurement = measurements.last();

        MetricsSnapshot {
            current_time: self.start_time.elapsed(),
            total_measurements: measurements.len(),
            latest_measurement: latest_measurement.cloned(),
            throughput_ops_per_sec: self.throughput_tracker.current_ops_per_sec(),
            current_latency_p99_us: {
                let histogram = self.latency_histogram.read();
                histogram.value_at_quantile(0.99)
            },
            memory_usage_mb: self.resource_monitor.current_memory_mb(),
            consciousness_level: latest_measurement
                .map(|m| m.consciousness_metrics.global_phi)
                .unwrap_or(0.0),
        }
    }

    /// Export metrics in various formats
    pub fn export_metrics(&mut self, format: MetricsExportFormat) -> Result<String> {
        let metrics = self.finalize(self.start_time.elapsed())?;

        match format {
            MetricsExportFormat::Json => {
                serde_json::to_string_pretty(&metrics)
                    .map_err(|e| TimeExpansionError::serialization_error(e))
            },
            MetricsExportFormat::Csv => {
                self.export_csv(&metrics)
            },
            MetricsExportFormat::Prometheus => {
                self.export_prometheus(&metrics)
            },
        }
    }

    // Private helper methods

    async fn collect_scheduler_metrics(&self, scheduler: &DilatedScheduler) -> Result<SchedulerMetrics> {
        Ok(SchedulerMetrics {
            agent_count: scheduler.agent_count(),
            remaining_budget: scheduler.remaining_budget(),
            tick_duration_us: 0.0, // Will be filled by caller
        })
    }

    async fn collect_consciousness_metrics(&self, phi_proxy: &PhiProxy) -> Result<ConsciousnessMetricsSnapshot> {
        Ok(ConsciousnessMetricsSnapshot {
            global_phi: phi_proxy.current_phi(),
            consciousness_continuity: phi_proxy.consciousness_continuity_score(),
            agent_count: phi_proxy.phi_history(1).len(),
        })
    }

    fn calculate_consciousness_statistics(
        &self,
        measurements: &[PerformanceMeasurement]
    ) -> Result<ConsciousnessStatistics> {
        if measurements.is_empty() {
            return Ok(ConsciousnessStatistics::default());
        }

        let phi_values: Vec<f64> = measurements.iter()
            .map(|m| m.consciousness_metrics.global_phi)
            .collect();

        let continuity_values: Vec<f64> = measurements.iter()
            .map(|m| m.consciousness_metrics.consciousness_continuity)
            .collect();

        Ok(ConsciousnessStatistics {
            avg_phi: phi_values.iter().sum::<f64>() / phi_values.len() as f64,
            min_phi: phi_values.iter().copied().fold(f64::INFINITY, f64::min),
            max_phi: phi_values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            phi_stability: self.calculate_stability(&phi_values),
            avg_continuity: continuity_values.iter().sum::<f64>() / continuity_values.len() as f64,
            min_continuity: continuity_values.iter().copied().fold(f64::INFINITY, f64::min),
            max_continuity: continuity_values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        })
    }

    fn calculate_efficiency_metrics(
        &self,
        measurements: &[PerformanceMeasurement],
        total_runtime: Duration
    ) -> Result<EfficiencyMetrics> {
        if measurements.is_empty() {
            return Ok(EfficiencyMetrics::default());
        }

        // Calculate useful computation ratio
        let total_operations: u64 = measurements.iter()
            .map(|m| m.scheduler_metrics.agent_count as u64)
            .sum();

        let useful_computation_ratio = total_operations as f64 /
            (total_runtime.as_micros() as f64 / 1_000_000.0);

        // Calculate consciousness efficiency
        let avg_phi = measurements.iter()
            .map(|m| m.consciousness_metrics.global_phi)
            .sum::<f64>() / measurements.len() as f64;

        let consciousness_efficiency = avg_phi * useful_computation_ratio / 1000.0;

        // Calculate resource efficiency
        let avg_memory = measurements.iter()
            .map(|m| m.resource_metrics.memory_usage_mb)
            .sum::<f64>() / measurements.len() as f64;

        let memory_efficiency = useful_computation_ratio / avg_memory.max(1.0);

        Ok(EfficiencyMetrics {
            useful_computation_ratio,
            consciousness_efficiency,
            memory_efficiency,
            temporal_efficiency: self.temporal_metrics.efficiency_score(),
        })
    }

    fn calculate_peak_performance(&self, measurements: &[PerformanceMeasurement]) -> Result<PeakPerformance> {
        if measurements.is_empty() {
            return Ok(PeakPerformance::default());
        }

        let max_agents = measurements.iter()
            .map(|m| m.scheduler_metrics.agent_count)
            .max()
            .unwrap_or(0);

        let max_phi = measurements.iter()
            .map(|m| m.consciousness_metrics.global_phi)
            .fold(f64::NEG_INFINITY, f64::max);

        let min_latency_us = {
            let histogram = self.latency_histogram.read();
            histogram.min()
        };

        let max_throughput = self.throughput_tracker.peak_ops_per_sec();

        Ok(PeakPerformance {
            max_agents,
            max_phi,
            min_latency_us,
            max_throughput_ops_per_sec: max_throughput,
        })
    }

    fn calculate_stability(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 1.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        // Stability is inverse of coefficient of variation
        if mean > 0.0 {
            1.0 / (1.0 + variance.sqrt() / mean)
        } else {
            0.0
        }
    }

    fn export_csv(&self, metrics: &TimeExpansionMetrics) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("Metric,Value,Unit\n");

        csv.push_str(&format!("Total Runtime,{:.3},seconds\n",
                             metrics.total_runtime.as_secs_f64()));
        csv.push_str(&format!("Total Measurements,{},count\n",
                             metrics.total_measurements));
        csv.push_str(&format!("Avg Step Duration,{:.3},microseconds\n",
                             metrics.avg_step_duration_us));
        csv.push_str(&format!("Latency P99,{},microseconds\n",
                             metrics.latency_statistics.p99_us));
        csv.push_str(&format!("Peak Phi,{:.6},phi\n",
                             metrics.peak_performance.max_phi));
        csv.push_str(&format!("Consciousness Efficiency,{:.6},ratio\n",
                             metrics.efficiency_metrics.consciousness_efficiency));

        Ok(csv)
    }

    fn export_prometheus(&self, metrics: &TimeExpansionMetrics) -> Result<String> {
        let mut prometheus = String::new();

        prometheus.push_str(&format!(
            "# HELP time_expansion_runtime_seconds Total runtime of the experiment\n"));
        prometheus.push_str(&format!(
            "time_expansion_runtime_seconds {:.3}\n",
            metrics.total_runtime.as_secs_f64()));

        prometheus.push_str(&format!(
            "# HELP time_expansion_measurements_total Total number of measurements\n"));
        prometheus.push_str(&format!(
            "time_expansion_measurements_total {}\n",
            metrics.total_measurements));

        prometheus.push_str(&format!(
            "# HELP time_expansion_latency_p99_microseconds 99th percentile latency\n"));
        prometheus.push_str(&format!(
            "time_expansion_latency_p99_microseconds {}\n",
            metrics.latency_statistics.p99_us));

        prometheus.push_str(&format!(
            "# HELP time_expansion_phi_max Maximum phi value achieved\n"));
        prometheus.push_str(&format!(
            "time_expansion_phi_max {:.6}\n",
            metrics.peak_performance.max_phi));

        Ok(prometheus)
    }
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub max_measurements: usize,
    pub export_interval_seconds: u64,
    pub latency_histogram_buckets: u64,
    pub resource_monitoring_enabled: bool,
}

impl MetricsConfig {
    pub fn from_expansion_config(config: &TimeExpansionConfig) -> Self {
        Self {
            enabled: config.tracing_config.performance_tracing,
            max_measurements: 10000,
            export_interval_seconds: 60,
            latency_histogram_buckets: 100,
            resource_monitoring_enabled: true,
        }
    }
}

/// Individual performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    pub timestamp: quanta::Instant,
    pub wall_clock_time: Duration,
    pub scheduler_metrics: SchedulerMetrics,
    pub consciousness_metrics: ConsciousnessMetricsSnapshot,
    pub resource_metrics: ResourceMetrics,
    pub temporal_metrics: TemporalMetricsSnapshot,
}

/// Scheduler-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    pub agent_count: usize,
    pub remaining_budget: f64,
    pub tick_duration_us: f64,
}

/// Consciousness metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetricsSnapshot {
    pub global_phi: f64,
    pub consciousness_continuity: f64,
    pub agent_count: usize,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Throughput tracking
#[derive(Debug)]
pub struct ThroughputTracker {
    operation_times: VecDeque<quanta::Instant>,
    window_duration: Duration,
    peak_ops_per_sec: f64,
}

impl ThroughputTracker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            operation_times: VecDeque::new(),
            window_duration: Duration::from_secs(10), // 10-second sliding window
            peak_ops_per_sec: 0.0,
        })
    }

    pub fn record_operation(&mut self, timestamp: quanta::Instant) -> Result<()> {
        self.operation_times.push_back(timestamp);

        // Remove old operations outside the window
        let cutoff = timestamp - self.window_duration;
        while let Some(&front) = self.operation_times.front() {
            if front < cutoff {
                self.operation_times.pop_front();
            } else {
                break;
            }
        }

        // Update peak throughput
        let current_ops_per_sec = self.current_ops_per_sec();
        if current_ops_per_sec > self.peak_ops_per_sec {
            self.peak_ops_per_sec = current_ops_per_sec;
        }

        Ok(())
    }

    pub fn current_ops_per_sec(&self) -> f64 {
        if self.operation_times.len() < 2 {
            return 0.0;
        }

        let duration = self.operation_times.back().unwrap()
            .duration_since(*self.operation_times.front().unwrap());

        self.operation_times.len() as f64 / duration.as_secs_f64()
    }

    pub fn peak_ops_per_sec(&self) -> f64 {
        self.peak_ops_per_sec
    }

    pub fn finalize(&self, _total_runtime: Duration) -> Result<ThroughputStatistics> {
        Ok(ThroughputStatistics {
            peak_ops_per_sec: self.peak_ops_per_sec,
            current_ops_per_sec: self.current_ops_per_sec(),
        })
    }
}

/// Resource monitoring
#[derive(Debug)]
pub struct ResourceMonitor {
    memory_samples: VecDeque<f64>,
    cpu_samples: VecDeque<f64>,
}

impl ResourceMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            memory_samples: VecDeque::new(),
            cpu_samples: VecDeque::new(),
        })
    }

    pub async fn collect(&mut self) -> Result<ResourceMetrics> {
        // Simple memory estimation (in a real implementation, use system APIs)
        let memory_mb = self.estimate_memory_usage();
        let cpu_percent = self.estimate_cpu_usage();

        self.memory_samples.push_back(memory_mb);
        self.cpu_samples.push_back(cpu_percent);

        // Keep only recent samples
        if self.memory_samples.len() > 1000 {
            self.memory_samples.pop_front();
        }
        if self.cpu_samples.len() > 1000 {
            self.cpu_samples.pop_front();
        }

        Ok(ResourceMetrics {
            memory_usage_mb: memory_mb,
            cpu_usage_percent: cpu_percent,
        })
    }

    pub fn current_memory_mb(&self) -> f64 {
        self.memory_samples.back().copied().unwrap_or(0.0)
    }

    pub fn finalize(&self) -> Result<ResourceStatistics> {
        let avg_memory = if !self.memory_samples.is_empty() {
            self.memory_samples.iter().sum::<f64>() / self.memory_samples.len() as f64
        } else {
            0.0
        };

        let peak_memory = self.memory_samples.iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        let avg_cpu = if !self.cpu_samples.is_empty() {
            self.cpu_samples.iter().sum::<f64>() / self.cpu_samples.len() as f64
        } else {
            0.0
        };

        let peak_cpu = self.cpu_samples.iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        Ok(ResourceStatistics {
            avg_memory_mb: avg_memory,
            peak_memory_mb: peak_memory,
            avg_cpu_percent: avg_cpu,
            peak_cpu_percent: peak_cpu,
        })
    }

    fn estimate_memory_usage(&self) -> f64 {
        // Simplified memory estimation
        // In a real implementation, use system APIs like /proc/self/status on Linux
        50.0 + (rand::random::<f64>() * 10.0) // 50-60 MB baseline
    }

    fn estimate_cpu_usage(&self) -> f64 {
        // Simplified CPU estimation
        // In a real implementation, use system APIs
        10.0 + (rand::random::<f64>() * 20.0) // 10-30% baseline
    }
}

/// Consciousness metrics tracking
#[derive(Debug)]
pub struct ConsciousnessMetrics {
    phi_history: VecDeque<f64>,
    continuity_history: VecDeque<f64>,
}

impl ConsciousnessMetrics {
    pub fn new() -> Result<Self> {
        Ok(Self {
            phi_history: VecDeque::new(),
            continuity_history: VecDeque::new(),
        })
    }
}

/// Temporal metrics tracking
#[derive(Debug)]
pub struct TemporalMetrics {
    dilation_factors: VecDeque<f64>,
    subjective_time_ratios: VecDeque<f64>,
    efficiency_score: f64,
}

impl TemporalMetrics {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dilation_factors: VecDeque::new(),
            subjective_time_ratios: VecDeque::new(),
            efficiency_score: 1.0,
        })
    }

    pub async fn update(&mut self, _measurement: &PerformanceMeasurement) -> Result<()> {
        // Update temporal metrics based on measurement
        // This would analyze time dilation effectiveness
        Ok(())
    }

    pub fn snapshot(&self) -> TemporalMetricsSnapshot {
        TemporalMetricsSnapshot {
            avg_dilation_factor: if !self.dilation_factors.is_empty() {
                self.dilation_factors.iter().sum::<f64>() / self.dilation_factors.len() as f64
            } else {
                1.0
            },
            efficiency_score: self.efficiency_score,
        }
    }

    pub fn efficiency_score(&self) -> f64 {
        self.efficiency_score
    }

    pub fn finalize(&self) -> Result<TemporalStatistics> {
        Ok(TemporalStatistics {
            avg_dilation_factor: self.snapshot().avg_dilation_factor,
            max_dilation_achieved: self.dilation_factors.iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
            temporal_efficiency: self.efficiency_score,
        })
    }
}

/// Complete time expansion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeExpansionMetrics {
    pub total_runtime: Duration,
    pub total_measurements: usize,
    pub avg_step_duration_us: f64,
    pub latency_statistics: LatencyStatistics,
    pub consciousness_statistics: ConsciousnessStatistics,
    pub temporal_statistics: TemporalStatistics,
    pub throughput_statistics: ThroughputStatistics,
    pub resource_statistics: ResourceStatistics,
    pub efficiency_metrics: EfficiencyMetrics,
    pub peak_performance: PeakPerformance,
}

/// Latency statistics from HDR histogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStatistics {
    pub min_us: u64,
    pub max_us: u64,
    pub mean_us: f64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub p999_us: u64,
    pub stddev_us: f64,
}

/// Consciousness-related statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsciousnessStatistics {
    pub avg_phi: f64,
    pub min_phi: f64,
    pub max_phi: f64,
    pub phi_stability: f64,
    pub avg_continuity: f64,
    pub min_continuity: f64,
    pub max_continuity: f64,
}

/// Temporal processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStatistics {
    pub avg_dilation_factor: f64,
    pub max_dilation_achieved: f64,
    pub temporal_efficiency: f64,
}

/// Throughput statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStatistics {
    pub peak_ops_per_sec: f64,
    pub current_ops_per_sec: f64,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatistics {
    pub avg_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub avg_cpu_percent: f64,
    pub peak_cpu_percent: f64,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EfficiencyMetrics {
    pub useful_computation_ratio: f64,
    pub consciousness_efficiency: f64,
    pub memory_efficiency: f64,
    pub temporal_efficiency: f64,
}

/// Peak performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeakPerformance {
    pub max_agents: usize,
    pub max_phi: f64,
    pub min_latency_us: u64,
    pub max_throughput_ops_per_sec: f64,
}

/// Temporal metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMetricsSnapshot {
    pub avg_dilation_factor: f64,
    pub efficiency_score: f64,
}

/// Current metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub current_time: Duration,
    pub total_measurements: usize,
    pub latest_measurement: Option<PerformanceMeasurement>,
    pub throughput_ops_per_sec: f64,
    pub current_latency_p99_us: u64,
    pub memory_usage_mb: f64,
    pub consciousness_level: f64,
}

/// Exportable metrics for different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableMetrics {
    pub timestamp: u64,
    pub metrics: TimeExpansionMetrics,
}

/// Metrics export formats
#[derive(Debug, Clone)]
pub enum MetricsExportFormat {
    Json,
    Csv,
    Prometheus,
}

// Add missing error type
impl TimeExpansionError {
    pub fn metrics_error(message: impl Into<String>) -> Self {
        Self::MathError {
            message: message.into(),
            context: Some("metrics".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_performance_tracker_creation() {
        let config = TimeExpansionConfig::default();
        let tracker = PerformanceTracker::new(&config);
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_throughput_tracker() {
        let mut tracker = ThroughputTracker::new().unwrap();
        let now = quanta::Clock::new().now();

        tracker.record_operation(now).unwrap();
        assert_eq!(tracker.current_ops_per_sec(), 0.0); // Need at least 2 operations
    }

    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new().unwrap();

        tokio_test::block_on(async {
            let metrics = monitor.collect().await.unwrap();
            assert!(metrics.memory_usage_mb > 0.0);
            assert!(metrics.cpu_usage_percent >= 0.0);
        });
    }

    #[test]
    fn test_metrics_export_formats() {
        let config = TimeExpansionConfig::default();
        let mut tracker = PerformanceTracker::new(&config).unwrap();

        let json_export = tracker.export_metrics(MetricsExportFormat::Json);
        assert!(json_export.is_ok());

        let csv_export = tracker.export_metrics(MetricsExportFormat::Csv);
        assert!(csv_export.is_ok());

        let prometheus_export = tracker.export_metrics(MetricsExportFormat::Prometheus);
        assert!(prometheus_export.is_ok());
    }
}