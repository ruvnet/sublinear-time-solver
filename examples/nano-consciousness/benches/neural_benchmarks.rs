use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nano_consciousness::neural::{ConsciousnessNetwork, ActivationFunction, architectures};
use ndarray::Array1;

fn benchmark_neural_forward_pass(c: &mut Criterion) {
    let mut group = c.benchmark_group("neural_forward_pass");
    group.sample_size(100);

    let network_sizes = [
        (8, 16, 8),
        (16, 32, 16),
        (32, 64, 32),
        (64, 128, 64),
    ];

    for &(input_size, hidden_size, output_size) in network_sizes.iter() {
        let mut network = architectures::simple_consciousness_net(input_size, hidden_size, output_size);
        let input = Array1::from((0..input_size).map(|i| (i as f64 / input_size as f64).sin()).collect::<Vec<_>>());

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("forward_pass", format!("{}x{}x{}", input_size, hidden_size, output_size)),
            &input,
            |b, input| {
                b.iter(|| {
                    let output = network.forward(black_box(input));
                    black_box(output)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_phi_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phi_calculation");
    group.sample_size(50);

    let network_sizes = [16, 32, 64, 128];

    for &size in network_sizes.iter() {
        let mut network = architectures::iit_inspired_net(size);
        let input = Array1::from((0..size).map(|i| (i as f64 / size as f64).sin()).collect::<Vec<_>>());

        // Establish network state
        network.forward(&input);

        group.bench_with_input(
            BenchmarkId::new("phi_calculation", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let phi = network.calculate_phi();
                    black_box(phi)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_strange_loop_dynamics(c: &mut Criterion) {
    let mut group = c.benchmark_group("strange_loop_dynamics");
    group.sample_size(30);

    let mut network = architectures::simple_consciousness_net(16, 32, 16);
    let input = Array1::from((0..16).map(|i| (i as f64 / 16.0).sin()).collect::<Vec<_>>());

    let depths = [1, 2, 3, 5, 10];

    for &depth in depths.iter() {
        group.bench_with_input(
            BenchmarkId::new("strange_loop_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let output = network.strange_loop_dynamics(black_box(&input), black_box(depth));
                    black_box(output)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_global_workspace_activation(c: &mut Criterion) {
    let mut group = c.benchmark_group("global_workspace_activation");
    group.sample_size(100);

    let workspace_sizes = [16, 32, 64, 128];

    for &workspace_size in workspace_sizes.iter() {
        let mut network = architectures::global_workspace_net(16, workspace_size, 8);
        let input = Array1::from((0..16).map(|i| (i as f64 / 16.0).sin()).collect::<Vec<_>>());

        group.bench_with_input(
            BenchmarkId::new("workspace_size", workspace_size),
            &input,
            |b, input| {
                b.iter(|| {
                    let activation = network.global_workspace_activation(black_box(input));
                    black_box(activation)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_attention_mechanism(c: &mut Criterion) {
    let mut group = c.benchmark_group("attention_mechanism");
    group.sample_size(100);

    let mut network = architectures::simple_consciousness_net(16, 32, 16);
    let input = Array1::from((0..16).map(|i| (i as f64 / 16.0).sin()).collect::<Vec<_>>());

    let attention_patterns = [
        ("uniform", Array1::ones(16)),
        ("focused", {
            let mut attn = Array1::zeros(16);
            attn[0] = 1.0;
            attn[1] = 1.0;
            attn
        }),
        ("gaussian", Array1::from((0..16).map(|i| {
            let center = 8.0;
            let sigma = 2.0;
            (-(i as f64 - center).powi(2) / (2.0 * sigma.powi(2))).exp()
        }).collect::<Vec<_>>())),
    ];

    for (name, attention) in attention_patterns.iter() {
        group.bench_with_input(
            BenchmarkId::new("attention_pattern", name),
            &(&input, attention),
            |b, (input, attention)| {
                b.iter(|| {
                    let output = network.attention_mechanism(black_box(input), black_box(attention));
                    black_box(output)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_network_training(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_training");
    group.sample_size(20);

    let mut network = architectures::simple_consciousness_net(16, 32, 16);
    let input = Array1::from((0..16).map(|i| (i as f64 / 16.0).sin()).collect::<Vec<_>>());
    let target = Array1::from((0..16).map(|i| (i as f64 / 16.0).cos()).collect::<Vec<_>>());

    group.bench_function("single_training_step", |b| {
        b.iter(|| {
            let loss = network.train(black_box(&input), black_box(&target));
            black_box(loss)
        })
    });

    // Batch training benchmark
    let batch_size = 10;
    let inputs: Vec<Array1<f64>> = (0..batch_size)
        .map(|i| Array1::from((0..16).map(|j| ((i + j) as f64 / 16.0).sin()).collect::<Vec<_>>()))
        .collect();
    let targets: Vec<Array1<f64>> = (0..batch_size)
        .map(|i| Array1::from((0..16).map(|j| ((i + j) as f64 / 16.0).cos()).collect::<Vec<_>>()))
        .collect();

    group.throughput(Throughput::Elements(batch_size as u64));
    group.bench_function("batch_training", |b| {
        b.iter(|| {
            let loss = network.train_batch(black_box(&inputs), black_box(&targets));
            black_box(loss)
        })
    });

    group.finish();
}

fn benchmark_activation_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("activation_functions");
    group.sample_size(1000);

    let test_values = Array1::from(vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0]);

    let activations = [
        ActivationFunction::Sigmoid,
        ActivationFunction::Tanh,
        ActivationFunction::ReLU,
        ActivationFunction::LeakyReLU(0.1),
        ActivationFunction::Linear,
    ];

    for activation in activations.iter() {
        group.bench_with_input(
            BenchmarkId::new("activation", format!("{:?}", activation)),
            &test_values,
            |b, values| {
                b.iter(|| {
                    let results: Vec<f64> = values.iter()
                        .map(|&x| activation.apply(black_box(x)))
                        .collect();
                    black_box(results)
                })
            },
        );
    }

    // Softmax benchmark (different from element-wise activations)
    let softmax_input = Array1::from((0..10).map(|i| i as f64).collect::<Vec<_>>());
    group.bench_function("softmax", |b| {
        b.iter(|| {
            let output = ActivationFunction::softmax(black_box(&softmax_input));
            black_box(output)
        })
    });

    group.finish();
}

fn benchmark_network_mutation(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_mutation");
    group.sample_size(50);

    let mutation_rates = [0.01, 0.05, 0.1, 0.2];
    let mutation_strengths = [0.001, 0.01, 0.1];

    for &rate in mutation_rates.iter() {
        for &strength in mutation_strengths.iter() {
            let mut network = architectures::simple_consciousness_net(16, 32, 16);

            group.bench_with_input(
                BenchmarkId::new("mutation", format!("rate_{}_strength_{}", rate, strength)),
                &(rate, strength),
                |b, &(rate, strength)| {
                    b.iter(|| {
                        network.mutate(black_box(rate), black_box(strength));
                    })
                },
            );
        }
    }

    group.finish();
}

fn benchmark_network_architectures(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_architectures");
    group.sample_size(50);

    let input = Array1::from((0..16).map(|i| (i as f64 / 16.0).sin()).collect::<Vec<_>>());

    let architectures_to_test = [
        ("simple_consciousness", || architectures::simple_consciousness_net(16, 32, 8)),
        ("global_workspace", || architectures::global_workspace_net(16, 64, 8)),
        ("iit_inspired", || architectures::iit_inspired_net(16)),
    ];

    for (name, create_network) in architectures_to_test.iter() {
        let mut network = create_network();

        group.bench_with_input(
            BenchmarkId::new("architecture", name),
            &input,
            |b, input| {
                b.iter(|| {
                    let output = network.forward(black_box(input));
                    black_box(output)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_large_networks(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_networks");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    let large_configs = [
        (64, 128, 64),
        (128, 256, 128),
        (256, 512, 256),
    ];

    for &(input_size, hidden_size, output_size) in large_configs.iter() {
        let mut network = architectures::simple_consciousness_net(input_size, hidden_size, output_size);
        let input = Array1::from((0..input_size).map(|i| (i as f64 / input_size as f64).sin()).collect::<Vec<_>>());

        group.throughput(Throughput::Elements(input_size as u64));
        group.bench_with_input(
            BenchmarkId::new("large_network", format!("{}x{}x{}", input_size, hidden_size, output_size)),
            &input,
            |b, input| {
                b.iter(|| {
                    let output = network.forward(black_box(input));
                    let phi = network.calculate_phi();
                    black_box((output, phi))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_neural_forward_pass,
    benchmark_phi_calculation,
    benchmark_strange_loop_dynamics,
    benchmark_global_workspace_activation,
    benchmark_attention_mechanism,
    benchmark_network_training,
    benchmark_activation_functions,
    benchmark_network_mutation,
    benchmark_network_architectures,
    benchmark_large_networks
);

criterion_main!(benches);