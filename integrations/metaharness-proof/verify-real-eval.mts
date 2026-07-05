#!/usr/bin/env -S npx tsx
/**
 * Independent verifier for the REAL evaluation round.
 *
 * The strongest verification in this package: it RE-EXECUTES the repo's real
 * solver (and the baseline) on the sealed systems, recomputes the measured
 * residuals and the gate decision, and asserts they reproduce. It does not
 * trust the producer's numbers — it regenerates them by running real code.
 *
 * Run: npx tsx verify-real-eval.mts
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { evaluate } from './src/real-task.mts';
import { scoreRawOutcomes } from './src/score-map.mjs';
import { hashJson, sha256 } from './src/hash.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'real-eval');
const read = (rel: string) => JSON.parse(readFileSync(join(OUT, rel), 'utf8'));

let failures = 0;
const check = (name: string, ok: boolean, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const eq = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

console.log('Verifying REAL evaluation by RE-EXECUTING the repo solver:\n');

const receipt = read('RECEIPT.json');
const systems = read('systems.json');

check('systems hash matches sealed', sha256(JSON.stringify(systems.map((s: any) => ({ seed: s.seed, matrix: s.matrix, b: s.b })))) === receipt.systemsHash);

// Re-run BOTH solvers on the sealed systems — real execution, not trust.
const baselineRaw = await evaluate('baseline', systems);
const candidateRaw = await evaluate('candidate', systems);

const baseSolved = baselineRaw.filter((r) => r.solvedMeasured).length;
const candSolved = candidateRaw.filter((r) => r.solvedMeasured).length;
check('re-run baseline verified-solves reproduce', baseSolved === receipt.baseline.verifiedSolves, `${baseSolved}/${systems.length}`);
check('re-run candidate verified-solves reproduce', candSolved === receipt.candidate.verifiedSolves, `${candSolved}/${systems.length}`);
check('measured residuals reproduce (baseline)', eq(baselineRaw.map((r) => r.residual), read('measured/baseline.json').map((r: any) => r.residual)));
check('measured residuals reproduce (candidate)', eq(candidateRaw.map((r) => r.residual), read('measured/candidate.json').map((r: any) => r.residual)));

// Re-gate from the freshly measured outcomes.
const parentResults = scoreRawOutcomes('baseline-buggy-neumann', null, baselineRaw);
const childResults = scoreRawOutcomes('candidate-repo-solver', 'baseline', candidateRaw);
const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 7 });
check('gate decision reproduces (bit-for-bit)', eq(decision, receipt.decision));
check('decision receipt hash reproduces', hashJson(decision) === receipt.decisionReceiptHash);

// The real learning signal: candidate genuinely solves what baseline cannot.
check('candidate really solves MORE than baseline (measured)', candSolved > baseSolved, `${candSolved} > ${baseSolved}`);
check('promotion is on real verified correctness', decision.promote === true && decision.reasons.some((r: string) => /verified-solve rate/.test(r)));

console.log(`\n  re-executed: baseline ${baseSolved}/${systems.length}, candidate ${candSolved}/${systems.length} verified solves`);
console.log(`  gate promote=${decision.promote} (${decision.reasons.length} clauses), reproduced from real re-execution`);
console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: real evaluation, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
