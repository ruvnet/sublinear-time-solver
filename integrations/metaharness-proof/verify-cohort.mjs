#!/usr/bin/env node
/**
 * Independent verifier for the cohort knowledge base.
 *
 * Reads only the sealed ./cohort. For every node it recomputes the candidate
 * manifest hash and RE-RUNS the real gate from the sealed raw outcomes, then
 * RECOMPUTES the mutation-effectiveness and regression-ancestry analytics from
 * those re-run decisions and asserts they match the sealed analytics. The
 * knowledge base is therefore itself verifiable — not a trusted summary.
 *
 * Run: node verify-cohort.mjs
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { manifestOfDir } from './src/hash.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'cohort');
const GEN0 = join(HERE, 'generation-0');
const read = (rel, base = OUT) => JSON.parse(readFileSync(join(base, rel), 'utf8'));

let failures = 0;
const check = (name, ok, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);

console.log('Verifying cohort knowledge base (independent replay):\n');

const dag = read('dag.json');
const gen0 = read('RECEIPT.json', GEN0);
const anchorSuite = read('holdout/suite.json', GEN0);
check('anchor frozen == gen-0 root holdout', hashTasks(anchorSuite) === gen0.artifacts.inputHoldoutHash && dag.anchorHash === gen0.artifacts.inputHoldoutHash);
check('DAG root == immutable gen-0 root', dag.root === gen0.bundleRootHash);

// Reconstruct the shared champion (parent) results from sealed raw outcomes.
const parentResults = scoreRawOutcomes('champion', gen0.bundleRootHash, read('champion.raw.json'));

// Re-run every node's gate from sealed inputs; rebuild the analytics.
const effAcc = {};
const rejects = [];
for (const nodeInfo of dag.nodes) {
  const dir = `nodes/${nodeInfo.id}`;
  const receipt = read(`${dir}/RECEIPT.json`);
  const manifest = manifestOfDir(join(OUT, dir, 'variant')).manifestHash;
  check(`${nodeInfo.id} manifest reproduces`, manifest === receipt.candidateManifestHash);

  const childResults = scoreRawOutcomes(nodeInfo.id, 'champion', read(`${dir}/candidate.raw.json`));
  const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: receipt.seed });
  check(`${nodeInfo.id} decision reproduces (bit-for-bit)`, eq(decision, receipt.decision));
  check(`${nodeInfo.id} parent links to root`, receipt.parent === gen0.bundleRootHash);

  const s = receipt.mutation.surface;
  effAcc[s] ??= { surface: s, attempts: 0, promotions: 0, deltaSum: 0 };
  effAcc[s].attempts += 1;
  if (decision.promote) effAcc[s].promotions += 1;
  effAcc[s].deltaSum += decision.meanDelta;
  if (!decision.promote) rejects.push({ id: receipt.id, surface: s, failedClauses: decision.reasons, ancestor: receipt.parent });
}

// Recomputed analytics must equal the sealed analytics.
const recomputedEff = Object.values(effAcc)
  .map((e) => ({ surface: e.surface, attempts: e.attempts, promotions: e.promotions, meanDelta: Number((e.deltaSum / e.attempts).toFixed(6)) }))
  .sort((a, b) => b.meanDelta - a.meanDelta);
check('mutation-effectiveness recomputes from node receipts', eq(recomputedEff, read('mutation-effectiveness.json').bySurface));
check('regression-ancestry recomputes from node receipts', eq(rejects, read('regression-ancestry.json').rejected));

console.log('\nRecomputed mutation-effectiveness (evidence for mutation-class bias):');
for (const e of recomputedEff) console.log(`   ${e.surface.padEnd(15)} attempts=${e.attempts} promotions=${e.promotions} meanΔ=${e.meanDelta.toFixed(3)}`);
if (rejects.length) {
  console.log('Regression ancestry (why a direction was abandoned):');
  for (const r of rejects) console.log(`   ${r.id} [${r.surface}] ← ${r.failedClauses[0]}`);
}

console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: cohort of ${dag.nodes.length} nodes, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
