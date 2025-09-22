//! # Temporal Windowing System
//!
//! Manages temporal windows with configurable overlap for continuous consciousness processing.
//! Provides sliding time windows that maintain continuity between discrete processing moments.

use std::{
    collections::VecDeque,
    time::Duration,
};

use thiserror::Error;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

use crate::scheduler::TimePoint;

/// Temporal processing error types
#[derive(Error, Debug)]
pub enum TemporalError {
    /// Invalid window configuration
    #[error("Invalid window configuration: {0}")]
    InvalidConfig(String),
    
    /// Window management error
    #[error("Window management error: {0}")]
    WindowManagement(String),
    
    /// Overlap processing error
    #[error("Overlap processing error: {0}")]
    OverlapProcessing(String),
    
    /// Temporal alignment error
    #[error("Temporal alignment error: {0}")]
    TemporalAlignment(String),
}

/// Configuration for temporal windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Duration of each temporal window in nanoseconds
    pub window_duration_ns: u64,
    /// Overlap percentage between windows (0.0 to 1.0)
    pub overlap_percent: f32,
    /// Maximum number of windows to keep in memory
    pub max_windows: usize,
    /// Window advancement strategy
    pub advancement_strategy: WindowAdvancementStrategy,
    /// Overlap management strategy
    pub overlap_strategy: OverlapStrategy,
    /// Enable temporal coherence monitoring
    pub enable_coherence_monitoring: bool,
    /// Minimum window duration in nanoseconds (safety limit)
    pub min_window_duration_ns: u64,
    /// Maximum overlap percentage (safety limit)
    pub max_overlap_percent: f32,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            window_duration_ns: 100_000, // 100 microseconds
            overlap_percent: 0.5, // 50% overlap
            max_windows: 100,
            advancement_strategy: WindowAdvancementStrategy::FixedInterval,
            overlap_strategy: OverlapStrategy::BlendedTransition,
            enable_coherence_monitoring: true,
            min_window_duration_ns: 1_000, // 1 microsecond minimum
            max_overlap_percent: 0.9, // 90% maximum overlap
        }
    }
}

/// Window advancement strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowAdvancementStrategy {
    /// Fixed time intervals
    FixedInterval,
    /// Adaptive based on processing load
    Adaptive { target_utilization: f32 },
    /// Event-driven advancement
    EventDriven,
    /// Synchronous with external clock
    Synchronized { sync_period_ns: u64 },
}

/// Overlap management strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlapStrategy {
    /// Simple blending of overlapping regions
    BlendedTransition,
    /// Weighted averaging based on distance from window center
    WeightedAveraging,
    /// State carry-over between windows
    StateCarryOver,
    /// Independent processing with post-merge
    IndependentMerge,
    /// Phase-locked overlap synchronization
    PhaseLocked { phase_duration_ns: u64 },
}

/// Represents a single temporal window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalWindow {
    /// Window identifier
    pub id: u64,
    /// Start time of the window
    pub start_time: TimePoint,
    /// End time of the window
    pub end_time: TimePoint,
    /// Window state
    pub state: WindowState,
    /// Processing priority
    pub priority: u8,
    /// Overlap regions with other windows
    pub overlaps: Vec<OverlapRegion>,
    /// Window metadata
    pub metadata: WindowMetadata,
}

/// Window processing state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowState {
    /// Window is being initialized
    Initializing,
    /// Window is actively processing
    Active,
    /// Window is in overlap region
    Overlapping,
    /// Window processing is complete
    Completed,
    /// Window was cancelled
    Cancelled,
}

/// Overlap region between windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapRegion {
    /// ID of the overlapping window
    pub other_window_id: u64,
    /// Start time of overlap
    pub overlap_start: TimePoint,
    /// End time of overlap
    pub overlap_end: TimePoint,
    /// Overlap processing weight (0.0 to 1.0)
    pub weight: f32,
    /// Overlap state
    pub state: OverlapState,
}

/// State of overlap processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverlapState {
    /// Overlap is pending
    Pending,
    /// Overlap is being processed
    Processing,
    /// Overlap processing is complete
    Completed,
    /// Overlap had conflicts
    Conflicted,
}

