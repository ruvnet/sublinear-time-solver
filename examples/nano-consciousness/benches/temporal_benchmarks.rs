use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nano_consciousness::temporal::{TemporalProcessor, NeuralState, TemporalPattern};
use ndarray::Array1;
use std::time::Duration;

fn benchmark_temporal_window_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_window_processing");
    group.sample_size(50);

    let window_sizes = [
        Duration::from_millis(50),
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(500),
    ];

    for &window_size in window_sizes.iter() {
        let mut processor = TemporalProcessor::new(
            window_size,
            0.5,
            50,
            0.3,
        );

        let test_state = NeuralState::new(
            Array1::from(vec![0.5, 0.8, 0.2, 0.9]),
            Array1::from(vec![0.3, 0.7]),
            Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
            0.6,
            0.4,
        );

        group.bench_with_input(
            BenchmarkId::new("window_size", format!("{}ms", window_size.as_millis())),
            &test_state,
            |b, state| {
                b.iter(|| {
                    processor.add_state(black_box(state.clone()));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_temporal_binding_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_binding_calculation");
    group.sample_size(100);

    let state_counts = [5, 10, 20, 50];

    for &count in state_counts.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            50,
            0.3,
        );

        // Add multiple states
        for i in 0..count {
            let state = NeuralState::new(
                Array1::from(vec![
                    (i as f64 / count as f64).sin(),
                    (i as f64 / count as f64).cos(),
                    (i as f64 / count as f64) * 0.5,
                    1.0 - (i as f64 / count as f64),
                ]),
                Array1::from(vec![0.3, 0.7]),
                Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
                0.6 + (i as f64 / count as f64) * 0.3,
                0.4 + (i as f64 / count as f64) * 0.2,
            );
            processor.add_state(state);
        }

        group.bench_with_input(
            BenchmarkId::new("state_count", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let binding = processor.calculate_temporal_binding();
                    black_box(binding)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_stream_continuity_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stream_continuity_calculation");
    group.sample_size(100);

    let continuity_patterns = [
        ("smooth", |i: usize| 0.5 + (i as f64 * 0.1).sin() * 0.1),
        ("step", |i: usize| if i % 10 < 5 { 0.3 } else { 0.7 }),
        ("random", |i: usize| 0.5 + ((i * 7) % 100) as f64 / 200.0 - 0.25),
        ("linear", |i: usize| (i as f64 / 50.0).min(1.0)),
    ];

    for (pattern_name, pattern_fn) in continuity_patterns.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            50,
            0.3,
        );

        // Generate pattern
        for i in 0..30 {
            let consciousness_level = pattern_fn(i);
            let state = NeuralState::new(
                Array1::from(vec![consciousness_level, consciousness_level * 0.8, consciousness_level * 0.6, consciousness_level * 0.4]),
                Array1::from(vec![0.3, 0.7]),
                Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
                consciousness_level,
                consciousness_level * 0.8,
            );
            processor.add_state(state);
        }

        group.bench_with_input(
            BenchmarkId::new("continuity_pattern", pattern_name),
            &pattern_name,
            |b, _| {
                b.iter(|| {
                    let continuity = processor.calculate_stream_continuity();
                    black_box(continuity)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_temporal_coherence_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_coherence_calculation");
    group.sample_size(50);

    let window_counts = [2, 5, 10, 20];

    for &count in window_counts.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(50), // Smaller windows to create more of them
            0.2, // Less overlap to create more windows
            count,
            0.3,
        );

        // Generate enough states to create multiple windows
        for i in 0..(count * 10) {
            let state = NeuralState::new(
                Array1::from(vec![0.5, 0.8, 0.2, 0.9]),
                Array1::from(vec![0.3, 0.7]),
                Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
                0.6 + (i as f64 / (count * 10) as f64) * 0.2,
                0.4,
            );
            processor.add_state(state);

            // Small delay to ensure different timestamps
            std::thread::sleep(Duration::from_nanos(100));
        }

        group.bench_with_input(
            BenchmarkId::new("window_count", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let coherence = processor.calculate_temporal_coherence();
                    black_box(coherence)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_future_state_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("future_state_prediction");
    group.sample_size(50);

    let mut processor = TemporalProcessor::new(
        Duration::from_millis(100),
        0.5,
        50,
        0.3,
    );

    // Add states with a clear trend
    for i in 0..10 {
        let trend_value = i as f64 / 10.0;
        let state = NeuralState::new(
            Array1::from(vec![trend_value, trend_value * 0.8, trend_value * 0.6, trend_value * 0.4]),
            Array1::from(vec![0.3, 0.7]),
            Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
            trend_value,
            trend_value * 0.8,
        );
        processor.add_state(state);
    }

    let prediction_horizons = [
        Duration::from_millis(10),
        Duration::from_millis(50),
        Duration::from_millis(100),
        Duration::from_millis(500),
    ];

    for &horizon in prediction_horizons.iter() {
        group.bench_with_input(
            BenchmarkId::new("prediction_horizon", format!("{}ms", horizon.as_millis())),
            &horizon,
            |b, &horizon| {
                b.iter(|| {
                    let prediction = processor.predict_future_state(black_box(horizon));
                    black_box(prediction)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_pattern_recognition(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_recognition");
    group.sample_size(100);

    let pattern_types = [
        ("oscillatory", |i: usize| (i as f64 * 0.5).sin() * 0.5 + 0.5),
        ("trending", |i: usize| (i as f64 / 20.0).min(1.0)),
        ("stable", |_: usize| 0.5),
        ("chaotic", |i: usize| ((i * 7 + i * i) % 100) as f64 / 100.0),
        ("periodic", |i: usize| if i % 5 == 0 { 1.0 } else { 0.2 }),
    ];

    for (pattern_name, pattern_fn) in pattern_types.iter() {
        let sequence: Vec<f64> = (0..20).map(|i| pattern_fn(i)).collect();

        group.bench_with_input(
            BenchmarkId::new("pattern_type", pattern_name),
            &sequence,
            |b, sequence| {
                b.iter(|| {
                    let pattern = TemporalPattern::from_sequence(black_box(sequence));
                    black_box(pattern)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_neural_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("neural_state_operations");
    group.sample_size(200);

    let state1 = NeuralState::new(
        Array1::from(vec![0.5, 0.8, 0.2, 0.9, 0.1, 0.7, 0.3, 0.6]),
        Array1::from(vec![0.3, 0.7]),
        Array1::from(vec![1.0, 0.5, 0.8, 0.2, 0.4, 0.9, 0.1, 0.6]),
        0.6,
        0.4,
    );

    let state2 = NeuralState::new(
        Array1::from(vec![0.4, 0.7, 0.3, 0.8, 0.2, 0.6, 0.4, 0.5]),
        Array1::from(vec![0.4, 0.6]),
        Array1::from(vec![0.9, 0.6, 0.7, 0.3, 0.5, 0.8, 0.2, 0.5]),
        0.7,
        0.5,
    );

    group.bench_function("similarity_calculation", |b| {
        b.iter(|| {
            let similarity = state1.similarity(black_box(&state2));
            black_box(similarity)
        })
    });

    group.bench_function("temporal_distance", |b| {
        b.iter(|| {
            let distance = state1.temporal_distance(black_box(&state2));
            black_box(distance)
        })
    });

    group.finish();
}

fn benchmark_temporal_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_cleanup");
    group.sample_size(20);

    let state_counts = [100, 500, 1000, 5000];

    for &count in state_counts.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            50,
            0.3,
        );

        // Add many states
        for i in 0..count {
            let state = NeuralState::new(
                Array1::from(vec![0.5, 0.8, 0.2, 0.9]),
                Array1::from(vec![0.3, 0.7]),
                Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
                0.6,
                0.4,
            );
            processor.add_state(state);

            if i % 100 == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }

        let cutoff_time = nano_consciousness::scheduler::NanoTimestamp::now();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("cleanup_state_count", count),
            &cutoff_time,
            |b, &cutoff_time| {
                b.iter(|| {
                    processor.cleanup_old_data(black_box(cutoff_time));
                })
            },
        );
    }

    group.finish();
}

fn benchmark_window_overlap_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_overlap_ratios");
    group.sample_size(50);

    let overlap_ratios = [0.0, 0.25, 0.5, 0.75, 0.9];

    for &ratio in overlap_ratios.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            ratio,
            20,
            0.3,
        );

        let test_state = NeuralState::new(
            Array1::from(vec![0.5, 0.8, 0.2, 0.9]),
            Array1::from(vec![0.3, 0.7]),
            Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
            0.6,
            0.4,
        );

        group.bench_with_input(
            BenchmarkId::new("overlap_ratio", format!("{:.1}", ratio)),
            &test_state,
            |b, state| {
                b.iter(|| {
                    for _ in 0..10 {
                        processor.add_state(black_box(state.clone()));
                        std::thread::sleep(Duration::from_millis(5));
                    }
                })
            },
        );
    }

    group.finish();
}

fn benchmark_large_temporal_datasets(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_temporal_datasets");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let dataset_sizes = [1000, 5000, 10000];

    for &size in dataset_sizes.iter() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            100,
            0.3,
        );

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("dataset_size", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let state = NeuralState::new(
                            Array1::from(vec![
                                (i as f64 / size as f64).sin(),
                                (i as f64 / size as f64).cos(),
                                (i as f64 / size as f64) * 0.5,
                                1.0 - (i as f64 / size as f64),
                            ]),
                            Array1::from(vec![0.3, 0.7]),
                            Array1::from(vec![1.0, 0.5, 0.8, 0.2]),
                            0.5 + (i as f64 / size as f64) * 0.3,
                            0.4 + (i as f64 / size as f64) * 0.2,
                        );
                        processor.add_state(black_box(state));

                        // Occasional sleep to simulate real-time processing
                        if i % 100 == 0 {
                            std::thread::sleep(Duration::from_nanos(1000));
                        }
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_temporal_window_processing,
    benchmark_temporal_binding_calculation,
    benchmark_stream_continuity_calculation,
    benchmark_temporal_coherence_calculation,
    benchmark_future_state_prediction,
    benchmark_pattern_recognition,
    benchmark_neural_state_operations,
    benchmark_temporal_cleanup,
    benchmark_window_overlap_ratios,
    benchmark_large_temporal_datasets
);

criterion_main!(benches);