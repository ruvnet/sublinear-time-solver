#!/usr/bin/env node
/**
 * single-round proof-of-mechanism — independent verifier (the acceptance test).
 *
 * Reads ONLY the sealed ./generation-0 bundle. Recomputes every hash, re-runs
 * the REAL decidePromotion on the sealed raw outcomes, and asserts the sealed
 * decision matches bit-for-bit — then explains WHY the candidate passed/failed
 * from the receipt alone. It never reads a service log or trusts a
 * producer-claimed value.
 *
 * Exit 0 = independently verified. Exit 1 = a claim did not reproduce.
 *
 * Run: node verify-receipt.mjs [path-to-generation-0]
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { manifestOfDir, hashJson } from './src/hash.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = process.argv[2] ? process.argv[2] : join(HERE, 'generation-0');
const read = (rel) => JSON.parse(readFileSync(join(OUT, rel), 'utf8'));

let failures = 0;
const check = (name, ok, detail = '') => {
  console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`);
  if (!ok) failures++;
};
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);

console.log(`Verifying receipt bundle: ${OUT}\n`);

const receipt = read('RECEIPT.json');
const A = receipt.artifacts;

// 1. input holdout hash — recompute from the sealed suite.
const suite = read('holdout/suite.json');
const holdoutHash = hashTasks(suite);
check('input holdout hash reproduces', holdoutHash === A.inputHoldoutHash, holdoutHash.slice(0, 20) + '…');
check('holdout_hash.json agrees', read('receipts/holdout_hash.json').inputHoldoutHash === holdoutHash);

// 2 & 3. manifest hashes — recompute from sealed variant contents.
const baseHash = manifestOfDir(join(OUT, 'variants/baseline')).manifestHash;
const candHash = manifestOfDir(join(OUT, 'variants/candidate')).manifestHash;
check('baseline manifest hash reproduces', baseHash === A.baselineManifestHash, baseHash.slice(0, 20) + '…');
check('candidate manifest hash reproduces', candHash === A.candidateManifestHash, candHash.slice(0, 20) + '…');

// 5. decision receipt — RE-RUN the real gate on the sealed raw outcomes and
//    compare to the sealed decision. This is the core independence check.
const parent = scoreRawOutcomes('baseline', null, read('results/baseline.raw.json'));
const child = scoreRawOutcomes('candidate', 'baseline', read('results/candidate.raw.json'));
const sealed = read('receipts/decision_receipt.json');
const recomputed = decidePromotion({ parentResults: parent, childResults: child, cleanReplay: sealed.cleanReplay, seed: sealed.seed });
check('decision reproduces (promote)', recomputed.promote === sealed.decision.promote);
check('decision reproduces (full object, bit-for-bit)', eq(recomputed, sealed.decision));
check('sealed result hashes match recomputed', sealed.inputs.parentResultsHash === hashJson(parent) && sealed.inputs.childResultsHash === hashJson(child));

// 4. meetsPromotionRule version present & stamped.
const rule = read('receipts/promotion_rule.json');
check('meetsPromotionRule version stamped', rule.meetsPromotionRule === A.meetsPromotionRule && /^adr-076@/.test(rule.meetsPromotionRule), rule.meetsPromotionRule);

// 6. SHADOW registration id + no-auto-serve.
const shadow = read('receipts/shadow_registration.json');
const serving = read('receipts/serving_manifest.json');
check('SHADOW registration id present', shadow.shadowId === A.shadowRegistrationId, shadow.shadowId);
check('candidate is SHADOW, not servable', shadow.status === 'SHADOW' && shadow.servable === false);
check('NO auto-serve path: shadow id not in served set', !serving.served.includes(shadow.shadowId) && serving.served.length === 0);

// 7. cost receipt hash + bundle root.
check('cost receipt hash reproduces', hashJson(read('receipts/cost_receipt.json')) === A.costReceiptHash);
check('decision receipt hash reproduces', hashJson(sealed) === A.decisionReceiptHash);
check('bundle root hash reproduces', hashJson(A) === receipt.bundleRootHash);

// --- The independent explanation of the verdict ------------------------------
console.log('\nWhy the candidate ' + (recomputed.promote ? 'PASSED' : 'FAILED') + ' (recomputed from the bundle, not service logs):');
for (const r of recomputed.reasons) console.log('   • ' + r);
console.log(`   meanDelta=${recomputed.meanDelta}  lower95=${recomputed.lower95}  pValue=${recomputed.pValue}`);

console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
