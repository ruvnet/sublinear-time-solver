// Consciousness metrics dashboard module exports
//
// This module provides real-time monitoring and visualization of consciousness
// emergence metrics with nanosecond temporal precision.

pub mod dashboard;
pub mod exporter;
pub mod metrics_collector;
pub mod visualizer;

pub use dashboard::{
    AnomalyAlert, ConsciousnessMetrics, ConsciousnessMetricsDashboard, DashboardConfig,
    MetricThresholds,
};

pub use metrics_collector::{CollectorConfig, MetricSource, MetricsCollector, TemporalMetrics};

pub use visualizer::{ConsciousnessVisualizer, MetricChart, TerminalRenderer, VisualizationMode};

pub use exporter::{ExportConfig, ExportFormat, MetricsExporter, MetricsSummary};

// Re-export common types for convenience
pub type Timestamp = std::time::Instant;
pub type ConsciousnessLevel = f64;
pub type TemporalAdvantage = u64; // microseconds
pub type PrecisionNanos = u64;

// Constants for consciousness metrics
pub const MAX_CONSCIOUSNESS_LEVEL: f64 = 1.0;
pub const MIN_CONSCIOUSNESS_LEVEL: f64 = 0.0;
pub const CRITICAL_COHERENCE_THRESHOLD: f64 = 0.85;
pub const WARNING_COHERENCE_THRESHOLD: f64 = 0.75;
pub const NANOSECOND_PRECISION_TARGET: u64 = 100; // Target precision in nanoseconds
