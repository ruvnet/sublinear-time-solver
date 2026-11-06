use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nanosecond_scheduler::{Config, Scheduler, Task};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

fn benchmark_tick_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_overhead");

    for tick_rate in [100, 500, 1000, 5000].iter() {
        let config = Config {
            tick_rate_ns: *tick_rate,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}ns", tick_rate)),
            tick_rate,
            |b, _| {
                let scheduler = Scheduler::new(config.clone());
                b.iter(|| {
                    scheduler.tick();
                });
            },
        );
    }
    group.finish();
}

fn benchmark_task_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_throughput");
    group.throughput(Throughput::Elements(1000));

    for batch_size in [10, 50, 100, 500].iter() {
        let config = Config {
            max_tasks_per_tick: *batch_size,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_tasks", batch_size)),
            batch_size,
            |b, &batch_size| {
                let scheduler = Scheduler::new(config.clone());
                let counter = Arc::new(AtomicU64::new(0));

                // Pre-schedule tasks
                for _ in 0..batch_size {
                    let counter_clone = counter.clone();
                    scheduler.schedule(Task::new(
                        move || {
                            counter_clone.fetch_add(1, Ordering::Relaxed);
                        },
                        Duration::from_nanos(0),
                    ));
                }

                b.iter(|| {
                    scheduler.tick();
                });
            },
        );
    }
    group.finish();
}

fn benchmark_strange_loop_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("strange_loop");

    for lipschitz in [0.5, 0.7, 0.9, 0.99].iter() {
        let config = Config {
            lipschitz_constant: *lipschitz,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("k_{}", lipschitz)),
            lipschitz,
            |b, _| {
                let scheduler = Scheduler::new(config.clone());
                b.iter(|| {
                    for _ in 0..100 {
                        scheduler.tick();
                    }
                    black_box(scheduler.strange_loop_state())
                });
            },
        );
    }
    group.finish();
}

fn benchmark_temporal_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_windows");

    for window_size in [10, 50, 100, 500].iter() {
        let config = Config {
            window_size: *window_size,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("window_{}", window_size)),
            window_size,
            |b, _| {
                let scheduler = Scheduler::new(config.clone());

                // Pre-fill windows
                for _ in 0..*window_size {
                    scheduler.tick();
                }

                b.iter(|| {
                    scheduler.tick();
                    black_box(scheduler.temporal_overlap())
                });
            },
        );
    }
    group.finish();
}

fn benchmark_parallel_execution(c: &mut Criterion) {
    #[cfg(feature = "parallel")]
    {
        let mut group = c.benchmark_group("parallel_execution");
        group.throughput(Throughput::Elements(1000));

        for parallel in [false, true].iter() {
            let config = Config {
                parallel: *parallel,
                max_tasks_per_tick: 100,
                ..Default::default()
            };

            group.bench_with_input(
                BenchmarkId::from_parameter(if *parallel { "parallel" } else { "serial" }),
                parallel,
                |b, _| {
                    let scheduler = Scheduler::new(config.clone());
                    let counter = Arc::new(AtomicU64::new(0));

                    // Schedule CPU-intensive tasks
                    for _ in 0..100 {
                        let counter_clone = counter.clone();
                        scheduler.schedule(Task::new(
                            move || {
                                // Simulate work
                                let mut sum = 0u64;
                                for i in 0..100 {
                                    sum = sum.wrapping_add(i);
                                }
                                counter_clone.fetch_add(sum, Ordering::Relaxed);
                            },
                            Duration::from_nanos(0),
                        ));
                    }

                    b.iter(|| {
                        scheduler.tick();
                    });
                },
            );
        }
        group.finish();
    }
}

criterion_group!(
    benches,
    benchmark_tick_overhead,
    benchmark_task_throughput,
    benchmark_strange_loop_convergence,
    benchmark_temporal_windows,
    benchmark_parallel_execution
);
criterion_main!(benches);