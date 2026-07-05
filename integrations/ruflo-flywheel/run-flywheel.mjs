#!/usr/bin/env node
/**
 * RUFLO GUIDANCE FLYWHEEL — evolve a bounded retrieval policy through the REAL
 * `@claude-flow/guidance` evolution pipeline that ruflo 3.24.0 ships, and seal
 * a replayable receipt for the promotion.
 *
 * This is the gist's flywheel, applied to THIS repo's guidance:
 *   "A bounded retrieval policy can now evolve, prove it beat the previous
 *    policy, and leave a replayable receipt for every promotion."
 *
 * Pipeline (all real modules, no reimplementation):
 *   1. compile   — GuidanceCompiler turns GUIDANCE.md into a policy bundle
 *                  (constitution + intent-tagged shards + manifest)
 *   2. measure   — the real ShardRetriever runs two bounded policies over a
 *                  held-out task suite; intent-precision@k is measured
 *   3. propose   — EvolutionPipeline.propose signs the policy change (HMAC)
 *   4. simulate  — golden traces replayed through baseline vs candidate
 *   5. compare   — the gate approves only on no-regression AND drift <= limit
 *   6. stage     — canary -> partial -> full rollout with per-stage drift gates
 *   7. promote   — all stages pass => auto-promote; receipt + lineage sealed
 *
 * Defaults match the gist: $0 cost, no network (local HashEmbeddingProvider),
 * fail-closed (any regression or excess drift rejects), mutation is explicit.
 *
 * Run: node run-flywheel.mjs   (writes ./flywheel)
 */
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { compileAndMeasure, driveFlywheel, K, MAX_DIVERGENCE, HELD_OUT_TASKS } from './src/flywheel-core.mjs';
import { hashJson } from './src/hash.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'flywheel');
const write = (rel, v) => { const p = join(OUT, rel); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, typeof v === 'string' ? v : JSON.stringify(v, null, 2) + '\n'); };

const src = readFileSync(join(HERE, 'GUIDANCE.md'), 'utf8');
const { bundle, bundleDigest, base, cand } = await compileAndMeasure(src);
console.log(`compiled: ${bundle.shards.length} shards, ${bundle.manifest.constitutionRules} constitution rules  (digest ${bundleDigest.slice(0, 12)}…)`);
console.log(`held-out intent-precision@${K}: baseline ${base.intentPrecision.toFixed(3)} -> candidate ${cand.intentPrecision.toFixed(3)}`);

const { proposal, sim, cmp, stageLog, promoted, decision, decisionHash } = driveFlywheel(base, cand);
console.log(`gate: divergence ${sim.divergenceScore.toFixed(4)} (limit ${MAX_DIVERGENCE}), approved=${cmp.approved} — ${cmp.reason}`);
console.log(`${promoted ? 'PROMOTED' : 'NOT promoted'}: proposal status = ${proposal.status}`);

const receipt = {
  kind: 'ruflo-guidance-flywheel-receipt',
  target: 'ruvnet/sublinear-time-solver',
  flywheel: 'real @claude-flow/guidance EvolutionPipeline shipped by ruflo 3.24.0 (propose→simulate→compare→stage→promote)',
  correction: "the flywheel machinery IS shipped in npm — inside @claude-flow/guidance (ruflo's transitive dep): EvolutionPipeline, ProofChain, ArtifactLedger, EnforcementGates, TrustAccumulator, ContinueGate. It is not a top-level `ruflo flywheel` command; it is the guidance control plane + evolution module.",
  defaults: { costUsd: 0, network: 'none (local HashEmbeddingProvider)', failClosed: true, mutation: 'explicit (this run opted in)' },
  bundleDigest,
  policy: { knob: 'intent routing', baseline: 'intent-blind (intent=general)', candidate: 'intent-routed (intent=task)' },
  heldOut: {
    k: K, tasks: HELD_OUT_TASKS.length,
    baselineIntentPrecision: base.intentPrecision,
    candidateIntentPrecision: cand.intentPrecision,
    improvement: cand.intentPrecision - base.intentPrecision,
    noRegression: cand.intentPrecision >= base.intentPrecision,
  },
  proposal: {
    proposalId: proposal.proposalId, kind: proposal.kind, title: proposal.title, description: proposal.description,
    author: proposal.author, targetPath: proposal.targetPath, diff: proposal.diff, rationale: proposal.rationale,
    riskAssessment: proposal.riskAssessment, createdAt: proposal.createdAt, status: proposal.status,
    signature: proposal.signature, signingKeyRef: 'DEMO_SIGNING_KEY (committed non-secret; see src/flywheel-core.mjs)',
  },
  simulation: {
    baselineTraceHash: sim.baselineTraceHash, candidateTraceHash: sim.candidateTraceHash,
    divergenceScore: sim.divergenceScore, decisionDiffs: sim.decisionDiffs.length,
  },
  decision,
  decisionHash,
  stageLog,
  reproducible: 'bundle digest, held-out precision, divergence, gate verdict, promotion, and the proposal signature are reproduced by verify-flywheel.mjs. proposalId + createdAt are non-deterministic (UUID/clock) and are excluded from the reproducible decision.',
};
write('RECEIPT.json', receipt);
write('policies.json', { baseline: base, candidate: cand });
write('bundle-projection.json', { digest: bundleDigest, shards: bundle.shards.map((s) => ({ id: s.rule.id, intents: s.rule.intents, text: s.rule.text })) });
write('lineage.json', { kind: 'guidance-policy-lineage', chain: [
  { id: 'baseline', policy: 'intent-blind', intentPrecision: base.intentPrecision },
  ...(promoted ? [{ id: 'candidate', parent: 'baseline', policy: 'intent-routed', intentPrecision: cand.intentPrecision, promotedBy: 'EvolutionPipeline', decisionHash }] : []),
] });

console.log(`\nSealed ./flywheel — receipt, policies, bundle projection, lineage.`);
console.log(`  held-out precision ${base.intentPrecision.toFixed(3)} -> ${cand.intentPrecision.toFixed(3)}, drift ${sim.divergenceScore.toFixed(4)} <= ${MAX_DIVERGENCE}, ${promoted ? 'promoted' : 'held'}.`);
