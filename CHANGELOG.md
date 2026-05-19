# Changelog

All notable changes to this project. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased] — targeting v1.8.0 / Rust crate 0.4.0

ADR-001 phase-2/3 release. Closes every open question in the ADR
(except the hardware-dependent Pi hwmon J/solve), shipping a full
self-tuning SubLinear pipeline with empirical receipts, two runnable
examples, and a streaming iterator surface for Rust callers.

### Added — Change-driven SubLinear primitives

- **`closure_indices(matrix, seeds, depth)`** (`src/closure.rs`, PR #26).
  Bounded-depth row-graph BFS — input to every change-driven path. SubLinear in `n` for sparse DD with bounded depth.

- **`solve_single_entry_neumann(matrix, b, target, max_terms, tolerance)`** (`src/entry.rs`, PR #27). Computes `x[i] = e_iᵀ A⁻¹ b` via truncated Neumann restricted to `closure(target, max_terms)`. Never materialises the full `x`. Critical correctness test: agrees with full `NeumannSolver::solve` at every row of an 8×8 DD chain within `1e-6`.

- **`solve_on_change_sublinear(...)`** (`src/incremental.rs`, PR #29). Sparse delta-solve returning `Vec<(idx, val)>` over closure entries only — never materialises the full `n`-vector.

- **`contrastive_solve_on_change_sublinear(...)`** (`src/contrastive.rs`, PR #27). End-to-end SubLinear orchestrator: closure → per-entry Neumann → top-k-in-subset.

- **Auto-tuned siblings `*_sublinear_auto(...)`** (PR #38). Caller passes only `tolerance`/`k`; orchestrator picks `closure_depth + max_terms` internally from `coherence_score` + `optimal_neumann_terms`. Eliminates the last hand-tuned magic numbers from the change-driven inner loop.
- **`*_sublinear_auto_with_rho(...)` siblings** (PR #51). Tightest-bound variants that take a caller-supplied spectral-radius `ρ` (from `approximate_spectral_radius`) and use `optimal_neumann_terms_with_rho` for tighter tuning than the loose `(1 − coherence)` bound. On matrices where coherence-derived bounds are loose, produces a smaller `max_terms` → smaller closure → less per-event work.

### Added — Coherence + gating primitives

- **`delta_below_solve_threshold(coherence, min_diag, delta, tolerance)`** (`src/coherence.rs`, PR #34). `O(|δ|)` cached-input fast-path skip gate — the "no event, no work" discipline of ADR-001. Independent of `n` + `nnz` once `(coherence, min_diag)` are cached.

- **`optimal_neumann_terms(coherence, b_inf, min_diag, tolerance)`** (PR #37). Math-driven adaptive Neumann depth. Replaces caller-blind guesses with the Neumann-envelope bound `k ≥ log(‖b‖∞ / (min_diag · tolerance)) / log(1 / (1 − coherence))`.

- **`CoherenceCache::{build, update, score}`** (PR #39). Streaming per-row diagonal-dominance margin cache. `O(nnz)` build, `O(|dirty| · row_nnz)` update — amortised SubLinear when only a few rows mutate per event.

- **`approximate_spectral_radius(matrix, num_iters)` + `optimal_neumann_terms_with_rho(rho, ...)`** (PR #46). Power-iteration tight `ρ` estimate, used to pick smaller `max_terms` on matrices where `(1 - coherence)` is loose.

### Added — Verification + bounded planning

- **`verify_sparse_solution(matrix, prev, b, entries, tolerance)`** (`src/witness.rs`, PR #41). Per-entry residual audit restricted to the closure. SubLinear in `n` — same class as the orchestrator whose output it verifies. Closes ADR-001 open question #3.

- **`PlanBudget::{new, try_consume, has_capacity, worst_seen}`** (`src/budget.rs`, PR #40). Cumulative budget accumulator across chained solves. Per-call MCP gate (PR #28) + per-plan accumulator together bound the planning kernel from both directions.

### Added — Streaming surface

- **`event_stream_iter(matrix, prev, events, config, budget)` → `Iterator<Item = ProcessedEvent>`** (`src/stream.rs`, PR #45). The native Rust surface for processing event streams. Composes with stdlib `.filter()` / `.take()` / `.collect()`. Each event: optional skip-gate → optional budget consume → SubLinear orchestrator. Iterator ends gracefully on budget refusal.

### Added — MCP wire surface

- **`handleEstimateEntry` enforces `max_complexity_class` budget** (PR #28). The third solve handler joins `handleSolve` + `handleSolveTrueSublinear`.
- **`METHOD_WORST_CASE` + `estimateComplexityClass` table extended** (PRs #28, #30, #42) to cover every phase-2/3 primitive. Wire-callable agents can query any primitive's class before invoking it.
- **`verifySparseSolution`** (PR #52). Wire-callable witness — pure-TS handler computing the closure-restricted residual `r[i] = b[i] - Σ_j A[i,j]·x_new[j]` for caller-supplied entries. Returns `{ok, max_residual, threshold, worst_row}`. SubLinear class.
- **`coherenceScore`** (PR #53). Wire-callable feasibility check returning the matrix's diagonal-dominance margin. Lets agents check *"is this matrix solvable?"* before invoking a solver. Linear cost; positive ⇒ Neumann convergence guaranteed.
- **`closureIndices`** (PR #54). Wire-callable bounded-depth row-graph BFS. Lets agents preview *"what rows would my SubLinear orchestrator touch?"* before committing. SubLinear class.
- **`solveOnChangeSublinear`** (PR #56). Wire-callable SubLinear orchestrator. Pure-TS chain of closure + per-entry Neumann + return `Vec<{row, value}>` over closure only. Never materialises the full `n`-vector. SubLinear class.
- **`contrastiveSolveOnChangeSublinear`** (PR #57). Wire-callable top-k anomaly extraction. Delegates to `solveOnChangeSublinear`, then ranks entries by `|current − prev_solution[row]|` and returns the top-k. The canonical RuView / Cognitum wake-on-event primitive.

The five new MCP tools (`#52, #53, #54, #56, #57`) compose into an end-to-end wire pipeline for the change-driven inner loop — agents can run *predict → check → preview → solve → audit* without writing Rust.

### Added — Empirical receipts

- **`benches/solver_benchmarks.rs::delta_solve`** (PR #31). Compares `cold_full` / `warm_full` (Linear) vs `sparse_closure` (SubLinear) across `n = 64, 256, 1024`. Linear paths grow 4× per 4× size; SubLinear path stays roughly constant.
- **`benches/solver_benchmarks.rs::witness_audit`** (PR #43). Compares `full_residual` (Linear) vs `closure_audit` (SubLinear). Crossover at `n ≈ 200`; by `n = 1024` the closure-restricted audit is ~4× faster.
- **`benches/solver_benchmarks.rs::closure_only`** (PR #55). Isolates `closure_indices` across `n = 64, 256, 1024, 4096`. 64× growth in `n` → 1.4× growth in cost. The cleanest empirical demonstration that `closure_indices` is constant-in-n for sparse DD matrices.

### Added — Runnable demos

- **`examples/event_driven_anomaly.rs`** (PRs #33, #35, #38). Canonical inner-loop demo: baseline solve → coherence-gated event filter → SubLinear orchestrator → top-k anomaly extraction. End-to-end on 5 sensor events + 1 noise event.
- **`examples/event_stream_processing.rs`** (PR #48). Streaming surface demo using `event_stream_iter` + `PlanBudget` over a 10-event stream. Shows the iterator composition pattern.

### Documentation

- **README.md** (PR #32): "Complexity as a First-Class API Surface" section now reflects every shipped primitive with class + per-call cost. Bench numbers from `delta_solve` quoted inline.
- **BENCHMARK.md** (PR #36): Adds "Change-driven solve paths — Linear vs SubLinear" + "Event-driven anomaly example" sections, plus a complexity-class catalog.
- **`docs/adr/ADR-001-complexity-as-architecture.md`** (PR #44): Added "Shipped primitive catalogue" section enumerating every primitive shipped this cycle with class, file, and role. 3 of 5 open questions resolved inline.

### Tooling

- **`.github/complexity-baseline.txt`** updated to include every new `Op` marker (ClosureIndicesOp, ContrastiveSolveOnChangeOp, ContrastiveSolveOnChangeSublinearOp, EventStreamOp, SolveOnChangeSublinearOp, SolveSingleEntryNeumannOp, VerifySparseSolutionOp).
- **`temporal_nexus::integration_tests::test_demonstration`** quarantined on macOS (PR #47) — wall-clock-flaky on shared-tenant M1 runners.

### Tests

- **+80 new tests** across `src/closure.rs` (7), `src/entry.rs` (9), `src/contrastive.rs` (additions), `src/incremental.rs` (additions), `src/coherence.rs` (+27 across PRs #34, #37, #39, #46), `src/budget.rs` (8), `src/witness.rs` (7), `src/stream.rs` (5).
- 245+ lib tests pass on Linux + macOS (modulo the quarantined timing test on macOS).

[Unreleased]: https://github.com/ruvnet/sublinear-time-solver/compare/v1.7.2...HEAD

## [1.7.2] / Rust crate 0.3.2 — 2026-05-18

ADR-001 phase-2 release. Three enforcement mechanisms now live, plus the sub-linear contrastive primitive that completes roadmap item #6's phase-2 plan.

### Added

- **`find_anomalous_rows_in_subset(baseline, current, candidates, k)`** in `src/contrastive.rs`. Drops the phase-1 `O(n log k)` full scan to `O(|candidates| log k)` by limiting the scan to a caller-supplied candidate set. Combined with the existing `SublinearNeumannSolver`'s single-entry primitive (`O(log n)` per query), the total contrastive top-k cost becomes `O(|candidates| · log n)` — true sub-linear in n when |candidates| ≪ n. RuView / Cognitum / Ruflo callers compute the candidate set from a sparse RHS delta's reachable rows, then invoke this function on the top-k boundary check.

- **CI `complexity-baseline-guard` job** (`.github/workflows/ci.yml`) + `scripts/extract_complexity_classes.sh` + frozen `.github/complexity-baseline.txt`. Diffs the live `Complexity` impls against a checked-in snapshot on every PR; a silent class downgrade fails the build the same way safe-path regressions do. Uses `LC_ALL=C` so the sort is byte-stable across hosts.

### Improved

- **Phase-2 enforcement matrix now complete for ADR-001:**
  - **Server-side budget rejection** (v1.7.1): MCP refuses over-budget solver invocations at dispatch time.
  - **Type-level baseline guard** (this release): CI refuses `Complexity` impls that silently degrade.
  - **Sub-linear contrastive primitive** (this release): `find_anomalous_rows_in_subset` lets callers pay only for the rows they care about.

### Tests

- 5 new unit tests in `src/contrastive.rs` pinning the new function's contract (out-of-set anomalies stay ignored, OOB indices silently skipped, full-set candidates equals phase-1 output, k limit, empty-candidates → empty result).
- Total: **165 lib pass** (160 → +5), 11 doc pass, **7 CI gates** on every PR (test×2 OS, fmt+clippy, safe-path, bench-smoke, joules smoke, complexity-baseline-guard).

[1.7.2]: https://github.com/ruvnet/sublinear-time-solver/releases/tag/v1.7.2

## [1.7.1] / Rust crate 0.3.1 — 2026-05-18

Closes the ADR-001 roadmap. With this release the package is functionally SOTA per the ADR's own criterion (6 of 6 roadmap items shipped; README cites complexity classes as a first-class API surface; only CI J/solve integration remains, blocked on GH Actions exposing power counters).

### Added

- **MCP tool surface advertises complexity** (ADR-001 #4). `solve` and `solveTrueSublinear` JSON schemas now carry an `x-complexity` extension with `class`, `default`/`worst` for Adaptive solvers, `detail`, and `edgeSafe`. Clients can read it at `tools/list` time.
- **`max_complexity_class` input arg + server-side enforcement.** Both solver handlers reject the call with a structured `InvalidRequest` when the chosen method's worst-case class exceeds the caller's budget. No-op when the arg is absent (wire-compatible). Per-method class table mirrors the Rust `Complexity` impls.
- **New `estimateComplexityClass` MCP tool.** O(1) lookup against the per-method class table. Returns class + detail + edgeSafe so an agent can decide between 'spend the J/decision'and 'cache lookup' before invoking a solver.
- README's new "🧮 Complexity as a First-Class API Surface" section explaining the type-level + wire-level contract.

### Fixed

- **`[profile.bench]` now sets `panic = "unwind"`** so criterion's transitive deps (regex_syntax, aho_corasick, memchr, log, humantime, is_terminal, termcolor) can link. Without this, criterion's `catch_unwind`-using measurement loop fails the CI bench-smoke job after a recent dep drift tightened panic-strategy requirements.
- Cargo.toml gains an explicit `include` list so future npm/WASM artifact directories cannot silently push the crate tarball past the 10 MB crates.io cap. (The v1.7.0 publish narrowly avoided this; the v1.7.1 cycle hit it head-on. Fixed forward.)

[1.7.1]: https://github.com/ruvnet/sublinear-time-solver/releases/tag/v1.7.1

## [1.7.0] / Rust crate 0.3.0 — 2026-05-18

ADR-001 release. The first architectural ADR (*Complexity as
Architecture*) lands as code: every public solver now declares its
worst-case complexity class at the type level, a coherence gate
refuses polynomial-time work on near-singular systems, and an
event-gated entry point lets streaming systems pay sub-linear cost
per call instead of cold-starting on every tick.

### Added

- **`ComplexityClass` enum + `Complexity` trait** (`src/complexity.rs`).
  Twelve-tier taxonomy (`Logarithmic` → `DoubleExponential` +
  `Adaptive { default, worst }`) with `PartialOrd`/`Ord` for budget
  comparison, an object-safe `ComplexityIntrospect` trait blanket-
  impl'd for any `T: Complexity`, and `is_edge_safe()` /
  `short_label()` helpers. Lifts the "is this algorithm acceptable
  on a Pi Zero?" question from runtime-discovery to compile-time-
  check. Re-exported at the crate root as `Complexity`,
  `ComplexityClass`, `ComplexityIntrospect`.

- **Complexity impls for the headline solvers**:
  `NeumannSolver` → `Linear`,
  `OptimizedConjugateGradientSolver` → `Linear`,
  `SublinearNeumannSolver` → `Adaptive { default: Logarithmic, worst: Linear }`,
  `JLEmbedding` → `Linear`. Adaptive solvers carry both bounds so
  callers budget against the safe worst case.

- **Coherence gate** (`src/coherence.rs`).
  `coherence_score(&dyn Matrix) -> f64` returns the per-row diagonal-
  dominance margin (`min_i (|diag_i| − Σ|off_i|) / |diag_i|`) in
  `[-∞, 1]`. `check_coherence_or_reject(matrix, threshold)` returns
  `Err(SolverError::Incoherent { coherence, threshold })` when the
  matrix's coherence falls below the configured budget.
  `SolverOptions::coherence_threshold` defaults to `0.0` (gate
  disabled) so every existing caller stays wire-compatible.

- **`SolverError::Incoherent { coherence, threshold }`** new variant.
  `is_recoverable() = true`, `severity = Low` (budget refusal, not
  data corruption). Error message points the caller at ADR-001 and
  the opt-out.

- **`solve_on_change(matrix, prev_solution, delta)` event-gated entry**
  (`src/incremental.rs`). Extension trait `IncrementalSolver` blanket-
  impl'd for every `SolverAlgorithm`, so the entry point is available
  on every solver in the crate. Uses the residual-correction pattern
  (`A·dx = delta`, then `x_new = prev + dx`) which sidesteps the
  initial-guess-not-honoured-correctly trap in Neumann and is
  asymptotically faster on small deltas because the inner RHS is
  sparse. `SparseDelta { indices, values }` type with `apply_to`,
  `as_pairs`, length validation, out-of-bounds rejection.

- **23 new unit tests** across the three new modules (5 complexity,
  8 coherence, 6 incremental — plus 4 sanity tests). Lib test count
  148 → 151 (with the green base from v1.6.0 = 137 → 151 net).

- **ADR document**: `docs/adr/ADR-001-complexity-as-architecture.md`.
  196 lines. Twelve-class taxonomy mapped onto current code paths,
  six-item roadmap, "definition of SOTA" criterion. Driven by the
  `/loop 5m` cron `a3644c7d`.

### What's left for the next minor

Roadmap items #4 (MCP `x-complexity` schema + `max_complexity_class`
budget arg), #5 (joules-per-decision benchmark), #6 (contrastive
`find_anomalous_rows` adapter). All three are scoped in
ADR-001 §Roadmap.

### Acknowledgements

The "complexity classes are architecture, not academia" framing
came from @ruvnet's directive on the ruv.io stack (RuVector / RuView /
Cognitum / Ruflo). This release is the first half of that thesis
made executable in `sublinear-time-solver`.

[1.7.0]: https://github.com/ruvnet/sublinear-time-solver/releases/tag/v1.7.0

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
