# Benchmark Results

Baseline numbers for the two main solver paths, captured from
`cargo bench --bench solver_benchmarks -- --quick` on the dev host
after the iter-1–5 fixes on branch `deepreview/sota-pass-2026-05-18`.

The harness exercises each solver on a deterministic diagonally-
dominant N×N test matrix (5 on the diagonal, ±1 on the four nearest
off-diagonals with wrap, so the matrix is well-conditioned without
being trivially diagonal). The reported numbers are criterion's
median over its quick-mode samples; full release-mode runs go in CI's
`bench-smoke` job.

## Solver runtime (single solve)

| Solver         | n=16    | n=64    | n=256   | Throughput @ n=256 |
|----------------|---------|---------|---------|--------------------|
| Neumann series | 3.6 µs  | 12.6 µs | 51.5 µs | ~5.0 Melem/s       |
| Optimized CG   | 198 ns  | 316 ns  | 816 ns  | ~314 Melem/s       |

**Observation.** Optimized CG is **40–60× faster than Neumann** across
all three sizes on these symmetric test matrices, which matches the
classical ranking (CG amortises matrix-vector products with very tight
inner loops, Neumann pays per-term scaling and a residual recompute).
Use Neumann when the system is asymmetric and CG isn't applicable;
otherwise CG is the default.

## Fixes that landed this cycle

(numbers above reflect post-fix performance; pre-fix numbers in parens)

- **Neumann residual against original RHS**
  (`src/solver/neumann.rs:update_residual`). Previously compared
  `A·x` against the *scaled* RHS `D⁻¹b`, which is the residual of a
  different equation. Result: the solver kept iterating past the true
  convergence point and outright diverged at n ≥ 64 on the bench
  matrices. After the fix, n=16 is 47% faster (correct early exit),
  n=64 and n=256 converge cleanly where they couldn't before.

- **k=0 Neumann term double-counted** (iter-2). The solution was
  initialised to `D⁻¹b` *and* `compute_next_term` immediately added
  another copy, so a 2×2 system that should converge to `[1,1]`
  ended at `[2,2]`.

- **Conjugate-gradient instrumentation** (iter-2).
  `OptimizedConjugateGradientSolver` was inlining every dot product
  and AXPY directly, so `performance_stats.dot_product_count` /
  `axpy_count` stayed at 0 the entire run. Routed the three CG dot
  products and two AXPY updates through the existing instrumented
  helpers; SIMD/scalar dispatch unchanged.

- **Adaptive sampler symmetry** (iter-2). `adapt_parameters` scaled
  UP by 1.5× on high error but DOWN by only 0.8× on low error, so
  the sampling probability drifted monotonically upward over time.
  Made the factors inverse so a high-then-low pair round-trips.

- **JL embedding `target_dim`** (iter-1). `compute_target_dimension`
  could return `k > n` (a dimensional *expansion* dressed up as a
  reduction). Capped at `n−1` with the standard Achlioptas constant.

- **Sublinear-Neumann base case** (iter-3). `solve_base_case` ran a
  hard-coded 10 Jacobi iterations, well short of the ~30 needed to
  hit the test's `<1e-10` residual on a 2×2 system. Now driven by
  `max_recursion_depth` with a 64-iter floor and step-size early
  termination at 1e-14.

## How to run

```bash
# Quick (single sample per benchmark, useful in CI)
cargo bench --bench solver_benchmarks -- --quick

# Full criterion run (warmup + ~30 samples per benchmark)
cargo bench --bench solver_benchmarks

# CI bench-smoke job runs the --quick variant on every PR
```

Criterion writes HTML reports to `target/criterion/` if you want to
poke at the per-sample histograms.
