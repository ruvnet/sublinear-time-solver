//! # Cognitive Pattern System
//!
//! Defines different modes of AI cognitive processing that can be applied to subjective
//! agents for specialized temporal reasoning and consciousness expansion.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use nalgebra::{DMatrix, DVector};
use rand::{thread_rng, Rng};

use crate::{SubjectiveResult, SubjectiveTimeError};

/// Cognitive processing patterns for different types of thinking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CognitivePattern {
    /// Creative synthesis - combining diverse perspectives for novel solutions
    CreativeSynthesis,

    /// Systems thinking - understanding interconnections and emergent properties
    SystemsThinking,

    /// Convergent reasoning - focused analysis toward optimal solutions
    ConvergentReasoning,

    /// Divergent thinking - exploring multiple possibilities and alternatives
    DivergentThinking,

    /// Lateral thinking - approaching problems from unexpected angles
    LateralThinking,

    /// Critical analysis - rigorous evaluation and logical reasoning
    CriticalAnalysis,

    /// Empathetic reasoning - understanding perspectives and emotional context
    EmpatheticReasoning,
}

impl Default for CognitivePattern {
    fn default() -> Self {
        Self::SystemsThinking
    }
}

/// Configuration for cognitive pattern processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveConfig {
    /// Base processing intensity (0.0 to 1.0)
    pub base_intensity: f64,

    /// Adaptation learning rate
    pub learning_rate: f64,

    /// Memory retention factor
    pub memory_retention: f64,

    /// Cross-pattern integration enabled
    pub cross_pattern_integration: bool,

    /// Pattern switching threshold
    pub switching_threshold: f64,
}

impl Default for CognitiveConfig {
    fn default() -> Self {
        Self {
            base_intensity: 0.7,
            learning_rate: 0.01,
            memory_retention: 0.9,
            cross_pattern_integration: true,
            switching_threshold: 0.3,
        }
    }
}

/// Result of cognitive pattern processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveResult {
    /// Applied cognitive pattern
    pub pattern: CognitivePattern,

    /// Processing effectiveness score (0.0 to 1.0)
    pub effectiveness: f64,

    /// Cognitive load generated
    pub cognitive_load: f64,

    /// Pattern-specific metrics
    pub pattern_metrics: HashMap<String, f64>,

    /// Processing duration in nanoseconds
    pub duration_ns: u64,

    /// Confidence in result quality
    pub confidence: f64,
}

/// Error types for cognitive processing
#[derive(Error, Debug)]
pub enum CognitiveError {
    #[error("Invalid pattern configuration: {0}")]
    Configuration(String),

    #[error("Pattern processing failed: {0}")]
    Processing(String),

    #[error("Pattern adaptation error: {0}")]
    Adaptation(String),
}

/// Cognitive pattern processor
pub struct CognitiveProcessor {
    config: CognitiveConfig,
    pattern_history: Vec<(CognitivePattern, f64)>, // (pattern, effectiveness)
    adaptation_weights: HashMap<CognitivePattern, f64>,
    cross_pattern_memory: DMatrix<f64>,
}

impl CognitiveProcessor {
    /// Create a new cognitive processor
    pub fn new(config: CognitiveConfig) -> Self {
        let mut adaptation_weights = HashMap::new();

        // Initialize equal weights for all patterns
        for pattern in Self::all_patterns() {
            adaptation_weights.insert(pattern, 1.0);
        }

        // Initialize cross-pattern memory matrix
        let pattern_count = Self::all_patterns().len();
        let cross_pattern_memory = DMatrix::from_fn(pattern_count, pattern_count, |_, _| 0.1);

        Self {
            config,
            pattern_history: Vec::new(),
            adaptation_weights,
            cross_pattern_memory,
        }
    }

