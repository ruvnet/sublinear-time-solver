#!/usr/bin/env node
/**
 * Independent verifier for plateau detection.
 *
 * Reads only ./plateau. Re-runs the real gate on every sealed candidate to
 * recompute each generation's stats, then recomputes the pure plateau detector
 * and asserts the sealed history and verdict reproduce exactly. The plateau
 * signal is therefore verifiable, not a trusted claim.
 *
 * Run: node verify-plateau.mjs
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { scoreRawOutcomes } from './src/score-map.mjs';
import { detectPlateau } from './src/plateau.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'plateau');
const read = (rel) => JSON.parse(readFileSync(join(OUT, rel), 'utf8'));

let failures = 0;
const check = (name, ok, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const variance = (xs) => { const m = xs.reduce((s, v) => s + v, 0) / xs.length; return xs.reduce((s, v) => s + (v - m) ** 2, 0) / xs.length; };

console.log('Verifying plateau detection (independent replay):\n');

const sealedHistory = read('history.json');
const sealedPlateau = read('plateau.json');
const parentResults = scoreRawOutcomes('champion', null, read('champion.raw.json'));

// Re-gate every candidate; rebuild each generation's stats from real decisions.
const history = [];
for (const gen of sealedHistory.generations) {
  const g = gen.generation;
  let promotions = 0, bestDelta = -Infinity;
  const finalScores = [];
  for (let k = 0; k < gen.attempts; k++) {
    const childResults = scoreRawOutcomes(`g${g}_c${k}`, 'champion', read(`generations/g${g}/cand${k}.raw.json`));
    finalScores.push(childResults[0].finalScore);
    const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 100 + g * 10 + k });
    if (decision.promote) promotions += 1;
    bestDelta = Math.max(bestDelta, decision.meanDelta);
  }
  history.push({ generation: g, attempts: gen.attempts, promotions, bestDelta: Number(bestDelta.toFixed(6)), scoreVariance: Number(variance(finalScores).toFixed(9)) });
}

check('per-generation history reproduces from re-gated candidates', eq(history, sealedHistory.generations));

// Recompute the detector (full verdict + per-prefix trace).
const verdict = detectPlateau(history, sealedPlateau.config);
const trace = [];
for (let n = 1; n <= history.length; n++) trace.push({ upToGeneration: n - 1, ...detectPlateau(history.slice(0, n), sealedPlateau.config) });
check('plateau verdict reproduces (bit-for-bit)', eq(verdict, sealedPlateau.verdict));
check('plateau per-prefix trace reproduces', eq(trace, sealedPlateau.trace));
check('final classification is local-optimum', verdict.plateau === true && verdict.classification === 'local-optimum');
check('all three plateau clauses hold', verdict.clauses.medianImprovementBelowEpsilon && verdict.clauses.promotionRateBelowMax && verdict.clauses.varianceShrinking);

const first = trace.find((t) => t.plateau);
console.log(`\n  recomputed: plateau=${verdict.plateau} (${verdict.classification}); first declared at generation ${first ? first.upToGeneration : '(never)'}`);
console.log(`  medianImprovement=${verdict.medianImprovement} < ${sealedPlateau.config.epsilon}, promotionRate=${verdict.promotionRate} < ${sealedPlateau.config.maxPromotionRate}, varianceShrinking=${verdict.varianceShrinking}`);

console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: ${history.length}-generation history, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
