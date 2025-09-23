// Training and Learning Module for Sublinear LLM
// Enables the system to learn new knowledge and improve responses

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub input: String,
    pub expected_output: String,
    pub knowledge_triples: Vec<LearnedTriple>,
    pub feedback_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source: String, // "training", "feedback", "reinforcement"
    pub usage_count: u32,
    pub success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub learning_rate: f32,
    pub confidence_threshold: f32,
    pub max_triples: usize,
    pub enable_reinforcement: bool,
    pub enable_pattern_learning: bool,
    pub enable_semantic_expansion: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            confidence_threshold: 0.6,
            max_triples: 1000,
            enable_reinforcement: true,
            enable_pattern_learning: true,
            enable_semantic_expansion: true,
        }
    }
}

pub struct LearningEngine {
    pub learned_triples: HashMap<String, LearnedTriple>,
    pub pattern_memory: HashMap<String, Vec<String>>,
    pub semantic_clusters: HashMap<String, HashSet<String>>,
    pub feedback_history: Vec<FeedbackRecord>,
    pub config: TrainingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub query: String,
    pub response: String,
    pub score: f32,
    pub timestamp: u64,
    pub improvements: Vec<String>,
}

impl LearningEngine {
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            learned_triples: HashMap::new(),
            pattern_memory: HashMap::new(),
            semantic_clusters: HashMap::new(),
            feedback_history: Vec::new(),
            config,
        }
    }

    // Learn from training data
    pub fn train(&mut self, data: &TrainingData) -> TrainingResult {
        let mut learned_count = 0;
        let mut improved_count = 0;

        // 1. Extract patterns from input-output pairs
        let patterns = self.extract_patterns(&data.input, &data.expected_output);

        // 2. Learn new knowledge triples
        for triple in &data.knowledge_triples {
            let id = format!("{}_{}_{}", triple.subject, triple.predicate, triple.object);

            if let Some(existing) = self.learned_triples.get_mut(&id) {
                // Reinforce existing knowledge
                existing.confidence = (existing.confidence + triple.confidence * self.config.learning_rate)
                    .min(1.0);
                existing.usage_count += 1;
                existing.success_rate = (existing.success_rate + data.feedback_score) / 2.0;
                improved_count += 1;
            } else if self.learned_triples.len() < self.config.max_triples {
                // Add new knowledge
                self.learned_triples.insert(id, triple.clone());
                learned_count += 1;
            }
        }

        // 3. Learn patterns for better response generation
        let patterns_count = patterns.len();
        for (pattern_type, examples) in patterns {
            self.pattern_memory
                .entry(pattern_type)
                .or_insert_with(Vec::new)
                .extend(examples);
        }

        // 4. Build semantic clusters for related concepts
        if self.config.enable_semantic_expansion {
            self.expand_semantic_knowledge(&data.input);
        }

        TrainingResult {
            learned_triples: learned_count,
            improved_triples: improved_count,
            total_triples: self.learned_triples.len(),
            patterns_learned: patterns_count,
            confidence_avg: self.calculate_average_confidence(),
        }
    }

    // Learn from user feedback
    pub fn learn_from_feedback(&mut self, query: &str, response: &str, score: f32) -> LearningResult {
        // Record feedback
        self.feedback_history.push(FeedbackRecord {
            query: query.to_string(),
            response: response.to_string(),
            score,
            timestamp: js_sys::Date::now() as u64,
            improvements: Vec::new(),
        });

        // Adjust confidence based on feedback
        if self.config.enable_reinforcement {
            self.reinforce_learning(query, score);
        }

        // Learn new patterns from successful responses
        if score > 0.8 && self.config.enable_pattern_learning {
            let patterns = self.extract_patterns(query, response);
            for (pattern_type, examples) in patterns {
                self.pattern_memory
                    .entry(pattern_type)
                    .or_insert_with(Vec::new)
                    .extend(examples);
            }
        }

        LearningResult {
            feedback_incorporated: true,
            confidence_adjusted: self.config.enable_reinforcement,
            patterns_updated: score > 0.8,
            overall_improvement: self.calculate_improvement_rate(),
        }
    }

    // Fine-tune on specific domain
    pub fn fine_tune(&mut self, domain: &str, examples: Vec<TrainingData>) -> FineTuneResult {
        let mut domain_triples = Vec::new();
        let mut _domain_patterns: HashMap<String, Vec<String>> = HashMap::new();

        for example in examples {
            // Train on each example
            let result = self.train(&example);

            // Collect domain-specific knowledge
            for triple in example.knowledge_triples {
                if triple.confidence > self.config.confidence_threshold {
                    domain_triples.push(triple);
                }
            }
        }

        // Create domain-specific cluster
        let domain_concepts: HashSet<String> = domain_triples
            .iter()
            .flat_map(|t| vec![t.subject.clone(), t.object.clone()])
            .collect();

        self.semantic_clusters.insert(domain.to_string(), domain_concepts.clone());

        FineTuneResult {
            domain: domain.to_string(),
            triples_added: domain_triples.len(),
            concepts_learned: domain_concepts.len(),
            domain_confidence: self.calculate_domain_confidence(domain),
        }
    }

    // Extract patterns from input-output pairs
    fn extract_patterns(&self, input: &str, output: &str) -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();

        // Detect question types
        let input_lower = input.to_lowercase();
        if input_lower.starts_with("what") {
            patterns.entry("definition".to_string())
                .or_insert_with(Vec::new)
                .push(output.to_string());
        } else if input_lower.starts_with("how") {
            patterns.entry("explanation".to_string())
                .or_insert_with(Vec::new)
                .push(output.to_string());
        } else if input_lower.starts_with("why") {
            patterns.entry("reasoning".to_string())
                .or_insert_with(Vec::new)
                .push(output.to_string());
        }

        // Learn response patterns
        if output.contains("because") {
            patterns.entry("causal".to_string())
                .or_insert_with(Vec::new)
                .push(output.to_string());
        }

        patterns
    }

    // Expand semantic knowledge through clustering
    fn expand_semantic_knowledge(&mut self, text: &str) {
        let words: Vec<String> = text.split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 3)
            .collect();

        for word in &words {
            // Find related concepts
            for (_, triple) in &self.learned_triples {
                if triple.subject.contains(word) || triple.object.contains(word) {
                    self.semantic_clusters
                        .entry(word.clone())
                        .or_insert_with(HashSet::new)
                        .insert(triple.subject.clone());

                    self.semantic_clusters
                        .get_mut(word)
                        .unwrap()
                        .insert(triple.object.clone());
                }
            }
        }
    }

    // Reinforce learning based on feedback
    fn reinforce_learning(&mut self, query: &str, score: f32) {
        let adjustment = (score - 0.5) * self.config.learning_rate;

        // Adjust confidence of related triples
        for (_, triple) in self.learned_triples.iter_mut() {
            if query.contains(&triple.subject) || query.contains(&triple.object) {
                triple.confidence = (triple.confidence + adjustment).clamp(0.1, 1.0);
                triple.usage_count += 1;
                triple.success_rate = (triple.success_rate * 0.9) + (score * 0.1); // Exponential moving average
            }
        }
    }

    fn calculate_average_confidence(&self) -> f32 {
        if self.learned_triples.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.learned_triples.values()
            .map(|t| t.confidence)
            .sum();

        sum / self.learned_triples.len() as f32
    }

    fn calculate_improvement_rate(&self) -> f32 {
        if self.feedback_history.len() < 2 {
            return 0.0;
        }

        let recent_scores: Vec<f32> = self.feedback_history
            .iter()
            .rev()
            .take(10)
            .map(|f| f.score)
            .collect();

        let avg_recent = recent_scores.iter().sum::<f32>() / recent_scores.len() as f32;
        let avg_overall = self.feedback_history.iter()
            .map(|f| f.score)
            .sum::<f32>() / self.feedback_history.len() as f32;

        avg_recent - avg_overall
    }

    fn calculate_domain_confidence(&self, domain: &str) -> f32 {
        if let Some(concepts) = self.semantic_clusters.get(domain) {
            let related_triples: Vec<&LearnedTriple> = self.learned_triples
                .values()
                .filter(|t| concepts.contains(&t.subject) || concepts.contains(&t.object))
                .collect();

            if related_triples.is_empty() {
                return 0.0;
            }

            related_triples.iter()
                .map(|t| t.confidence)
                .sum::<f32>() / related_triples.len() as f32
        } else {
            0.0
        }
    }

    // Export learned knowledge for persistence
    pub fn export_knowledge(&self) -> String {
        serde_json::to_string(&ExportedKnowledge {
            learned_triples: self.learned_triples.values().cloned().collect(),
            pattern_memory: self.pattern_memory.clone(),
            semantic_clusters: self.semantic_clusters.iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
                .collect(),
            feedback_stats: FeedbackStats {
                total_feedback: self.feedback_history.len(),
                average_score: self.feedback_history.iter()
                    .map(|f| f.score)
                    .sum::<f32>() / self.feedback_history.len().max(1) as f32,
                improvement_rate: self.calculate_improvement_rate(),
            },
        }).unwrap_or_else(|_| "{}".to_string())
    }

    // Import learned knowledge
    pub fn import_knowledge(&mut self, json: &str) -> Result<(), String> {
        let imported: ExportedKnowledge = serde_json::from_str(json)
            .map_err(|e| e.to_string())?;

        for triple in imported.learned_triples {
            let id = format!("{}_{}_{}", triple.subject, triple.predicate, triple.object);
            self.learned_triples.insert(id, triple);
        }

        self.pattern_memory.extend(imported.pattern_memory);

        for (key, values) in imported.semantic_clusters {
            self.semantic_clusters.insert(key, values.into_iter().collect());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingResult {
    pub learned_triples: usize,
    pub improved_triples: usize,
    pub total_triples: usize,
    pub patterns_learned: usize,
    pub confidence_avg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResult {
    pub feedback_incorporated: bool,
    pub confidence_adjusted: bool,
    pub patterns_updated: bool,
    pub overall_improvement: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneResult {
    pub domain: String,
    pub triples_added: usize,
    pub concepts_learned: usize,
    pub domain_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedKnowledge {
    pub learned_triples: Vec<LearnedTriple>,
    pub pattern_memory: HashMap<String, Vec<String>>,
    pub semantic_clusters: HashMap<String, Vec<String>>,
    pub feedback_stats: FeedbackStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackStats {
    pub total_feedback: usize,
    pub average_score: f32,
    pub improvement_rate: f32,
}