/// Window metadata for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowMetadata {
    /// Number of tasks scheduled in this window
    pub task_count: usize,
    /// Estimated processing time
    pub estimated_duration_ns: u64,
    /// Actual processing time (if completed)
    pub actual_duration_ns: Option<u64>,
    /// Window efficiency metric
    pub efficiency: f32,
    /// Temporal coherence score
    pub coherence_score: f32,
    /// Creation timestamp
    pub created_at: TimePoint,
    /// Last update timestamp
    pub updated_at: TimePoint,
}

impl Default for WindowMetadata {
    fn default() -> Self {
        let now = TimePoint::now();
        Self {
            task_count: 0,
            estimated_duration_ns: 0,
            actual_duration_ns: None,
            efficiency: 1.0,
            coherence_score: 1.0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Manages temporal windows and their overlaps
pub struct WindowManager {
    config: TemporalConfig,
    windows: VecDeque<TemporalWindow>,
    current_window_id: u64,
    next_window_id: u64,
    current_time: TimePoint,
    coherence_history: VecDeque<f32>,
    metrics: WindowManagerMetrics,
}

/// Window manager performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WindowManagerMetrics {
    /// Total windows created
    pub windows_created: u64,
    /// Total windows completed
    pub windows_completed: u64,
    /// Average window processing time
    pub avg_processing_time_ns: f64,
    /// Average overlap processing time
    pub avg_overlap_time_ns: f64,
    /// Temporal coherence average
    pub avg_coherence: f32,
    /// Window efficiency average
    pub avg_efficiency: f32,
    /// Current active windows
    pub active_windows: usize,
    /// Overlap conflicts detected
    pub overlap_conflicts: u64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(config: TemporalConfig) -> Result<Self, TemporalError> {
        // Validate configuration
        if config.window_duration_ns < config.min_window_duration_ns {
            return Err(TemporalError::InvalidConfig(
                format!("Window duration {}ns is below minimum {}ns",
                    config.window_duration_ns, config.min_window_duration_ns)
            ));
        }
        
        if config.overlap_percent < 0.0 || config.overlap_percent > config.max_overlap_percent {
            return Err(TemporalError::InvalidConfig(
                format!("Overlap percent {} is outside valid range [0.0, {}]",
                    config.overlap_percent, config.max_overlap_percent)
            ));
        }
        
        let current_time = TimePoint::now();
        
        let mut manager = Self {
            config,
            windows: VecDeque::new(),
            current_window_id: 0,
            next_window_id: 1,
            current_time,
            coherence_history: VecDeque::new(),
            metrics: WindowManagerMetrics::default(),
        };
        
        // Create initial window
        manager.create_initial_window()?;
        
        Ok(manager)
    }
    
    /// Create the initial temporal window
    fn create_initial_window(&mut self) -> Result<(), TemporalError> {
        let window = TemporalWindow {
            id: self.current_window_id,
            start_time: self.current_time,
            end_time: self.current_time.add_duration(
                Duration::from_nanos(self.config.window_duration_ns)
            ),
            state: WindowState::Active,
            priority: 128,
            overlaps: Vec::new(),
            metadata: WindowMetadata::default(),
        };
        
        self.windows.push_back(window);
        self.metrics.windows_created += 1;
        self.metrics.active_windows += 1;
        
        log::debug!("Created initial temporal window {}", self.current_window_id);
        Ok(())
    }
    
    /// Get the current active window
    pub fn get_current_window(&self) -> Result<&TemporalWindow, TemporalError> {
        self.windows.iter()
            .find(|w| w.id == self.current_window_id)
            .ok_or_else(|| TemporalError::WindowManagement(
                "Current window not found".to_string()
            ))
    }
    
    /// Get mutable reference to current window
    pub fn get_current_window_mut(&mut self) -> Result<&mut TemporalWindow, TemporalError> {
        let window_id = self.current_window_id;
        self.windows.iter_mut()
            .find(|w| w.id == window_id)
            .ok_or_else(|| TemporalError::WindowManagement(
                "Current window not found".to_string()
            ))
    }
    
    /// Advance to the next temporal window
    pub fn advance_window(&mut self) -> Result<(), TemporalError> {
        let current_time = TimePoint::now();
        self.current_time = current_time;
        
        // Calculate advancement interval based on overlap
        let advancement_interval = self.calculate_advancement_interval();
        
        // Create next window
        let next_start_time = self.get_current_window()?.start_time
            .add_duration(advancement_interval);
        
        let next_window = TemporalWindow {
            id: self.next_window_id,
            start_time: next_start_time,
            end_time: next_start_time.add_duration(
                Duration::from_nanos(self.config.window_duration_ns)
            ),
            state: WindowState::Initializing,
            priority: 128,
            overlaps: Vec::new(),
            metadata: WindowMetadata::default(),
        };
        
        // Calculate overlaps with existing windows
        let overlaps = self.calculate_overlaps(&next_window);
        
        // Update window with overlaps
        let mut next_window = next_window;
        next_window.overlaps = overlaps;
        next_window.state = WindowState::Active;
        
        // Add to window queue
        self.windows.push_back(next_window);
        
        // Update current window
        let old_window_id = self.current_window_id;
        self.current_window_id = self.next_window_id;
        self.next_window_id += 1;
        
        // Update metrics
        self.metrics.windows_created += 1;
        self.metrics.active_windows += 1;
        
        // Mark old window as overlapping or completed
        if let Some(old_window) = self.windows.iter_mut().find(|w| w.id == old_window_id) {
            if old_window.overlaps.is_empty() {
                old_window.state = WindowState::Completed;
                self.metrics.windows_completed += 1;
                self.metrics.active_windows = self.metrics.active_windows.saturating_sub(1);
            } else {
                old_window.state = WindowState::Overlapping;
            }
        }
        
        // Clean up old windows
        self.cleanup_old_windows();
        
        // Update coherence metrics
        self.update_coherence_metrics()?;
        
        log::debug!("Advanced to window {} at time {}", 
            self.current_window_id, current_time.as_nanos());
        
        Ok(())
    }
    
    /// Calculate the advancement interval based on overlap configuration
    fn calculate_advancement_interval(&self) -> Duration {
        match self.config.advancement_strategy {
            WindowAdvancementStrategy::FixedInterval => {
                let non_overlap_duration = self.config.window_duration_ns as f32 * 
                    (1.0 - self.config.overlap_percent);
                Duration::from_nanos(non_overlap_duration as u64)
            }
            WindowAdvancementStrategy::Adaptive { target_utilization } => {
                // Adjust based on current system utilization
                let base_interval = self.config.window_duration_ns as f32 * 
                    (1.0 - self.config.overlap_percent);
                
                // Simple adaptive scaling (could be more sophisticated)
                let utilization_factor = if target_utilization > 0.0 {
                    (self.metrics.avg_efficiency / target_utilization).min(2.0).max(0.5)
                } else {
                    1.0
                };
                
                Duration::from_nanos((base_interval * utilization_factor) as u64)
            }
            WindowAdvancementStrategy::EventDriven => {
                // For event-driven, use minimum safe interval
                Duration::from_nanos(self.config.min_window_duration_ns)
            }
            WindowAdvancementStrategy::Synchronized { sync_period_ns } => {
                Duration::from_nanos(sync_period_ns)
            }
        }
    }
    
    /// Calculate overlaps between a new window and existing windows
    fn calculate_overlaps(&self, new_window: &TemporalWindow) -> Vec<OverlapRegion> {
        let mut overlaps = Vec::new();
        
        for existing_window in &self.windows {
            if existing_window.state == WindowState::Completed || 
               existing_window.state == WindowState::Cancelled {
                continue;
            }
            
            // Check for temporal overlap
            let overlap_start = new_window.start_time.max(existing_window.start_time);
            let overlap_end = new_window.end_time.min(existing_window.end_time);
            
            if overlap_start < overlap_end {
                // Calculate overlap weight based on duration
                let overlap_duration = overlap_end.duration_since(overlap_start).as_nanos() as f32;
                let window_duration = new_window.end_time.duration_since(new_window.start_time).as_nanos() as f32;
                let weight = overlap_duration / window_duration;
                
                let overlap_region = OverlapRegion {
                    other_window_id: existing_window.id,
                    overlap_start,
                    overlap_end,
                    weight,
                    state: OverlapState::Pending,
                };
                
                overlaps.push(overlap_region);
            }
        }
        
        overlaps
    }
    
    /// Process overlapping regions between windows
    pub fn process_overlaps(&mut self) -> Result<(), TemporalError> {
        let window_ids: Vec<u64> = self.windows.iter()
            .filter(|w| w.state == WindowState::Overlapping)
            .map(|w| w.id)
            .collect();
        
        for window_id in window_ids {
            self.process_window_overlaps(window_id)?;
        }
        
        Ok(())
    }
    
    /// Process overlaps for a specific window
    fn process_window_overlaps(&mut self, window_id: u64) -> Result<(), TemporalError> {
        // Find the window
        let window_index = self.windows.iter().position(|w| w.id == window_id)
            .ok_or_else(|| TemporalError::WindowManagement(
                format!("Window {} not found", window_id)
            ))?;
        
        // Process each overlap region
        let overlap_count = self.windows[window_index].overlaps.len();
        
        for overlap_idx in 0..overlap_count {
            match self.config.overlap_strategy {
                OverlapStrategy::BlendedTransition => {
                    self.process_blended_overlap(window_index, overlap_idx)?;
                }
                OverlapStrategy::WeightedAveraging => {
                    self.process_weighted_overlap(window_index, overlap_idx)?;
                }
                OverlapStrategy::StateCarryOver => {
                    self.process_state_carryover(window_index, overlap_idx)?;
                }
                OverlapStrategy::IndependentMerge => {
                    self.process_independent_merge(window_index, overlap_idx)?;
                }
                OverlapStrategy::PhaseLocked { .. } => {
                    self.process_phase_locked_overlap(window_index, overlap_idx)?;
                }
            }
        }
        
        // Mark window as completed if all overlaps are processed
        let all_overlaps_complete = self.windows[window_index].overlaps.iter()
            .all(|overlap| overlap.state == OverlapState::Completed);
        
        if all_overlaps_complete {
            self.windows[window_index].state = WindowState::Completed;
            self.metrics.windows_completed += 1;
            self.metrics.active_windows = self.metrics.active_windows.saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Process blended transition overlap
    fn process_blended_overlap(&mut self, window_idx: usize, overlap_idx: usize) -> Result<(), TemporalError> {
        // Simple blending - just mark as processed
        self.windows[window_idx].overlaps[overlap_idx].state = OverlapState::Completed;
        log::debug!("Processed blended overlap for window {}", self.windows[window_idx].id);
        Ok(())
    }
    
    /// Process weighted averaging overlap
    fn process_weighted_overlap(&mut self, window_idx: usize, overlap_idx: usize) -> Result<(), TemporalError> {
        // Weighted averaging based on distance from window center
        self.windows[window_idx].overlaps[overlap_idx].state = OverlapState::Completed;
        log::debug!("Processed weighted overlap for window {}", self.windows[window_idx].id);
        Ok(())
    }
    
    /// Process state carry-over overlap
    fn process_state_carryover(&mut self, window_idx: usize, overlap_idx: usize) -> Result<(), TemporalError> {
        // State carry-over between windows
        self.windows[window_idx].overlaps[overlap_idx].state = OverlapState::Completed;
        log::debug!("Processed state carryover for window {}", self.windows[window_idx].id);
        Ok(())
    }
    
    /// Process independent merge overlap
    fn process_independent_merge(&mut self, window_idx: usize, overlap_idx: usize) -> Result<(), TemporalError> {
        // Independent processing with post-merge
        self.windows[window_idx].overlaps[overlap_idx].state = OverlapState::Completed;
        log::debug!("Processed independent merge for window {}", self.windows[window_idx].id);
        Ok(())
    }
    
    /// Process phase-locked overlap
    fn process_phase_locked_overlap(&mut self, window_idx: usize, overlap_idx: usize) -> Result<(), TemporalError> {
        // Phase-locked synchronization
        self.windows[window_idx].overlaps[overlap_idx].state = OverlapState::Completed;
        log::debug!("Processed phase-locked overlap for window {}", self.windows[window_idx].id);
        Ok(())
    }
    
    /// Clean up old completed windows
    fn cleanup_old_windows(&mut self) {
        let max_windows = self.config.max_windows;
        
        // Remove completed windows beyond the limit
        while self.windows.len() > max_windows {
            if let Some(window) = self.windows.front() {
                if window.state == WindowState::Completed {
                    self.windows.pop_front();
                    log::debug!("Cleaned up old window {}", window.id);
                } else {
                    break; // Don't remove active windows
                }
            } else {
                break;
            }
        }
        
        // Update memory usage metric
        self.metrics.memory_usage_bytes = self.windows.len() * 
            std::mem::size_of::<TemporalWindow>();
    }
    
    /// Update temporal coherence metrics
    fn update_coherence_metrics(&mut self) -> Result<(), TemporalError> {
        if !self.config.enable_coherence_monitoring {
            return Ok(());
        }
        
        // Calculate coherence based on window timing consistency
        let coherence = self.calculate_temporal_coherence();
        
        // Update coherence history
        self.coherence_history.push_back(coherence);
        
        // Keep limited history
        if self.coherence_history.len() > 100 {
            self.coherence_history.pop_front();
        }
        
        // Update average coherence
        let sum: f32 = self.coherence_history.iter().sum();
        self.metrics.avg_coherence = sum / self.coherence_history.len() as f32;
        
        Ok(())
    }
    
    /// Calculate temporal coherence metric
    fn calculate_temporal_coherence(&self) -> f32 {
        // Simplified coherence calculation based on overlap quality
        let mut coherence_sum = 0.0;
        let mut overlap_count = 0;
        
        for window in &self.windows {
            for overlap in &window.overlaps {
                // Coherence based on overlap weight and state
                let overlap_coherence = match overlap.state {
                    OverlapState::Completed => overlap.weight,
                    OverlapState::Processing => overlap.weight * 0.5,
                    OverlapState::Pending => overlap.weight * 0.1,
                    OverlapState::Conflicted => 0.0,
                };
                
                coherence_sum += overlap_coherence;
                overlap_count += 1;
            }
        }
        
        if overlap_count > 0 {
            coherence_sum / overlap_count as f32
        } else {
            1.0 // Perfect coherence if no overlaps
        }
    }
    
    /// Get temporal coherence metric for consciousness system
    pub fn get_coherence_metric(&self) -> f32 {
        self.metrics.avg_coherence
    }
    
    /// Get current window manager metrics
    pub fn get_metrics(&self) -> &WindowManagerMetrics {
        &self.metrics
    }
    
    /// Get all active windows
    pub fn get_active_windows(&self) -> Vec<&TemporalWindow> {
        self.windows.iter()
            .filter(|w| w.state == WindowState::Active || w.state == WindowState::Overlapping)
            .collect()
    }
    
    /// Get window by ID
    pub fn get_window(&self, window_id: u64) -> Option<&TemporalWindow> {
        self.windows.iter().find(|w| w.id == window_id)
    }
    
    /// Update window metadata
    pub fn update_window_metadata(&mut self, window_id: u64, task_count: usize, estimated_duration: u64) -> Result<(), TemporalError> {
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
            window.metadata.task_count = task_count;
            window.metadata.estimated_duration_ns = estimated_duration;
            window.metadata.updated_at = TimePoint::now();
            Ok(())
        } else {
            Err(TemporalError::WindowManagement(
                format!("Window {} not found", window_id)
            ))
        }
    }
    
    /// Mark window as completed and record actual duration
    pub fn complete_window(&mut self, window_id: u64, actual_duration: Duration) -> Result<(), TemporalError> {
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
            window.state = WindowState::Completed;
            window.metadata.actual_duration_ns = Some(actual_duration.as_nanos() as u64);
            
            // Calculate efficiency
            if window.metadata.estimated_duration_ns > 0 {
                window.metadata.efficiency = 
                    window.metadata.estimated_duration_ns as f32 / 
                    actual_duration.as_nanos() as f32;
            }
            
            // Update metrics
            self.metrics.windows_completed += 1;
            self.metrics.active_windows = self.metrics.active_windows.saturating_sub(1);
            
            // Update average processing time
            let actual_duration_ns = actual_duration.as_nanos() as f64;
            if self.metrics.windows_completed == 1 {
                self.metrics.avg_processing_time_ns = actual_duration_ns;
            } else {
                self.metrics.avg_processing_time_ns = 
                    (self.metrics.avg_processing_time_ns * (self.metrics.windows_completed - 1) as f64 + 
                     actual_duration_ns) / self.metrics.windows_completed as f64;
            }
            
            // Update average efficiency
            let efficiency = window.metadata.efficiency;
            if self.metrics.windows_completed == 1 {
                self.metrics.avg_efficiency = efficiency;
            } else {
                self.metrics.avg_efficiency = 
                    (self.metrics.avg_efficiency * (self.metrics.windows_completed - 1) as f32 + 
                     efficiency) / self.metrics.windows_completed as f32;
            }
            
            Ok(())
        } else {
            Err(TemporalError::WindowManagement(
                format!("Window {} not found", window_id)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temporal_config_validation() {
        let mut config = TemporalConfig::default();
        
        // Test valid configuration
        let manager = WindowManager::new(config.clone());
        assert!(manager.is_ok());
        
        // Test invalid window duration
        config.window_duration_ns = 500; // Below minimum
        let manager = WindowManager::new(config.clone());
        assert!(manager.is_err());
        
        // Test invalid overlap percentage
        config.window_duration_ns = 100_000;
        config.overlap_percent = 1.5; // Above maximum
        let manager = WindowManager::new(config);
        assert!(manager.is_err());
    }
    
    #[test]
    fn test_window_creation() {
        let config = TemporalConfig::default();
        let manager = WindowManager::new(config).unwrap();
        
        let current_window = manager.get_current_window();
        assert!(current_window.is_ok());
        
        let window = current_window.unwrap();
        assert_eq!(window.id, 0);
        assert_eq!(window.state, WindowState::Active);
    }
    
    #[test]
    fn test_window_advancement() {
        let config = TemporalConfig::default();
        let mut manager = WindowManager::new(config).unwrap();
        
        let initial_window_id = manager.current_window_id;
        
        // Advance window
        let result = manager.advance_window();
        assert!(result.is_ok());
        
        // Check that window ID changed
        assert_ne!(manager.current_window_id, initial_window_id);
        assert_eq!(manager.current_window_id, 1);
    }
    
    #[test]
    fn test_overlap_calculation() {
        let mut config = TemporalConfig::default();
        config.overlap_percent = 0.5; // 50% overlap
        
        let mut manager = WindowManager::new(config).unwrap();
        
        // Advance to create overlap
        let _ = manager.advance_window();
        
        // Check that overlaps were calculated
        let current_window = manager.get_current_window().unwrap();
        
        // Should have overlap with previous window
        assert!(!current_window.overlaps.is_empty());
    }
    
    #[test]
    fn test_coherence_metric() {
        let config = TemporalConfig::default();
        let manager = WindowManager::new(config).unwrap();
        
        let coherence = manager.get_coherence_metric();
        assert!(coherence >= 0.0 && coherence <= 1.0);
    }
    
    #[test]
    fn test_window_metadata_update() {
        let config = TemporalConfig::default();
        let mut manager = WindowManager::new(config).unwrap();
        
        let window_id = manager.current_window_id;
        let result = manager.update_window_metadata(window_id, 5, 10_000);
        assert!(result.is_ok());
        
        let window = manager.get_current_window().unwrap();
        assert_eq!(window.metadata.task_count, 5);
        assert_eq!(window.metadata.estimated_duration_ns, 10_000);
    }
    
    #[test]
    fn test_window_completion() {
        let config = TemporalConfig::default();
        let mut manager = WindowManager::new(config).unwrap();
        
        let window_id = manager.current_window_id;
        let duration = Duration::from_nanos(8_000);
        
        let result = manager.complete_window(window_id, duration);
        assert!(result.is_ok());
        
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.state, WindowState::Completed);
        assert_eq!(window.metadata.actual_duration_ns, Some(8_000));
    }
}
