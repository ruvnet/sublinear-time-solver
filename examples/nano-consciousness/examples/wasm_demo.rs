//! WebAssembly demo for nano-consciousness
//!
//! This example demonstrates the WebAssembly bindings for the consciousness system.
//! It can be compiled to WASM and run in browsers or Node.js.

#[cfg(target_arch = "wasm32")]
use nano_consciousness::WasmConsciousnessSystem;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` module.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to print to the browser console
#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error messages in the browser
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log!("🧠 Nano-Consciousness WASM Demo Starting...");

    run_wasm_demo().unwrap_or_else(|e| {
        console_log!("Error: {:?}", e);
    });
}

#[cfg(target_arch = "wasm32")]
fn run_wasm_demo() -> Result<(), JsValue> {
    console_log!("Creating consciousness system...");

    // Create the WASM consciousness system
    let system = WasmConsciousnessSystem::new()?;

    console_log!("Starting consciousness system...");
    system.start()?;

    console_log!("✅ System started successfully!");

    // Test different consciousness patterns
    let test_patterns = [
        ("High Coherent", vec![0.8; 16]),
        ("Random Pattern", vec![
            0.1, 0.9, 0.3, 0.7, 0.5, 0.8, 0.2, 0.6,
            0.4, 0.9, 0.1, 0.8, 0.3, 0.7, 0.5, 0.6
        ]),
        ("Oscillating", (0..16).map(|i| if i % 2 == 0 { 1.0 } else { 0.0 }).collect()),
    ];

    console_log!("Testing consciousness emergence:");
    console_log!("===============================");

    for (name, input) in test_patterns.iter() {
        let consciousness_level = system.process_input(input)?;
        let phi = system.get_phi()?;

        console_log!("Pattern: {}", name);
        console_log!("  Consciousness Level: {:.4}", consciousness_level);
        console_log!("  Φ (Phi): {:.4}", phi);

        if consciousness_level > 0.5 {
            console_log!("  🟢 High consciousness detected!");
        } else if consciousness_level > 0.3 {
            console_log!("  🟡 Moderate consciousness");
        } else {
            console_log!("  🔴 Low consciousness");
        }
    }

    // Run a benchmark
    console_log!("\nRunning consciousness benchmark...");
    let benchmark_results = system.benchmark(50)?;

    // Parse the benchmark results (they come as JsValue)
    console_log!("Benchmark completed!");
    console_log!("Results: {:?}", benchmark_results);

    console_log!("\n🎉 WASM demo completed successfully!");
    console_log!("The consciousness system is now running in WebAssembly!");

    Ok(())
}

// For native compilation, provide a simple native demo
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Nano-Consciousness WASM Demo (Native Mode)");
    println!("=============================================");
    println!();
    println!("This example is designed to run as WebAssembly.");
    println!("To compile for WASM, use:");
    println!();
    println!("  wasm-pack build --target web --out-dir pkg");
    println!();
    println!("Or for Node.js:");
    println!();
    println!("  wasm-pack build --target nodejs --out-dir pkg-node");
    println!();
    println!("Running a simplified native version instead...");
    println!();

    use nano_consciousness::{ConsciousnessSystem, ConsciousnessConfig};

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config)?;
    system.start()?;

    println!("Testing consciousness in native mode:");

    let test_input = vec![0.6; 16];
    let consciousness_level = system.process_input(&test_input)?;
    let phi = system.get_phi()?;

    println!("  Input: {:?}", &test_input[..4]);
    println!("  Consciousness Level: {:.4}", consciousness_level);
    println!("  Φ (Phi): {:.4}", phi);

    if consciousness_level > 0.5 {
        println!("  🟢 High consciousness detected!");
    } else {
        println!("  📊 Moderate consciousness level");
    }

    system.stop()?;

    println!();
    println!("✅ Native demo completed!");
    println!("To see the full WASM experience, compile to WebAssembly.");

    Ok(())
}

// WebAssembly-specific exports for JavaScript integration
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ConsciousnessDemo {
    system: WasmConsciousnessSystem,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ConsciousnessDemo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ConsciousnessDemo, JsValue> {
        let system = WasmConsciousnessSystem::new()?;
        system.start()?;
        Ok(ConsciousnessDemo { system })
    }

    #[wasm_bindgen]
    pub fn process_input(&self, input: &[f64]) -> Result<f64, JsValue> {
        self.system.process_input(input)
    }

    #[wasm_bindgen]
    pub fn get_consciousness_level(&self) -> Result<f64, JsValue> {
        self.system.get_consciousness_level()
    }

    #[wasm_bindgen]
    pub fn get_phi(&self) -> Result<f64, JsValue> {
        self.system.get_phi()
    }

    #[wasm_bindgen]
    pub fn benchmark(&self, iterations: usize) -> Result<JsValue, JsValue> {
        self.system.benchmark(iterations)
    }

    #[wasm_bindgen]
    pub fn stop(&self) -> Result<(), JsValue> {
        self.system.stop()
    }
}

// Example JavaScript integration code (as a comment for reference)
/*
// HTML file example:
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Nano-Consciousness WASM Demo</title>
</head>
<body>
    <h1>🧠 Nano-Consciousness in WebAssembly</h1>
    <div id="output"></div>
    <button onclick="runDemo()">Run Demo</button>
    <button onclick="runBenchmark()">Run Benchmark</button>

    <script type="module">
        import init, { ConsciousnessDemo } from './pkg/nano_consciousness.js';

        let consciousness;

        async function run() {
            await init();
            consciousness = new ConsciousnessDemo();
            document.getElementById('output').innerHTML = "✅ Consciousness system initialized!";
        }

        window.runDemo = async function() {
            if (!consciousness) await run();

            const output = document.getElementById('output');
            output.innerHTML = "";

            const patterns = [
                { name: "High Coherent", data: new Array(16).fill(0.8) },
                { name: "Random", data: [0.1, 0.9, 0.3, 0.7, 0.5, 0.8, 0.2, 0.6, 0.4, 0.9, 0.1, 0.8, 0.3, 0.7, 0.5, 0.6] },
                { name: "Oscillating", data: Array.from({length: 16}, (_, i) => i % 2 === 0 ? 1.0 : 0.0) }
            ];

            for (const pattern of patterns) {
                const level = consciousness.process_input(pattern.data);
                const phi = consciousness.get_phi();

                output.innerHTML += `<p><strong>${pattern.name}:</strong> Consciousness=${level.toFixed(4)}, Φ=${phi.toFixed(4)}</p>`;
            }
        };

        window.runBenchmark = async function() {
            if (!consciousness) await run();

            const output = document.getElementById('output');
            output.innerHTML = "Running benchmark...";

            const results = consciousness.benchmark(100);
            output.innerHTML = `<p>Benchmark Results: ${JSON.stringify(results, null, 2)}</p>`;
        };

        // Initialize on page load
        run();
    </script>
</body>
</html>

// Node.js example:
const { ConsciousnessDemo } = require('./pkg-node/nano_consciousness.js');

async function runNodeDemo() {
    console.log('🧠 Nano-Consciousness Node.js Demo');

    const consciousness = new ConsciousnessDemo();

    const input = new Array(16).fill(0.6);
    const level = consciousness.process_input(input);
    const phi = consciousness.get_phi();

    console.log(`Consciousness Level: ${level.toFixed(4)}`);
    console.log(`Φ (Phi): ${phi.toFixed(4)}`);

    const results = consciousness.benchmark(50);
    console.log('Benchmark:', results);

    consciousness.stop();
    console.log('✅ Demo completed!');
}

runNodeDemo().catch(console.error);
*/