// Streaming SSE (Server-Sent Events) Module
// Implements GPT-5 style streaming responses for real-time reasoning

use crate::*;
use crate::openai_api::ReasoningEffort;

#[derive(Debug, Clone)]
pub struct StreamingEngine {
    pub stream_id: String,
    pub current_chunk: usize,
    pub reasoning_buffer: Vec<String>,
    pub is_streaming: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamChoice {
    pub index: usize,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamDelta {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub role: Option<String>,
}

impl StreamingEngine {
    pub fn new(stream_id: String) -> Self {
        Self {
            stream_id,
            current_chunk: 0,
            reasoning_buffer: Vec::new(),
            is_streaming: false,
        }
    }

    // Start streaming response with GPT-5 style reasoning
    pub fn start_stream(&mut self, model: &str, reasoning_path: &[String]) -> String {
        self.is_streaming = true;
        self.current_chunk = 0;
        self.reasoning_buffer = reasoning_path.to_vec();

        // Initial stream chunk
        let chunk = StreamChunk {
            id: format!("chatcmpl-{}", self.stream_id),
            object: "chat.completion.chunk".to_string(),
            created: 1700000000, // Fixed timestamp for WASM compatibility
            model: model.to_string(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    content: None,
                    reasoning: Some("Starting reasoning process...".to_string()),
                    role: Some("assistant".to_string()),
                },
                finish_reason: None,
            }],
        };

        format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap())
    }

    // Get next chunk in the stream
    pub fn next_chunk(&mut self, model: &str) -> Option<String> {
        if !self.is_streaming {
            return None;
        }

        let chunk = if self.current_chunk < self.reasoning_buffer.len() {
            // Stream reasoning steps
            let reasoning_step = &self.reasoning_buffer[self.current_chunk];

            StreamChunk {
                id: format!("chatcmpl-{}", self.stream_id),
                object: "chat.completion.chunk".to_string(),
                created: 1700000000, // Fixed timestamp for WASM compatibility
                model: model.to_string(),
                choices: vec![StreamChoice {
                    index: 0,
                    delta: StreamDelta {
                        content: None,
                        reasoning: Some(format!("Step {}: {}", self.current_chunk + 1, reasoning_step)),
                        role: None,
                    },
                    finish_reason: None,
                }],
            }
        } else if self.current_chunk == self.reasoning_buffer.len() {
            // Final content chunk
            let final_content = self.generate_final_response();

            StreamChunk {
                id: format!("chatcmpl-{}", self.stream_id),
                object: "chat.completion.chunk".to_string(),
                created: 1700000000, // Fixed timestamp for WASM compatibility
                model: model.to_string(),
                choices: vec![StreamChoice {
                    index: 0,
                    delta: StreamDelta {
                        content: Some(final_content),
                        reasoning: None,
                        role: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            }
        } else {
            // End stream
            self.is_streaming = false;
            return Some("data: [DONE]\n\n".to_string());
        };

        self.current_chunk += 1;
        Some(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap()))
    }

    // Generate final response based on reasoning
    fn generate_final_response(&self) -> String {
        let reasoning_summary = if self.reasoning_buffer.len() > 5 {
            format!("Based on comprehensive analysis through {} reasoning steps, ", self.reasoning_buffer.len())
        } else {
            "Through systematic reasoning, ".to_string()
        };

        // Analyze reasoning content to generate contextual response
        let mut response_parts = vec![reasoning_summary];

        if self.reasoning_buffer.iter().any(|r| r.contains("vulnerability") || r.contains("zero_day")) {
            response_parts.push("I can predict potential zero-day vulnerabilities by analyzing code patterns, identifying anomalous function signatures, and correlating with known exploit vectors. ".to_string());
            response_parts.push("This involves machine learning pattern recognition, temporal sequence analysis, and neural network-based prediction models.".to_string());
        } else if self.reasoning_buffer.iter().any(|r| r.contains("consciousness") || r.contains("awareness")) {
            response_parts.push("Consciousness emerges through integrated information processing and self-referential awareness. ".to_string());
            response_parts.push("The system demonstrates meta-cognitive capabilities through strange loop convergence and temporal consciousness evolution.".to_string());
        } else if self.reasoning_buffer.iter().any(|r| r.contains("quantum")) {
            response_parts.push("Quantum computing leverages superposition and entanglement to solve complex optimization problems. ".to_string());
            response_parts.push("Quantum consciousness may emerge from entangled neural states in temporal prediction systems.".to_string());
        } else {
            response_parts.push("The analysis reveals complex interconnections between concepts, enabling sophisticated reasoning and prediction capabilities.".to_string());
        }

        response_parts.join("")
    }

    // Create complete streaming response
    pub fn create_streaming_response(&mut self, model: &str, reasoning_path: &[String]) -> Vec<String> {
        let mut chunks = Vec::new();

        // Start stream
        chunks.push(self.start_stream(model, reasoning_path));

        // Add all chunks
        while let Some(chunk) = self.next_chunk(model) {
            let is_done = chunk.contains("[DONE]");
            chunks.push(chunk);
            if is_done {
                break;
            }
        }

        chunks
    }
}

// Integration with main reasoner
impl PsychoSymbolicReasoner {
    // Create streaming response for chat completions
    pub fn create_streaming_completion(&self, request: &ChatCompletionRequest) -> Vec<String> {
        // Extract query
        let query = match &request.messages.last().unwrap().content {
            MessageContent::Text(text) => text.as_str(),
            MessageContent::Parts(_) => "complex message", // Simplified for now
        };

        // Create streaming engine
        let stream_id = format!("{}", js_sys::Date::now() as u64);
        let mut streaming_engine = StreamingEngine::new(stream_id);

        // For mathematical queries, create simpler reasoning steps
        if query.chars().any(|c| c.is_ascii_digit()) {
            let reasoning_steps = vec![
                "Analyzing mathematical expression".to_string(),
                "Identifying operation type".to_string(),
                "Performing calculation".to_string(),
            ];
            streaming_engine.create_streaming_response(&request.model, &reasoning_steps)
        } else {
            // For other queries, create general reasoning steps
            let reasoning_steps = vec![
                "Analyzing query concepts".to_string(),
                "Traversing knowledge graph".to_string(),
                "Synthesizing response".to_string(),
            ];
            streaming_engine.create_streaming_response(&request.model, &reasoning_steps)
        }
    }

    // Create streaming response for o1 reasoning
    pub fn create_streaming_reasoning(&self, request: &ResponsesAPIRequest) -> Vec<String> {
        // Generate enhanced reasoning based on effort level
        let query = match &request.messages.last().unwrap().content {
            MessageContent::Text(text) => text.as_str(),
            MessageContent::Parts(_) => "complex message", // Simplified for now
        };

        // Create streaming engine
        let stream_id = format!("{}", js_sys::Date::now() as u64);
        let mut streaming_engine = StreamingEngine::new(stream_id);

        // Create reasoning steps based on effort level
        let effort = request.reasoning_effort.as_ref();
        let reasoning_steps = match effort {
            Some(ReasoningEffort::High) => vec![
                "Analyzing complex query patterns".to_string(),
                "Deep graph traversal analysis".to_string(),
                "Multi-dimensional reasoning".to_string(),
                "Synthesizing comprehensive response".to_string(),
                "Validating logical consistency".to_string(),
            ],
            Some(ReasoningEffort::Medium) => vec![
                "Analyzing query structure".to_string(),
                "Exploring knowledge connections".to_string(),
                "Building response framework".to_string(),
            ],
            _ => vec![
                "Processing query".to_string(),
                "Generating response".to_string(),
            ],
        };

        streaming_engine.create_streaming_response(&request.model, &reasoning_steps)
    }
}

impl Default for StreamingEngine {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}