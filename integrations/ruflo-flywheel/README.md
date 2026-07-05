# ruflo guidance flywheel — bounded retrieval policy evolution, with receipts

This integrates the **self-optimizing flywheel** described in
[the ruflo 3.24.0 gist](https://gist.github.com/ruvnet/f8e2851fd307df5d5de7b5c70c37fa0c):

> *A bounded retrieval policy can now evolve, prove it beat the previous policy,
> and leave a replayable receipt for every promotion … git for executable agent
> policy decisions.*

It runs that flywheel on **this repo's own contributor guidance** and seals a
receipt you can independently replay.

## Honest correction first

An earlier note in this branch concluded the flywheel machinery "appears absent
from the npm build" because the words *flywheel / receipt / canary* are not
top-level `ruflo` commands. **That was wrong.** The machinery is shipped — it
lives inside `@claude-flow/guidance` (a transitive dependency of `ruflo`'s
`@claude-flow/cli`), surfaced through the `ruflo guidance` control plane:

| gist concept | real shipped module (`@claude-flow/guidance`) |
| --- | --- |
| evolve a policy | `EvolutionPipeline` (propose → simulate → compare → stage → promote/rollback) |
| signed champion | `propose()` HMAC-SHA256 signature; `TrustAccumulator` |
| evidence gate | `compare()` (no-regression + drift limit); `EnforcementGates` |
| staged / canary safety | `stage()` / `advanceStage()` canary → partial → full with per-stage drift gates |
| deterministic replay | `ProofChain`; golden-trace `simulate()` |
| receipt coverage / lineage | `ArtifactLedger`, `PersistentLedger`, `EventStore` |
| fail-closed continue | `ContinueGate` |

This package drives the **real** `EvolutionPipeline` — no reimplementation.

### Part #1 — the signed champion, captured live

The gist describes *two* shipped things. Part #2 (the opt-in flywheel) is what
this package drives. Part #1 — *"a signed retrieval config champion that auto
applies on upgrade when authenticity and compatibility pass"* — manifested on
its own: running `ruflo guidance` against this repo auto-wrote a signed champion
to `.claude/proven-config.json`. The exact artifact ruflo adopted is captured
verbatim under [`proven-config/`](./proven-config/) as evidence (I did **not**
author its numbers — ruflo did):

```json
{ "schema": "ruflo.proven-config/v1",
  "policy": { "alpha": 0.3, "subjectWeight": 1, "mmrLambda": 0.5, "bodyWeight": 1.5, "typePenaltyFactor": 0.5 },
  "compatibility": { "ruflo": ">=3.24.0" },
  "receipt": { "heldOutDelta": 0.0738, "redblue": "PASS", "drift": 0,
               "canary": { "rollbackRate": 0, "costPerTask": 0 }, "receiptCoverage": 1 } }
```

That receipt is exactly the gate the gist lists — held-out delta, red/blue
significance, drift, canary safety, full receipt coverage — for the champion
ruflo signed and adopted. The live `.claude/` copies are gitignored (they are
ruflo's runtime state, left in place for it); the captured copy here is the
committed record.

## What evolves

The **bounded retrieval policy**: given a task, which guidance shards does the
control plane inject? `GUIDANCE.md` compiles (via the real `GuidanceCompiler`)
into a bundle of intent-tagged shards. Two policies differ by one knob:

- **baseline — intent-blind.** Passes `intent: 'general'`; no shard matches, so
  the retriever's intent boost (`+0.15` per matching shard, see `scoreShards`)
  never fires. Ranking falls back to the default `HashEmbeddingProvider`, which
  the package itself documents as test-only with *"no real semantic meaning."*
- **candidate — intent-routed.** Passes the task's intent (as an agent would
  from an issue label), activating the real `+0.15` boost.

Metric: **intent-precision@2** on a held-out task suite — of the 2 shards
injected, the fraction whose rule intent matches the task's ground-truth intent.
A narrow, deterministic quality proxy (not human relevance judgment). The boost
competes with embedding similarity, so the gain is *measured, not tautological*.

## Result

```
compiled: 8 shards, 2 constitution rules
held-out intent-precision@2: baseline 0.250 -> candidate 1.000
gate: divergence 0.0375 (limit 0.2), approved=true — no metric regressions
PROMOTED through canary -> partial -> full
```

The real `EvolutionPipeline` **promoted** the candidate: held-out precision rose
0.25 → 1.00, behavioural drift (0.0375) stayed inside the drift limit and the
strictest canary stage gate (0.2), and no metric regressed. The promotion is
sealed in `flywheel/RECEIPT.json` with the signed proposal, simulation, gate
decision, staged-rollout log, and a `guidance-policy-lineage`.

## Independent verification

`verify-flywheel.mjs` trusts none of the sealed output. From `GUIDANCE.md` it
recompiles the bundle (reproduces the digest), re-runs both policies on the real
retriever (reproduces 0.25 and 1.00), re-drives the real pipeline (reproduces
divergence, the approve verdict, the promotion, and the decision hash
bit-for-bit), and recomputes the proposal's HMAC to prove it is authentic and
untampered. **12/12 checks pass.** The only non-reproduced fields are the
proposal's UUID and timestamp — which the gate never acts on.

```bash
npm install            # pulls @claude-flow/guidance (ruflo's flywheel module)
npm run flywheel       # compile -> measure -> propose -> simulate -> gate -> stage -> promote
npm run verify:flywheel  # re-execute everything and reproduce the promotion
npm run prove:flywheel   # both
```

## Defaults (matching the gist)

- **cost `$0`** — no LLM, no paid API in the loop.
- **network `none`** — local `HashEmbeddingProvider`; nothing leaves the process.
- **fail-closed** — any metric regression or drift over the limit *rejects*; a
  stage whose divergence exceeds its threshold auto-rolls-back.
- **mutation explicit** — nothing evolves unless you run the flywheel.

## Honest scope

- The `HashEmbeddingProvider` is test-only, so the *baseline* ranking is
  deliberately weak; the win is attributable to the real intent-boost mechanism,
  not to semantic embeddings (which would need a model-backed provider).
- Ground-truth intents are author-assigned per task (a deterministic oracle),
  not human relevance judgments — stated plainly so the metric isn't oversold.
- This proves one promotion end-to-end on the real pipeline. Turning it into a
  *compounding* loop (each promoted policy becoming the next round's baseline
  across many rounds) is the same next-milestone gap called out in
  `../metaharness-proof` — the rails are here; sustained rounds are future work.

## Files

- `GUIDANCE.md` — this repo's guidance, compiled into the policy bundle
- `src/policy.mjs` — the two bounded policies + intent-precision metric
- `src/flywheel-core.mjs` — shared compile/measure/drive logic (run & verify share it)
- `src/bundle.mjs` — timestamp-free deterministic bundle digest
- `src/hash.mjs` — canonical JSON hashing
- `run-flywheel.mjs` — drives the real pipeline, seals `flywheel/`
- `verify-flywheel.mjs` — independent re-execution (the acceptance test)
- `flywheel/` — sealed receipt, policies, bundle projection, lineage
