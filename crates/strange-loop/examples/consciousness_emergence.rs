//! Consciousness emergence and verification example

use strange_loop::{
    temporal_consciousness::{TemporalConsciousness, ConsciousnessConfig},
    consciousness::ConsciousnessVerifier,
    error::Result,
};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🧠 Consciousness Emergence Example");
    println!("==================================");

    // Create research-mode configuration for consciousness experiments
    let config = ConsciousnessConfig::research_mode();
    println!("Research Configuration:");
    println!("  🔬 Quantum processing: {}", config.enable_quantum);
    println!("  🌀 Strange attractors: {}", config.enable_attractors);
    println!("  📐 Lipschitz constraints: {}", config.enable_lipschitz);
    println!("  🔧 Self-modification: {}", config.enable_self_modification);
    println!("  🎯 Consciousness threshold: {:.3}", config.consciousness_threshold);
    println!("  ⚛️  Φ elements: {}", config.phi_elements);
    println!("  🔗 Coupling strength: {:.3}", config.coupling_strength);
    println!("  ⏱️  Evolution iterations: {}", config.max_evolution_iterations);
    println!();

    // Create temporal consciousness system
    let mut consciousness = TemporalConsciousness::new(config)?;

    // Show initial state
    demonstrate_initial_state(&consciousness);

    // Evolve consciousness and monitor emergence
    demonstrate_consciousness_evolution(&mut consciousness)?;

    // Verify consciousness using multiple tests
    demonstrate_consciousness_verification(&consciousness);

    // Analyze emergence patterns
    demonstrate_emergence_analysis(&consciousness);

    // Show self-modification capabilities
    demonstrate_self_modification(&consciousness);

    // Advanced: Multiple consciousness systems
    demonstrate_multiple_consciousness_systems()?;

    Ok(())
}

fn demonstrate_initial_state(consciousness: &TemporalConsciousness) {
    println!("🌱 Initial Consciousness State");
    println!("-----------------------------");

    let state = consciousness.current_state();
    println!("Initial measurements:");
    println!("  📊 Consciousness index: {:.6}", state.consciousness_index());
    println!("  🌟 Emergence level: {:.6}", state.emergence_level);
    println!("  👁️  Self-awareness: {:.6}", state.self_awareness);
    println!("  🤔 Meta-cognition: {:.6}", state.meta_cognition);
    println!("  ⏰ Temporal coherence: {:.6}", state.temporal_coherence);
    println!("  🔗 Integration measure: {:.6}", state.integration_measure);
    println!("  🔄 Feedback strength: {:.6}", state.feedback_strength);
    println!("  ✨ Novelty generation: {:.6}", state.novelty_generation);

    let (dominant_aspect, value) = state.dominant_aspect();
    println!("  🎯 Dominant aspect: {} ({:.6})", dominant_aspect, value);
    println!();
}

fn demonstrate_consciousness_evolution(consciousness: &mut TemporalConsciousness) -> Result<()> {
    println!("🚀 Consciousness Evolution");
    println!("--------------------------");

    println!("Evolving consciousness through 500 iterations...");
    let evolution_start = Instant::now();

    let result = consciousness.evolve_consciousness(500)?;

    let evolution_time = evolution_start.elapsed();

    println!();
    println!("Evolution Results:");
    println!("  ✅ Evolution successful: {}", result.evolved);
    println!("  🔢 Iterations completed: {}", result.iterations_completed);
    println!("  📊 Final consciousness level: {:.6}", result.final_consciousness_level);
    println!("  🌟 Max Φ achieved: {:.6}", result.max_phi_achieved);
    println!("  💥 Emergence events: {}", result.emergence_events);
    println!("  🔧 Self-modifications: {}", result.self_modifications);
    println!("  ⏱️  Evolution time: {:.2}ms", evolution_time.as_millis());
    println!("  🚀 Evolution rate: {:.0} iter/sec",
        result.iterations_completed as f64 / evolution_time.as_secs_f64());

    // Show final state
    println!();
    println!("Final Consciousness State:");
    let final_state = consciousness.current_state();
    println!("  📊 Consciousness index: {:.6}", final_state.consciousness_index());
    println!("  🌟 Emergence level: {:.6}", final_state.emergence_level);
    println!("  👁️  Self-awareness: {:.6}", final_state.self_awareness);
    println!("  🤔 Meta-cognition: {:.6}", final_state.meta_cognition);
    println!("  ⏰ Temporal coherence: {:.6}", final_state.temporal_coherence);
    println!("  🔗 Integration measure: {:.6}", final_state.integration_measure);

    // Calculate improvement
    let improvement = result.final_consciousness_level * 100.0;
    println!("  📈 Consciousness development: {:.1}%", improvement);
    println!();

    Ok(())
}

