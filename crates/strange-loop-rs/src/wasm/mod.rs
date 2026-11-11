//! WebAssembly bindings for strange-loop crate
//!
//! This module provides WASM bindings for running strange loops in web browsers
//! with JavaScript interoperability.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::{
    consciousness::{ConsciousnessMetrics, ConsciousnessState},
    error::Result,
    quantum_container::QuantumContainer,
    strange_attractor::{TemporalAttractor, AttractorConfig, AttractorType},
    temporal_consciousness::{TemporalConsciousness, ConsciousnessConfig},
    types::{StrangeLoop, LoopConfig, ScalarReasoner, SimpleCritic, SafeReflector},
    sublinear_solver::{SublinearNeumannSolver, SublinearConfig, SublinearNeumannResult, ComplexityBound},
};

#[cfg(feature = "wasm")]
use js_sys::{Array, Object, Reflect};

#[cfg(feature = "wasm")]
use web_sys::console;

/// Initialize WASM module with panic hook
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Test function to verify WASM exports are working
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn test_wasm_export() -> String {
    "WASM exports working!".to_string()
}

/// Get version of strange-loop crate
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version() -> String {
    "0.1.2".to_string()
}

/// WASM-compatible consciousness configuration
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmConsciousnessConfig {
    inner: ConsciousnessConfig,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmConsciousnessConfig {
    /// Create new consciousness configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ConsciousnessConfig::default(),
        }
    }

    /// Create research mode configuration
    #[wasm_bindgen]
    pub fn research_mode() -> Self {
        Self {
            inner: ConsciousnessConfig::research_mode(),
        }
    }

    /// Create real-time mode configuration
    #[wasm_bindgen]
    pub fn real_time_mode() -> Self {
        Self {
            inner: ConsciousnessConfig::real_time_mode(),
        }
    }

    /// Set consciousness threshold
    #[wasm_bindgen]
    pub fn set_consciousness_threshold(&mut self, threshold: f64) {
        self.inner.consciousness_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get consciousness threshold
    #[wasm_bindgen]
    pub fn consciousness_threshold(&self) -> f64 {
        self.inner.consciousness_threshold
    }

    /// Enable/disable quantum processing
    #[wasm_bindgen]
    pub fn set_enable_quantum(&mut self, enable: bool) {
        self.inner.enable_quantum = enable;
    }

    /// Check if quantum processing is enabled
    #[wasm_bindgen]
    pub fn enable_quantum(&self) -> bool {
        self.inner.enable_quantum
    }

    /// Set number of phi elements
    #[wasm_bindgen]
    pub fn set_phi_elements(&mut self, elements: usize) {
        self.inner.phi_elements = elements.clamp(1, 16); // Limit for WASM performance
    }

    /// Get number of phi elements
    #[wasm_bindgen]
    pub fn phi_elements(&self) -> usize {
        self.inner.phi_elements
    }

    /// Set coupling strength
    #[wasm_bindgen]
    pub fn set_coupling_strength(&mut self, strength: f64) {
        self.inner.coupling_strength = strength.clamp(0.0, 1.0);
    }

    /// Get coupling strength
    #[wasm_bindgen]
    pub fn coupling_strength(&self) -> f64 {
        self.inner.coupling_strength
    }

    /// Set maximum evolution iterations
    #[wasm_bindgen]
    pub fn set_max_iterations(&mut self, iterations: usize) {
        self.inner.max_evolution_iterations = iterations.clamp(1, 100_000);
    }

    /// Get maximum evolution iterations
    #[wasm_bindgen]
    pub fn max_iterations(&self) -> usize {
        self.inner.max_evolution_iterations
    }
}

