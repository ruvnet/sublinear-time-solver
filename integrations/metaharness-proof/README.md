# single-round proof-of-mechanism

A single synthetic evolve round sealed into an independently-verifiable
**generation-0** receipt bundle, built on the real
[`@metaharness/darwin`](https://www.npmjs.com/package/metaharness) promotion
gate (`hashTasks`, `scoreBenchmark`, `decidePromotion` — ADR-076).

**This is exactly, and only, a single-round proof-of-mechanism.**
It is **not** flywheel proof, **not** compounding learning, **not** production
learning. The candidate, its mutation, and its per-task outcomes are transparent
synthetic fixtures. Only the *mechanism* is real. Do not use its (synthetic)
promotion as marketing evidence.

## Purpose (narrow, on purpose)

- **prove gate wiring** — the real ADR-076 `decidePromotion` decides, over real hashed inputs
- **prove receipt persistence** — the full decision + hashes are sealed to disk, not just logged
- **prove SHADOW registration** — the gate-promoted candidate is registered SHADOW
- **prove no auto-serve path** — the serving manifest is empty; gate-promoted ≠ served

## The seven artifacts

Every one is emitted into `generation-0/` (the run is aborted as unfit for
purpose if any is missing):

| Artifact | Where | Source |
|---|---|---|
| input holdout hash | `receipts/holdout_hash.json` | `bench.hashTasks(suite)` (library) |
| baseline manifest hash | `receipts/manifest_hashes.json` | SHA-256 content manifest of `variants/baseline` |
| candidate manifest hash | `receipts/manifest_hashes.json` | SHA-256 content manifest of `variants/candidate` |
| `meetsPromotionRule` version | `receipts/promotion_rule.json` | `adr-076@<darwin version>` |
| decision receipt | `receipts/decision_receipt.json` | real `bench.decidePromotion(...)` output |
| SHADOW registration id | `receipts/shadow_registration.json` | derived from candidate manifest + holdout hash |
| cost receipt | `receipts/cost_receipt.json` | metered per-task cost proxy (never self-reported) |

`RECEIPT.json` is the bundle index: the seven artifact fingerprints and a single
`bundleRootHash` over them. **Generation zero is the immutable root of the
evolution graph** — it never changes; replay starts here.

## Acceptance test

`verify-receipt.mjs` reads *only* the sealed bundle, recomputes every hash,
**re-runs the real `decidePromotion` on the sealed raw outcomes**, and asserts
the sealed decision reproduces bit-for-bit — then prints *why* the candidate
passed or failed from the receipt alone. It never reads a service log or trusts
a producer-claimed value.

Verified properties (see the commands below):
- **deterministic** — two runs produce an identical `bundleRootHash`
- **portable** — the bundle verifies from any directory / a copy
- **tamper-evident** — mutating any sealed byte makes `verify` fail (it is not a rubber stamp)

## Run

```bash
cd integrations/metaharness-proof
npm install
npm run proof     # seal generation-0/
npm test          # acceptance test: independently verify the bundle
npm run prove     # proof + verify in one step
```

## Compounding lineage (generation 1)

`run-lineage.mjs` appends **generation-1** onto the immutable generation-0 root,
making the "version control for operating policies" structure concrete and
independently verifiable. `verify-lineage.mjs` verifies the whole chain.

What is **real** at gen-1:
- **autonomous candidate generation** — the candidate is produced by the real
  `DeterministicMutator` (machine-generated, not hand-authored); the receipt
  records the mutated surface and summary
- **frozen anchor suite** — gen-1 is scored against the *same* holdout suite as
  gen-0; the run aborts unless its hash equals gen-0's `inputHoldoutHash`
- **hash-chained parent link** — `gen-1.parent === gen-0.bundleRootHash`, and
  gen-1's own root hash binds that parent in (append-only, tamper-evident)
- the real ADR-076 gate and full receipt bundle, plus `lineage.json` (the chain index)

What is still **synthetic** — the honest boundary: the per-task **outcomes**.
gen-1 proves the compounding *structure* and *autonomous mutation*, **not** that
the improvement is real. See the milestone below.

```bash
npm run lineage          # append generation-1
npm run verify:lineage   # verify the full chain (also `npm test`)
npm run prove:lineage    # proof + lineage + chain verify, end to end
```

## Knowledge base — mutation effectiveness & regression ancestry (cohort)

`run-cohort.mjs` turns the lineage from an audit trail into a **knowledge base**.
From the immutable gen-0 root it autonomously generates a cohort of
machine-mutated candidates (real `DeterministicMutator`, one per seed), gates
each against the frozen anchor suite, and derives two analytics:

- **mutation-effectiveness** — per mutation *surface*: `{attempts, promotions,
  meanDelta}`, sorted by payoff. This is the evidence a future optimizer would
  use to bias mutation toward high-payoff classes (meta-learning grounded in
  evidence, not intuition).
- **regression-ancestry** — every *rejected* candidate mapped to the gate clause
  it failed and its ancestor: why a design direction was abandoned.

`verify-cohort.mjs` re-runs every node's gate from sealed inputs and
**recomputes both analytics from the node receipts**, asserting they match — so
the knowledge base is itself verifiable, not a trusted summary. Deterministic.

```bash
npm run cohort           # generate + gate the machine-mutated cohort
npm run verify:cohort    # recompute analytics from node receipts and verify
```

Example (synthetic outcomes keyed by surface, so payoff differs by class):

```
retryPolicy     attempts=1 promotions=1 meanΔ=0.148
toolPolicy      attempts=1 promotions=1 meanΔ=0.140
contextBuilder  attempts=1 promotions=1 meanΔ=0.132
reviewer        attempts=4 promotions=4 meanΔ=0.094
planner         attempts=1 promotions=0 meanΔ=-0.606   ← regressed, abandoned
```

## Plateau detection (statistical, not intuition)

`run-plateau.mjs` builds a real-gated multi-generation history with diminishing
returns and applies a rigorous plateau detector (`src/plateau.mjs`, design item
#2). A plateau is declared only when ALL three hold over a rolling window:

1. median per-generation improvement < ε
2. promotion rate over the window < max
3. candidate-score variance shrinking

That separates a **local optimum** (all three) from a **noisy benchmark**
(flat improvement but non-shrinking variance) or **still-improving**
(promotions continue) — the classification is emitted, not just a boolean.

`verify-plateau.mjs` re-gates every candidate, rebuilds the per-generation
stats, and recomputes the detector — asserting the sealed history and verdict
reproduce. Example run:

```
gen  bestΔ    promo/att  variance
0    0.0760   3/3        4.27e-5
5    0.0008   0/3        1.07e-7
final verdict: plateau=true  classification=local-optimum  (first declared at gen 4)
```

```bash
npm run plateau && npm run verify:plateau
```

## REAL evaluation on the sublinear repo (synthetic → real, crossed here)

`run-real-eval.mts` is the one part of this package whose outcomes are **not
fixtures**. It uses `ruvnet/sublinear-time-solver` as the task and evaluates by
**executing the repo's real solver**:

- **task**: solve 8 real diagonally-dominant systems `A x = b` to residual < 1e-6
- **candidate**: the repo's real `SublinearSolver` (fixed Neumann)
- **baseline**: the pre-fix buggy Neumann (missing sign — converges to `A x = D b`)
- **score**: `solved := measured ‖A x − b‖ < 1e-6`, computed with the repo's real
  `MatrixOperations` — deterministic and reproducible
- **gate**: the real ADR-076 `decidePromotion`

Measured result (real, not fixtured):

```
baseline (buggy Neumann)  verified-solves: 0/8   median residual 1.5e+01
candidate (repo solver)   verified-solves: 8/8   median residual 6.7e-13
gate promote: true — verified-solve rate 0 → 1, statistically real
```

`verify-real-eval.mts` is the strongest verifier in this package: it
**re-executes both solvers** on the sealed systems, recomputes the residuals and
the gate decision, and asserts they reproduce (9/9). It trusts no
producer-claimed number — it regenerates them by running real code.

```bash
npm run real-eval          # measure by running the real solver
npm run verify:real-eval   # re-execute the solver and reproduce the decision
```

**Honest scope.** This makes *evaluation* real: real code, real systems, real
measured correctness, gated for real. The two variants were authored by the
operator, so it is **real evaluation, not autonomous discovery**. The remaining
step to a turning flywheel is an autonomous mutator proposing the candidate
(instead of a human) and a real agent executing it — the `evaluate()` seam in
`src/real-task.mts` is exactly where that plugs in.

## Autonomous discovery on the real solver (the line, crossed)

`run-autonomous.mts` closes the loop: **no human authors the winning change.**
The system searches a configuration space over the real `SublinearSolver`
(`method` / `maxIterations` / `epsilon`), and real measured evaluation selects
the winner:

1. a seeded **mutation operator proposes** a cohort of candidate configs (it
   does not know which will win)
2. **each config is evaluated by running the real solver** on 8 real linear
   systems of varying difficulty — `solved := measured ‖A x − b‖ < 1e-6`
3. the real **ADR-076 gate selects** the best config that beats the champion

The champion is under-powered (`maxIterations:3` → solves 0/8). From 12
machine-proposed configs, the gate discovered:

```
DISCOVERED (by the gate, not by a human):
  {"method":"neumann","maxIterations":75,"epsilon":1e-9}  →  solves 8/8
  meanΔ=0.700, lower95=0.700 (statistically real), verified-solve rate 0 → 1
```

`verify-autonomous.mts` **re-executes the entire pipeline** from sealed inputs —
re-derives the proposals from the seed (proving they weren't cherry-picked),
re-runs the real solver on every config, re-gates, and recomputes the gate's
selection — asserting the discovery reproduces bit-for-bit, that it was
machine-proposed (no human injected it), and that it genuinely solves more than
the champion. 12/12 checks pass.

```bash
npm run autonomous          # search: propose → real-solver eval → gate select
npm run verify:autonomous   # re-execute the pipeline and reproduce the discovery
```

**Honest scope.** This is autonomous discovery of the *configuration-search*
kind (like AutoML): the operator and search space are designed, but the winning
config is not authored by a human — it emerges from measured evaluation and the
gate selects it. It is not yet arbitrary code synthesis by an LLM agent. But it
is a genuine, validated, self-contained autonomous-improvement loop on the real
sublinear solver: the system proposes, real execution measures, the gate
decides, and the result reproduces under independent re-execution.

## Novel discovery — open-ended LLM code synthesis (the frontier, crossed)

`run-novel.mjs` crosses the configuration-search frontier: the proposer is a
real LLM (`claude -p`) that emits **arbitrary solver code**, not a point in a
designed grid. The champion is plain **Jacobi** (`novel/genome/champion.mjs`),
which *diverges* on symmetric-positive-definite systems that are not
diagonally dominant — so no amount of "more iterations" can win. Only a genuine
change of numerical **method** can, and the proposer is not told which method.

The loop:

1. the champion genome is run in a **sandboxed subprocess** (hard timeout, no
   host access) on 8 SPD-non-dominant systems → it solves **0/8**
2. `claude -p` is asked to improve it; each proposal passes a **static safety
   gate** (pure numeric ESM only — no imports, I/O, eval, or template literals)
   before it is allowed to run
3. each candidate genome is executed the same sandboxed way; `solved := measured
   ‖A x − b‖ < 1e-6`
4. the real **ADR-076 gate** selects any candidate that genuinely beats the
   champion; the winner is sealed with a replayable receipt + lineage

What Claude autonomously discovered (no human authored it):

```
DISCOVERED (proposed by claude -p, selected by the gate):
  a preconditioned BiCGSTAB solver  →  solves 8/8  (champion Jacobi: 0/8)
  It recognised that Jacobi diverges on SPD-non-dominant systems and that
  iteration count cannot fix divergence, so it changed the METHOD to a
  Krylov-subspace solver with a Jacobi preconditioner.
```

**Honest reproducibility boundary.** LLM output is non-deterministic — the
*proposal* is not reproducible, and the receipt says so plainly. What **is**
reproducible, and what `verify-novel.mjs` checks by **re-execution**, is the
discovered *code* and its measured superiority: it re-hashes the sealed genome,
re-runs both champion and discovered solver in the sandbox, reproduces the
solve counts (0/8 vs 8/8), reproduces the gate decision bit-for-bit
(`hashJson(decision) === receipt.discovery.decisionHash`), confirms the
discovery genuinely solves more, and confirms it passed the static safety gate.
9/9 checks pass. The claim is narrow and true: *the discovered solver works, by
re-running it* — regardless of how it was proposed.

```bash
npm run novel          # claude -p proposes arbitrary solvers → sandbox eval → gate
npm run verify:novel   # re-execute the sealed genomes and reproduce the discovery
```

## What is proven, and the frontier that remains

Proven and independently verifiable (re-execution, not trust):
- **mechanism** — 7 artifacts, gate, receipts, shadow, no-serve
- **structure** — immutable hash-chained lineage, mutation-effectiveness &
  regression-ancestry knowledge base, statistical plateau detection
- **real evaluation** — the repo's real solver run on real systems, measured
- **autonomous discovery** — the system proposes configs, real execution
  measures, the gate selects a genuinely-better one the operator did not author
- **novel discovery** — a real LLM proposes *arbitrary* solver code; a genuinely
  new algorithm (preconditioned BiCGSTAB) is discovered, sandboxed, gate-selected,
  and its superiority reproduced by independent re-execution

The honest remaining boundary is no longer *what* can be discovered — arbitrary
code now can be — but *reproducibility of the proposal itself*: because the
generator is a stochastic LLM, the discovery is validated by re-executing the
sealed code, not by replaying the generation. Turning single-round novel
discovery into *compounding* learning (a discovered solver becoming the next
round's champion, across many rounds, each surviving a frozen anchor suite) is
the next milestone — the loop harness is built; it needs sustained rounds and an
API budget to run them.

## What comes next (not in this package)

- **F-P1 / F-P2** consume `generation-0/` as a frozen fixture.
- **A-P3b** is what turns this from a one-round mechanism proof into real,
  compounding learning. The meaningful milestone is not "generation 1" — it is
  the system autonomously discovering a *second* independently-verified
  improvement that survives a frozen anchor suite and joins the immutable
  lineage without human intervention. This package only proves the plumbing that
  such a milestone would ride on.

## Files

- `run-proof.mjs` — producer: drives one synthetic round through the real gate, seals `generation-0/`
- `verify-receipt.mjs` — independent verifier (the acceptance test)
- `src/round.mjs` — the synthetic fixture (holdout suite, variant policies, per-task outcomes)
- `src/score-map.mjs` — the shared raw-outcome → `BenchmarkResult` scoring pipeline (real `scoreBenchmark`)
- `src/hash.mjs` — deterministic content/JSON hashing
- `run-novel.mjs` / `verify-novel.mjs` — open-ended LLM discovery engine + re-execution validator
- `src/llm-proposer.mjs` — `claude -p` proposer + static safety gate (`isSafe`)
- `src/genome-runner.mjs` — sandboxed subprocess evaluator for one solver genome
- `novel/genome/champion.mjs` — the modest Jacobi champion the LLM improves on
- `novel/` — sealed discovery: `RECEIPT.json`, `genome/discovered.mjs`, `proposals/`, `measured/`, `lineage.json`
- `generation-0/` — the sealed, committed immutable-root fixture
