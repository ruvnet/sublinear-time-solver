#!/usr/bin/env node
/**
 * Sandboxed evaluator for one solver genome. Runs as a SEPARATE subprocess so
 * a runaway genome is killed by the parent's timeout. Imports the genome,
 * solves every system, and prints measured outcomes as JSON.
 *
 * Usage: node genome-runner.mjs <genome.mjs> <systems.json>
 * Output (stdout): JSON [{ taskId, residual, solved, ms }]
 */
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { performance } from 'node:perf_hooks';

const TOL = 1e-6;
const [, , genomePath, systemsPath] = process.argv;

function matvec(matrix, x) {
  const r = new Array(matrix.rows).fill(0);
  const { values, rowIndices, colIndices } = matrix;
  for (let k = 0; k < values.length; k++) r[rowIndices[k]] += values[k] * x[colIndices[k]];
  return r;
}
function residual(matrix, x, b) {
  if (!Array.isArray(x) || x.length !== matrix.rows) return Infinity;
  const ax = matvec(matrix, x);
  let s = 0;
  for (let i = 0; i < matrix.rows; i++) { const d = ax[i] - b[i]; if (!Number.isFinite(d)) return Infinity; s += d * d; }
  return Math.sqrt(s);
}

const systems = JSON.parse(readFileSync(systemsPath, 'utf8'));
const mod = await import(pathToFileURL(genomePath).href);
if (typeof mod.solve !== 'function') { console.log(JSON.stringify({ error: 'genome does not export solve()' })); process.exit(0); }

const out = [];
for (const sys of systems) {
  let residualVal = Infinity, ms = 0;
  try {
    const t0 = performance.now();
    const x = mod.solve(sys.matrix, sys.b);
    ms = performance.now() - t0;
    residualVal = residual(sys.matrix, x, sys.b);
  } catch (e) {
    residualVal = Infinity;
  }
  out.push({ taskId: `sys-${sys.seed}`, residual: Number.isFinite(residualVal) ? residualVal : 1e9, solved: residualVal < TOL, ms: Number(ms.toFixed(2)) });
}
console.log(JSON.stringify(out));
