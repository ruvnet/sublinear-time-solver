/**
 * Shared flywheel logic used by BOTH run-flywheel.mjs (seal) and
 * verify-flywheel.mjs (independent re-execution), so the verifier reproduces
 * the run bit-for-bit on the deterministic parts.
 */
import { createHmac } from 'node:crypto';
import { createCompiler, createRetriever, createEvolutionPipeline } from '@claude-flow/guidance';
import { scorePolicy, HELD_OUT_TASKS } from './policy.mjs';
import { digestBundle } from './bundle.mjs';
import { hashJson } from './hash.mjs';

export const K = 2;
export const MAX_DIVERGENCE = 0.2; // drift limit; also the strictest (canary) stage gate
// A FIXED, non-secret demonstration HMAC key. Committed on purpose so the
// verifier can recompute the proposal signature and prove authenticity.
export const DEMO_SIGNING_KEY = 'ruflo-flywheel-demo-key-not-a-secret';

/** Compile GUIDANCE.md, run both bounded policies, return bundle + measurements. */
export async function compileAndMeasure(src) {
  const bundle = await createCompiler({ autoGenerateIds: true }).compile(src);
  const retriever = createRetriever();
  await retriever.loadBundle(bundle);
  await retriever.indexShards();
  const base = await scorePolicy(retriever, 'baseline', K);
  const cand = await scorePolicy(retriever, 'candidate', K);
  return { bundle, bundleDigest: digestBundle(bundle), base, cand };
}

/** Deterministic golden-trace evaluator cache (async retrieval precomputed). */
export function buildCache(base, cand) {
  const cache = {};
  for (const p of [base, cand]) {
    for (const t of p.perTask) {
      (cache[t.id] ??= {})[p.policy] = {
        traceHash: hashJson(t.selectedRuleIds),
        metrics: { intentPrecision: t.of ? t.matched / t.of : 0 },
        decisions: t.selectedRuleIds,
      };
    }
  }
  return cache;
}

/** Recompute the pipeline's HMAC over a proposal's signed body (see evolution.js). */
export function proposalSignature(proposal, key = DEMO_SIGNING_KEY) {
  const body = {
    proposalId: proposal.proposalId, kind: proposal.kind, title: proposal.title,
    description: proposal.description, author: proposal.author, targetPath: proposal.targetPath,
    diff: proposal.diff, rationale: proposal.rationale, riskAssessment: proposal.riskAssessment,
    createdAt: proposal.createdAt,
  };
  return createHmac('sha256', key).update(JSON.stringify(body)).digest('hex');
}

/** Drive the real EvolutionPipeline through propose→simulate→compare→stage→promote. */
export function driveFlywheel(base, cand) {
  const cache = buildCache(base, cand);
  const evo = createEvolutionPipeline({ signingKey: DEMO_SIGNING_KEY, maxDivergence: MAX_DIVERGENCE });
  const proposal = evo.propose({
    kind: 'policy-update',
    title: 'intent-routed retrieval',
    description: 'Supply the task intent to the shard retriever so its intent boost activates, instead of intent-blind retrieval.',
    author: 'ruflo-flywheel',
    targetPath: 'retrieval.policy',
    diff: { before: { intentRouting: false }, after: { intentRouting: true } },
    rationale: `Held-out intent-precision@${K} rises ${base.intentPrecision.toFixed(3)} -> ${cand.intentPrecision.toFixed(3)} with no regression.`,
    riskAssessment: { level: 'low', factors: ['retrieval-only', 'no code path change', 'reversible'] },
  });
  const sim = evo.simulate(proposal.proposalId, HELD_OUT_TASKS, (t, cfg) => cache[t.id][cfg]);
  const cmp = evo.compare(proposal.proposalId, sim);

  let rollout = null;
  const stageLog = [];
  if (cmp.approved) {
    rollout = evo.stage(proposal.proposalId);
    for (let i = 0; i < rollout.stages.length; i++) {
      const adv = evo.advanceStage(rollout.rolloutId, { divergence: sim.divergenceScore });
      stageLog.push({ stage: rollout.stages[i].name, ...adv });
      if (adv.rolledBack) break;
    }
  }
  const finalProposal = evo.getProposal(proposal.proposalId);
  const finalRollout = rollout ? evo.getRollout(rollout.rolloutId) : null;
  const promoted = finalProposal.status === 'promoted';
  const decision = {
    approved: cmp.approved, reason: cmp.reason,
    divergenceScore: sim.divergenceScore, maxDivergence: MAX_DIVERGENCE,
    metrics: sim.metricsComparison, promoted, rolloutStatus: finalRollout?.status ?? null,
  };
  return { proposal: finalProposal, sim, cmp, stageLog, promoted, decision, decisionHash: hashJson(decision) };
}

export { HELD_OUT_TASKS };
