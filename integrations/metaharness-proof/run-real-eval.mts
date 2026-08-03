#!/usr/bin/env -S npx tsx
/**
 * REAL evaluation round on ruvnet/sublinear-time-solver.
 *
 * Crosses the synthetic → real boundary: outcomes are MEASURED by executing the
 * repo's real solver on real linear systems (see src/real-task.mts), not
 * hand-authored. Candidate = the repo's real SublinearSolver; baseline = the
 * pre-fix buggy Neumann. Correctness is a measured residual; the real ADR-076
 * gate decides. Honest scope: the two variants were authored by the operator
 * (me), so this proves real EVALUATION, not autonomous discovery.
 *
 * Run: npx tsx run-real-eval.mts   (writes ./real-eval)
 */
import { mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { SEEDS, N, makeSystem, evaluate } from './src/real-task.mts';
import { scoreRawOutcomes } from './src/score-map.mjs';
import { hashJson, sha256 } from './src/hash.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'real-eval');
const write = (rel: string, v: unknown) => { const p = join(OUT, rel); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, JSON.stringify(v, null, 2) + '\n'); };

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

const systems = SEEDS.map((seed) => ({ seed, n: N, ...makeSystem(N, seed) }));
write('systems.json', systems.map((s) => ({ seed: s.seed, n: s.n, matrix: s.matrix, b: s.b })));

const baselineRaw = await evaluate('baseline', systems);
const candidateRaw = await evaluate('candidate', systems);
write('measured/baseline.json', baselineRaw);
write('measured/candidate.json', candidateRaw);

const parentResults = scoreRawOutcomes('baseline-buggy-neumann', null, baselineRaw);
const childResults = scoreRawOutcomes('candidate-repo-solver', 'baseline', candidateRaw);
const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 7 });

const baseSolved = baselineRaw.filter((r) => r.solvedMeasured).length;
const candSolved = candidateRaw.filter((r) => r.solvedMeasured).length;
const median = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

const receipt = {
  kind: 'real-evaluation-receipt',
  target: 'ruvnet/sublinear-time-solver',
  evaluation: 'REAL — measured by executing the repo solver on real systems (not fixtures)',
  agent: 'human/AI-in-the-loop; variants authored by the operator. Autonomous discovery is the next step.',
  task: `solve ${systems.length} real diagonally-dominant systems A x = b to residual < 1e-6`,
  baseline: { id: 'baseline-buggy-neumann', verifiedSolves: baseSolved, of: systems.length, medianResidual: median(baselineRaw.map((r) => r.residual)) },
  candidate: { id: 'candidate-repo-solver', verifiedSolves: candSolved, of: systems.length, medianResidual: median(candidateRaw.map((r) => r.residual)) },
  systemsHash: sha256(JSON.stringify(systems.map((s) => ({ seed: s.seed, matrix: s.matrix, b: s.b })))),
  decision,
  decisionReceiptHash: hashJson(decision),
  promote: decision.promote,
};
write('RECEIPT.json', receipt);

console.log('REAL evaluation on sublinear-time-solver (measured, not fixtured):');
console.log(`  baseline (buggy Neumann)  verified-solves: ${baseSolved}/${systems.length}  median residual ${receipt.baseline.medianResidual.toExponential(2)}`);
console.log(`  candidate (repo solver)   verified-solves: ${candSolved}/${systems.length}  median residual ${receipt.candidate.medianResidual.toExponential(2)}`);
console.log(`  gate promote: ${decision.promote} — ${decision.reasons.length} clause(s)`);
