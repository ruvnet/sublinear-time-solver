#!/usr/bin/env node
/**
 * Independent verifier for the whole lineage chain (gen-0 → gen-1 → …).
 *
 * For each generation it recomputes every hash and re-runs the real gate from
 * sealed inputs; then it checks the chain itself: each generation's parent link
 * points at its predecessor's bundle root, the anchor suite is frozen across
 * the whole chain, and the immutable root is unchanged. Reads only sealed
 * bundles + lineage.json — never a service log.
 *
 * Run: node verify-lineage.mjs
 */
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { manifestOfDir, hashJson } from './src/hash.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const read = (p) => JSON.parse(readFileSync(p, 'utf8'));

let failures = 0;
const check = (name, ok, detail = '') => {
  console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`);
  if (!ok) failures++;
};
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);

/** Verify one sealed generation bundle in isolation; return its receipt. */
function verifyBundle(dir, gen) {
  console.log(`\ngeneration ${gen}  (${dir.split('/').pop()})`);
  const receipt = read(join(dir, 'RECEIPT.json'));
  const A = receipt.artifacts;

  const suite = read(join(dir, 'holdout/suite.json'));
  check('holdout hash reproduces', hashTasks(suite) === A.inputHoldoutHash);
  check('baseline manifest reproduces', manifestOfDir(join(dir, 'variants/baseline')).manifestHash === A.baselineManifestHash);
  check('candidate manifest reproduces', manifestOfDir(join(dir, 'variants/candidate')).manifestHash === A.candidateManifestHash);

  const sealed = read(join(dir, 'receipts/decision_receipt.json'));
  const parent = scoreRawOutcomes('baseline', null, read(join(dir, 'results/baseline.raw.json')));
  const child = scoreRawOutcomes('candidate', 'baseline', read(join(dir, 'results/candidate.raw.json')));
  const recomputed = decidePromotion({ parentResults: parent, childResults: child, cleanReplay: sealed.cleanReplay, seed: sealed.seed });
  check('decision reproduces (bit-for-bit)', eq(recomputed, sealed.decision));

  const shadow = read(join(dir, 'receipts/shadow_registration.json'));
  const serving = read(join(dir, 'receipts/serving_manifest.json'));
  check('no auto-serve (shadow id not served)', !serving.served.includes(shadow.shadowId) && serving.served.length === 0);

  // Root hash: generations with a parent bind the parent into the root.
  const root = receipt.parent ? hashJson({ ...A, parent: receipt.parent }) : hashJson(A);
  check('bundle root hash reproduces', root === receipt.bundleRootHash);
  return receipt;
}

console.log('Verifying lineage chain (independent replay):');

const gen0 = verifyBundle(join(HERE, 'generation-0'), 0);
const hasGen1 = existsSync(join(HERE, 'generation-1', 'RECEIPT.json'));
const gen1 = hasGen1 ? verifyBundle(join(HERE, 'generation-1'), 1) : null;

// --- chain integrity ---------------------------------------------------------
console.log('\nchain integrity');
const lineage = read(join(HERE, 'lineage.json'));
check('immutable root == gen-0 bundle root', lineage.immutableRoot === gen0.bundleRootHash);
check('gen-0 has no parent (root of the graph)', !gen0.parent);
if (gen1) {
  check('gen-1 parent links to gen-0 root', gen1.parent === gen0.bundleRootHash);
  check('anchor suite frozen across chain', gen1.frozenAnchorHash === gen0.artifacts.inputHoldoutHash && gen1.artifacts.inputHoldoutHash === gen0.artifacts.inputHoldoutHash);
  check('gen-1 candidate is machine-generated (autonomous)', gen1.autonomousMutation?.mutator === 'DeterministicMutator' && !!gen1.autonomousMutation?.surface);
  check('lineage index matches bundle roots', lineage.chain[0].bundleRootHash === gen0.bundleRootHash && lineage.chain[1].bundleRootHash === gen1.bundleRootHash && lineage.chain[1].parent === gen0.bundleRootHash);
}

console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: lineage chain of ${gen1 ? 2 : 1} generation(s), ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
