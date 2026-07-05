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

## The line this does NOT cross (and what would)

Everything here proves *mechanism and structure*. It does **not** prove the
flywheel turns, because every candidate outcome is a synthetic fixture. Crossing
that line requires **real evaluation**: machine-generated candidates executed
against real tasks by a real agent, scored by the real sandbox — not hand-set
outcomes. That needs an agent runtime, an API budget, and a real frozen task
suite. The meaningful milestone (per the design notes) is the system
autonomously discovering a **second independently-verified improvement that
survives the frozen anchor suite and joins the immutable lineage without human
intervention** — with *real* outcomes. This package builds and verifies the
rails that milestone rides on; it is deliberately not that milestone.

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
- `generation-0/` — the sealed, committed immutable-root fixture
