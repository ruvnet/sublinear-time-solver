#!/usr/bin/env -S npx tsx
/**
 * Validator for AUTONOMOUS DISCOVERY.
 *
 * Re-executes the entire pipeline from sealed inputs: re-derives the proposed
 * configs from the seed (proving they were not cherry-picked), RE-RUNS the real
 * solver to re-measure every config, re-gates, and recomputes which config the
 * gate selects — asserting the discovery reproduces. Also asserts the discovery
 * was among the machine-proposed configs (no human injected it) and genuinely
 * solves more than the champion.
 *
 * Run: npx tsx verify-autonomous.mts
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { bench } from '@metaharness/darwin';

import { CHAMPION, proposeCohort, buildSystems, evaluateConfig } from './src/autonomous-task.mts';
import { scoreRawOutcomes } from './src/score-map.mjs';
import { hashJson, sha256 } from './src/hash.mjs';

const { decidePromotion } = bench;
const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'autonomous');
const GEN0 = join(HERE, 'generation-0');
const read = (rel: string) => JSON.parse(readFileSync(join(OUT, rel), 'utf8'));

let failures = 0;
const check = (name: string, ok: boolean, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const eq = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

console.log('Validating AUTONOMOUS DISCOVERY by re-executing the pipeline:\n');

const receipt = read('RECEIPT.json');
const sealedProposed = read('proposed-configs.json');
const systems = buildSystems();

// 1. Systems + immutable-root link.
check('systems hash matches sealed', sha256(JSON.stringify(systems.map((s) => ({ seed: s.seed, matrix: s.matrix, b: s.b })))) === receipt.systemsHash);
check('links to immutable gen-0 root', receipt.immutableRoot === JSON.parse(readFileSync(join(GEN0, 'RECEIPT.json'), 'utf8')).bundleRootHash);

// 2. Proposals reproduce from the seed (not cherry-picked).
const proposed = proposeCohort(sealedProposed.configs.length, sealedProposed.proposeSeed);
check('proposed configs reproduce from seed', eq(proposed, sealedProposed.configs));
check('champion matches', eq(CHAMPION, read('champion.config.json')));

// 3. RE-RUN the real solver for champion + every candidate; re-gate.
const championRaw = await evaluateConfig(CHAMPION, systems);
const parentResults = scoreRawOutcomes('champion', receipt.immutableRoot, championRaw);
const championSolves = championRaw.filter((r) => r.solvedMeasured).length;
check('champion measurement reproduces', championSolves === receipt.champion.verifiedSolves, `${championSolves}/${systems.length}`);

const recomputed = [];
for (let i = 0; i < proposed.length; i++) {
  const raw = await evaluateConfig(proposed[i], systems);
  const childResults = scoreRawOutcomes(`cand-${i}`, 'champion', raw);
  const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 7 });
  recomputed.push({ index: i, config: proposed[i], solves: raw.filter((r) => r.solvedMeasured).length, promote: decision.promote, meanDelta: decision.meanDelta, lower95: decision.lower95, decision });
}
check('all candidate measurements + decisions reproduce', eq(recomputed.map(({ decision, ...c }) => c), read('candidates.json')));

// 4. Recompute the gate's selection — the discovery must reproduce.
const promoted = recomputed.filter((c) => c.promote).sort((a, b) => b.meanDelta - a.meanDelta);
const discovery = promoted[0] ?? null;
check('a config was discovered', !!discovery);
if (discovery && receipt.discovery) {
  check('discovered config reproduces (gate selection)', eq(discovery.config, receipt.discovery.config), JSON.stringify(discovery.config));
  check('discovery decision reproduces (bit-for-bit)', hashJson(discovery.decision) === receipt.discovery.decisionHash);
  // 5. Autonomy + real improvement.
  check('discovery was machine-PROPOSED (no human injected it)', proposed.some((c) => eq(c, discovery.config)));
  check('discovery genuinely solves MORE than champion (measured)', discovery.solves > championSolves, `${discovery.solves} > ${championSolves}`);
  check('gate promoted on verified-solve improvement', discovery.decision.reasons.some((r: string) => /verified-solve rate/.test(r)));
}

console.log(`\n  re-executed: champion solves ${championSolves}/${systems.length}; gate selected ${discovery ? JSON.stringify(discovery.config) : '(none)'}`);
if (discovery) console.log(`  discovered config solves ${discovery.solves}/${systems.length} — reproduced by re-running the real solver`);
console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: autonomous discovery, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
