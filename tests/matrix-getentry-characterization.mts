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

// --- checkDiagonalDominance / isSymmetric behavior lock ---------------------
// Naive references replicating the ORIGINAL semantics exactly:
//  - diagonal via getEntry (first-match / implicit zero)
//  - off-diagonal row/col sums over ALL stored entries (duplicates included),
//    using |value|, excluding the diagonal coordinate.
function refDominance(m: Matrix): { isRowDD: boolean; isColDD: boolean; strength: number } {
  if (m.rows !== m.cols) return { isRowDD: false, isColDD: false, strength: 0 };
  const n = m.rows;
  const rowSum = (r: number) => {
    let s = 0;
    if (m.format === 'dense') { for (let j = 0; j < n; j++) if (j !== r) s += Math.abs((m as DenseMatrix).data[r][j]); }
    else { const sp = m as SparseMatrix; for (let k = 0; k < sp.values.length; k++) if (sp.rowIndices[k] === r && sp.colIndices[k] !== r) s += Math.abs(sp.values[k]); }
    return s;
  };
  const colSum = (c: number) => {
    let s = 0;
    if (m.format === 'dense') { for (let i = 0; i < n; i++) if (i !== c) s += Math.abs((m as DenseMatrix).data[i][c]); }
    else { const sp = m as SparseMatrix; for (let k = 0; k < sp.values.length; k++) if (sp.colIndices[k] === c && sp.rowIndices[k] !== c) s += Math.abs(sp.values[k]); }
    return s;
  };
  let isRowDD = true, isColDD = true, minRow = Infinity, minCol = Infinity;
  for (let i = 0; i < n; i++) {
    const d = Math.abs(refGetEntry(m, i, i));
    if (d === 0) { return { isRowDD: false, isColDD: false, strength: 0 }; }
    const rs = d - rowSum(i), cs = d - colSum(i);
    if (rs < 0) isRowDD = false; else minRow = Math.min(minRow, rs / d);
    if (cs < 0) isColDD = false; else minCol = Math.min(minCol, cs / d);
  }
  return { isRowDD, isColDD, strength: Math.max(isRowDD ? minRow : 0, isColDD ? minCol : 0) };
}
function refSymmetric(m: Matrix, tol = 1e-10): boolean {
  if (m.rows !== m.cols) return false;
  for (let i = 0; i < m.rows; i++) for (let j = i + 1; j < m.cols; j++) {
    if (Math.abs(refGetEntry(m, i, j) - refGetEntry(m, j, i)) > tol) return false;
  }
  return true;
}

// Build a diagonally-dominant COO so the DD branch is exercised too.
function ddCoo(n: number, rng: () => number): SparseMatrix {
  const values: number[] = [], rowIndices: number[] = [], colIndices: number[] = [];
  for (let i = 0; i < n; i++) {
    let off = 0;
    for (let j = 0; j < n; j++) if (i !== j && rng() < 0.2) { const v = Math.round(rng() * 3) + 1; values.push(v); rowIndices.push(i); colIndices.push(j); off += v; }
    values.push(off + 5); rowIndices.push(i); colIndices.push(i); // dominant diagonal
  }
  return { rows: n, cols: n, values, rowIndices, colIndices, format: 'coo' };
}

const drng = makeLcg(11);
let ddChecks = 0;
for (const n of [1, 4, 12, 30]) {
  const mats: Matrix[] = [randomCoo(n, drng), randomDense(n, drng), ddCoo(n, drng)];
  for (const m of mats) {
    assert.deepEqual(MatrixOperations.checkDiagonalDominance(m), refDominance(m), `dominance mismatch (${m.format}, n=${n})`);
    assert.equal(MatrixOperations.isSymmetric(m), refSymmetric(m), `symmetry mismatch (${m.format}, n=${n})`);
    ddChecks += 2;
  }
}
console.log(`dominance/symmetry characterization: ${ddChecks} checks passed`);
