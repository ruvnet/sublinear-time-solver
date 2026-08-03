#!/usr/bin/env -S npx tsx
/**
 * AUTONOMOUS DISCOVERY on the real sublinear solver.
 *
 * The system proposes candidate configurations (seeded mutation operator),
 * evaluates each by RUNNING THE REAL SOLVER on real systems, and the real
 * ADR-076 gate selects the winner. No human authors the winning config — it
 * emerges from measured evaluation. The discovered config joins the lineage.
 *
 * Run: npx tsx run-autonomous.mts   (writes ./autonomous)
 */
import { mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
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
const PROPOSE_SEED = 20240705;
const COHORT = 12;
const write = (rel: string, v: unknown) => { const p = join(OUT, rel); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, JSON.stringify(v, null, 2) + '\n'); };

const immutableRoot = JSON.parse(readFileSync(join(GEN0, 'RECEIPT.json'), 'utf8')).bundleRootHash;

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

const systems = buildSystems();
write('systems.json', systems.map((s) => ({ seed: s.seed, dominance: s.dominance, n: s.n, matrix: s.matrix, b: s.b })));
write('champion.config.json', CHAMPION);

// The mutator PROPOSES; it does not know which wins.
const proposed = proposeCohort(COHORT, PROPOSE_SEED);
write('proposed-configs.json', { proposeSeed: PROPOSE_SEED, configs: proposed });

// Evaluate the champion by running the real solver.
const championRaw = await evaluateConfig(CHAMPION, systems);
write('measured/champion.json', championRaw);
const parentResults = scoreRawOutcomes('champion', immutableRoot, championRaw);
const championSolves = championRaw.filter((r) => r.solvedMeasured).length;

// Evaluate every proposed config; gate each against the champion.
const candidates = [];
for (let i = 0; i < proposed.length; i++) {
  const cfg = proposed[i];
  const raw = await evaluateConfig(cfg, systems);
  write(`measured/candidate-${i}.json`, raw);
  const childResults = scoreRawOutcomes(`cand-${i}`, 'champion', raw);
  const decision = decidePromotion({ parentResults, childResults, cleanReplay: true, seed: 7 });
  candidates.push({ index: i, config: cfg, solves: raw.filter((r) => r.solvedMeasured).length, promote: decision.promote, meanDelta: decision.meanDelta, lower95: decision.lower95, decision });
}
write('candidates.json', candidates.map(({ decision, ...c }) => c));

// The gate — not the operator — selects. Discovery = best-promoted by meanDelta.
const promoted = candidates.filter((c) => c.promote).sort((a, b) => b.meanDelta - a.meanDelta);
const discovery = promoted[0] ?? null;

const receipt = {
  kind: 'autonomous-discovery-receipt',
  target: 'ruvnet/sublinear-time-solver',
  immutableRoot,
  method: 'seeded config mutation → real-solver evaluation → real ADR-076 gate selection',
  authored: 'NO human authored the winning config; it emerged from measured evaluation',
  proposeSeed: PROPOSE_SEED,
  champion: { config: CHAMPION, verifiedSolves: championSolves, of: systems.length },
  proposedCount: proposed.length,
  promotedCount: promoted.length,
  discovery: discovery && {
    config: discovery.config,
    verifiedSolves: discovery.solves,
    of: systems.length,
    meanDelta: discovery.meanDelta,
    lower95: discovery.lower95,
    reasons: discovery.decision.reasons,
    decisionHash: hashJson(discovery.decision),
  },
  systemsHash: sha256(JSON.stringify(systems.map((s) => ({ seed: s.seed, matrix: s.matrix, b: s.b })))),
  promote: !!discovery,
};
write('RECEIPT.json', receipt);

console.log('AUTONOMOUS DISCOVERY on the real sublinear solver:');
console.log(`  champion config: ${JSON.stringify(CHAMPION)}  → solves ${championSolves}/${systems.length}`);
console.log(`  system proposed ${proposed.length} configs; ${promoted.length} beat the champion at the gate.`);
if (discovery) {
  console.log(`  DISCOVERED (by the gate, not by me): ${JSON.stringify(discovery.config)}`);
  console.log(`     → solves ${discovery.solves}/${systems.length}, meanΔ=${discovery.meanDelta.toFixed(3)}, lower95=${discovery.lower95}`);
} else {
  console.log('  no config beat the champion this round.');
}
