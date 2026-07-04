/**
 * Characterization test for MatrixOperations.getEntry and the graph analysis
 * helpers it feeds. Locks current behavior so the getEntry-index refactor can
 * be proven to change speed, not results.
 *
 * Run: npx tsx tests/matrix-getentry-characterization.mts
 */
import assert from 'node:assert/strict';
import { MatrixOperations } from '../src/core/matrix.ts';
import type { Matrix, SparseMatrix, DenseMatrix } from '../src/core/types.ts';

function makeLcg(seed = 1) {
  let s = seed >>> 0;
  return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);
}

// Reference getEntry: the ORIGINAL naive semantics (first-match-wins for COO,
// implicit zero on miss). The refactor must match this exactly.
function refGetEntry(matrix: Matrix, row: number, col: number): number {
  if (matrix.format === 'dense') return (matrix as DenseMatrix).data[row][col];
  const s = matrix as SparseMatrix;
  for (let k = 0; k < s.values.length; k++) {
    if (s.rowIndices[k] === row && s.colIndices[k] === col) return s.values[k];
  }
  return 0;
}

// Build a random COO matrix that deliberately includes DUPLICATE (i,j) entries
// and implicit zeros, to pin the first-match-wins + implicit-zero behavior.
function randomCoo(n: number, rng: () => number): SparseMatrix {
  const values: number[] = [];
  const rowIndices: number[] = [];
  const colIndices: number[] = [];
  const push = (i: number, j: number, v: number) => { rowIndices.push(i); colIndices.push(j); values.push(v); };
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      if (rng() < 0.3) push(i, j, Math.round(rng() * 10) - 5 || 1);
    }
  }
  // inject some duplicate coordinates with different values
  for (let d = 0; d < n; d++) {
    const i = Math.floor(rng() * n), j = Math.floor(rng() * n);
    push(i, j, 100 + d);
    push(i, j, 200 + d); // duplicate: original getEntry returns the FIRST
  }
  return { rows: n, cols: n, values, rowIndices, colIndices, format: 'coo' };
}

function randomDense(n: number, rng: () => number): DenseMatrix {
  const data: number[][] = [];
  for (let i = 0; i < n; i++) {
    const row: number[] = [];
    for (let j = 0; j < n; j++) row.push(rng() < 0.5 ? 0 : Math.round(rng() * 10) - 5);
    data.push(row);
  }
  return { rows: n, cols: n, data, format: 'dense' };
}

const rng = makeLcg(42);
let checks = 0;

for (const n of [1, 5, 20, 50]) {
  for (const matrix of [randomCoo(n, rng), randomDense(n, rng)]) {
    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        const got = MatrixOperations.getEntry(matrix, i, j);
        const want = refGetEntry(matrix, i, j);
        assert.equal(got, want, `getEntry(${matrix.format}, ${i}, ${j}) = ${got}, expected ${want}`);
        checks++;
      }
    }
  }
}

// Reference implementations of the graph helpers (private in graph.ts) so we
// can pin their numeric output across the refactor.
const nodeDegree = (m: Matrix, node: number) => {
  let d = 0;
  for (let j = 0; j < m.cols; j++) d += refGetEntry(m, node, j);
  return d;
};
const countEdges = (m: Matrix) => {
  let e = 0;
  for (let i = 0; i < m.rows; i++) for (let j = 0; j < m.cols; j++) e += refGetEntry(m, i, j);
  return e / 2;
};
const modularity = (m: Matrix, assign: number[]) => {
  const n = m.rows, mm = countEdges(m);
  let q = 0;
  for (let i = 0; i < n; i++) for (let j = 0; j < n; j++) {
    if (assign[i] === assign[j]) q += refGetEntry(m, i, j) - (nodeDegree(m, i) * nodeDegree(m, j)) / (2 * mm);
  }
  return q / (2 * mm);
};

// Emit reference modularity values for a few graphs — the refactored graph.ts
// (degree-hoisted) must reproduce these exactly (checked by the sibling test).
const refModularity: Record<string, number> = {};
const grng = makeLcg(7);
for (const n of [4, 8, 16]) {
  const m = randomCoo(n, grng);
  const assign = Array.from({ length: n }, (_, i) => i % 3);
  refModularity[`n${n}`] = modularity(m, assign);
}

console.log(`getEntry characterization: ${checks} checks passed`);
console.log('reference modularity:', JSON.stringify(refModularity));
