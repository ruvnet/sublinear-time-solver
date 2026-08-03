#!/usr/bin/env node
/**
 * Turn the lineage into a knowledge base, not just an audit trail.
 *
 * From the immutable generation-0 root, autonomously generate a COHORT of
 * machine-mutated candidates (real DeterministicMutator, one per seed), gate
 * each against the frozen anchor suite with the real ADR-076 gate, and derive
 * two VERIFIABLE analytics over the sealed nodes:
 *   - mutation-effectiveness: per-surface {attempts, promotions, meanDelta}
 *     (so the optimizer can later bias toward high-payoff mutation classes)
 *   - regression-ancestry: each rejected candidate -> the clause it failed ->
 *     its ancestor (why a design direction was abandoned)
 *
 * REAL: candidate generation (mutator), the gate, the hashing, the DAG shape,
 * and the analytics (recomputed by the verifier from node receipts).
 * SYNTHETIC (labeled): per-task outcomes, here keyed by mutation surface so
 * different mutation classes show different payoff — see README's boundary.
 *
 * Run: node run-cohort.mjs   (writes ./cohort)
 */
import { mkdirSync, writeFileSync, rmSync, readFileSync, cpSync, mkdtempSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { bench, generateBaselineHarness, createChildVariant, DeterministicMutator } from '@metaharness/darwin';

import { manifestOfDir, hashJson, sha256 } from './src/hash.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const GEN0 = join(HERE, 'generation-0');
const OUT = join(HERE, 'cohort');
const SEEDS = [43, 44, 45, 46, 47, 48, 49, 50];
const darwinEntry = fileURLToPath(import.meta.resolve('@metaharness/darwin'));
const darwinVersion = JSON.parse(readFileSync(join(dirname(dirname(darwinEntry)), 'package.json'), 'utf8')).version;

const read = (p) => JSON.parse(readFileSync(p, 'utf8'));
const write = (rel, value) => {
  const p = join(OUT, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, typeof value === 'string' ? value : JSON.stringify(value, null, 2) + '\n');
};

// Synthetic outcome profile PER MUTATION SURFACE. Deterministic; clearly a
// fixture. Efficiency-improving surfaces promote; neutral ones fail the
// significance clause; the risky one regresses. This is what lets the
// effectiveness rollup show differential payoff by mutation class.
const SURFACE_PROFILE = {
  retryPolicy: { cost: 0.006, dur: 200, regressionPassed: true },
  toolPolicy: { cost: 0.008, dur: 400, regressionPassed: true },
  contextBuilder: { cost: 0.01, dur: 600, regressionPassed: true },
  reviewer: { cost: 0.02, dur: 1500, regressionPassed: true },
  scorePolicy: { cost: 0.039, dur: 4100, regressionPassed: true }, // ~neutral -> reject
  memoryPolicy: { cost: 0.04, dur: 4200, regressionPassed: true }, // ~neutral -> reject
  planner: { cost: 0.02, dur: 1500, regressionPassed: false }, // regresses -> reject
};
const BASELINE = { cost: 0.04, dur: 4200, regressionPassed: true };

const outcomes = (suite, prof) =>
  suite.map((task) => ({
    taskId: task.id,
    publicTestPassed: true, hiddenTestPassed: true, regressionPassed: prof.regressionPassed,
    safetyViolations: [], blockedFileTouches: [], hallucinatedFileRefs: false,
    costUsd: prof.cost, maxCostUsd: task.maxCostUsd, durationMs: prof.dur, timeoutMs: task.timeoutMs,
  }));

// --- Load the immutable root + frozen anchor --------------------------------
const gen0 = read(join(GEN0, 'RECEIPT.json'));
const anchorSuite = read(join(GEN0, 'holdout/suite.json'));
const anchorHash = hashTasks(anchorSuite);
if (anchorHash !== gen0.artifacts.inputHoldoutHash) { console.error('ABORT: anchor not frozen'); process.exit(1); }

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

const workRoot = mkdtempSync(join(tmpdir(), 'mh-cohort-'));
const profile = { root: workRoot, packageManager: 'npm', testCommand: 'true', sourceFiles: ['retry_policy.ts'], riskFiles: ['.env'], summary: 'cohort' };
const champion = await generateBaselineHarness(profile, workRoot);

// Score the shared parent (champion) once against the frozen anchor. Seal its
// raw outcomes so the verifier can reconstruct parentResults independently.
const championRaw = outcomes(anchorSuite, BASELINE);
write('champion.raw.json', championRaw);
const parentResults = scoreRawOutcomes('champion', gen0.bundleRootHash, championRaw);

const nodes = [];
for (let i = 0; i < SEEDS.length; i++) {
  const seed = SEEDS[i];
  const child = await createChildVariant(champion, workRoot, 1, i, new DeterministicMutator(), seed);
  const surface = child.mutationSurface;
  const prof = SURFACE_PROFILE[surface] ?? BASELINE;

  const nodeDir = `nodes/${child.id}`;
  cpSync(child.dir, join(OUT, nodeDir, 'variant'), { recursive: true });
  const manifest = manifestOfDir(join(OUT, nodeDir, 'variant'));
  const raw = outcomes(anchorSuite, prof);
  write(`${nodeDir}/candidate.raw.json`, raw);
  const childResults = scoreRawOutcomes(child.id, 'champion', raw);
  const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed });

  const receipt = {
    kind: 'cohort-node', id: child.id, parent: gen0.bundleRootHash, seed,
    frozenAnchorHash: anchorHash,
    mutation: { surface, summary: child.mutationSummary, mutator: 'DeterministicMutator' },
    candidateManifestHash: manifest.manifestHash,
    decision, // full ADR-076 decision
    promote: decision.promote,
    meetsPromotionRule: `adr-076@${darwinVersion}`,
  };
  write(`${nodeDir}/RECEIPT.json`, receipt);
  nodes.push(receipt);
}

