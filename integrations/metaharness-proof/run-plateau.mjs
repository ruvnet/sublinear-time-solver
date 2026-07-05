#!/usr/bin/env node
/**
 * Statistical plateau detection over a real-gated evolution history (item #2).
 *
 * Builds a 6-generation history where each generation's candidates offer a
 * DIMINISHING efficiency gain over a fixed champion (early gens win big; late
 * gens are within noise). Every generation's candidates are gated by the real
 * ADR-076 gate; the per-generation stats (bestDelta, promotion rate, score
 * variance) are derived from those real decisions. The pure detector
 * (src/plateau.mjs) then classifies the tail as local-optimum / still-improving
 * / noisy / inconclusive.
 *
 * REAL: the gate, the per-generation stats, the detector. SYNTHETIC (labeled):
 * the per-task outcomes, shaped to diminish so a plateau actually forms.
 *
 * Run: node run-plateau.mjs   (writes ./plateau)
 */
import { mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { scoreRawOutcomes } from './src/score-map.mjs';
import { detectPlateau, DEFAULT_PLATEAU_CONFIG } from './src/plateau.mjs';

const { hashTasks, decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const GEN0 = join(HERE, 'generation-0');
const OUT = join(HERE, 'plateau');
const read = (p) => JSON.parse(readFileSync(p, 'utf8'));
const write = (rel, v) => { const p = join(OUT, rel); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, JSON.stringify(v, null, 2) + '\n'); };

const anchorSuite = read(join(GEN0, 'holdout/suite.json'));
const anchorHash = hashTasks(anchorSuite);

// Fixed champion; per-generation candidate cost approaches it (diminishing gain).
const CHAMPION_COST = 0.04;
// gen:      0      1      2      3       4       5
const TARGET = [0.006, 0.012, 0.022, 0.032, 0.0385, 0.0398];
const JITTER = [0.004, 0.003, 0.002, 0.001, 0.0004, 0.0002]; // converging candidates
const CANDIDATES = 3;

const mkRaw = (cost) => anchorSuite.map((task) => ({
  taskId: task.id, publicTestPassed: true, hiddenTestPassed: true, regressionPassed: true,
  safetyViolations: [], blockedFileTouches: [], hallucinatedFileRefs: false,
  costUsd: Number(cost.toFixed(6)), maxCostUsd: task.maxCostUsd, durationMs: 200, timeoutMs: task.timeoutMs,
}));
const variance = (xs) => { const m = xs.reduce((s, v) => s + v, 0) / xs.length; return xs.reduce((s, v) => s + (v - m) ** 2, 0) / xs.length; };

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

const championRaw = mkRaw(CHAMPION_COST);
write('champion.raw.json', championRaw);
const parentResults = scoreRawOutcomes('champion', null, championRaw);

const history = [];
for (let g = 0; g < TARGET.length; g++) {
  let promotions = 0;
  let bestDelta = -Infinity;
  const finalScores = [];
  for (let k = 0; k < CANDIDATES; k++) {
    const cost = TARGET[g] + (k - (CANDIDATES - 1) / 2) * JITTER[g]; // spread around target
    const raw = mkRaw(cost);
    write(`generations/g${g}/cand${k}.raw.json`, raw);
    const childResults = scoreRawOutcomes(`g${g}_c${k}`, 'champion', raw);
    finalScores.push(childResults[0].finalScore);
    const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 100 + g * 10 + k });
    if (decision.promote) promotions += 1;
    bestDelta = Math.max(bestDelta, decision.meanDelta);
  }
  history.push({
    generation: g,
    attempts: CANDIDATES,
    promotions,
    bestDelta: Number(bestDelta.toFixed(6)),
    scoreVariance: Number(variance(finalScores).toFixed(9)),
  });
}

write('history.json', { kind: 'evolution-history', anchorHash, champion: { costUsd: CHAMPION_COST }, generations: history });

// Detector over the full history, plus a per-prefix trace showing WHEN it fires.
const trace = [];
for (let n = 1; n <= history.length; n++) trace.push({ upToGeneration: n - 1, ...detectPlateau(history.slice(0, n)) });
const verdict = detectPlateau(history);
write('plateau.json', { kind: 'plateau-verdict', config: DEFAULT_PLATEAU_CONFIG, verdict, trace });

console.log('plateau detection over a diminishing-returns history:');
console.log('  gen  bestΔ    promo/att  variance');
for (const h of history) console.log(`  ${h.generation}    ${h.bestDelta.toFixed(4).padStart(7)}  ${h.promotions}/${h.attempts}        ${h.scoreVariance.toExponential(2)}`);
console.log(`\n  final verdict: plateau=${verdict.plateau}  classification=${verdict.classification}`);
console.log(`  medianImprovement=${verdict.medianImprovement}  promotionRate=${verdict.promotionRate}  varianceShrinking=${verdict.varianceShrinking}`);
const firstPlateau = trace.find((t) => t.plateau);
console.log('  first declared plateau at generation:', firstPlateau ? firstPlateau.upToGeneration : '(never)');