    /// Process using a specific cognitive pattern
    pub async fn process_with_pattern(
        &mut self,
        pattern: CognitivePattern,
        input_data: &serde_json::Value,
        context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<CognitiveResult> {
        let start_time = std::time::Instant::now();

        // Apply pattern-specific processing
        let pattern_result = self.apply_pattern_processing(pattern, input_data, context).await?;

        // Calculate effectiveness based on result and context
        let effectiveness = self.calculate_effectiveness(&pattern_result, context);

        // Update adaptation weights based on effectiveness
        self.update_adaptation_weights(pattern, effectiveness);

        // Record in pattern history
        self.pattern_history.push((pattern, effectiveness));
        if self.pattern_history.len() > 100 {
            self.pattern_history.remove(0); // Keep recent history
        }

        let duration_ns = start_time.elapsed().as_nanos() as u64;

        Ok(CognitiveResult {
            pattern,
            effectiveness,
            cognitive_load: pattern_result.cognitive_load,
            pattern_metrics: pattern_result.metrics,
            duration_ns,
            confidence: pattern_result.confidence,
        })
    }

    /// Automatically select best pattern for given input
    pub async fn auto_select_pattern(
        &mut self,
        input_data: &serde_json::Value,
        context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<CognitivePattern> {
        let mut pattern_scores = HashMap::new();

        // Score each pattern based on suitability for input
        for pattern in Self::all_patterns() {
            let suitability = self.calculate_pattern_suitability(pattern, input_data, context)?;
            let adaptation_weight = self.adaptation_weights.get(&pattern).copied().unwrap_or(1.0);
            let total_score = suitability * adaptation_weight;

            pattern_scores.insert(pattern, total_score);
        }

        // Select pattern with highest score
        let selected_pattern = pattern_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(pattern, _)| pattern)
            .unwrap_or_default();

        Ok(selected_pattern)
    }

    /// Get pattern processing statistics
    pub fn get_pattern_stats(&self) -> HashMap<CognitivePattern, PatternStats> {
        let mut stats = HashMap::new();

        for pattern in Self::all_patterns() {
            let pattern_history: Vec<f64> = self.pattern_history
                .iter()
                .filter(|(p, _)| *p == pattern)
                .map(|(_, eff)| *eff)
                .collect();

            let stats_entry = if !pattern_history.is_empty() {
                PatternStats {
                    usage_count: pattern_history.len(),
                    average_effectiveness: pattern_history.iter().sum::<f64>() / pattern_history.len() as f64,
                    max_effectiveness: pattern_history.iter().fold(0.0, |a, &b| a.max(b)),
                    min_effectiveness: pattern_history.iter().fold(1.0, |a, &b| a.min(b)),
                    adaptation_weight: self.adaptation_weights.get(&pattern).copied().unwrap_or(1.0),
                }
            } else {
                PatternStats::default()
            };

            stats.insert(pattern, stats_entry);
        }

        stats
    }

    /// Apply specific pattern processing logic
    async fn apply_pattern_processing(
        &self,
        pattern: CognitivePattern,
        input_data: &serde_json::Value,
        context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        match pattern {
            CognitivePattern::CreativeSynthesis => {
                self.process_creative_synthesis(input_data, context).await
            },
            CognitivePattern::SystemsThinking => {
                self.process_systems_thinking(input_data, context).await
            },
            CognitivePattern::ConvergentReasoning => {
                self.process_convergent_reasoning(input_data, context).await
            },
            CognitivePattern::DivergentThinking => {
                self.process_divergent_thinking(input_data, context).await
            },
            CognitivePattern::LateralThinking => {
                self.process_lateral_thinking(input_data, context).await
            },
            CognitivePattern::CriticalAnalysis => {
                self.process_critical_analysis(input_data, context).await
            },
            CognitivePattern::EmpatheticReasoning => {
                self.process_empathetic_reasoning(input_data, context).await
            },
        }
    }

    /// Creative synthesis processing
    async fn process_creative_synthesis(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate creative perspective generation
        let perspective_count = 3 + thread_rng().gen_range(0..=5);
        metrics.insert("perspectives_generated".to_string(), perspective_count as f64);

        // Simulate novelty measurement
        let novelty_score = 0.4 + thread_rng().gen_range(0.0..0.6);
        metrics.insert("novelty_score".to_string(), novelty_score);

        // Simulate synthesis complexity
        let synthesis_complexity = perspective_count as f64 * 0.15;
        metrics.insert("synthesis_complexity".to_string(), synthesis_complexity);

        // Creative synthesis has higher cognitive load due to multiple perspectives
        let cognitive_load = 0.6 + (synthesis_complexity * 0.4);

        // Confidence varies with novelty
        let confidence = 0.5 + (novelty_score * 0.4);

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Systems thinking processing
    async fn process_systems_thinking(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate system component analysis
        let component_count = 4 + thread_rng().gen_range(0..=6);
        metrics.insert("components_analyzed".to_string(), component_count as f64);

        // Simulate interconnection mapping
        let interconnection_density = thread_rng().gen_range(0.3..0.9);
        metrics.insert("interconnection_density".to_string(), interconnection_density);

        // Simulate emergent property detection
        let emergent_properties = thread_rng().gen_range(1..=4);
        metrics.insert("emergent_properties".to_string(), emergent_properties as f64);

        // Systems thinking has moderate cognitive load
        let cognitive_load = 0.5 + (interconnection_density * 0.3);

        // High confidence due to systematic analysis
        let confidence = 0.7 + thread_rng().gen_range(0.0..0.2);

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Convergent reasoning processing
    async fn process_convergent_reasoning(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate reasoning steps
        let reasoning_steps = 5 + thread_rng().gen_range(0..=10);
        metrics.insert("reasoning_steps".to_string(), reasoning_steps as f64);

        // Simulate solution confidence buildup
        let solution_confidence = 0.6 + thread_rng().gen_range(0.0..0.4);
        metrics.insert("solution_confidence".to_string(), solution_confidence);

        // Simulate logical consistency check
        let logical_consistency = 0.8 + thread_rng().gen_range(0.0..0.2);
        metrics.insert("logical_consistency".to_string(), logical_consistency);

        // Convergent reasoning has lower cognitive load due to focus
        let cognitive_load = 0.4 + (reasoning_steps as f64 * 0.02);

        // High confidence due to logical rigor
        let confidence = solution_confidence * logical_consistency;

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Divergent thinking processing
    async fn process_divergent_thinking(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate idea generation
        let ideas_generated = 8 + thread_rng().gen_range(0..=12);
        metrics.insert("ideas_generated".to_string(), ideas_generated as f64);

        // Simulate idea diversity
        let idea_diversity = 0.5 + thread_rng().gen_range(0.0..0.5);
        metrics.insert("idea_diversity".to_string(), idea_diversity);

        // Simulate exploration breadth
        let exploration_breadth = ideas_generated as f64 * idea_diversity;
        metrics.insert("exploration_breadth".to_string(), exploration_breadth);

        // Divergent thinking has high cognitive load due to exploration
        let cognitive_load = 0.7 + (idea_diversity * 0.3);

        // Moderate confidence due to exploration nature
        let confidence = 0.6 + thread_rng().gen_range(-0.1..0.2);

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Lateral thinking processing
    async fn process_lateral_thinking(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate unconventional approach discovery
        let unconventional_approaches = 2 + thread_rng().gen_range(0..=4);
        metrics.insert("unconventional_approaches".to_string(), unconventional_approaches as f64);

        // Simulate perspective shift intensity
        let perspective_shift = 0.4 + thread_rng().gen_range(0.0..0.6);
        metrics.insert("perspective_shift".to_string(), perspective_shift);

        // Simulate breakthrough potential
        let breakthrough_potential = perspective_shift * thread_rng().gen_range(0.5..1.0);
        metrics.insert("breakthrough_potential".to_string(), breakthrough_potential);

        // Lateral thinking has variable cognitive load
        let cognitive_load = 0.5 + (perspective_shift * 0.4);

        // Variable confidence due to unconventional nature
        let confidence = 0.4 + (breakthrough_potential * 0.4);

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Critical analysis processing
    async fn process_critical_analysis(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate evidence evaluation
        let evidence_points = 6 + thread_rng().gen_range(0..=8);
        metrics.insert("evidence_points".to_string(), evidence_points as f64);

        // Simulate argument strength assessment
        let argument_strength = 0.6 + thread_rng().gen_range(0.0..0.4);
        metrics.insert("argument_strength".to_string(), argument_strength);

        // Simulate bias detection
        let bias_detection_score = 0.5 + thread_rng().gen_range(0.0..0.5);
        metrics.insert("bias_detection_score".to_string(), bias_detection_score);

        // Critical analysis has moderate cognitive load
        let cognitive_load = 0.5 + (evidence_points as f64 * 0.03);

        // High confidence due to rigorous analysis
        let confidence = argument_strength * 0.7 + bias_detection_score * 0.3;

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Empathetic reasoning processing
    async fn process_empathetic_reasoning(
        &self,
        input_data: &serde_json::Value,
        _context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<PatternProcessingResult> {
        let mut metrics = HashMap::new();

        // Simulate perspective understanding
        let perspectives_understood = 3 + thread_rng().gen_range(0..=4);
        metrics.insert("perspectives_understood".to_string(), perspectives_understood as f64);

        // Simulate emotional context recognition
        let emotional_context_score = 0.4 + thread_rng().gen_range(0.0..0.6);
        metrics.insert("emotional_context_score".to_string(), emotional_context_score);

        // Simulate empathy depth
        let empathy_depth = emotional_context_score * thread_rng().gen_range(0.7..1.0);
        metrics.insert("empathy_depth".to_string(), empathy_depth);

        // Empathetic reasoning has moderate cognitive load
        let cognitive_load = 0.4 + (empathy_depth * 0.4);

        // Good confidence in understanding others
        let confidence = 0.6 + (emotional_context_score * 0.3);

        Ok(PatternProcessingResult {
            metrics,
            cognitive_load,
            confidence,
        })
    }

    /// Calculate pattern suitability for given input
    fn calculate_pattern_suitability(
        &self,
        pattern: CognitivePattern,
        input_data: &serde_json::Value,
        context: Option<&HashMap<String, f64>>
    ) -> SubjectiveResult<f64> {
        // Base suitability from pattern characteristics
        let base_suitability = match pattern {
            CognitivePattern::CreativeSynthesis => {
                if input_data.to_string().contains("creative") || input_data.to_string().contains("novel") {
                    0.8
                } else {
                    0.4
                }
            },
            CognitivePattern::SystemsThinking => {
                if input_data.to_string().contains("system") || input_data.to_string().contains("complex") {
                    0.8
                } else {
                    0.6
                }
            },
            CognitivePattern::ConvergentReasoning => {
                if input_data.to_string().contains("solve") || input_data.to_string().contains("optimal") {
                    0.8
                } else {
                    0.5
                }
            },
            _ => 0.5, // Default suitability
        };

        // Adjust based on context if available
        let context_adjustment = if let Some(ctx) = context {
            if let Some(complexity) = ctx.get("complexity") {
                match pattern {
                    CognitivePattern::SystemsThinking => complexity * 0.2,
                    CognitivePattern::CreativeSynthesis => (1.0 - complexity) * 0.2,
                    _ => 0.0,
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok((base_suitability + context_adjustment).max(0.1).min(1.0))
    }

    /// Calculate effectiveness of pattern processing
    fn calculate_effectiveness(
        &self,
        result: &PatternProcessingResult,
        _context: Option<&HashMap<String, f64>>
    ) -> f64 {
        // Base effectiveness from confidence
        let base_effectiveness = result.confidence * 0.6;

        // Adjust for cognitive load efficiency (higher load should show better results if successful)
        let load_efficiency = if result.cognitive_load > 0.5 {
            result.confidence * 0.3 // High load should produce high confidence
        } else {
            0.2 // Low load is efficient but may be less thorough
        };

        // Combine factors
        (base_effectiveness + load_efficiency + 0.2).min(1.0)
    }

    /// Update adaptation weights based on effectiveness
    fn update_adaptation_weights(&mut self, pattern: CognitivePattern, effectiveness: f64) {
        let learning_rate = self.config.learning_rate;
        let current_weight = self.adaptation_weights.get(&pattern).copied().unwrap_or(1.0);

        // Adjust weight based on effectiveness
        let adjustment = (effectiveness - 0.5) * learning_rate; // 0.5 is neutral effectiveness
        let new_weight = (current_weight + adjustment).max(0.1).min(2.0);

        self.adaptation_weights.insert(pattern, new_weight);
    }

    /// Get all available cognitive patterns
    fn all_patterns() -> Vec<CognitivePattern> {
        vec![
            CognitivePattern::CreativeSynthesis,
            CognitivePattern::SystemsThinking,
            CognitivePattern::ConvergentReasoning,
            CognitivePattern::DivergentThinking,
            CognitivePattern::LateralThinking,
            CognitivePattern::CriticalAnalysis,
            CognitivePattern::EmpatheticReasoning,
        ]
    }
}

/// Internal pattern processing result
#[derive(Debug)]
struct PatternProcessingResult {
    metrics: HashMap<String, f64>,
    cognitive_load: f64,
    confidence: f64,
}

/// Statistics for a cognitive pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternStats {
    pub usage_count: usize,
    pub average_effectiveness: f64,
    pub max_effectiveness: f64,
    pub min_effectiveness: f64,
    pub adaptation_weight: f64,
}

impl CognitivePattern {
    /// Get human-readable description of the pattern
    pub fn description(&self) -> &'static str {
        match self {
            Self::CreativeSynthesis => "Combines diverse perspectives for innovative solutions",
            Self::SystemsThinking => "Analyzes interconnections and emergent properties",
            Self::ConvergentReasoning => "Focuses analysis toward optimal solutions",
            Self::DivergentThinking => "Explores multiple possibilities and alternatives",
            Self::LateralThinking => "Approaches problems from unexpected angles",
            Self::CriticalAnalysis => "Rigorous evaluation and logical reasoning",
            Self::EmpatheticReasoning => "Understanding perspectives and emotional context",
        }
    }

    /// Get typical cognitive load range for this pattern
    pub fn typical_cognitive_load_range(&self) -> (f64, f64) {
        match self {
            Self::CreativeSynthesis => (0.6, 0.9),
            Self::SystemsThinking => (0.5, 0.8),
            Self::ConvergentReasoning => (0.4, 0.7),
            Self::DivergentThinking => (0.7, 1.0),
            Self::LateralThinking => (0.5, 0.9),
            Self::CriticalAnalysis => (0.5, 0.8),
            Self::EmpatheticReasoning => (0.4, 0.8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cognitive_processor_creation() {
        let config = CognitiveConfig::default();
        let processor = CognitiveProcessor::new(config);

        let stats = processor.get_pattern_stats();
        assert_eq!(stats.len(), 7); // All patterns should be present
    }

    #[tokio::test]
    async fn test_pattern_processing() {
        let mut processor = CognitiveProcessor::new(CognitiveConfig::default());
        let test_input = serde_json::json!({"task": "creative problem solving"});

        let result = processor.process_with_pattern(
            CognitivePattern::CreativeSynthesis,
            &test_input,
            None
        ).await.unwrap();

        assert_eq!(result.pattern, CognitivePattern::CreativeSynthesis);
        assert!(result.effectiveness >= 0.0 && result.effectiveness <= 1.0);
        assert!(result.cognitive_load >= 0.0 && result.cognitive_load <= 2.0); // Allow for higher cognitive load in complex patterns
    }

    #[tokio::test]
    async fn test_auto_pattern_selection() {
        let mut processor = CognitiveProcessor::new(CognitiveConfig::default());
        let creative_input = serde_json::json!({"task": "creative novel solution needed"});

        let selected_pattern = processor.auto_select_pattern(&creative_input, None).await.unwrap();

        // Should prefer creative synthesis for creative tasks
        assert!(matches!(selected_pattern, CognitivePattern::CreativeSynthesis));
    }
}