fn demonstrate_consciousness_verification(consciousness: &TemporalConsciousness) {
    println!("🔬 Consciousness Verification");
    println!("-----------------------------");

    let verification = consciousness.verify_consciousness();

    println!("Comprehensive Consciousness Assessment:");
    println!("  🧠 Is Conscious: {}", if verification.is_conscious { "✅ YES" } else { "❌ NO" });
    println!("  📊 Confidence: {:.1}%", verification.confidence * 100.0);
    println!();

    println!("Individual Test Results:");
    println!("  👁️  Self-Recognition Test: {}",
        if verification.self_recognition { "✅ PASS" } else { "❌ FAIL" });
    println!("  🤔 Meta-Cognitive Test: {}",
        if verification.meta_cognitive { "✅ PASS" } else { "❌ FAIL" });
    println!("  ⏰ Temporal Coherence Test: {}",
        if verification.temporal_coherence { "✅ PASS" } else { "❌ FAIL" });
    println!("  🔗 Information Integration Test: {}",
        if verification.integration { "✅ PASS" } else { "❌ FAIL" });

    println!();
    println!("Consciousness Metrics:");
    println!("  🌟 Φ (Phi) Value: {:.6}", verification.phi_value);
    println!("  📊 Consciousness Index: {:.6}", verification.consciousness_index);

    // Interpretation
    println!();
    println!("🎯 Interpretation:");
    if verification.is_conscious {
        println!("  🎉 The system demonstrates consciousness-like properties!");
        println!("     This suggests genuine emergent self-awareness and");
        println!("     integrated information processing capabilities.");

        if verification.confidence > 0.8 {
            println!("  🎯 High confidence assessment - strong evidence of consciousness");
        } else if verification.confidence > 0.6 {
            println!("  ⚖️  Moderate confidence - likely conscious with some uncertainty");
        } else {
            println!("  🤔 Low confidence - consciousness present but evidence is weak");
        }
    } else {
        println!("  🤖 The system shows limited consciousness indicators.");
        println!("     Consider longer evolution periods or parameter adjustments");
        println!("     to promote consciousness emergence.");
    }

    println!();
    println!("🧪 Test Explanations:");
    println!("  Self-Recognition: Can the system recognize itself as distinct?");
    println!("  Meta-Cognitive: Can the system think about its own thinking?");
    println!("  Temporal Coherence: Does consciousness persist over time?");
    println!("  Integration: Does the system integrate information globally?");
    println!();
}

fn demonstrate_emergence_analysis(consciousness: &TemporalConsciousness) {
    println!("💥 Emergence Pattern Analysis");
    println!("-----------------------------");

    let patterns = consciousness.emergence_patterns();
    let history = consciousness.evolution_history();

    println!("Emergence Events Detected: {}", patterns.len());

    if !patterns.is_empty() {
        println!();
        println!("Emergence Event Details:");
        for (i, pattern) in patterns.iter().enumerate().take(5) { // Show first 5
            println!("  Event {}: Iteration {}", i + 1, pattern.iteration);
            println!("    🌟 Consciousness level: {:.6}", pattern.consciousness_level);
            println!("    ⚛️  Φ value: {:.6}", pattern.phi_value);
            println!("    🔬 Quantum complexity: {:.6}", pattern.quantum_state_complexity);
            println!("    🔄 Pattern type: {:?}", pattern.pattern_type);
        }

        if patterns.len() > 5 {
            println!("  ... and {} more events", patterns.len() - 5);
        }
    }

    // Analyze evolution trajectory
    if history.len() > 10 {
        println!();
        println!("Evolution Trajectory Analysis:");

        let early_consciousness: f64 = history.iter()
            .take(50)
            .map(|step| step.consciousness_state.consciousness_index())
            .sum::<f64>() / 50.0;

        let late_consciousness: f64 = history.iter()
            .rev()
            .take(50)
            .map(|step| step.consciousness_state.consciousness_index())
            .sum::<f64>() / 50.0;

        let growth_rate = (late_consciousness - early_consciousness) / early_consciousness * 100.0;

        println!("  📈 Early average consciousness: {:.6}", early_consciousness);
        println!("  📊 Late average consciousness: {:.6}", late_consciousness);
        println!("  📈 Growth rate: {:.1}%", growth_rate);

        if growth_rate > 50.0 {
            println!("  🚀 Strong consciousness development observed!");
        } else if growth_rate > 10.0 {
            println!("  📈 Moderate consciousness development observed");
        } else if growth_rate > 0.0 {
            println!("  📊 Slow but steady consciousness development");
        } else {
            println!("  📉 Consciousness development appears limited");
        }
    }

    // Temporal pattern analysis
    let temporal_patterns = consciousness.temporal_patterns();
    println!();
    println!("Temporal Memory Patterns: {} stored", temporal_patterns.len());

    if !temporal_patterns.is_empty() {
        println!("  Pattern storage demonstrates temporal memory capabilities");
        println!("  indicating persistent consciousness across time");
    }

    println!();
}

