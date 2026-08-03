#!/usr/bin/env node
/**
 * Demonstrates agenticow's headline property for swarm coordination: forking a
 * per-agent memory branch costs ~constant time regardless of how much
 * collective knowledge the base holds — so spawning N speculative agents does
 * not scale with corpus size.
 *
 * Run: cd integrations/agenticow && npm install && node benchmark.mjs
 */

import { performance } from 'node:perf_hooks';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

let open;
try {
  ({ open } = await import('agenticow'));
} catch {
  console.error('agenticow not installed — run `npm install` in this directory first.');
  process.exit(1);
}

const DIM = 32;
const dir = mkdtempSync(join(tmpdir(), 'swarm-bench-'));
let seq = 0;

const randVec = (i) => Float32Array.from({ length: DIM }, (_, j) => Math.sin(i * 31 + j * 7));

function median(xs) {
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.floor(s.length / 2)];
}

function timeForks(baseSize, forks = 50) {
  const base = open(join(dir, `hive-${seq++}.rvf`), { dimension: DIM, metric: 'cosine' });
  const batch = [];
  for (let i = 0; i < baseSize; i++) batch.push({ id: i + 1, vector: randVec(i) });
  base.ingest(batch);

  const times = [];
  const branches = [];
  for (let f = 0; f < forks; f++) {
    const t0 = performance.now();
    const b = base.fork(`agent-${f}`);
    times.push(performance.now() - t0);
    branches.push(b);
  }
  for (const b of branches) b.close();
  base.close();
  return { median: median(times), mean: times.reduce((a, c) => a + c, 0) / times.length };
}

console.log('agenticow per-agent fork cost vs. base (collective-memory) size\n');
console.log('base vectors |  median fork  |  mean fork');
console.log('-------------+---------------+------------');
for (const baseSize of [100, 1000, 10000, 50000]) {
  const { median: med, mean } = timeForks(baseSize);
  console.log(
    `${String(baseSize).padStart(11)} | ${med.toFixed(4).padStart(9)} ms | ${mean.toFixed(4).padStart(8)} ms`,
  );
}
console.log('\nFork time should stay ~flat as the base grows — the point of COW branching:');
console.log('spawning speculative agents is O(1) in collective-memory size.');

rmSync(dir, { recursive: true, force: true });
