# Zeptosecond Measurement Validation

**Test, optimize and prove** the smallest time interval ever measured: the
**247 zeptosecond** (247 × 10⁻²¹ s) transit of an X-ray photon across a
hydrogen molecule — Grundmann et al., *"Zeptosecond birth time delay in
molecular photoionization"*, **Science 370**, 339–341 (2020), measured at the
PETRA III source (DESY, Hamburg).

This suite proves the number from first principles, stress-tests the numerics,
optimizes the simulation hot path, and integrates the ruvnet stack
([`ruvector`](https://github.com/ruvnet/ruvector)) for measurement-event
indexing.

## The proof

The measured value is exactly the light-travel time across the H₂ bond:

```
t = R / c = 74.14 pm / 299 792 458 m/s = 247.30 zs      (measured: 247 zs, Δ ≈ 0.12%)
```

Inverse check: `c × 247 zs = 74.05 pm` — the H₂ internuclear distance, recovered
to within 0.13%.

Cross-checks implemented and asserted in [`zeptosecond.test.mjs`](./zeptosecond.test.mjs):

| Check | Result |
|---|---|
| Forward: `R/c` vs measured 247 zs | 247.30 zs, agrees to 0.12% |
| Inverse: `c · 247 zs` vs bond length | 74.05 pm vs 74.14 pm |
| Orientation model `Δt(θ) = R·cos(θ)/c` | max at alignment = full transit; 0 when perpendicular |
| Photoelectron de Broglie wavelength (≈784 eV) | 43.8 pm < 74.14 pm → two-center interference resolvable |
| Energy–time uncertainty `ΔE ≥ ħ/(2·247 zs)` | ≈ 1.33 keV → X-ray regime, hence PETRA III |
| Margolus–Levitin `τ_min = h/(4E)` at 800 eV | ≈ 1292 zs — consistent with the repo's `src/temporal_nexus/quantum/speed_limits.rs`; the 247 zs is a propagation delay read out interferometrically, not a state orthogonalization, so no bound is violated |
| Monte Carlo over isotropic orientations | converges to the analytic `R/(2c) = 123.65 zs` |

## The numerics ("test")

IEEE-754 doubles have ~16 significant digits, so **a 247 zs interval added to a
1 s epoch silently vanishes**:

```js
1.0 + 247e-21 === 1.0   // true — the measurement disappears
```

`ZeptoClock` (BigInt-backed, exact) extends this repo's nanosecond-scheduler
philosophy 12 orders of magnitude further down: 1 s + 247 zs is stored exactly
as `1_000_000_000_000_000_000_247n` zs, and a million stacked 247 zs events
accumulate with zero drift at ~41 M ticks/s.

## The optimization ("optimize")

`benchmark.mjs` compares a naive Monte Carlo estimator (per-event allocation,
`acos`/`cos` round-trip, functional reduction) against an optimized one
(direct `cosθ ~ U[-1,1]` sampling, fused pass, zero allocation). Measured on
this environment (Node 22, 5M events × 5 rounds):

```
naive        best 544.3 ms    9.2 M events/s   mean 123.625 zs
optimized    best 104.6 ms   47.8 M events/s   mean 123.625 zs
speedup: 5.20x
```

A dedicated test proves the optimization changes **cost, not physics**: under
identical randomness both estimators return the same statistic to < 1e-9
relative difference, and both converge to the analytic 123.65 zs expectation.

## The ruvnet stack ("prove with ruvector & friends")

- **[`ruvector`](https://www.npmjs.com/package/ruvector)** (native
  SIMD/Rust backend): [`ruvector-integration.test.mjs`](./ruvector-integration.test.mjs)
  indexes simulated measurement events as feature vectors
  `[cosθ, sinθ, |cosθ|]` in a `VectorDb` and proves that nearest-neighbor
  retrieval is physics-consistent — every kNN neighbor of a probe event agrees
  with it in birth-time delay to < 25 zs (over a 0–247 zs range), and aligned
  vs perpendicular molecular populations separate perfectly. The suite skips
  gracefully if `ruvector` isn't installed. Note: `VectorDb` persists to
  `./ruvector.db` and shares state across instances by default; the tests
  isolate via per-test `storagePath`.
- **`temporal-lead-solver` / `strange-loops` / `@ruvnet/bmssp`** are already
  first-class dependencies of this repo; the Margolus–Levitin cross-check
  mirrors `MargolousLevitinValidator` from `src/temporal_nexus/quantum/speed_limits.rs`
  so JS and Rust validation stay in lockstep.
- **[`agentic-flow`](https://www.npmjs.com/package/agentic-flow)**: this suite
  is deliberately flat ESM with zero required dependencies so agent swarms can
  run it as a verification step (`npx agentic-flow` + `npm test` here). It is
  not a hard dependency — its install footprint is far larger than the suite
  itself.

## Run it

```bash
cd validation/zeptosecond
node --test .              # 19 core physics/numerics tests, no deps
npm install && npm test    # + 2 ruvector integration tests (21 total)
npm run bench              # naive vs optimized Monte Carlo + ZeptoClock throughput
```

## Files

- `zeptosecond-physics.mjs` — constants (CODATA 2018), transit-time model,
  orientation-dependent birth-time delay, quantum-limit bounds, `ZeptoClock`,
  Monte Carlo estimators, event-vector generator
- `zeptosecond.test.mjs` — the proof suite (node:test)
- `ruvector-integration.test.mjs` — ruvnet vector-database integration proof
- `benchmark.mjs` — optimization benchmark