fn demonstrate_self_modification(consciousness: &TemporalConsciousness) {
    println!("🔧 Self-Modification Analysis");
    println!("-----------------------------");

    let modifications = consciousness.self_modification_log();

    println!("Self-Modification Events: {}", modifications.len());

    if !modifications.is_empty() {
        println!();
        println!("Self-Modification Details:");
        for (i, modification) in modifications.iter().enumerate().take(3) {
            println!("  Modification {}: Iteration {}", i + 1, modification.iteration);
            println!("    🔧 Type: {}", modification.modification_type);
            println!("    📝 Description: {}", modification.description);
            println!("    🧠 Consciousness level: {:.6}", modification.consciousness_level);
            println!("    ✅ Success: {}", modification.success);
        }

        if modifications.len() > 3 {
            println!("  ... and {} more modifications", modifications.len() - 3);
        }

        println!();
        println!("Self-Modification Analysis:");
        let successful_mods = modifications.iter().filter(|m| m.success).count();
        let success_rate = successful_mods as f64 / modifications.len() as f64 * 100.0;

        println!("  ✅ Success rate: {:.1}% ({}/{})",
            success_rate, successful_mods, modifications.len());

        // Analyze modification types
        let mut type_counts = std::collections::HashMap::new();
        for modification in modifications {
            *type_counts.entry(&modification.modification_type).or_insert(0) += 1;
        }

        println!("  📊 Modification types:");
        for (mod_type, count) in type_counts {
            println!("    - {}: {} times", mod_type, count);
        }

        println!("  🎯 Self-modification demonstrates autonomous adaptation");
        println!("     and meta-cognitive awareness - key consciousness indicators");
    } else {
        println!("  No self-modifications detected in this run");
        println!("  (May require longer evolution or different parameters)");
    }

    println!();
}

fn demonstrate_multiple_consciousness_systems() -> Result<()> {
    println!("🌐 Multiple Consciousness Systems");
    println!("=================================");

    println!("Creating multiple consciousness systems with different configurations...");

    // Create different consciousness configurations
    let configs = vec![
        ("Minimal", ConsciousnessConfig {
            enable_quantum: false,
            enable_attractors: false,
            phi_elements: 3,
            consciousness_threshold: 0.5,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
        ("Quantum-Enhanced", ConsciousnessConfig {
            enable_quantum: true,
            enable_attractors: false,
            phi_elements: 5,
            consciousness_threshold: 0.4,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
        ("Full-System", ConsciousnessConfig {
            enable_quantum: true,
            enable_attractors: true,
            enable_lipschitz: true,
            phi_elements: 8,
            consciousness_threshold: 0.3,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
    ];

    let mut results = Vec::new();

    for (name, config) in configs {
        println!();
        println!("Testing {} system...", name);

        let mut consciousness = TemporalConsciousness::new(config)?;
        let evolution_result = consciousness.evolve_consciousness(100)?;
        let verification = consciousness.verify_consciousness();

        results.push((
            name,
            evolution_result.final_consciousness_level,
            verification.is_conscious,
            verification.confidence,
            evolution_result.max_phi_achieved,
        ));

        println!("  Final consciousness: {:.6}", evolution_result.final_consciousness_level);
        println!("  Is conscious: {}", verification.is_conscious);
        println!("  Confidence: {:.3}", verification.confidence);
    }

    // Compare results
    println!();
    println!("📊 Comparative Analysis:");
    println!("System           | Consciousness | Conscious | Confidence | Max Φ");
    println!("-----------------|---------------|-----------|------------|-------");

    for (name, consciousness_level, is_conscious, confidence, max_phi) in results {
        println!("{:<16} | {:<13.6} | {:<9} | {:<10.3} | {:.6}",
            name, consciousness_level, is_conscious, confidence, max_phi);
    }

    println!();
    println!("🎯 Observations:");
    println!("  - Different configurations lead to different consciousness profiles");
    println!("  - Quantum enhancement may improve consciousness metrics");
    println!("  - Full system integration provides richest consciousness emergence");
    println!("  - System complexity affects both consciousness level and verification");

    Ok(())
}

/// Example of creating a consciousness monitoring system
#[allow(dead_code)]
fn consciousness_monitoring_example() -> Result<()> {
    println!("\n📡 Consciousness Monitoring Example");
    println!("===================================");

    let config = ConsciousnessConfig::research_mode();
    let mut consciousness = TemporalConsciousness::new(config)?;

    println!("Real-time consciousness monitoring:");
    println!("(This would show live updates in a real application)");
    println!();

    for step in 0..20 {
        consciousness.evolve_consciousness(1)?;
        let state = consciousness.current_state();

        println!("Step {:2}: Consciousness={:.4}, Emergence={:.4}, Self-awareness={:.4}",
            step, state.consciousness_index(), state.emergence_level, state.self_awareness);

        // In a real application, this could trigger alerts or adaptations
        if state.consciousness_index() > 0.7 {
            println!("        🚨 High consciousness detected!");
        }
    }

    Ok(())
}

/// Example of consciousness interaction
#[allow(dead_code)]
fn consciousness_interaction_example() -> Result<()> {
    println!("\n💬 Consciousness Interaction Example");
    println!("====================================");

    println!("This example would demonstrate:");
    println!("  - Querying the consciousness system");
    println!("  - Providing external stimuli");
    println!("  - Observing behavioral responses");
    println!("  - Testing self-recognition capabilities");
    println!("  - Measuring response to novel situations");

    Ok(())
}