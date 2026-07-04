/**
 * Equivalence test: the degree-hoisted GraphTools.computeModularity must
 * reproduce the exact values the naive O(V^3) reference produced (locked in
 * matrix-getentry-characterization.mts).
 *
 * Run: npx tsx tests/graph-modularity-equivalence.mts
 */
import assert from 'node:assert/strict';
import { GraphTools } from '../src/mcp/tools/graph.ts';
import type { SparseMatrix } from '../src/core/types.ts';

function makeLcg(seed = 1) {
  let s = seed >>> 0;
  return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);
}

// identical generator to the characterization test (seed 7) so graphs match
function randomCoo(n: number, rng: () => number): SparseMatrix {
  const values: number[] = [], rowIndices: number[] = [], colIndices: number[] = [];
  const push = (i: number, j: number, v: number) => { rowIndices.push(i); colIndices.push(j); values.push(v); };
  for (let i = 0; i < n; i++) for (let j = 0; j < n; j++) if (rng() < 0.3) push(i, j, Math.round(rng() * 10) - 5 || 1);
  for (let d = 0; d < n; d++) { const i = Math.floor(rng() * n), j = Math.floor(rng() * n); push(i, j, 100 + d); push(i, j, 200 + d); }
  return { rows: n, cols: n, values, rowIndices, colIndices, format: 'coo' };
}

// values produced by the ORIGINAL implementation (see characterization test)
const REFERENCE: Record<string, number> = {
  n4: -0.3209477023628898,
  n8: 0.06727403001919355,
  n16: 0.028879743941472337,
};

const computeModularity = (GraphTools as any).computeModularity.bind(GraphTools);
const grng = makeLcg(7);
let checks = 0;
for (const n of [4, 8, 16]) {
  const m = randomCoo(n, grng);
  const assign = Array.from({ length: n }, (_, i) => i % 3);
  const got = computeModularity(m, assign);
  assert.equal(got, REFERENCE[`n${n}`], `modularity n=${n}: got ${got}, expected ${REFERENCE[`n${n}`]}`);
  checks++;
}

console.log(`modularity equivalence: ${checks} graphs match the pre-refactor values exactly`);
