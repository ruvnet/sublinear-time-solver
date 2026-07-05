#!/usr/bin/env node
/**
 * Validator for NOVEL DISCOVERY.
 *
 * The LLM generation is non-deterministic, so this does NOT reproduce the
 * proposal. Instead it validates the DISCOVERY: re-execute the sealed champion
 * and the sealed discovered genome on the sealed systems, reproduce the
 * measured solve counts and the gate decision, and confirm the discovered code
 * genuinely solves more than the champion and passed the static safety gate.
 * That is the honest, verifiable claim: the discovered CODE works, by
 * re-running it — regardless of how it was proposed.
 *
 * Run: node verify-novel.mjs
 */
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { bench } from '@metaharness/darwin';

import { scoreRawOutcomes } from './src/score-map.mjs';
import { hashJson, sha256 } from './src/hash.mjs';
import { isSafe } from './src/llm-proposer.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'novel');

let failures = 0;
const check = (name, ok, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };

console.log('Validating NOVEL DISCOVERY by re-executing the sealed genomes:\n');

const receipt = JSON.parse(readFileSync(join(OUT, 'RECEIPT.json'), 'utf8'));
const systems = JSON.parse(readFileSync(join(OUT, 'systems.json'), 'utf8'));
check('systems hash matches sealed', sha256(JSON.stringify(systems.map((s) => ({ seed: s.seed, matrix: s.matrix, b: s.b })))) === receipt.systemsHash);

if (!receipt.discovery) {
  console.log('\n  receipt records NO promoted discovery this run (champion held). Nothing to reproduce.');
  console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'} (no-discovery run): ${failures} check(s) failed.`);
  process.exit(failures === 0 ? 0 : 1);
}

function runGenome(file) {
  const raw = JSON.parse(execFileSync('node', [join(HERE, 'src/genome-runner.mjs'), file, join(OUT, 'systems.json')], { encoding: 'utf8', timeout: 15000, maxBuffer: 4 * 1024 * 1024 }));
  return raw.map((m) => ({ taskId: m.taskId, solvedMeasured: !!m.solved, publicTestPassed: !!m.solved, hiddenTestPassed: !!m.solved, regressionPassed: true, safetyViolations: [], blockedFileTouches: [], hallucinatedFileRefs: false, costUsd: 0.001, maxCostUsd: 0.05, durationMs: 100, timeoutMs: 5000 }));
}

const discoveredFile = join(OUT, 'genome/discovered.mjs');
check('discovered genome present', existsSync(discoveredFile));
check('discovered genome hash matches receipt', sha256(readFileSync(discoveredFile)) === receipt.discovery.genomeSha256);
check('discovered genome passes the static safety gate', isSafe(readFileSync(discoveredFile, 'utf8')).ok);

// Re-execute both genomes.
const champRaw = runGenome(join(OUT, 'genome/champion.mjs'));
const discRaw = runGenome(discoveredFile);
const champSolves = champRaw.filter((r) => r.solvedMeasured).length;
const discSolves = discRaw.filter((r) => r.solvedMeasured).length;
check('champion re-execution reproduces solve count', champSolves === receipt.champion.solves, `${champSolves}/${systems.length}`);
check('discovered re-execution reproduces solve count', discSolves === receipt.discovery.solves, `${discSolves}/${systems.length}`);

// Re-gate and reproduce the decision.
const decision = decidePromotion({ parentResults: scoreRawOutcomes('champion', null, champRaw), childResults: scoreRawOutcomes('discovered', 'champion', discRaw), cleanReplay: true, seed: 7 });
check('gate decision reproduces (bit-for-bit)', hashJson(decision) === receipt.discovery.decisionHash);
check('discovery genuinely solves MORE than champion', discSolves > champSolves, `${discSolves} > ${champSolves}`);
check('promotion is real (gate promote=true)', decision.promote === true);

console.log(`\n  re-executed: champion ${champSolves}/${systems.length}, discovered ${discSolves}/${systems.length}; gate promote=${decision.promote}`);
console.log(`  the discovered solver was proposed by an LLM (non-reproducible) but its measured superiority is reproduced by re-execution.`);
console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: novel discovery, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
