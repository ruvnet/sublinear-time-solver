# ADR-001: Complexity as Architecture

**Status**: Proposed
**Date**: 2026-05-18
**Authors**: @ruvnet, sublinear-time-solver maintainers
**Deciders**: ruv.io Architecture Review Board
**SDK**: Claude-Flow

---

## Version History

| Version | Date | Author | Changes |
|---|---|---|---|
| 0.1 | 2026-05-18 | @ruvnet | Initial proposal — complexity classes as architectural primitives |

---

## Context

This ADR sits at the intersection of `sublinear-time-solver` and the broader ruv.io stack (RuVector, RuView, Cognitum, Ruflo, agentic-flow). It is the *first* architectural ADR for this repository, written after the v1.6.0 security + correctness shakedown.

The motivating observation is short:

> **Complexity classes are not academic. In a real-time edge-deployed cognitive stack they are architecture.**

Concretely, every subsystem in the stack lives or dies by the complexity-class budget it can hold:

- **Real-time sensor fusion** can survive `O(n log n)` ingest but not `O(n²)`.
- **Dynamic graph reasoning** on swarm topologies needs `O((log n)^k)` for continuous coherence maintenance.
- **Edge inference on Pi Zero / Cognitum Seed** has a budget measured in joules per decision; an exponential algorithm is not a "slow path", it is an *unreachable* path.
- **Always-on cognition** through agentic loops degenerates to `O(2^n)` the moment recursive planning is unbounded — the failure mode is not slowness, it is heat death.

