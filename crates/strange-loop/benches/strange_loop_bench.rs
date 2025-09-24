//! Benchmarks for strange-loop crate

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use strange_loop::{
    consciousness::ConsciousnessMetrics,
    lipschitz_loop::{LipschitzLoop, LipschitzParams, LoopTopology},
    quantum_container::QuantumContainer,
    strange_attractor::{TemporalAttractor, AttractorConfig, AttractorType},
    temporal_consciousness::{TemporalConsciousness, ConsciousnessConfig},
    types::{StrangeLoop, LoopConfig, ScalarReasoner, SimpleCritic, SafeReflector},
};
use nalgebra::Vector3;
use std::collections::HashMap;

fn bench_strange_loop_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("strange_loop_convergence");

    for iterations in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("scalar_reasoner", iterations),
            iterations,
            |b, &iterations| {
                b.iter(|| {
                    let reasoner = ScalarReasoner::new(0.0, 0.1);
                    let critic = SimpleCritic::new();
                    let reflector = SafeReflector::new();

                    let config = LoopConfig {
                        max_iterations: iterations,
                        max_duration_ns: 1_000_000_000, // 1 second
                        convergence_threshold: 1e-9,
                        lipschitz_constant: 0.9,
                        enable_consciousness: false,
                        enable_quantum: false,
                        enable_simd: true,
                    };

                    let mut strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);
                    let mut context = HashMap::from([("x".to_string(), black_box(10.0))]);

                    black_box(strange_loop.run(&mut context))
                });
            },
        );
    }
    group.finish();
}