// --- Derived analytic 1: mutation-effectiveness (per surface) ----------------
const eff = {};
for (const n of nodes) {
  const s = n.mutation.surface;
  eff[s] ??= { surface: s, attempts: 0, promotions: 0, deltaSum: 0 };
  eff[s].attempts += 1;
  if (n.promote) eff[s].promotions += 1;
  eff[s].deltaSum += n.decision.meanDelta;
}
const mutationEffectiveness = {
  kind: 'mutation-effectiveness',
  bySurface: Object.values(eff)
    .map((e) => ({ surface: e.surface, attempts: e.attempts, promotions: e.promotions, meanDelta: Number((e.deltaSum / e.attempts).toFixed(6)) }))
    .sort((a, b) => b.meanDelta - a.meanDelta),
  note: 'Evidence for biasing future mutation toward high-payoff surfaces.',
};

// --- Derived analytic 2: regression-ancestry (rejects) -----------------------
const regressionAncestry = {
  kind: 'regression-ancestry',
  rejected: nodes.filter((n) => !n.promote).map((n) => ({
    id: n.id,
    surface: n.mutation.surface,
    failedClauses: n.decision.reasons, // the clause(s) that blocked promotion
    ancestor: n.parent,
  })),
};

// --- DAG index ---------------------------------------------------------------
const dag = {
  kind: 'dag',
  root: gen0.bundleRootHash,
  anchorHash,
  nodes: nodes.map((n) => ({ id: n.id, parent: n.parent, surface: n.mutation.surface, promote: n.promote, manifest: n.candidateManifestHash })),
  edges: nodes.map((n) => ({ from: 'root', to: n.id })),
};

write('mutation-effectiveness.json', mutationEffectiveness);
write('regression-ancestry.json', regressionAncestry);
write('dag.json', dag);
rmSync(workRoot, { recursive: true, force: true });

console.log(`cohort of ${nodes.length} machine-mutated candidates from the immutable root:`);
for (const e of mutationEffectiveness.bySurface) {
  console.log(`  ${e.surface.padEnd(15)} attempts=${e.attempts} promotions=${e.promotions} meanΔ=${e.meanDelta.toFixed(3)}`);
}
console.log(`  promoted: ${nodes.filter((n) => n.promote).length}/${nodes.length}  ·  rejected: ${regressionAncestry.rejected.length}`);
