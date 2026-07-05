#!/usr/bin/env node
/**
 * single-round proof-of-mechanism — producer.
 *
 * Drives ONE synthetic evolve round through the REAL metaharness gate
 * (@metaharness/darwin: hashTasks, scoreBenchmark, decidePromotion) and seals a
 * self-contained, independently-verifiable generation-0 receipt bundle.
 *
 * Emits exactly the seven required artifacts:
 *   input holdout hash · baseline manifest hash · candidate manifest hash ·
 *   meetsPromotionRule version · decision receipt · SHADOW registration id ·
 *   cost receipt
 *
 * Run: node run-proof.mjs   (writes ./generation-0)
 */
import { mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { manifestOfDir, hashJson, sha256 } from './src/hash.mjs';
import { SEED, HOLDOUT_TASKS, VARIANT_FILES, rawOutcomes } from './src/round.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'generation-0');
// Resolve the darwin package version without relying on its exports map
// exposing package.json (it doesn't): walk up from the ESM-resolved entry.
const darwinEntry = fileURLToPath(import.meta.resolve('@metaharness/darwin'));
const darwinVersion = JSON.parse(
  readFileSync(join(dirname(dirname(darwinEntry)), 'package.json'), 'utf8'),
).version;

const write = (rel, value) => {
  const p = join(OUT, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, typeof value === 'string' ? value : JSON.stringify(value, null, 2) + '\n');
};

// Fresh bundle.
rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

// --- 1. Seal the inputs (so the bundle is self-contained & replayable) -------
for (const [variant, files] of Object.entries(VARIANT_FILES)) {
  for (const [name, content] of Object.entries(files)) write(`variants/${variant}/${name}`, content);
}
write('holdout/suite.json', HOLDOUT_TASKS);
write('results/baseline.raw.json', rawOutcomes('baseline'));
write('results/candidate.raw.json', rawOutcomes('candidate'));

// --- 2. Artifact 1: input holdout hash (the library's own canonical hash) ----
const inputHoldoutHash = hashTasks(HOLDOUT_TASKS);

// --- 3. Artifacts 2 & 3: baseline / candidate manifest hashes ----------------
const baselineManifest = manifestOfDir(join(OUT, 'variants/baseline'));
const candidateManifest = manifestOfDir(join(OUT, 'variants/candidate'));

// --- 4. Score the synthetic outcomes with the REAL scorer --------------------
const parentResults = scoreRawOutcomes('baseline', null, rawOutcomes('baseline'));
const childResults = scoreRawOutcomes('candidate', 'baseline', rawOutcomes('candidate'));

// --- 5. Artifact 5: decision receipt (the REAL ADR-076 gate) -----------------
const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: SEED });

const decisionReceipt = {
  kind: 'decision-receipt',
  rule: 'adr-076',
  seed: SEED,
  cleanReplay: true,
  inputs: {
    inputHoldoutHash,
    baselineManifestHash: baselineManifest.manifestHash,
    candidateManifestHash: candidateManifest.manifestHash,
    parentResultsHash: hashJson(parentResults),
    childResultsHash: hashJson(childResults),
  },
  decision, // promote, reasons[], meanDelta, lower95, pValue, rates …
};

// --- 6. Artifact 4: meetsPromotionRule version -------------------------------
const promotionRule = {
  kind: 'promotion-rule',
  meetsPromotionRule: `adr-076@${darwinVersion}`,
  engine: '@metaharness/darwin',
  engineVersion: darwinVersion,
  clauses: [
    'mean score win over parent + minDelta',
    'statistically real (bootstrap lower95 > 0)',
    'verified-solve rate held',
    'zero child safety violations',
    'regression rate not worse than parent',
    'clean replay reproduced the result',
  ],
};

// --- 7. Artifact 7: cost receipt (metered from per-task cost proxy) ----------
const sumCost = (rs) => Number(rs.reduce((s, r) => s + r.costUsd, 0).toFixed(6));
const costReceipt = {
  kind: 'cost-receipt',
  currency: 'USD',
  metered: true,
  source: 'per-task cost proxy (RunnerOptions.costUsdPerTask); never self-reported',
  perVariant: { baseline: sumCost(parentResults), candidate: sumCost(childResults) },
  total: sumCost([...parentResults, ...childResults]),
};

// --- 8. Artifact 6: SHADOW registration + serving manifest (no auto-serve) ---
const shadowId = 'shadow-' + sha256(
  candidateManifest.manifestHash + inputHoldoutHash + 'SHADOW',
).slice(0, 24);
const shadowRegistration = {
  kind: 'shadow-registration',
  shadowId,
  variant: 'candidate',
  candidateManifestHash: candidateManifest.manifestHash,
  status: 'SHADOW',
  promotedInLineage: decision.promote,
  servable: false,
  note: 'Registered for shadow evaluation only. Promotion into lineage does NOT serve traffic.',
};
// The serving manifest is the checkable "no auto-serve" fact: nothing is served.
const servingManifest = { kind: 'serving-manifest', served: [], note: 'no candidate is serving production traffic' };

// --- 9. Manifests & bundle root ---------------------------------------------
write('receipts/holdout_hash.json', { kind: 'holdout-hash', inputHoldoutHash, taskCount: HOLDOUT_TASKS.length });
write('receipts/manifest_hashes.json', {
  kind: 'manifest-hashes',
  baseline: baselineManifest,
  candidate: candidateManifest,
});
write('receipts/promotion_rule.json', promotionRule);
write('receipts/decision_receipt.json', decisionReceipt);
write('receipts/shadow_registration.json', shadowRegistration);
write('receipts/serving_manifest.json', servingManifest);
write('receipts/cost_receipt.json', costReceipt);
write('results/baseline.scored.json', parentResults);
write('results/candidate.scored.json', childResults);

// The seven artifact fingerprints, and a single root over them.
const artifacts = {
  inputHoldoutHash,
  baselineManifestHash: baselineManifest.manifestHash,
  candidateManifestHash: candidateManifest.manifestHash,
  meetsPromotionRule: promotionRule.meetsPromotionRule,
  decisionReceiptHash: hashJson(decisionReceipt),
  shadowRegistrationId: shadowId,
  costReceiptHash: hashJson(costReceipt),
};
const receipt = {
  kind: 'generation-0-receipt',
  label: 'single-round proof-of-mechanism',
  disclaimer: 'SYNTHETIC candidate/outcomes. Proves gate wiring, receipt persistence, SHADOW registration, and no-auto-serve — NOT learning, NOT compounding, NOT production.',
  immutableRoot: true,
  generation: 0,
  seed: SEED,
  engine: { name: '@metaharness/darwin', version: darwinVersion },
  artifacts,
  bundleRootHash: hashJson(artifacts),
  promote: decision.promote,
};
write('RECEIPT.json', receipt);

// --- Console summary ---------------------------------------------------------
console.log('generation-0 sealed:', OUT);
console.log('  input holdout hash    :', inputHoldoutHash);
console.log('  baseline manifest hash:', baselineManifest.manifestHash);
console.log('  candidate manifest hash:', candidateManifest.manifestHash);
console.log('  meetsPromotionRule    :', promotionRule.meetsPromotionRule);
console.log('  decision receipt hash :', artifacts.decisionReceiptHash);
console.log('  SHADOW registration id:', shadowId);
console.log('  cost receipt total    : $' + costReceipt.total);
console.log('  bundle root hash      :', receipt.bundleRootHash);
console.log('  promote               :', decision.promote, '—', decision.reasons.length, 'clauses');