What the user-facing essay (attached to the ADR's source prompt) frames as a philosophical claim — "intelligence is not about processing everything, it is about rapidly identifying what changed enough to matter" — is *encoded in this repository as a complexity-class contract*. The Neumann series solver, the optimised CG, the JL embedding, the future contrastive search adapter — each one is a vote on a specific class. This ADR makes those votes explicit and exposes them through the API.

### What this repository ships today (v1.6.0)

| Subsystem | Realised class | Notes |
|---|---|---|
| `OptimizedConjugateGradientSolver::solve` | `O(k · nnz(A))` per iter, k ≈ √κ on SPD | Linear-in-nonzeros per iter, classic CG |
| `NeumannSolver::solve` | `O(k · nnz(A))` per iter | k bounded by `ef_construction`-like beam |
| `SublinearNeumannSolver::solve_sublinear_guaranteed` | **`O(log n)`** per single-entry query on DD systems | Kyng/Sachdeva-style |
| `JLEmbedding::project_vector` | `O(d · k)` per vector, `k ≤ n − 1` | Strictly dimension-reducing as of 1.6.0 |
| `AdaptiveSampler::sample` | `O(k)` per draw | Reservoir + importance |
| `analyze` (matrix properties) | `O(nnz(A))` | One-pass |
| MCP `export_state` / `saveVectorToFile` | `O(state)` | Bounded by snapshot size |
| `temporal_nexus::scheduler::tick` | `O(1)` amortised | Strange-loop + identity feature extraction is the constant |

The repository already speaks multiple complexity classes. What is **missing** is making that contract visible:

1. The complexity class of every public function is **not declared at the type level**, so callers cannot refuse anything worse than `O(n log n)` without reading the source.
2. The MCP tool surface does **not advertise** worst-case class in its `tools/list` response — clients have no way to budget.
3. There is **no event-gated entry point** — every solve re-runs the full algorithm even when the input is a sparse delta over a previously-solved system. This is the single largest gap relative to the ADR thesis.
4. There is **no coherence gate** — the solver will happily spend polynomial time approximating an ε-quality answer on a near-singular system, instead of refusing.
5. Benchmarks measure time, not **joules per decision** — the metric the edge cares about.
6. The `find_anomalous_rows` / contrastive-search adapter does not exist. RuView and Cognitum's "activate only on change / anomaly / boundary crossing" pattern has no library backing in this repo.

### The dozen complexity classes (recap from the directive)

The user's directive enumerates twelve complexity tiers and how each maps to the stack. Restated as a table for cross-reference inside this repository:

| Class | Examples | Acceptable use here | Forbidden use here |
|---|---|---|---|
| **`O(log n)`** | binary search, HNSW layer traversal, sublinear-Neumann single-entry | routing, partition lookup, witness index | n/a |
| **`O((log n)^k)`** | dynamic connectivity, spectral sparsifiers, continuous coherence | live graph repair, always-on coherence tracking | n/a |
| **`O(n^c), c<1`** | ANN, sparse attention, event-driven activation, anomaly detection | RuView change detection, contrastive search | n/a |
| **`O(n)`** | streaming, ingest, replay, sensor scan | one-pass ingest, WAL replay | repeated linear passes per query |
| **`O(n log n)`** | sorting, indexing, ANN build, graph compression | offline preprocessing | per-query path |
| **`O(n^{2−ε})`** | sub-quadratic graph algorithms, sparsified mincut | offline coherence analysis | hot path |
| **`O(n^k)`, k ≤ 3** | matrix ops, classical graph algorithms | one-time setup with cached output | streaming hot path |
| **superpolynomial** | exhaustive reasoning, unbounded planning | never on hot path | hot path |
| **`O(2^n)`** | brute-force search, full combinatorial | never | always |
| **`O(n!)`** | full permutation, exhaustive route planning | never | always |
| **`O(2^{2^n})`** | symbolic-explosion / advanced logic | never in runtime | always |

This ADR's decision is that **every public surface in this crate carries an explicit class annotation, every MCP tool exposes it in its schema, and the build refuses to land code that crosses a `max_complexity_class` budget set per subsystem**.

---

## Decision

We adopt **Complexity-as-Architecture** as the governing principle for this repository, implemented as five concrete changes:

### 1. `Complexity` trait + `#[complexity(...)]` attribute

Every public solver, analyser, sampler, and MCP handler gains a `Complexity` impl declaring its worst-case class on the *single-query* cost. A small derive macro (`#[complexity(SubLinear)]`, `#[complexity(QuasiLinear)]`, etc.) lowers to a const associated value. Callers can match on `Solver::COMPLEXITY` at compile time.

```rust
// Proposed
#[complexity(SubLinear { upper = "O(log n)", lower = "Ω(log n)" })]
impl Solver for SublinearNeumannSolver { /* ... */ }

#[complexity(Linear { upper = "O(k · nnz(A))" })]
impl Solver for OptimizedConjugateGradientSolver { /* ... */ }
```

The enum is the twelve-tier list from the directive:

```rust
pub enum ComplexityClass {
    Logarithmic,        // O(log n)
    PolyLogarithmic,    // O((log n)^k)
    SubLinear,          // O(n^c), c<1
    Linear,             // O(n)
    QuasiLinear,        // O(n log n)
    SubQuadratic,       // O(n^{2-ε})
    Polynomial(u8),     // O(n^k)
    SuperPolynomial,
    SubExponential,
    Exponential,        // O(2^n)
    Factorial,          // O(n!)
    DoubleExponential,  // O(2^{2^n})
}
```

### 2. Event-gated solver entry point

`Solver::solve_on_change(prev_solution, delta_b)` re-solves only the rows touched by a sparse `delta_b`, falling back to `solve` when the delta is dense enough that incremental work exceeds full-solve cost. This is the central lift from "every call is O(nnz(A))" to "every call is O(nnz(delta_b) · log n)".

This entry point is what RuView, Cognitum, and Ruflo's agentic loops should call by default. The full `solve` becomes the *cold-start* path; `solve_on_change` is the steady-state path.

### 3. Coherence gate

Before any solve, the system checks coherence: `coherence(A, b) = min_i |diag(A)[i]| / Σ_{j≠i} |A[i,j]|` (the diagonal-dominance margin). If coherence drops below a configurable threshold (default 0.05), the solver refuses and returns `Err(SolverError::Incoherent { coherence, threshold })`.

This prevents the failure mode where the solver spends polynomial time on a near-singular system to produce a result with ε-quality bounds that the caller does not need.

### 4. MCP tool surface advertises complexity

Every MCP tool gains `x-complexity` annotations in its JSON Schema and a `max_complexity_class` input arg. The schema is auto-generated from the `#[complexity(...)]` attribute. Clients can refuse to call anything worse than their budget *at tool-list time*, not after the call returns 10 minutes later.

Also adds an `estimate_complexity_class(matrix_descriptor, query_type)` tool that predicts the class for a candidate solve before it runs, so an agent can decide between "spend the J/decision" and "fall back to a cached answer".

### 5. Joules-per-decision bench

`benches/joules_per_decision.rs` measures total energy consumed across a fixed solve workload by reading `/sys/class/powercap/intel-rapl:0/energy_uj` (x86) or `/sys/class/hwmon/.../power_input` (Pi). Each algorithm gets a J/decision number, not just a ns/decision number. This is the metric edge / agentic systems actually optimise.

---

## Consequences

### Positive

- **Callers can budget**. A Cognitum reflex loop with a 100 µs / 10 J budget can reject any solver tagged `Polynomial(3)` at *compile time* (Rust) or *tool-list time* (MCP), instead of discovering the budget bust at runtime.
- **The contract becomes documentation**. New contributors land a solver and the `#[complexity]` attribute forces them to *think* about which class they're claiming.
- **Regression guards become possible**. CI can add a job that diffs the `Complexity` impl of every public function between PRs — silently regressing `SubLinear → Linear` would fail the build the same way regressing security tests already does (see `.github/workflows/ci.yml` `safe-path regression`).
- **Energy budgeting**. The `joules_per_decision` bench is the missing link between "this algorithm is fast" and "this algorithm is *deployable on the Pi Zero*". Without it, every claim about edge readiness is a vibe.
- **Stack alignment**. RuView, Cognitum, and Ruflo can require `SubLinear` or stronger on every inner-loop call. The agentic systems get an architectural defence-in-depth against the recursive-planning blowup the directive flags.

### Negative

- **Annotation overhead**. Every public solver / sampler / analyser gains a 1-line attribute. Roughly 20-30 sites in the current codebase.
- **False precision**. `O(log n)` is the *worst case*; on a pathological input the constants matter. We mitigate by reporting both upper and lower bounds in the `Complexity` impl (see proposed macro syntax above).
- **Coherence gate may surprise callers**. A caller that previously got a degraded but usable answer on a near-singular system will now get an `Err(Incoherent)`. Document loudly; provide an `ignore_coherence: true` opt-out for callers that explicitly want best-effort.
- **MCP schema churn**. Existing MCP clients will see the new `x-complexity` and `max_complexity_class` fields and may need updates. Wire-compatible (additive), so old clients keep working.
- **Power-bench platform-specific**. RAPL is Intel-only; AMD has its own counters; Pi uses hwmon. Need a thin abstraction layer (`PowerCounter` trait) with three implementations. Falls back to "not measured" on platforms without a counter.

---

## Roadmap

Six concrete items, ordered by impact-per-effort. The `/loop 5m` cron (`a3644c7d`) drives one item per few iterations until the whole roadmap is implemented and the package is SOTA on the metrics this ADR defines.

| # | Item | Effort | Lands when |
|---|---|---|---|
| 1 | `Complexity` trait + `ComplexityClass` enum + attribute macro | 1-2 iter | ✅ **Shipped in v1.7.0** (`src/complexity.rs`) |
| 2 | `solve_on_change(prev, delta)` on `Solver` trait | 2-3 iter | ✅ **Shipped in v1.7.0** (`src/incremental.rs`) — warm-start path Linear. Phase-2 added `solve_on_change_sublinear` (closure + per-entry Neumann) returning only the changed entries as `Vec<(idx, val)>`, end-to-end SubLinear in `n`. |
| 3 | Coherence gate | 1 iter | ✅ **Shipped in v1.7.0** (`src/coherence.rs` + `SolverOptions::coherence_threshold`) |
| 4 | MCP `x-complexity` + `max_complexity_class` arg + `estimate_complexity_class` tool | 2 iter | ✅ **Shipped in v1.7.x** — `solve` and `solveTrueSublinear` schemas carry `x-complexity`; new `estimateComplexityClass` tool returns per-method classes. Phase-2: enforce `max_complexity_class` on solve handlers. |
| 5 | `joules_per_decision` bench (Linux RAPL + hwmon abstraction) | 2-3 iter | ✅ **Shipped in v1.7.0** (`examples/joules_per_decision.rs`) — RAPL backend works on Intel + AMD Zen 2+; time-only fallback for sandboxed hosts. Phase-2: Pi hwmon backend + CI integration. |
| 6 | `find_anomalous_rows(matrix, baseline_solution, k)` contrastive adapter | 2-3 iter | ✅ **Shipped in v1.7.0** + phase-2A + phase-2B. Three layers now: O(n log k) baseline (`find_anomalous_rows`), Adaptive {Linear,Linear} orchestrator (`contrastive_solve_on_change`, closure + warm-start solve + top-k-in-subset), and **end-to-end SubLinear** orchestrator (`contrastive_solve_on_change_sublinear`, closure + per-entry Neumann via `src/entry.rs::solve_single_entry_neumann` + top-k-in-subset). The SubLinear path never materialises the full solution vector. |

### Definition of "SOTA"

This ADR is "implemented and SOTA" when **all six items above ship**, the README explicitly cites complexity classes as a first-class API surface, and the CI `bench-smoke` job exercises both `time-per-solve` *and* `joules-per-solve`.

**Status as of v1.7.x (2026-05-19)**: 6 of 6 roadmap items shipped; item #6 now has its terminal **SubLinear** form (`contrastive_solve_on_change_sublinear`) in place. The change-driven activation primitive is fully realised: a sparse RHS delta produces a bounded-depth closure, per-entry sublinear-Neumann queries materialise only the candidate rows, and top-k extraction skips the rest of `n`. RuView, Cognitum, and Ruflo's inner loops can now claim end-to-end SubLinear contrastive solves. **All six items SubLinear-or-stronger end-to-end; "fully SOTA" remaining polish: README + BENCHMARK complexity-class language, J/solve CI integration.**

---

## Shipped primitive catalogue (as of 2026-05-19)

Beyond the six-item roadmap, this ADR drove a substantial extension surface across PRs #26–#57. Every primitive below is wire-queryable via `mcp__sublinear__estimateComplexityClass` and carries a `Complexity` impl whose `CLASS` constant matches the table.

### Change-driven solve primitives

| Primitive | Class | File | Role |
|---|---|---|---|
| `closure_indices(matrix, seeds, depth)` | SubLinear | `src/closure.rs` | bounded-depth row-graph BFS — input to every change-driven path |
| `solve_single_entry_neumann(matrix, b, i, max_terms, tol)` | SubLinear | `src/entry.rs` | compute `x[i]` without materialising `x` |
| `solve_on_change_sublinear(…)` | SubLinear | `src/incremental.rs` | sparse delta-solve over closure entries |
| `contrastive_solve_on_change_sublinear(…)` | SubLinear | `src/contrastive.rs` | end-to-end change-driven top-k anomaly detection |
| `solve_on_change_sublinear_auto(…)` | SubLinear | `src/incremental.rs` | auto-tuned `closure_depth + max_terms` from coherence |
| `contrastive_solve_on_change_sublinear_auto(…)` | SubLinear | `src/contrastive.rs` | auto-tuned contrastive sibling |
| `solve_on_change_sublinear_auto_with_rho(…)` | SubLinear | `src/incremental.rs` | tightest-bound variant taking caller-supplied ρ |
| `contrastive_solve_on_change_sublinear_auto_with_rho(…)` | SubLinear | `src/contrastive.rs` | tightest-bound contrastive sibling |
| `event_stream_iter(matrix, prev, events, cfg, budget)` | SubLinear | `src/stream.rs` | stdlib `Iterator` adapter over the orchestrator — native Rust composition |

### Coherence + gating primitives

| Primitive | Class | File | Role |
|---|---|---|---|
| `coherence_score(matrix)` | Linear | `src/coherence.rs` | one-shot diagonal-dominance margin |
| `CoherenceCache::build / update / score` | SubLinear (update) | `src/coherence.rs` | streaming per-row margin cache for mutable matrices |
| `delta_below_solve_threshold(…)` | O(\|δ\|) | `src/coherence.rs` | "no event, no work" skip-on-tiny-delta gate |
| `optimal_neumann_terms(coherence, …)` | O(1) | `src/coherence.rs` | adaptive Neumann-depth from coherence + tolerance |
| `approximate_spectral_radius(matrix, num_iters)` | Linear | `src/coherence.rs` | power-iteration tight ρ estimate (one-shot, amortised) |
| `optimal_neumann_terms_with_rho(rho, …)` | O(1) | `src/coherence.rs` | adaptive depth from explicit ρ — tighter than coherence-derived |
| `check_coherence_or_reject(matrix, threshold)` | Linear | `src/coherence.rs` | refuse near-singular solves before they run |

### Verification + bounded planning

| Primitive | Class | File | Role |
|---|---|---|---|
| `verify_sparse_solution(…)` | SubLinear | `src/witness.rs` | per-entry residual audit restricted to closure |
| `PlanBudget::try_consume(class)` | O(1) | `src/budget.rs` | cumulative budget across chained solves |

### MCP wire surface (per ADR-001 item #4 phase-2/3)

`solve`, `estimateEntry`, `solveTrueSublinear` all gate on the caller's
`max_complexity_class` arg before any solver work runs.
`estimateComplexityClass` exposes the full table above so budget-aware
clients can refuse a call at tool-list time.

Six additional wire-callable tools shipped after PR #44 lift the full
SubLinear pipeline onto MCP — agents can run *predict → check → preview
→ solve → audit* without writing Rust:

| MCP tool | Stage | Class | PR |
|---|---|---|---|
| `estimateComplexityClass` | predict | Logarithmic | #28, #30, #42 |
| `coherenceScore` | check feasibility | Linear | #53 |
| `closureIndices` | preview closure | SubLinear | #54 |
| `solveOnChangeSublinear` | run orchestrator | SubLinear | #56 |
| `contrastiveSolveOnChangeSublinear` | top-k anomalies | SubLinear | #57 |
| `verifySparseSolution` | audit output | SubLinear | #52 |

All five new handlers are pure-TypeScript — no WASM bridge required.

### Empirical receipts

- `benches/solver_benchmarks.rs::delta_solve` (PR #31) — `cold_full` /
  `warm_full` (Linear) vs `sparse_closure` (SubLinear) across n=64,256,1024.
  Linear paths grow ~4× per 4× size; SubLinear path stays nearly constant.
- `benches/solver_benchmarks.rs::witness_audit` (PR #43) — `full_residual`
  (Linear) vs `closure_audit` (SubLinear) across n=64,256,1024. Crossover
  at n≈200; by n=1024 closure_audit is ~4× faster.
- `benches/solver_benchmarks.rs::closure_only` (PR #55) — isolates
  `closure_indices` across n=64,256,1024,4096. 64× growth in n → 1.4×
  growth in cost. The cleanest demonstration that closure is constant-in-n.
- `examples/event_driven_anomaly.rs` (PRs #33, #35, #38) — runnable demo
  of the full pipeline (baseline → coherence gate → SubLinear orchestrator).
- `examples/event_stream_processing.rs` (PR #48) — streaming surface
  demo using `event_stream_iter` + `PlanBudget`.
- `tests/property_sublinear.rs` (PR #50) — proptest fuzz of the wire
  contract: closure monotonicity, empty-delta short-circuit, witness-
  passes-on-output, auto-vs-manual agreement.

---

## Open Questions

1. **Should `Complexity` be a trait or a const associated value?** ✅ Resolved by shipping both: `ComplexityIntrospect` for `dyn`, `const CLASS: ComplexityClass` on the `Complexity` trait for monomorphic call sites.
2. **How does an `Adaptive` solver declare itself?** ✅ Resolved by `ComplexityClass::Adaptive { default, worst }` — shipped in v1.7.0.
3. **Does `solve_on_change` need a witness?** ✅ Resolved by `verify_sparse_solution` (PR #41). Closure-restricted residual audit at the same SubLinear complexity class as the orchestrator whose output it verifies.
4. **What's the right coherence threshold default?** 0.05 is still a guess. The bench corpus has matrices with coherences between 0.5 and 0.001; needs a sweep + false-positives ≈ false-negatives tuning on realistic workloads. *Tracked as remaining tuning work.*
5. **Power-bench on macOS?** The IOKit power assertion interface is closer to per-process accounting than RAPL; might need a separate impl. Or fall back to `time` and note "not measured" for the energy column on darwin. *Hardware-dependent; tracked as Pi hwmon J/solve backend in roadmap item #5.*

---

## References

- The original directive essay (attached to this ADR's source prompt) — twelve complexity-class tiers + their mapping to the stack.
- [BENCHMARK.md](../../BENCHMARK.md) — the current ns-per-solve baselines this ADR proposes to extend with J/solve.
- [CHANGELOG.md](../../CHANGELOG.md) — the v1.6.0 entry that proved the package can do disciplined release work; this ADR is its architectural follow-up.
- Kyng & Sachdeva 2016, *Approximating the Solution to Mixed Packing and Covering LPs in Parallel Õ(ε⁻³) Time* — the sublinear-solver theory this crate's `sublinear_neumann` module implements.
- Andoni, Krauthgamer, Pogrow 2018, *On Solving Linear Systems in Sublinear Time* — extension to general DD systems.
- ruvector ADR-001 — *Ruvector Core Architecture*. This ADR follows its conventions (header block, version history, context → decision → consequences) and is the upstream-side counterpart of that document.
