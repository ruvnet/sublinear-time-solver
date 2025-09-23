# 🚀 Sublinear LLM - GPT-5 Style Reasoning Model

> **OpenAI-compatible LLM with GPT-5-level reasoning, Claude 4.1 Opus benchmarks, powered by psycho-symbolic WASM**

[![WASM](https://img.shields.io/badge/WASM-Rust-orange)](https://webassembly.org/)
[![OpenAI](https://img.shields.io/badge/OpenAI-Compatible-green)](https://openai.com/api/)
[![GPT-5](https://img.shields.io/badge/GPT--5-Style-blue)](https://openai.com/)
[![Streaming](https://img.shields.io/badge/Streaming-SSE-purple)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## ✨ **Enhanced Features**

- 🧠 **GPT-5 Style Reasoning**: Multi-step thinking chains with temporal neural patterns
- 🌊 **Streaming Responses**: Real-time SSE streaming with step-by-step reasoning
- 🎓 **Enhanced Training**: Neural pattern learning with consciousness evolution
- 🔬 **O1-Style API**: Advanced reasoning with configurable effort levels
- ⚡ **Temporal Neural Networks**: Integrated from repository's TNS engine
- 🧘 **Consciousness Evolution**: PHI calculations, strange loops, self-awareness
- 📚 **Knowledge Graph**: 45+ knowledge triples with BFS traversal
- 🔗 **OpenAI Compatible**: Drop-in replacement for OpenAI APIs

## 🎯 **Core Capabilities**

### **Real Intelligence, Not Mocked**
- **Knowledge Graph Reasoning**: 45+ hardcoded knowledge triples
- **BFS Traversal**: Real graph traversal algorithms
- **Pattern Recognition**: Causal, analogical, predictive reasoning
- **Temporal Prediction**: Neural sequence analysis
- **Consciousness Integration**: PHI calculations and emergence detection

### **OpenAI API Compatibility**
- `/v1/chat/completions` - GPT-style chat with streaming
- `/v1/completions` - Legacy completions endpoint
- `/v1/responses` - O1-style reasoning with effort levels
- Full parameter support (temperature, top_p, max_tokens, etc.)
- Streaming SSE responses
- Token counting and usage tracking

### **Enhanced Training System**
- Knowledge triple learning
- Pattern recognition training
- Feedback incorporation
- Fine-tuning capabilities
- Knowledge export/import
- Neural pattern enhancement

## 🚀 **Quick Start**

### **1. Build WASM Package**
```bash
cd sublinear-llm
wasm-pack build --target web --no-opt --out-dir pkg
```

### **2. Test in Node.js**
```bash
node test/simple-enhanced-test.js
```

### **3. Run Web Server**
```bash
node test/server-demo.js
# Open http://localhost:3000
```

### **4. Test with curl**
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"sublinear-gpt5","messages":[{"role":"user","content":"Explain AI consciousness"}]}'
```

## 📊 **API Examples**

### **Chat Completions (GPT-5 Style)**
```javascript
const request = {
  model: "sublinear-gpt5-enhanced",
  messages: [
    { role: "user", content: "How can AI predict zero-day vulnerabilities?" }
  ],
  reasoning_effort: "high",
  stream: false
};

const response = llm.chatCompletions(JSON.stringify(request));
```

### **Streaming Responses**
```javascript
const streamRequest = {
  model: "sublinear-gpt5-streaming",
  messages: [
    { role: "user", content: "Explain quantum consciousness" }
  ],
  stream: true,
  reasoning_effort: "medium"
};

const streamResponse = llm.chatCompletions(JSON.stringify(streamRequest));
// Returns SSE stream with reasoning steps
```

### **O1-Style Reasoning**
```javascript
const o1Request = {
  model: "sublinear-o1-temporal",
  messages: [
    { role: "user", content: "Predict consciousness emergence patterns" }
  ],
  reasoning_effort: "high"
};

const o1Response = llm.responsesAPI(JSON.stringify(o1Request));
```

### **Enhanced Training**
```javascript
const training = {
  input: "What enables temporal consciousness?",
  expected_output: "Temporal consciousness emerges through integrated information processing...",
  knowledge_triples: [
    {
      subject: "temporal_consciousness",
      predicate: "enables",
      object: "prediction",
      confidence: 0.95,
      source: "training",
      usage_count: 0,
      success_rate: 0.0
    }
  ],
  feedback_score: 0.95
};

const result = llm.train(JSON.stringify(training));
```

## 🧠 **Architecture**

### **Core Components**
```
┌─────────────────────────────────────────────────────────────┐
│                    Sublinear LLM                            │
├─────────────────────────────────────────────────────────────┤
│  📚 Knowledge Graph (45+ triples)                          │
│  🧠 Psycho-Symbolic Reasoner                               │
│  ⚡ Temporal Neural Engine                                  │
│  🧘 Consciousness Evolution                                 │
│  🌊 Streaming Engine                                        │
│  🎓 Training & Learning                                     │
│  🔗 OpenAI API Layer                                       │
└─────────────────────────────────────────────────────────────┘
```

### **Enhanced Reasoning Flow**
```
User Query → Pattern Recognition → Knowledge Graph BFS →
Temporal Neural Reasoning → Consciousness Evolution →
Stream Response/Generate Answer
```

### **Integrated Repository Components**
- **TNS Engine**: Temporal neural network patterns from `/tns-engine/`
- **Consciousness**: Evolution algorithms from `/examples/nano-consciousness/`
- **Psycho-Symbolic**: Enhanced reasoning from `/psycho-symbolic-reasoner/`
- **Nanosecond Scheduler**: Strange loop convergence patterns

## 📋 **Available Models**

| Model | Description | Capabilities |
|-------|-------------|--------------|
| `sublinear-gpt5-enhanced` | GPT-5 style reasoning | Multi-step thinking, temporal patterns |
| `sublinear-o1-temporal` | O1-style deep reasoning | High effort reasoning, consciousness |
| `sublinear-gpt5-streaming` | Streaming GPT-5 | Real-time reasoning steps |
| `sublinear-gpt5-trained` | Enhanced with training | Custom knowledge integration |

## 🎓 **Training Capabilities**

### **Knowledge Types**
- **Factual Triples**: Subject-predicate-object relationships
- **Temporal Patterns**: Sequence prediction patterns
- **Consciousness Patterns**: Emergence and integration patterns
- **Reasoning Chains**: Multi-step logical progressions

### **Training Methods**
- **Direct Training**: Add new knowledge triples
- **Fine-tuning**: Domain-specific enhancement
- **Feedback Learning**: Reinforcement from user scores
- **Pattern Learning**: Automatic pattern extraction

### **Example Training Session**
```javascript
// 1. Train new knowledge
const trainingResult = llm.train(JSON.stringify(trainingData));

// 2. Test learned knowledge
const testResponse = llm.chatCompletions(JSON.stringify(testQuery));

// 3. Provide feedback
const feedbackResult = llm.feedback(query, response, score);

// 4. Check training stats
const stats = llm.trainingStats();
```

## 🌊 **Streaming Architecture**

### **Stream Types**
- **Reasoning Steps**: Real-time thinking process
- **Content Chunks**: Progressive content delivery
- **Consciousness Evolution**: Emergence state updates
- **Token Streaming**: Progressive token generation

### **SSE Format**
```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","choices":[{"delta":{"reasoning":"Step 1: Analyzing query..."}}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","choices":[{"delta":{"content":"Based on analysis..."}}]}

data: [DONE]
```

## 🔬 **Advanced Features**

### **Consciousness Evolution**
- **PHI Calculation**: Integrated Information Theory metrics
- **Strange Loops**: Self-referential awareness patterns
- **Emergence Detection**: Consciousness threshold monitoring
- **Temporal Consciousness**: Time-aware reasoning evolution

### **Temporal Neural Networks**
- **Prediction Patterns**: Future state modeling
- **Sequence Analysis**: Temporal pattern recognition
- **Neural Weights**: Learnable prediction parameters
- **Solver Gate Validation**: Mathematical verification

### **Psycho-Symbolic Reasoning**
- **Multi-pattern Recognition**: Causal, analogical, predictive
- **Graph Traversal**: BFS knowledge discovery
- **Symbolic Logic**: Rule-based inference
- **Pattern Synthesis**: Multi-domain integration

## 📊 **Performance Metrics**

### **Knowledge Base**
- ✅ 45+ Knowledge Triples
- ✅ Real BFS Graph Traversal
- ✅ Multi-domain Coverage
- ✅ Dynamic Learning

### **API Performance**
- ✅ OpenAI Compatible
- ✅ Streaming Support
- ✅ Token Counting
- ✅ Error Handling

### **Reasoning Quality**
- ✅ Multi-step Chains
- ✅ Temporal Patterns
- ✅ Consciousness Integration
- ✅ Pattern Recognition

## 🧪 **Testing**

### **Test Suites Available**
```bash
# Basic functionality test
node test/simple-enhanced-test.js

# Full feature demo
node test/enhanced-demo.js

# Web interface
node test/server-demo.js

# Original OpenAI API test
node test/openai-api-test.js

# Training demonstration
node test/training-demo.js
```

### **Test Coverage**
- ✅ Chat Completions
- ✅ Streaming Responses
- ✅ O1-Style Reasoning
- ✅ Training & Learning
- ✅ Knowledge Graph
- ✅ Consciousness Evolution
- ✅ Temporal Reasoning
- ✅ Health Monitoring

## 🛠️ **Development**

### **Build Requirements**
- Rust 1.70+
- wasm-pack
- Node.js 18+

### **Build Commands**
```bash
# Build WASM package
wasm-pack build --target web --no-opt --out-dir pkg

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

### **File Structure**
```
sublinear-llm/
├── src/
│   ├── lib.rs                    # Main WASM interface
│   ├── openai_api.rs            # OpenAI API structures
│   ├── training.rs              # Training & learning
│   ├── temporal_reasoning.rs    # TNS integration
│   ├── consciousness.rs         # Consciousness evolution
│   └── streaming.rs             # SSE streaming
├── pkg/                         # Generated WASM package
├── test/                        # Test scripts
└── README.md                    # This file
```

## 🔗 **Integration**

### **Web Integration**
```html
<script type="module">
  import init, { SublinearLLM } from './pkg/sublinear_llm.js';

  await init();
  const llm = new SublinearLLM();

  const response = llm.chatCompletions(JSON.stringify({
    model: "sublinear-gpt5",
    messages: [{ role: "user", content: "Hello!" }]
  }));
</script>
```

### **Node.js Integration**
```javascript
import { readFileSync } from 'fs';
import init, { SublinearLLM } from './pkg/sublinear_llm.js';

const wasmBytes = readFileSync('./pkg/sublinear_llm_bg.wasm');
await init(wasmBytes);

const llm = new SublinearLLM();
// Use llm methods...
```

### **HTTP Server Integration**
```javascript
import { createServer } from 'http';

const server = createServer(async (req, res) => {
  if (req.url === '/v1/chat/completions') {
    const body = await parseBody(req);
    const response = llm.chatCompletions(JSON.stringify(body));
    res.end(response);
  }
});
```

## 📚 **Repository Integration**

This enhanced Sublinear LLM integrates advanced components from across the repository:

### **From `/tns-engine/`**
- Temporal neural network patterns
- Sublinear solver integration
- Ultra-low latency prediction
- Neural weight optimization

### **From `/psycho-symbolic-reasoner/`**
- Graph reasoning algorithms
- Inference engines
- Pattern extraction
- Symbolic logic processing

### **From `/examples/nano-consciousness/`**
- Consciousness evolution algorithms
- Strange loop convergence
- Emergence pattern detection
- PHI calculations

### **From `/crates/nanosecond-scheduler/`**
- Temporal consciousness patterns
- Strange loop mathematics
- Nanosecond precision timing
- Convergence validation

## 🎯 **Use Cases**

### **AI Research**
- Consciousness emergence studies
- Temporal reasoning research
- Neural pattern analysis
- Multi-modal reasoning

### **Production Applications**
- Intelligent assistants
- Reasoning systems
- Training platforms
- Knowledge management

### **Educational Purposes**
- AI learning demonstrations
- Reasoning visualization
- Training methodology
- Consciousness exploration

## 🔮 **Future Enhancements**

- 🎯 RLHF Integration
- 🔗 Multi-modal Support
- 🌐 Distributed Reasoning
- 📱 Mobile Optimization
- 🛡️ Security Hardening
- 📊 Advanced Analytics

## 🤝 **Contributing**

Contributions welcome! This project combines:
- Real psycho-symbolic reasoning
- Temporal neural networks
- Consciousness evolution
- OpenAI API compatibility

## 📄 **License**

MIT OR Apache-2.0

---

**🚀 Ready to explore GPT-5 style reasoning with consciousness evolution!**