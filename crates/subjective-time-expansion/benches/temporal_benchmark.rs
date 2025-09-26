//! Performance benchmarks for the Subjective Time Expansion framework
//!
//! Comprehensive benchmarking suite testing scheduler performance, consciousness
//! measurement speed, and overall framework throughput.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;

use subjective_time_expansion::prelude::*;
use subjective_time_expansion::scheduler::{TemporalTask, TaskPriority};

/// Benchmark scheduler tick performance
fn bench_scheduler_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scheduler_performance");
    group.sample_size(100);

    let tick_durations = vec![
        Duration::from_nanos(10_000),  // 100kHz
        Duration::from_nanos(25_000),  // 40kHz
        Duration::from_nanos(50_000),  // 20kHz
        Duration::from_nanos(100_000), // 10kHz
    ];

    for tick_duration in tick_durations {
        let frequency = 1_000_000_000 / tick_duration.as_nanos() as u64;

        group.bench_with_input(
            BenchmarkId::new("tick_processing", format!("{}kHz", frequency / 1000)),
            &tick_duration,
            |b, &duration| {
                b.iter(|| {
                    rt.block_on(async {
                        let scheduler = TemporalScheduler::new(
                            SchedulerConfig::default()
                                .with_base_tick_duration(duration)
                                .with_max_agents(100)
                        );

                        // Simulate scheduler processing
                        black_box(scheduler);
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark agent spawning performance
fn bench_agent_spawning(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_spawning");
    group.sample_size(50);

    let agent_counts = vec![1, 10, 50, 100];

    for agent_count in agent_counts {
        group.bench_with_input(
            BenchmarkId::new("spawn_agents", agent_count),
            &agent_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let scheduler = TemporalScheduler::new(
                            SchedulerConfig::default()
                                .with_base_tick_duration(Duration::from_nanos(25_000))
                                .with_max_agents(count as usize)
                        );

                        let mut agents = Vec::new();
                        for i in 0..count {
                            let agent = scheduler.spawn_agent(
                                AgentConfig::new(format!("bench-agent-{}", i))
                                    .with_pattern(CognitivePattern::CreativeSynthesis)
                                    .with_dilation_factor(2.0)
                            ).await.unwrap();
                            agents.push(agent);
                        }

                        black_box(agents);
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consciousness measurement (Phi calculation)
fn bench_phi_measurement(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("phi_measurement");
    group.sample_size(200);

    let matrix_sizes = vec![4, 8, 16, 32];

    for size in matrix_sizes {
        group.bench_with_input(
            BenchmarkId::new("phi_calculation", format!("{}x{}", size, size)),
            &size,
            |b, &matrix_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let scheduler = TemporalScheduler::new(
                            SchedulerConfig::default()
                        );

                        let agent = scheduler.spawn_agent(
                            AgentConfig::new("phi-test-agent".to_string())
                                .with_pattern(CognitivePattern::SystemsThinking)
                        ).await.unwrap();

                        // Measure Phi
                        let phi = agent.measure_phi().await.unwrap();
                        black_box(phi);
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark task execution with different cognitive patterns
fn bench_task_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("task_execution");
    group.sample_size(100);

    let patterns = vec![
        CognitivePattern::CreativeSynthesis,
        CognitivePattern::SystemsThinking,
        CognitivePattern::ConvergentReasoning,
        CognitivePattern::DivergentThinking,
    ];

    for pattern in patterns {
        group.bench_with_input(
            BenchmarkId::new("execute_task", format!("{:?}", pattern)),
            &pattern,
            |b, &cognitive_pattern| {
                b.iter(|| {
                    rt.block_on(async {
                        let scheduler = TemporalScheduler::new(
                            SchedulerConfig::default()
                        );

                        let agent = scheduler.spawn_agent(
                            AgentConfig::new("task-bench-agent".to_string())
                                .with_pattern(cognitive_pattern)
                                .with_dilation_factor(2.0)
                        ).await.unwrap();

                        let task = TemporalTask {
                            id: "benchmark-task".to_string(),
                            agent_id: agent.id().to_string(),
                            scheduled_ns: 0,
                            subjective_duration_ns: Duration::from_millis(1).as_nanos() as u64,
                            priority: TaskPriority::Normal,
                            cognitive_pattern,
                            payload: serde_json::json!({
                                "benchmark": true,
                                "complexity": 0.5
                            }),
                        };

                        let result = agent.execute_task(task).await.unwrap();
                        black_box(result);
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark retrocausal simulation
fn bench_retrocausal_simulation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("retrocausal_simulation");
    group.sample_size(50);

    let horizon_durations = vec![
        Duration::from_micros(100),
        Duration::from_micros(500),
        Duration::from_millis(1),
        Duration::from_millis(5),
    ];

    for horizon in horizon_durations {
        group.bench_with_input(
            BenchmarkId::new("future_simulation", format!("{}μs", horizon.as_micros())),
            &horizon,
            |b, &simulation_horizon| {
                b.iter(|| {
                    rt.block_on(async {
                        let retro_loop = RetrocausalLoop::new(
                            simulation_horizon,
                            CognitivePattern::CreativeSynthesis
                        ).unwrap();

                        let current_result = serde_json::json!({
                            "decision": "test_decision",
                            "confidence": 0.7,
                            "parameters": [1.0, 2.0, 3.0]
                        });

                        let optimized = retro_loop.apply_future_constraints(
                            &current_result,
                            Duration::from_micros(100)
                        ).await.unwrap();

                        black_box(optimized);
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark overall system throughput
fn bench_system_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("system_throughput");
    group.sample_size(20);

    group.bench_function("concurrent_agents_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let scheduler = TemporalScheduler::new(
                    SchedulerConfig::default()
                        .with_base_tick_duration(Duration::from_nanos(25_000))
                        .with_max_agents(50)
                );

                let mut agents = Vec::new();
                for i in 0..10 {
                    let agent = scheduler.spawn_agent(
                        AgentConfig::new(format!("throughput-agent-{}", i))
                            .with_pattern(CognitivePattern::SystemsThinking)
                            .with_dilation_factor(1.5)
                    ).await.unwrap();
                    agents.push(agent);
                }

                // Process tasks concurrently
                let mut tasks = Vec::new();
                for (i, agent) in agents.iter().enumerate() {
                    let task = TemporalTask {
                        id: format!("throughput-task-{}", i),
                        agent_id: agent.id().to_string(),
                        scheduled_ns: 0,
                        subjective_duration_ns: Duration::from_millis(1).as_nanos() as u64,
                        priority: TaskPriority::Normal,
                        cognitive_pattern: CognitivePattern::SystemsThinking,
                        payload: serde_json::json!({
                            "throughput_test": true,
                            "agent_id": i
                        }),
                    };

                    let task_future = agent.execute_task(task);
                    tasks.push(task_future);
                }

                // Wait for all tasks to complete
                let results = futures::future::try_join_all(tasks).await.unwrap();
                black_box(results);
            })
        });
    });

    group.bench_function("high_frequency_phi_measurement", |b| {
        b.iter(|| {
            rt.block_on(async {
                let scheduler = TemporalScheduler::new(SchedulerConfig::default());
                let agent = scheduler.spawn_agent(
                    AgentConfig::new("phi-freq-test".to_string())
                        .with_pattern(CognitivePattern::CreativeSynthesis)
                ).await.unwrap();

                // Measure Phi 100 times rapidly
                let mut measurements = Vec::new();
                for _ in 0..100 {
                    let phi = agent.measure_phi().await.unwrap();
                    measurements.push(phi);
                }

                black_box(measurements);
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scheduler_performance,
    bench_agent_spawning,
    bench_phi_measurement,
    bench_task_execution,
    bench_retrocausal_simulation,
    bench_system_throughput
);

criterion_main!(benches);