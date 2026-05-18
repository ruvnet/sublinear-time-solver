//! Solver baseline benchmarks.
//!
//! Three quick comparisons that exercise the current public API and give
//! us numbers to publish in the README + CHANGELOG. Each benchmark builds
//! a diagonally dominant 64×64 dense-ish matrix and solves Ax = b with a
//! different solver. Criterion handles warmup, statistical analysis, and
//! variance reporting.
//!
//! Run with: `cargo bench --bench solver_benchmarks`
//! Quick mode (single sample, useful in CI):
//!   `cargo bench --bench solver_benchmarks -- --quick`

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;

use sublinear_solver::{
    Matrix, SparseMatrix, NeumannSolver, SolverAlgorithm, SolverOptions,
    OptimizedConjugateGradientSolver, OptimizedSparseMatrix,
};
use sublinear_solver::optimized_solver::OptimizedSolverConfig;

/// Build a deterministic diagonally-dominant N×N test matrix
/// (5 on the diagonal, +1 / +1 / -1 / -1 on the four nearest off-diagonals
/// with wrap, so the matrix is well-conditioned without being trivially
/// diagonal).
fn build_test_triplets(n: usize) -> Vec<(usize, usize, f64)> {
    let mut t = Vec::with_capacity(n * 5);
    for i in 0..n {
        t.push((i, i, 5.0));
        t.push((i, (i + 1) % n, 1.0));
        t.push((i, (i + 2) % n, 1.0));
        t.push((i, (i + n - 1) % n, -1.0));
        t.push((i, (i + n - 2) % n, -1.0));
    }
    t
}

fn bench_neumann_series(c: &mut Criterion) {
    let mut group = c.benchmark_group("neumann_series");
    // With the correct-residual fix landed (update_residual compares
    // against the original `b`, not `D⁻¹b`), Neumann converges at the
    // larger sizes too. Bench across the same three n's as CG so the
    // numbers line up.
    for &n in &[16usize, 64, 256] {
        let triplets = build_test_triplets(n);
        let matrix = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let b: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
        let solver = NeumannSolver::new(64, 1e-10);
        let opts = SolverOptions {
            max_iterations: 200,
            tolerance: 1e-4,
            ..SolverOptions::default()
        };

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |bh, _| {
            bh.iter(|| {
                // Tolerate non-convergence at the bench's `tolerance =
                // 1e-4`; we're measuring iteration throughput, not
                // strict convergence (correctness covered by tests).
                let r = solver
                    .solve(black_box(&matrix), black_box(&b), black_box(&opts));
                black_box(r.is_ok());
            });
        });
    }
    group.finish();
}

fn bench_optimized_cg(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimized_cg");
    for &n in &[16usize, 64, 256] {
        let triplets = build_test_triplets(n);
        // Symmetrise so CG is well-defined: A + A^T halves off-diagonal
        // asymmetry. (Cheap and good enough for the smoke benchmark.)
        let mut sym = std::collections::HashMap::<(usize, usize), f64>::new();
        for &(i, j, v) in &triplets {
            *sym.entry((i, j)).or_insert(0.0) += v / 2.0;
            *sym.entry((j, i)).or_insert(0.0) += v / 2.0;
        }
        let sym_triplets: Vec<_> = sym.into_iter().map(|((i, j), v)| (i, j, v)).collect();
        let matrix = OptimizedSparseMatrix::from_triplets(sym_triplets, n, n).unwrap();
        let b: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
        let cfg = OptimizedSolverConfig::default();

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |bh, _| {
            bh.iter(|| {
                let mut solver = OptimizedConjugateGradientSolver::new(cfg.clone());
                let r = solver.solve(black_box(&matrix), black_box(&b)).unwrap();
                black_box(r.iterations);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_neumann_series, bench_optimized_cg);
criterion_main!(benches);