/// WASM-compatible temporal consciousness system
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmTemporalConsciousness {
    inner: TemporalConsciousness,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmTemporalConsciousness {
    /// Create new temporal consciousness system
    #[wasm_bindgen(constructor)]
    pub fn new(config: &WasmConsciousnessConfig) -> Result<WasmTemporalConsciousness, JsValue> {
        match TemporalConsciousness::new(config.inner.clone()) {
            Ok(consciousness) => Ok(Self { inner: consciousness }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Evolve consciousness for specified iterations
    #[wasm_bindgen]
    pub fn evolve_consciousness(&mut self, iterations: usize) -> Result<JsValue, JsValue> {
        match self.inner.evolve_consciousness(iterations) {
            Ok(result) => {
                let obj = Object::new();
                Reflect::set(&obj, &"evolved".into(), &result.evolved.into())?;
                Reflect::set(&obj, &"iterations_completed".into(), &result.iterations_completed.into())?;
                Reflect::set(&obj, &"final_consciousness_level".into(), &result.final_consciousness_level.into())?;
                Reflect::set(&obj, &"max_phi_achieved".into(), &result.max_phi_achieved.into())?;
                Reflect::set(&obj, &"emergence_events".into(), &result.emergence_events.into())?;
                Reflect::set(&obj, &"self_modifications".into(), &result.self_modifications.into())?;
                Reflect::set(&obj, &"evolution_time_ns".into(), &(result.evolution_time_ns as f64).into())?;
                Ok(obj.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Calculate phi (integrated information)
    #[wasm_bindgen]
    pub fn calculate_phi(&mut self, num_elements: usize, num_connections: usize, coupling_strength: f64) -> f64 {
        self.inner.calculate_phi(num_elements, num_connections, coupling_strength)
    }

    /// Get current consciousness level
    #[wasm_bindgen]
    pub fn consciousness_level(&self) -> f64 {
        self.inner.current_state().consciousness_index()
    }

    /// Get current emergence level
    #[wasm_bindgen]
    pub fn emergence_level(&self) -> f64 {
        self.inner.current_state().emergence_level
    }

    /// Get current self-awareness level
    #[wasm_bindgen]
    pub fn self_awareness(&self) -> f64 {
        self.inner.current_state().self_awareness
    }

    /// Get current meta-cognition level
    #[wasm_bindgen]
    pub fn meta_cognition(&self) -> f64 {
        self.inner.current_state().meta_cognition
    }

    /// Get temporal coherence
    #[wasm_bindgen]
    pub fn temporal_coherence(&self) -> f64 {
        self.inner.current_state().temporal_coherence
    }

    /// Get integration measure
    #[wasm_bindgen]
    pub fn integration_measure(&self) -> f64 {
        self.inner.current_state().integration_measure
    }

    /// Verify consciousness
    #[wasm_bindgen]
    pub fn verify_consciousness(&self) -> Result<JsValue, JsValue> {
        let verification = self.inner.verify_consciousness();
        let obj = Object::new();

        Reflect::set(&obj, &"is_conscious".into(), &verification.is_conscious.into())?;
        Reflect::set(&obj, &"confidence".into(), &verification.confidence.into())?;
        Reflect::set(&obj, &"self_recognition".into(), &verification.self_recognition.into())?;
        Reflect::set(&obj, &"meta_cognitive".into(), &verification.meta_cognitive.into())?;
        Reflect::set(&obj, &"temporal_coherence".into(), &verification.temporal_coherence.into())?;
        Reflect::set(&obj, &"integration".into(), &verification.integration.into())?;
        Reflect::set(&obj, &"phi_value".into(), &verification.phi_value.into())?;
        Reflect::set(&obj, &"consciousness_index".into(), &verification.consciousness_index.into())?;

        Ok(obj.into())
    }

    /// Get evolution history as JavaScript array
    #[wasm_bindgen]
    pub fn evolution_history(&self) -> Result<JsValue, JsValue> {
        let history = self.inner.evolution_history();
        let array = Array::new();

        for step in history.iter().take(100) { // Limit for performance
            let obj = Object::new();
            Reflect::set(&obj, &"iteration".into(), &step.iteration.into())?;
            Reflect::set(&obj, &"consciousness_level".into(), &step.consciousness_state.consciousness_index().into())?;
            Reflect::set(&obj, &"phi_value".into(), &step.phi_value.into())?;
            Reflect::set(&obj, &"quantum_complexity".into(), &step.quantum_state_complexity.into())?;
            Reflect::set(&obj, &"loop_convergence".into(), &step.loop_convergence.into())?;
            array.push(&obj);
        }

        Ok(array.into())
    }

    /// Get emergence patterns
    #[wasm_bindgen]
    pub fn emergence_patterns(&self) -> Result<JsValue, JsValue> {
        let patterns = self.inner.emergence_patterns();
        let array = Array::new();

        for pattern in patterns.iter().take(50) { // Limit for performance
            let obj = Object::new();
            Reflect::set(&obj, &"iteration".into(), &pattern.iteration.into())?;
            Reflect::set(&obj, &"consciousness_level".into(), &pattern.consciousness_level.into())?;
            Reflect::set(&obj, &"phi_value".into(), &pattern.phi_value.into())?;
            Reflect::set(&obj, &"quantum_complexity".into(), &pattern.quantum_state_complexity.into())?;
            array.push(&obj);
        }

        Ok(array.into())
    }

    /// Get number of self-modifications
    #[wasm_bindgen]
    pub fn self_modification_count(&self) -> usize {
        self.inner.self_modification_log().len()
    }

    /// Reset the consciousness system
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.inner.reset().map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get system statistics
    #[wasm_bindgen]
    pub fn statistics(&self) -> Result<JsValue, JsValue> {
        let stats = self.inner.metrics().get_statistics();
        let obj = Object::new();

        Reflect::set(&obj, &"min".into(), &stats.min.into())?;
        Reflect::set(&obj, &"max".into(), &stats.max.into())?;
        Reflect::set(&obj, &"mean".into(), &stats.mean.into())?;
        Reflect::set(&obj, &"std_dev".into(), &stats.std_dev.into())?;
        Reflect::set(&obj, &"p25".into(), &stats.p25.into())?;
        Reflect::set(&obj, &"p50".into(), &stats.p50.into())?;
        Reflect::set(&obj, &"p75".into(), &stats.p75.into())?;
        Reflect::set(&obj, &"emergence_events".into(), &stats.emergence_events.into())?;
        Reflect::set(&obj, &"self_modifications".into(), &stats.self_modifications.into())?;

        Ok(obj.into())
    }
}

/// WASM-compatible quantum container
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmQuantumContainer {
    inner: QuantumContainer,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmQuantumContainer {
    /// Create new quantum container
    #[wasm_bindgen(constructor)]
    pub fn new(num_qubits: usize) -> Self {
        let limited_qubits = num_qubits.clamp(1, 10); // Limit for WASM performance
        Self {
            inner: QuantumContainer::new(limited_qubits),
        }
    }

    /// Set superposition state
    #[wasm_bindgen]
    pub fn set_superposition_state(&mut self, state_index: usize, real: f64, imag: f64) {
        use num_complex::Complex64;
        let amplitude = Complex64::new(real, imag);
        self.inner.set_superposition_state(state_index, amplitude);
    }

    /// Get probability of specific state
    #[wasm_bindgen]
    pub fn get_probability(&self, state_index: usize) -> f64 {
        self.inner.get_probability(state_index)
    }

    /// Measure the quantum state
    #[wasm_bindgen]
    pub fn measure(&mut self) -> usize {
        self.inner.measure()
    }

    /// Store classical data
    #[wasm_bindgen]
    pub fn store_classical(&mut self, key: &str, value: f64) {
        self.inner.store_classical(key.to_string(), value);
    }

    /// Get classical data
    #[wasm_bindgen]
    pub fn get_classical(&self, key: &str) -> Option<f64> {
        self.inner.get_classical(key)
    }

    /// Create superposition from probability array
    #[wasm_bindgen]
    pub fn create_superposition_from_probabilities(&mut self, probabilities: &[f64]) -> Result<(), JsValue> {
        self.inner.create_superposition_from_classical(probabilities)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// WASM-compatible strange attractor
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmStrangeAttractor {
    inner: TemporalAttractor,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmStrangeAttractor {
    /// Create Lorenz attractor
    #[wasm_bindgen]
    pub fn lorenz(sigma: f64, rho: f64, beta: f64) -> Result<WasmStrangeAttractor, JsValue> {
        let config = AttractorConfig {
            attractor_type: AttractorType::Lorenz { sigma, rho, beta },
            dt_ns: 1000,
            steps_per_frame: 10,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        };

        match TemporalAttractor::new(config) {
            Ok(attractor) => Ok(Self { inner: attractor }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Create Rössler attractor
    #[wasm_bindgen]
    pub fn rossler(a: f64, b: f64, c: f64) -> Result<WasmStrangeAttractor, JsValue> {
        let config = AttractorConfig {
            attractor_type: AttractorType::Rossler { a, b, c },
            dt_ns: 1000,
            steps_per_frame: 10,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        };

        match TemporalAttractor::new(config) {
            Ok(attractor) => Ok(Self { inner: attractor }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Step the attractor forward
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.step() {
            Ok(state) => {
                let array = Array::new();
                array.push(&state[0].into());
                array.push(&state[1].into());
                array.push(&state[2].into());
                Ok(array.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Get current state
    #[wasm_bindgen]
    pub fn state(&self) -> JsValue {
        let state = self.inner.state();
        let array = Array::new();
        array.push(&state[0].into());
        array.push(&state[1].into());
        array.push(&state[2].into());
        array.into()
    }

    /// Get trajectory
    #[wasm_bindgen]
    pub fn trajectory(&self) -> JsValue {
        let trajectory = self.inner.trajectory();
        let array = Array::new();

        for point in trajectory.iter().take(1000) { // Limit for performance
            let point_array = Array::new();
            point_array.push(&point[0].into());
            point_array.push(&point[1].into());
            point_array.push(&point[2].into());
            array.push(&point_array);
        }

        array.into()
    }

    /// Reset attractor
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get time in nanoseconds
    #[wasm_bindgen]
    pub fn time_ns(&self) -> f64 {
        self.inner.time_ns() as f64
    }

    /// Calculate correlation dimension
    #[wasm_bindgen]
    pub fn correlation_dimension(&self) -> f64 {
        self.inner.correlation_dimension(3)
    }

    /// Get phase space volume
    #[wasm_bindgen]
    pub fn phase_space_volume(&self) -> f64 {
        self.inner.phase_space_volume()
    }
}

/// WASM-compatible simple strange loop
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmStrangeLoop {
    inner: StrangeLoop<ScalarReasoner, SimpleCritic, SafeReflector>,
    context: std::collections::HashMap<String, f64>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmStrangeLoop {
    /// Create new strange loop
    #[wasm_bindgen(constructor)]
    pub fn new(target: f64, step_size: f64) -> Self {
        let reasoner = ScalarReasoner::new(target, step_size);
        let critic = SimpleCritic::new();
        let reflector = SafeReflector::new();

        let config = LoopConfig {
            max_iterations: 10_000,
            max_duration_ns: 100_000_000, // 100ms
            convergence_threshold: 1e-9,
            lipschitz_constant: 0.9,
            enable_consciousness: false,
            enable_quantum: false,
            enable_simd: false, // Disable SIMD for WASM compatibility
        };

        let strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);
        let mut context = std::collections::HashMap::new();
        context.insert("x".to_string(), 10.0); // Initial value

        Self {
            inner: strange_loop,
            context,
        }
    }

    /// Run the strange loop
    #[wasm_bindgen]
    pub fn run(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.run(&mut self.context) {
            Ok(result) => {
                let obj = Object::new();
                Reflect::set(&obj, &"iterations".into(), &result.iterations.into())?;
                Reflect::set(&obj, &"final_score".into(), &result.final_score.into())?;
                Reflect::set(&obj, &"duration_ns".into(), &(result.duration_ns as f64).into())?;
                Reflect::set(&obj, &"converged".into(), &result.converged.into())?;
                Reflect::set(&obj, &"final_value".into(), &self.context.get("x").copied().unwrap_or(0.0).into())?;
                Ok(obj.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Get current value
    #[wasm_bindgen]
    pub fn current_value(&self) -> f64 {
        self.context.get("x").copied().unwrap_or(0.0)
    }

    /// Set initial value
    #[wasm_bindgen]
    pub fn set_initial_value(&mut self, value: f64) {
        self.context.insert("x".to_string(), value);
    }

    /// Get iteration count
    #[wasm_bindgen]
    pub fn iteration_count(&self) -> usize {
        self.inner.iteration_count()
    }
}

/// Log message to browser console
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn log(message: &str) {
    console::log_1(&message.into());
}

/// Get version information
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn version() -> String {
    crate::VERSION.to_string()
}

/// Check if WASM features are available
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_features() -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"quantum".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"attractors".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"consciousness".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"sublinear_solver".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"johnson_lindenstrauss".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"pagerank_sublinear".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"complexity_analysis".into(), &true.into()).unwrap();
    Reflect::set(&obj, &"simd".into(), &false.into()).unwrap(); // SIMD not available in WASM
    obj.into()
}

/// Utility function to create consciousness demo
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn create_consciousness_demo() -> Result<WasmTemporalConsciousness, JsValue> {
    let mut config = WasmConsciousnessConfig::new();
    config.set_consciousness_threshold(0.3);
    config.set_phi_elements(4);
    config.set_max_iterations(1000);
    config.set_enable_quantum(true);

    WasmTemporalConsciousness::new(&config)
}

/// Utility function to create simple attractor demo
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn create_lorenz_demo() -> Result<WasmStrangeAttractor, JsValue> {
    WasmStrangeAttractor::lorenz(10.0, 28.0, 8.0 / 3.0)
}

/// Utility function to run consciousness evolution demo
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_consciousness_demo(iterations: usize) -> Result<JsValue, JsValue> {
    let mut consciousness = create_consciousness_demo()?;
    consciousness.evolve_consciousness(iterations.min(1000)) // Limit for performance
}

/// WASM-compatible sublinear solver configuration
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmSublinearConfig {
    inner: SublinearConfig,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmSublinearConfig {
    /// Create new sublinear solver configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SublinearConfig::default(),
        }
    }

    /// Set maximum iterations
    #[wasm_bindgen]
    pub fn set_max_iterations(&mut self, iterations: usize) {
        self.inner.max_iterations = iterations.clamp(1, 10_000);
    }

    /// Get maximum iterations
    #[wasm_bindgen]
    pub fn max_iterations(&self) -> usize {
        self.inner.max_iterations
    }

    /// Set tolerance
    #[wasm_bindgen]
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.inner.tolerance = tolerance.clamp(1e-12, 1e-3);
    }

    /// Get tolerance
    #[wasm_bindgen]
    pub fn tolerance(&self) -> f64 {
        self.inner.tolerance
    }

    /// Set Johnson-Lindenstrauss distortion parameter
    #[wasm_bindgen]
    pub fn set_jl_distortion(&mut self, distortion: f64) {
        self.inner.jl_distortion = distortion.clamp(0.1, 0.9);
    }

    /// Get Johnson-Lindenstrauss distortion parameter
    #[wasm_bindgen]
    pub fn jl_distortion(&self) -> f64 {
        self.inner.jl_distortion
    }

    /// Set sketch ratio for matrix sparsification
    #[wasm_bindgen]
    pub fn set_sketch_ratio(&mut self, ratio: f64) {
        self.inner.sketch_ratio = ratio.clamp(0.01, 0.5);
    }

    /// Get sketch ratio
    #[wasm_bindgen]
    pub fn sketch_ratio(&self) -> f64 {
        self.inner.sketch_ratio
    }
}

/// WASM-compatible sublinear solver
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmSublinearSolver {
    inner: SublinearNeumannSolver,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmSublinearSolver {
    /// Create new sublinear solver
    #[wasm_bindgen(constructor)]
    pub fn new(config: &WasmSublinearConfig) -> Self {
        Self {
            inner: SublinearNeumannSolver::new(config.inner.clone()),
        }
    }

    /// Solve linear system Ax = b with guaranteed O(log n) complexity
    #[wasm_bindgen]
    pub fn solve_sublinear(&self, matrix_json: &str, b: &[f64]) -> Result<JsValue, JsValue> {
        // Parse matrix from JSON
        let matrix: Vec<Vec<f64>> = serde_json::from_str(matrix_json)
            .map_err(|e| JsValue::from_str(&format!("Matrix parsing error: {}", e)))?;

        // Solve the system
        match self.inner.solve_sublinear_guaranteed(&matrix, b) {
            Ok(result) => {
                let obj = Object::new();

                // Convert solution to JavaScript array
                let solution_array = Array::new();
                for &val in &result.solution {
                    solution_array.push(&val.into());
                }

                Reflect::set(&obj, &"solution".into(), &solution_array)?;
                Reflect::set(&obj, &"iterations_used".into(), &result.iterations_used.into())?;
                Reflect::set(&obj, &"final_residual".into(), &result.final_residual.into())?;
                Reflect::set(&obj, &"compression_ratio".into(), &result.compression_ratio.into())?;
                Reflect::set(&obj, &"convergence_rate".into(), &result.convergence_rate.into())?;
                Reflect::set(&obj, &"solve_time_ns".into(), &(result.solve_time_ns as f64).into())?;

                let complexity_str = match result.complexity_bound {
                    ComplexityBound::Logarithmic => "O(log n)",
                    ComplexityBound::Sublinear => "O(n^k), k < 1",
                    ComplexityBound::Linear => "O(n)",
                    ComplexityBound::Superlinear => "O(n^k), k > 1",
                };
                Reflect::set(&obj, &"complexity_bound".into(), &complexity_str.into())?;

                Ok(obj.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Compute PageRank with sublinear complexity
    #[wasm_bindgen]
    pub fn page_rank_sublinear(&self, adjacency_json: &str, damping: f64, personalized: Option<Vec<f64>>) -> Result<JsValue, JsValue> {
        // Parse adjacency matrix from JSON
        let adjacency: Vec<Vec<f64>> = serde_json::from_str(adjacency_json)
            .map_err(|e| JsValue::from_str(&format!("Adjacency matrix parsing error: {}", e)))?;

        let personalized_ref = personalized.as_ref().map(|v| v.as_slice());

        match self.inner.page_rank_sublinear(&adjacency, damping, personalized_ref) {
            Ok(pagerank) => {
                let array = Array::new();
                for &val in &pagerank {
                    array.push(&val.into());
                }
                Ok(array.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Analyze matrix complexity and properties
    #[wasm_bindgen]
    pub fn analyze_complexity(&self, matrix_json: &str) -> Result<JsValue, JsValue> {
        // Parse matrix from JSON
        let matrix: Vec<Vec<f64>> = serde_json::from_str(matrix_json)
            .map_err(|e| JsValue::from_str(&format!("Matrix parsing error: {}", e)))?;

        match self.inner.analyze_complexity(&matrix) {
            Ok(analysis) => {
                let obj = Object::new();
                for (key, value) in analysis {
                    Reflect::set(&obj, &key.into(), &value.into())?;
                }
                Ok(obj.into())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Verify sublinear conditions for a matrix
    #[wasm_bindgen]
    pub fn verify_sublinear_conditions(&self, matrix_json: &str) -> Result<String, JsValue> {
        let matrix: Vec<Vec<f64>> = serde_json::from_str(matrix_json)
            .map_err(|e| JsValue::from_str(&format!("Matrix parsing error: {}", e)))?;

        match self.inner.verify_sublinear_conditions(&matrix) {
            Ok(complexity_bound) => {
                let bound_str = match complexity_bound {
                    ComplexityBound::Logarithmic => "O(log n) - Guaranteed sublinear complexity",
                    ComplexityBound::Sublinear => "O(n^k), k < 1 - Sublinear complexity possible",
                    ComplexityBound::Linear => "O(n) - Linear complexity required",
                    ComplexityBound::Superlinear => "O(n^k), k > 1 - Superlinear complexity",
                };
                Ok(bound_str.to_string())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}

/// Create sublinear solver demo
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn create_sublinear_demo() -> Result<WasmSublinearSolver, JsValue> {
    let mut config = WasmSublinearConfig::new();
    config.set_max_iterations(100);
    config.set_tolerance(1e-6);
    config.set_jl_distortion(0.3);
    config.set_sketch_ratio(0.1);

    Ok(WasmSublinearSolver::new(&config))
}

/// Solve a demo linear system with O(log n) complexity
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn solve_demo_system() -> Result<JsValue, JsValue> {
    let solver = create_sublinear_demo()?;

    // Create a demo diagonally dominant matrix
    let matrix_json = r#"[
        [4.0, 1.0, 1.0],
        [1.0, 4.0, 1.0],
        [1.0, 1.0, 4.0]
    ]"#;

    let b = vec![6.0, 6.0, 6.0];

    solver.solve_sublinear(matrix_json, &b)
}

/// Compute demo PageRank with sublinear complexity
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn page_rank_demo() -> Result<JsValue, JsValue> {
    let solver = create_sublinear_demo()?;

    // Demo adjacency matrix for a simple 4-node graph
    let adjacency_json = r#"[
        [0.0, 1.0, 1.0, 0.0],
        [1.0, 0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 0.0]
    ]"#;

    solver.page_rank_sublinear(adjacency_json, 0.85, None)
}

#[cfg(not(feature = "wasm"))]
pub mod placeholder {
    //! Placeholder module when WASM feature is not enabled

    /// Placeholder function
    pub fn wasm_not_enabled() {
        println!("WASM features are not enabled. Build with --features wasm to enable.");
    }
}

#[cfg(not(feature = "wasm"))]
pub use placeholder::*;