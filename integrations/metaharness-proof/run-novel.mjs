#!/usr/bin/env node
/**
 * NOVEL DISCOVERY ENGINE — open-ended code proposals from a real LLM (claude -p),
 * evaluated by RUNNING the proposed code on real systems, selected by the real
 * ADR-076 gate, sealed with a replayable receipt + lineage.
 *
 * Unlike the config search (a fixed grid), the proposer can propose ARBITRARY
 * solver code — a new algorithm. The champion (Jacobi) diverges on SPD systems,
 * so "more iterations" cannot win; only a real method change can. LLM output is
 * non-deterministic, so the DISCOVERY is validated by re-executing the sealed
 * code, not by reproducing the LLM call (see verify-novel.mjs).
 *
 * Run: node run-novel.mjs   (writes ./novel)
 */
import { mkdirSync, writeFileSync, readFileSync, copyFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { bench } from '@metaharness/darwin';

import { propose } from './src/llm-proposer.mjs';
import { scoreRawOutcomes } from './src/score-map.mjs';
import { hashJson, sha256 } from './src/hash.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'novel');
const ROUNDS = 1;
const CANDIDATES = Number(process.argv[2] ?? 3);
const write = (rel, v) => { const p = join(OUT, rel); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, typeof v === 'string' ? v : JSON.stringify(v, null, 2) + '\n'); };

const systems = JSON.parse(readFileSync(join(OUT, 'systems.json'), 'utf8'));
const systemsHash = sha256(JSON.stringify(systems.map((s) => ({ seed: s.seed, matrix: s.matrix, b: s.b }))));

/** Run a genome file in a sandboxed subprocess with a hard timeout; return raw outcomes. */
function evaluateGenome(genomeFile) {
  let measured;
  try {
    const stdout = execFileSync('node', [join(HERE, 'src/genome-runner.mjs'), genomeFile, join(OUT, 'systems.json')], { encoding: 'utf8', timeout: 15000, maxBuffer: 4 * 1024 * 1024 });
    measured = JSON.parse(stdout);
  } catch { measured = systems.map((s) => ({ taskId: `sys-${s.seed}`, residual: 1e9, solved: false, ms: 0 })); }
  if (measured.error) measured = systems.map((s) => ({ taskId: `sys-${s.seed}`, residual: 1e9, solved: false, ms: 0 }));
  const raw = measured.map((m, i) => ({
    taskId: m.taskId, residual: m.residual, solvedMeasured: !!m.solved, ms: m.ms, kind: systems[i].kind,
    publicTestPassed: !!m.solved, hiddenTestPassed: !!m.solved, regressionPassed: true,
    safetyViolations: [], blockedFileTouches: [], hallucinatedFileRefs: false,
    costUsd: 0.001, maxCostUsd: 0.05, durationMs: 100, timeoutMs: 5000,
  }));
  return { raw, solves: raw.filter((r) => r.solvedMeasured).length };
}

const championFile = join(OUT, 'genome/champion.mjs');
let champEval = evaluateGenome(championFile);
write('measured/champion.json', champEval.raw);
const parentResults = scoreRawOutcomes('champion', null, champEval.raw);
const failingKinds = [...new Set(champEval.raw.filter((r) => !r.solvedMeasured).map((r) => r.kind))];

console.log(`champion solves ${champEval.solves}/${systems.length}; failing classes: ${failingKinds.join(', ')}`);

const proposalsLog = [];
const promotions = [];
let llmCalls = 0, rejected = 0;
let bestGenome = null, bestDecision = null, bestSolves = champEval.solves;

for (let r = 0; r < ROUNDS; r++) {
  for (let k = 0; k < CANDIDATES; k++) {
    const champSrc = readFileSync(championFile, 'utf8');
    llmCalls++;
    const p = propose(champSrc, champEval.solves, systems.length, failingKinds, proposalsLog.filter((x) => !x.improved).map((x) => x.note).slice(-3));
    if (!p.ok) { rejected++; proposalsLog.push({ round: r, k, ok: false, why: p.why, improved: false, note: 'rejected: ' + p.why }); console.log(`  cand ${k}: rejected (${p.why})`); continue; }
    const genomeFile = join(OUT, `proposals/round${r}/cand-${k}.mjs`);
    write(`proposals/round${r}/cand-${k}.mjs`, p.code);
    const ev = evaluateGenome(genomeFile);
    write(`measured/round${r}-cand-${k}.json`, ev.raw);
    const childResults = scoreRawOutcomes(`r${r}c${k}`, 'champion', ev.raw);
    const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 7 });
    const improved = decision.promote && ev.solves > champEval.solves;
    proposalsLog.push({ round: r, k, ok: true, solves: ev.solves, promote: decision.promote, improved, note: `solved ${ev.solves}/${systems.length}` });
    console.log(`  cand ${k}: solved ${ev.solves}/${systems.length}, gate promote=${decision.promote}`);
    if (improved && ev.solves > bestSolves) { bestSolves = ev.solves; bestGenome = genomeFile; bestDecision = decision; }
  }
}

let discovery = null;
if (bestGenome) {
  copyFileSync(bestGenome, join(OUT, 'genome/discovered.mjs'));
  promotions.push({ from: 'champion', to: 'discovered', solves: bestSolves, of: systems.length });
  discovery = {
    solves: bestSolves, of: systems.length,
    genomeSha256: sha256(readFileSync(bestGenome)),
    meanDelta: bestDecision.meanDelta, lower95: bestDecision.lower95, reasons: bestDecision.reasons,
    decisionHash: hashJson(bestDecision),
  };
}

const receipt = {
  kind: 'novel-discovery-receipt',
  target: 'ruvnet/sublinear-time-solver',
  proposer: 'claude -p (open-ended LLM code proposal — arbitrary algorithms, not a config grid)',
  authored: 'NO human authored the discovered solver; Claude proposed it, real execution validated it',
  nonReproducibleGeneration: 'LLM output is non-deterministic; the discovered CODE and its measured superiority are reproducible by re-execution (verify-novel.mjs)',
  systemsHash, suite: `${systems.length} systems (${failingKinds.length ? 'mixed dominant + SPD-non-dominant' : ''})`,
  champion: { solves: champEval.solves, of: systems.length },
  llmCalls, rejectedProposals: rejected,
  discovery,
  promote: !!discovery,
};
write('RECEIPT.json', receipt);
write('proposals-log.json', proposalsLog);
write('lineage.json', { kind: 'novel-lineage', immutableRootHint: existsSync(join(HERE, 'generation-0/RECEIPT.json')) ? JSON.parse(readFileSync(join(HERE, 'generation-0/RECEIPT.json'), 'utf8')).bundleRootHash : null, chain: [{ id: 'champion', solves: champEval.solves }, ...(discovery ? [{ id: 'discovered', parent: 'champion', solves: discovery.solves }] : [])] });

console.log(`\n${discovery ? 'DISCOVERED' : 'no improvement'}: ${discovery ? `a genome solving ${discovery.solves}/${systems.length} (champion ${champEval.solves}/${systems.length})` : `champion held at ${champEval.solves}/${systems.length}`}`);
console.log(`  ${llmCalls} LLM proposals, ${rejected} rejected by the safety gate.`);
