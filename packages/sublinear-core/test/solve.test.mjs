import test from 'node:test';
import assert from 'node:assert/strict';
import {
  solve, estimateEntry, csrFromDense, csrFromCoo, relativeResidual, diagonalDominance, SublinearError, spmv,
} from '../dist/esm/index.js';
import { denseSolve, randomDD, randomVector, maxAbsDiff } from './helpers.mjs';

const METHODS = ['jacobi', 'neumann', 'gauss-seidel'];

test('every method matches dense Gaussian elimination on DD systems', () => {
  for (const seed of [1, 2, 3, 4, 5]) {
    const n = 40;
    const A = randomDD(n, 6, seed);
    const b = randomVector(n, seed + 100);
    const exact = denseSolve(A, b);
    for (const method of METHODS) {
      const r = solve(A, b, { method, tolerance: 1e-12, maxIterations: 5000 });
      assert.ok(r.converged, `${method} seed ${seed} did not converge`);
      assert.ok(r.residual <= 1e-12);
      assert.ok(maxAbsDiff(r.x, exact) < 1e-9, `${method} error ${maxAbsDiff(r.x, exact)}`);
    }
  }
});

test('conjugate gradient and auto on symmetric positive definite systems', () => {
  const n = 60;
  const A = randomDD(n, 5, 7, { symmetric: true });
  const b = randomVector(n, 8);
  const exact = denseSolve(A, b);
  const cg = solve(A, b, { method: 'conjugate-gradient', tolerance: 1e-12 });
  assert.ok(cg.converged);
  assert.ok(maxAbsDiff(cg.x, exact) < 1e-9);
  const auto = solve(A, b, { tolerance: 1e-12 });
  assert.equal(auto.method, 'conjugate-gradient');
  const nonsym = randomDD(n, 5, 9);
  assert.equal(solve(nonsym, b).method, 'gauss-seidel');
});

test('reported residual is the true residual of x', () => {
  const A = randomDD(30, 4, 11);
  const b = randomVector(30, 12);
  const r = solve(A, b, { method: 'jacobi', maxIterations: 3, tolerance: 1e-14 });
  assert.equal(r.converged, false);
  assert.equal(r.iterations, 3);
  const csr = csrFromDense(A);
  assert.ok(Math.abs(relativeResidual(csr, r.x, b) - r.residual) < 1e-15);
});

test('COO duplicates are summed and dense/COO/CSR inputs agree', () => {
  const coo = { rows: 2, cols: 2, rowIndices: [0, 0, 1, 1, 0], colIndices: [0, 1, 1, 0, 0], values: [2, 1, 3, 1, 2] };
  const A = csrFromCoo(coo);
  assert.deepEqual([...A.rowPtr], [0, 2, 4]);
  assert.deepEqual([...A.values], [4, 1, 1, 3]);
  const b = [1, 2];
  const x1 = solve(coo, b).x;
  const x2 = solve([[4, 1], [1, 3]], b).x;
  const x3 = solve(A, b).x;
  assert.ok(maxAbsDiff(x1, x2) < 1e-12 && maxAbsDiff(x2, x3) < 1e-12);
  assert.deepEqual([...spmv(A, [1, 1])], [5, 4]);
});

test('diagonal dominance report', () => {
  const rep = diagonalDominance(csrFromDense([[3, -1, 1], [1, 2, 1], [0, 1, 5]]));
  assert.equal(rep.strictlyRowDominant, false);
  assert.equal(rep.weaklyRowDominant, true);
  assert.equal(rep.minMargin, 0);
  assert.equal(rep.jacobiNorm, 1);
});

test('estimateEntry is unbiased within 5 standard errors', () => {
  const n = 200;
  const A = randomDD(n, 4, 21, { margin: 2 });
  const b = randomVector(n, 22);
  const exact = denseSolve(A, b);
  for (const i of [0, 17, 199]) {
    const e = estimateEntry(A, b, i, { walks: 20000, seed: i + 1 });
    assert.ok(Math.abs(e.estimate - exact[i]) < 5 * e.standardError + 1e-12,
      `entry ${i}: ${e.estimate} vs ${exact[i]} (se ${e.standardError})`);
  }
  assert.throws(() => estimateEntry([[1, 2], [2, 1]], [1, 1], 0), (err) => err.code === 'NOT_DIAGONALLY_DOMINANT');
});

test('input validation rejects hostile or malformed input before allocating', () => {
  const bad = (fn, code) => assert.throws(fn, (e) => e instanceof SublinearError && e.code === code);
  bad(() => solve({ rows: 1e12, cols: 1e12, rowIndices: [], colIndices: [], values: [] }, []), 'LIMIT_EXCEEDED');
  bad(() => solve({ rows: 2, cols: 2, rowIndices: [0, 5], colIndices: [0, 1], values: [1, 1] }, [1, 1]), 'INVALID_DIMENSIONS');
  bad(() => solve([[1, NaN], [0, 1]], [1, 1]), 'INVALID_INPUT');
  bad(() => solve([[1, 0], [0, 1]], [1, Infinity]), 'INVALID_INPUT');
  bad(() => solve([[1, 0], [0, 1]], [1]), 'INVALID_DIMENSIONS');
  bad(() => solve([[1, 0], [0, 1]], [1, 1], { maxIterations: 1e9 }), 'LIMIT_EXCEEDED');
  bad(() => solve([[1, 0], [0, 1]], [1, 1], { tolerance: -1 }), 'INVALID_INPUT');
  bad(() => solve([[0, 1], [1, 0]], [1, 1], { method: 'jacobi' }), 'ZERO_DIAGONAL');
  bad(() => solve([[1, 2, 3]], [1]), 'INVALID_DIMENSIONS');
  bad(() => solve([[1, 0], [0, 1]], [1, 1], { method: 'bogus' }), 'INVALID_INPUT');
  const big = Array.from({ length: 6000 }, () => ({ length: 6000 }));
  bad(() => solve(big, []), 'LIMIT_EXCEEDED');
  bad(() => solve([[1]], [1], { limits: { maxDimension: -1 } }), 'INVALID_INPUT');
});

test('empty system', () => {
  const r = solve({ rows: 0, cols: 0, rowIndices: [], colIndices: [], values: [] }, []);
  assert.equal(r.x.length, 0);
  assert.equal(r.converged, true);
});
