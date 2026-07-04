#!/usr/bin/env node
/**
 * Optimization benchmark: naive vs optimized zeptosecond Monte Carlo.
 *
 * Run: node validation/zeptosecond/benchmark.mjs [events]
 */

import { performance } from 'node:perf_hooks';
import {
  monteCarloMeanDelayNaive,
  monteCarloMeanDelayOptimized,
  expectedMeanAbsDelay,
  secondsToZs,
  makeLcg,
  ZeptoClock,
} from './zeptosecond-physics.mjs';

const N = Number(process.argv[2] ?? 5_000_000);
const ROUNDS = 5;

function bench(label, fn) {
  fn(makeLcg(0)); // warm up JIT
  const times = [];
  let result;
  for (let r = 0; r < ROUNDS; r++) {
    const t0 = performance.now();
    result = fn(makeLcg(r + 1));
    times.push(performance.now() - t0);
  }
  const best = Math.min(...times);
  const eventsPerSec = N / (best / 1000);
  console.log(
    `${label.padEnd(12)} best ${best.toFixed(1).padStart(8)} ms  ` +
      `${(eventsPerSec / 1e6).toFixed(1).padStart(6)} M events/s  ` +
      `mean ${secondsToZs(result).toFixed(3)} zs`,
  );
  return { best, result };
}

console.log(`Zeptosecond Monte Carlo benchmark — ${N.toLocaleString()} events x ${ROUNDS} rounds`);
console.log(`analytic expectation: ${secondsToZs(expectedMeanAbsDelay()).toFixed(3)} zs\n`);

const naive = bench('naive', (rng) => monteCarloMeanDelayNaive(N, rng));
const optimized = bench('optimized', (rng) => monteCarloMeanDelayOptimized(N, rng));

console.log(`\nspeedup: ${(naive.best / optimized.best).toFixed(2)}x`);

// BigInt clock throughput — cost of exactness
const TICKS = 1_000_000;
const t0 = performance.now();
const clock = new ZeptoClock();
for (let i = 0; i < TICKS; i++) clock.advanceZs(247n);
const clockMs = performance.now() - t0;
console.log(
  `\nZeptoClock: ${TICKS.toLocaleString()} exact 247 zs ticks in ${clockMs.toFixed(1)} ms ` +
    `(${(TICKS / clockMs / 1000).toFixed(2)} M ticks/s), total = ${clock.zeptoseconds} zs`,
);
