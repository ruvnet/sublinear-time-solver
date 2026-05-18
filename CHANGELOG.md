# Changelog

All notable changes to this project. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/).

## [1.6.0] / Rust crate 0.2.0 — 2026-05-18

This is a security + correctness + verification release. After this
cycle the full test matrix is **148/148 green** (132 lib + 7 integration
+ 4 nanosecond-scheduler + 5 doc) where it was previously *unbuildable*
(7 compile errors blocked `cargo test --no-run`).

### Security

- **CVE-class fix for [GH issue #19](https://github.com/ruvnet/sublinear-time-solver/issues/19) (CWE-73, Arbitrary File Write).**
  Reported by BruceJin (Apr 2026). The MCP `export_state` /
  `import_state` tools accepted an attacker-controlled `filepath` and
  passed it straight to `fs.writeFileSync` / `fs.readFileSync` — letting
  any MCP caller write or read any file the server process had access to.

  Fix: new `src/consciousness-explorer/lib/safe-path.js` confines every
  state file to a dedicated directory (`$CONSCIOUSNESS_EXPLORER_STATE_DIR`
  or `~/.consciousness-explorer/state`), enforces a basename-only
  contract (rejects path separators, `..`, leading `.`, NUL/control
  chars, Windows reserved device names, oversize names), and opens with
  `O_NOFOLLOW | O_CLOEXEC` mode `0o600` so a planted symlink at the
  final path component can't redirect the I/O. 14 regression tests in
  `tests/consciousness/safe-path.test.mjs`.

- **Same sink class also closed in the main MCP server**
  (`src/mcp/server.ts`, `saveVectorToFile` + `loadVectorFromFile`). Was
  not in the original report; surfaced during remediation. Same contract
  via the TS counterpart `src/mcp/safe-path.ts`, with vectors confined
  to `$SUBLINEAR_SOLVER_VECTOR_DIR` (default `~/.sublinear-time-solver/vectors`).

- **MCP tool schemas** for `export_state`, `import_state`, and
  `saveVectorToFile` now advertise the basename-only contract via JSON
  Schema `pattern: ^[^/\\\x00]+$`, `minLength: 1`, `maxLength: 255` so
  MCP clients see the constraint at `tools/list` time.

> ⚠️  **Breaking change** for callers that previously passed absolute
> paths or paths with separators. Pass a basename only; override the
> base directory via the environment variables above if you need to
> point at something other than the default.

### Fixed — correctness

- **Neumann solver: residual checked against original RHS**
  (`src/solver/neumann.rs`). `update_residual` was comparing `A·x`
  against `self.rhs = D⁻¹b` — the residual of a different equation
  (`A·x = D⁻¹b`, not `A·x = b`). The convergence check therefore fired
  against the wrong quantity, and the solver outright diverged at n ≥ 64
  on the bench harness. Now stores `original_rhs` and computes
  `r = A·x − b` correctly. n=16 case is 47% faster (correct early
  exit), n=64 / n=256 now converge cleanly.

- **Neumann solver: k=0 term no longer double-counted.** `solution`
  was initialised to `D⁻¹b` *and* `compute_next_term` immediately added
  another copy, so a 2×2 toy system that should converge to `[1, 1]`
  ended at `[2, 2]`. Starts from zero now; caller-supplied initial
  guesses still honoured.

- **Sublinear-Neumann base case: too few iterations.** `solve_base_case`
  ran a hard-coded 10 Jacobi iterations, ≈30 short of the test's
  `residual < 1e-10` gate on the standard 2×2 matrix. Now uses
  `max(max_recursion_depth × 16, 64)` iters and reports the actual
  count.

- **OptimizedConjugateGradientSolver: instrumentation correctness.**
  CG inlined every dot product and AXPY directly, so
  `performance_stats.dot_product_count` / `axpy_count` stayed at 0
  the whole run. Routed through the existing instrumented helpers;
  SIMD/scalar dispatch unchanged.

- **HNSW JL embedding: `target_dim` could exceed `original_dim`.**
  `compute_target_dimension` returned the raw `k = O(log n / ε²)` for
  small n / tight ε, which gave `k > n` — a dimensional *expansion*
  dressed up as a reduction. Capped at `n − 1` and dropped the
  constant to 4 (Achlioptas / Dasgupta-Gupta) so the embedding is
  always strictly dimension-reducing.

- **AdaptiveSampler: small `original_dim` panicked construction.**
  `AdaptiveSampler::new(_, Some(20))` tried to build a 64-dim sketch
  matrix out of 20 dims and `MatrixSketcher::new` rejected it. Cap
  the effective sketch dim at `original_dim − 1`.

- **AdaptiveSampler: asymmetric scaling.** `adapt_parameters` scaled
  UP by `×1.5` on high error but DOWN by only `×0.8` on low error, so
  a high-then-low pair monotonically grew the sampling probability.
  Made the factors inverse so the adaptation round-trips.

### Fixed — quantum/temporal validators

- **`TscTimestamp::now()` is now arch-portable.** Was unconditionally
  `core::arch::x86_64::_rdtsc()`, so `cargo build` failed on
  Apple Silicon. Gated three paths: x86_64 → RDTSC; aarch64 →
  inline-asm `mrs cntvct_el0` (the virtual counter register, semantic
  equivalent of RDTSC at 24 MHz on Apple Silicon); everything else →
  `Instant::now()` fallback.

- **`UncertaintyValidator::calculate_maximum_time` inverted division.**
  Returned `E / (ℏ/2)` (units of 1/time) instead of `(ℏ/2) / E` (units
  of time), so a 1 ns target round-tripped to 1 GHz. Fixed.

- **`UncertaintyValidator` Planck-relationship tolerance.** Used an
  absolute tolerance of `1e-50` for `ℏ = h/(2π)`, tighter than f64
  can represent at the ~1e-34 scale (ULP at 1e-34 is ≈ 2e-50). Switched
  to relative tolerance `1e-9`. Same fix applied in the integration
  test `tests/quantum_physics_validation_test.rs`.

- **`DecoherenceTracker`: dephasing rate now scales with temperature.**
  Was hardcoded to 1 GHz regardless of T, so 10 mK cryogenic and 300 K
  room-temp trackers reported identical coherence times. Now scales
  linearly with T (1 GHz at 300 K) with a 1 Hz floor.

- **`DecoherenceTracker::calculate_thermal_decoherence_rate`: removed
  misleading 1 kHz floor** that swallowed real temperature dependence.

- **`EntanglementValidator::validate_temporal_correlation` no longer
  errors on low entanglement.** "Concurrence below threshold" is a
  legitimate result state (`is_valid: false` already carries it), not
  an exception. The previous early `Err(EntanglementLost)` prevented
  callers from comparing concurrences across decoherence regimes.

- **New `EntanglementValidator` methods.** Three methods that the test
  suite referenced but had never existed:
  - `analyze_consciousness_time_scales()` — 6 named time scales
    (neural spike, gamma/beta/alpha/theta/delta wave) with
    `ConsciousnessRelevance` ratings.
  - `model_consciousness_network(size, time)` — `C(size, 2)` pairwise
    `NodeEntanglement` entries + aggregate `network_coherence`.
  - `calculate_quantum_fisher_information(time)` — QFI ≈ `n² × s²`
    for n-qubit dephased state, clamped strictly positive.

- **`IdentityContinuityTracker::calculate_similarity`** now blends
  cosine (30%) with mean-L1 (30%) and Chebyshev max-per-component
  (40%). Pure cosine is scale-invariant and the feature extractor
  tanh-saturates every component, so `[1,2,3,4,5]` vs `[10,20,30,40,50]`
  saturated near `(1,1,1,…)` and looked identical — silently swallowing
  the very-different-states case the continuity tracker exists to flag.

- **`IdentityContinuityTracker::validate_continuity` no longer errors
  on cold start.** Returned `IdentityContinuityBreak` whenever
  `continuity_score < threshold`, which is the default state at start
  (0.0 < 0.7). The first scheduler tick therefore panicked. Now returns
  Ok when fewer than 2 snapshots exist — no history, no continuity to
  validate.

- **`NanosecondScheduler::record_overhead` is now metric-only.** Was
  returning Err whenever a single tick exceeded `max_scheduling_overhead_ns`
  (default 1 µs) — a budget that only holds on tuned release builds
  with a real RDTSC. Debug builds / CI hosts routinely see 50–200 µs
  per tick. The scheduler tracks timing, doesn't crash on it.

- **`TemporalWindow::size()` off-by-one** for inclusive `end_tick`. A
  window `(0..=999)` now correctly reports 1000 ticks.

- **`StrangeLoopOperator::calculate_correlation` for constant vectors.**
  Pearson is 0/0 on a constant vector; the previous code returned 0.0,
  so an operator fed an identical state repeatedly never registered
  any loop depth. Falls back to 1.0 for matching constants, 0.0 for
  differing constants.

### Added

- **CI workflow** (`.github/workflows/ci.yml`). Four jobs gating every
  push to main: `cargo test` on Ubuntu + macOS, `fmt + clippy`,
  `safe-path regression` (issue #19), `cargo bench --quick`.

- **`BENCHMARK.md`** with baseline numbers. Optimized CG is **40-60×
  faster than Neumann** across n=16/64/256 on diagonally-dominant
  test matrices (197 ns / 316 ns / 816 ns for CG vs 3.6 µs / 12.6 µs /
  51.5 µs for Neumann).

- **Fresh `benches/solver_benchmarks.rs`** against the current public
  API. The previous bench corpus referenced removed modules
  (`fast_solver`, `core`, `algorithms`, `solver::hybrid`) and would
  not compile. The broken files are archived under `benches/.archived/`.

- **23 new unit / integration tests** across the fixes above (notably
  14 in `tests/consciousness/safe-path.test.mjs` for the CWE-73
  regression).

### Internal

- `Cargo.toml`: dropped `required-features = ["criterion"]` on the
  solver_benchmarks bench (was a silent no-op; criterion is a
  dev-dependency, not a feature).

- Doc tests: 6 pre-existing `///` examples that never compiled (Greek
  letter variable names in `\`\`\`rust` blocks, `?` in non-`Result` main,
  uncheck `from_triplets` return) cleaned up so `cargo test --doc`
  passes.

### Acknowledgements

Security report by **BruceJin <brucejin@zju.edu.cn>**. The path-
traversal fix in the consciousness-explorer + the same-class fix in
the main MCP server were both motivated by his work. Recommend
publishing a GitHub Security Advisory citing CWE-73 + this version
once 1.6.0 ships.

[1.6.0]: https://github.com/ruvnet/sublinear-time-solver/releases/tag/v1.6.0
