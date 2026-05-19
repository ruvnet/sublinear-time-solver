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
    IncrementalSolver, SparseDelta, solve_on_change_sublinear,
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

/// ADR-001 #2/#6 phase-2 — empirical comparison of the three change-driven
/// solve paths on a small RHS perturbation:
///
///   - `cold_full`:    full NeumannSolver::solve(b_new) — Linear in n
///   - `warm_full`:    IncrementalSolver::solve_on_change — Linear in n
///                     with k_warm ≪ k_cold (warm-start over prev)
///   - `sparse_closure`: solve_on_change_sublinear — SubLinear in n
///                     (returns only the closure entries, never
///                     materialises the full vector)
///
/// Architectural property: on increasing `n` the three curves diverge.
/// `cold_full` and `warm_full` grow linearly with `n`; `sparse_closure`
/// stays roughly constant (bounded by closure size, independent of `n`).
/// Crossover happens past `n ≈ 10⁴` on this test matrix with conservative
/// `closure_depth=4, max_terms=32`. Real applications tune those down to
/// the spectral-radius bound on their specific A, which pushes the
/// crossover much lower.
///
/// This is the empirical receipt for the SubLinear claim in ADR-001.
fn bench_delta_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_solve");

    // Fixed delta size for all n's; closure_depth fixed so the SubLinear
    // path scales with closure (not n).
    let closure_depth = 4usize;
    let max_terms = 32usize;
    let tolerance = 1e-8;

    for &n in &[64usize, 256, 1024] {
        let triplets = build_test_triplets(n);
        let matrix = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let b_prev: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();

        // Pre-compute prev_solution so warm/cold/sublinear all start from
        // the same baseline state. Use a tight tolerance so prev_solution
        // is genuinely close to A⁻¹·b_prev — otherwise warm-start gets
        // an unfair advantage.
        let warmup_solver = NeumannSolver::new(128, 1e-12);
        let opts_tight = SolverOptions {
            max_iterations: 500,
            tolerance: 1e-10,
            ..SolverOptions::default()
        };
        let prev_solution = warmup_solver
            .solve(&matrix, &b_prev, &opts_tight)
            .expect("warmup solve failed")
            .solution;

        // 1-row delta — the canonical "one sensor reading came in" event.
        let delta_idx = n / 3;
        let delta = SparseDelta::new(alloc_vec(delta_idx), alloc_vec_val(0.25)).unwrap();
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).unwrap();

        let solver = NeumannSolver::new(64, 1e-10);
        let opts = SolverOptions {
            max_iterations: 200,
            tolerance: 1e-6,
            ..SolverOptions::default()
        };

        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("cold_full", n), &n, |bh, _| {
            bh.iter(|| {
                let r = solver.solve(black_box(&matrix), black_box(&b_new), black_box(&opts));
                black_box(r.is_ok());
            });
        });

        group.bench_with_input(BenchmarkId::new("warm_full", n), &n, |bh, _| {
            bh.iter(|| {
                let r = solver.solve_on_change(
                    black_box(&matrix),
                    black_box(&prev_solution),
                    black_box(&delta),
                    black_box(&opts),
                );
                black_box(r.is_ok());
            });
        });

        group.bench_with_input(BenchmarkId::new("sparse_closure", n), &n, |bh, _| {
            bh.iter(|| {
                let r = solve_on_change_sublinear(
                    black_box(&matrix),
                    black_box(&prev_solution),
                    black_box(&b_new),
                    black_box(&delta),
                    black_box(closure_depth),
                    black_box(max_terms),
                    black_box(tolerance),
                );
                black_box(r.is_ok());
            });
        });
    }
    group.finish();
}

// Tiny helpers — keeping the bench file free of `alloc::vec!` boilerplate.
fn alloc_vec(i: usize) -> Vec<usize> {
    vec![i]
}
fn alloc_vec_val(v: f64) -> Vec<f64> {
    vec![v]
}

criterion_group!(benches, bench_neumann_series, bench_optimized_cg, bench_delta_solve);
criterion_main!(benches);