fn bench_attractor_dynamics(c: &mut Criterion) {
    let mut group = c.benchmark_group("attractor_dynamics");

    let attractors = [
        ("lorenz", AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 }),
        ("rossler", AttractorType::Rossler { a: 0.2, b: 0.2, c: 5.7 }),
        ("chua", AttractorType::Chua { alpha: 15.6, beta: -1.143, gamma: -0.714 }),
    ];

    for (name, attractor_type) in attractors.iter() {
        for steps in [100, 500, 1000, 5000].iter() {
            group.throughput(Throughput::Elements(*steps as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_fixed_step", name), steps),
                steps,
                |b, &steps| {
                    let config = AttractorConfig {
                        attractor_type: attractor_type.clone(),
                        dt_ns: 1000,
                        steps_per_frame: 1,
                        adaptive_stepping: false,
                        tolerance: 1e-6,
                        max_deviation: 50.0,
                    };

                    b.iter(|| {
                        let mut attractor = TemporalAttractor::new(config.clone()).unwrap();
                        for _ in 0..steps {
                            black_box(attractor.step().unwrap());
                        }
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(format!("{}_adaptive_step", name), steps),
                steps,
                |b, &steps| {
                    let config = AttractorConfig {
                        attractor_type: attractor_type.clone(),
                        dt_ns: 1000,
                        steps_per_frame: 1,
                        adaptive_stepping: true,
                        tolerance: 1e-9,
                        max_deviation: 50.0,
                    };

                    b.iter(|| {
                        let mut attractor = TemporalAttractor::new(config.clone()).unwrap();
                        for _ in 0..steps {
                            black_box(attractor.step().unwrap());
                        }
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_quantum_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantum_operations");

    for qubits in [2, 3, 4, 5].iter() {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("measurements", qubits),
            qubits,
            |b, &qubits| {
                b.iter(|| {
                    let mut quantum = QuantumContainer::new(qubits);

                    // Create uniform superposition
                    let num_states = 1 << qubits;
                    let probabilities = vec![1.0 / num_states as f64; num_states];
                    quantum.create_superposition_from_classical(&probabilities).unwrap();

                    // Perform measurements
                    for _ in 0..1000 {
                        black_box(quantum.measure());
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("gate_operations", qubits),
            qubits,
            |b, &qubits| {
                use strange_loop::quantum_container::Gate;

                b.iter(|| {
                    let mut quantum = QuantumContainer::new(qubits);

                    // Apply gates to all qubits
                    for qubit in 0..qubits {
                        black_box(quantum.apply_gate(qubit, Gate::H).unwrap());
                        black_box(quantum.apply_gate(qubit, Gate::RZ(0.5)).unwrap());
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_lipschitz_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("lipschitz_loops");

    let topologies = [
        ("fixed_point", LoopTopology::FixedPoint),
        ("newton", LoopTopology::Newton),
        ("accelerated", LoopTopology::Accelerated),
        ("conjugate_gradient", LoopTopology::ConjugateGradient),
    ];

    for (name, topology) in topologies.iter() {
        group.bench_function(name, |b| {
            let params = LipschitzParams {
                lipschitz_constant: 0.8,
                tolerance: 1e-9,
                max_iterations: 1000,
                adaptive_estimation: true,
                damping: 0.99,
            };

            b.iter(|| {
                let mut loop_solver = LipschitzLoop::new(params.clone(), topology.clone()).unwrap();

                // Simple contractive function
                let function = |x: Vector3<f64>| black_box(0.7 * x);
                let initial_state = Vector3::new(10.0, 10.0, 10.0);

                black_box(loop_solver.execute(function, initial_state))
            });
        });
    }
    group.finish();
}

fn bench_consciousness_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("consciousness_evolution");

    let configs = [
        ("minimal", ConsciousnessConfig {
            enable_quantum: false,
            enable_attractors: false,
            enable_lipschitz: false,
            enable_self_modification: false,
            phi_elements: 2,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
        ("quantum_only", ConsciousnessConfig {
            enable_quantum: true,
            enable_attractors: false,
            enable_lipschitz: false,
            enable_self_modification: false,
            phi_elements: 4,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
        ("full_system", ConsciousnessConfig {
            enable_quantum: true,
            enable_attractors: true,
            enable_lipschitz: true,
            enable_self_modification: false, // Disable for consistent benchmarking
            phi_elements: 6,
            max_evolution_iterations: 100,
            ..ConsciousnessConfig::default()
        }),
    ];

    for (name, config) in configs.iter() {
        for iterations in [10, 25, 50, 100].iter() {
            group.throughput(Throughput::Elements(*iterations as u64));
            group.bench_with_input(
                BenchmarkId::new(name, iterations),
                iterations,
                |b, &iterations| {
                    b.iter(|| {
                        let mut consciousness = TemporalConsciousness::new(config.clone()).unwrap();
                        black_box(consciousness.evolve_consciousness(iterations).unwrap())
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_phi_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phi_calculation");

    for elements in [4, 6, 8, 10, 12].iter() {
        group.throughput(Throughput::Elements(*elements as u64));
        group.bench_with_input(
            BenchmarkId::new("integrated_information", elements),
            elements,
            |b, &elements| {
                b.iter(|| {
                    let mut metrics = ConsciousnessMetrics::new();
                    let connections = elements * (elements - 1) / 2;
                    let coupling_strength = 0.8;

                    black_box(metrics.calculate_phi(
                        black_box(elements),
                        black_box(connections),
                        black_box(coupling_strength)
                    ))
                });
            },
        );
    }
    group.finish();
}

fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    // Benchmark temporal pattern storage
    group.bench_function("temporal_pattern_storage", |b| {
        b.iter(|| {
            let config = ConsciousnessConfig::default();
            let mut consciousness = TemporalConsciousness::new(config).unwrap();

            // Generate patterns
            for i in 0..black_box(1000) {
                consciousness.evolve_consciousness(1).unwrap();
            }

            black_box(consciousness.temporal_patterns().len())
        });
    });

    // Benchmark attractor trajectory storage
    group.bench_function("attractor_trajectory", |b| {
        b.iter(|| {
            let config = AttractorConfig::default();
            let mut attractor = TemporalAttractor::new(config).unwrap();
            attractor.set_max_trajectory_length(1000);

            for _ in 0..black_box(1000) {
                attractor.step().unwrap();
            }

            black_box(attractor.trajectory().len())
        });
    });

    group.finish();
}

fn bench_simd_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_optimizations");

    // Benchmark with and without SIMD (if available)
    group.bench_function("vector_operations_scalar", |b| {
        b.iter(|| {
            let vectors: Vec<Vector3<f64>> = (0..1000)
                .map(|i| Vector3::new(i as f64, (i*2) as f64, (i*3) as f64))
                .collect();

            let mut result = Vector3::zeros();
            for v in &vectors {
                result += v;
            }

            black_box(result)
        });
    });

    #[cfg(feature = "simd")]
    group.bench_function("vector_operations_simd", |b| {
        use wide::f64x4;

        b.iter(|| {
            let data: Vec<f64> = (0..4000)
                .map(|i| i as f64)
                .collect();

            let mut sum = f64x4::ZERO;
            for chunk in data.chunks_exact(4) {
                let vec = f64x4::new([chunk[0], chunk[1], chunk[2], chunk[3]]);
                sum += vec;
            }

            black_box(sum.to_array())
        });
    });

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    group.bench_function("parallel_consciousness_evolution", |b| {
        use rayon::prelude::*;

        b.iter(|| {
            let configs: Vec<_> = (0..4)
                .map(|_| ConsciousnessConfig {
                    max_evolution_iterations: 25,
                    ..ConsciousnessConfig::default()
                })
                .collect();

            let results: Vec<_> = configs.into_par_iter()
                .map(|config| {
                    let mut consciousness = TemporalConsciousness::new(config).unwrap();
                    consciousness.evolve_consciousness(25).unwrap()
                })
                .collect();

            black_box(results)
        });
    });

    group.bench_function("parallel_attractor_computation", |b| {
        use rayon::prelude::*;

        b.iter(|| {
            let configs: Vec<_> = (0..4)
                .map(|_| AttractorConfig::default())
                .collect();

            let results: Vec<_> = configs.into_par_iter()
                .map(|config| {
                    let mut attractor = TemporalAttractor::new(config).unwrap();
                    for _ in 0..100 {
                        attractor.step().unwrap();
                    }
                    attractor.state()
                })
                .collect();

            black_box(results)
        });
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_creation_and_propagation", |b| {
        use strange_loop::error::LoopError;

        b.iter(|| {
            // Test error creation performance
            let errors = vec![
                LoopError::convergence_failure(1000),
                LoopError::timeout(1_000_000),
                LoopError::lipschitz_violation(1.2, 1.0),
                LoopError::invalid_policy("test error".to_string()),
                LoopError::quantum_error("quantum test".to_string()),
            ];

            // Test error checking
            for error in &errors {
                black_box(error.is_recoverable());
                black_box(error.is_fatal());
            }

            black_box(errors)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_strange_loop_convergence,
    bench_attractor_dynamics,
    bench_quantum_operations,
    bench_lipschitz_loops,
    bench_consciousness_evolution,
    bench_phi_calculation,
    bench_memory_operations,
    bench_simd_optimizations,
    bench_concurrent_operations,
    bench_error_handling,
);

criterion_main!(benches);