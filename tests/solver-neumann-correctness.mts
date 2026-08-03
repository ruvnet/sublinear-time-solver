/**
 * Correctness guard for the core Neumann solver (src/core/solver.ts).
 *
 * Regression test for a sign bug: the iteration matrix is
 * M = I - D^-1 A = -D^-1 * offdiag(A), but the term update used +offdiag,
 * summing (-M)^k instead of M^k and converging to the wrong vector.
 *
 * Run: npx tsx tests/solver-neumann-correctness.mts
 */
import assert from 'node:assert/strict';
import { SublinearSolver } from '../src/core/solver.ts';
import { SimpleSublinearSolver } from '../src/mcp/tools/simple-wasm-solver.ts';
import { MatrixOperations } from '../src/core/matrix.ts';
import type { Matrix } from '../src/core/types.ts';

function residual(matrix: Matrix, x: number[], b: number[]): number {
  const Ax = MatrixOperations.multiplyMatrixVector(matrix, x);
  return Math.sqrt(Ax.reduce((s, v, i) => s + (v - b[i]) ** 2, 0));
}

function denseResidual(A: number[][], x: number[], b: number[]): number {
  return Math.sqrt(A.map((row, i) => row.reduce((s, a, j) => s + a * x[j], 0) - b[i]).reduce((s, v) => s + v * v, 0));
}

async function solveResidual(matrix: Matrix, b: number[]): Promise<number> {
  const solver = new SublinearSolver({ method: 'neumann', epsilon: 1e-12, maxIterations: 2000 });
  const res = await solver.solve(matrix, b);
  return residual(matrix, res.solution, b);
}

const cases: Array<{ name: string; matrix: Matrix; b: number[]; expect?: number[] }> = [
  {
    name: 'tridiagonal 3x3',
    matrix: { rows: 3, cols: 3, format: 'coo',
      values: [4, -1, -1, 4, -1, -1, 4], rowIndices: [0, 0, 1, 1, 1, 2, 2], colIndices: [0, 1, 0, 1, 2, 1, 2] } as Matrix,
    b: [1, 2, 3],
    expect: [0.4642857, 0.8571429, 0.9642857],
  },
  {
    name: 'pure diagonal 2x2',
    matrix: { rows: 2, cols: 2, format: 'coo', values: [2, 3], rowIndices: [0, 1], colIndices: [0, 1] } as Matrix,
    b: [4, 9],
    expect: [2, 3],
  },
  {
    name: 'dense dominant 3x3',
    matrix: { rows: 3, cols: 3, format: 'dense', data: [[10, 1, 2], [1, 12, 3], [2, 3, 15]] } as unknown as Matrix,
    b: [1, 1, 1],
  },
];

let passed = 0;
for (const c of cases) {
  const solver = new SublinearSolver({ method: 'neumann', epsilon: 1e-12, maxIterations: 2000 });
  const res = await solver.solve(c.matrix, c.b);
  const r = residual(c.matrix, res.solution, c.b);
  assert.ok(r < 1e-6, `${c.name}: residual ${r.toExponential(2)} not < 1e-6 (solver did not converge to A x = b)`);
  if (c.expect) {
    for (let i = 0; i < c.expect.length; i++) {
      assert.ok(Math.abs(res.solution[i] - c.expect[i]) < 1e-4,
        `${c.name}: x[${i}] = ${res.solution[i]} != expected ${c.expect[i]}`);
    }
  }
  passed++;
}

// Guard the second Neumann implementation too (simple-wasm-solver's
// solveNeumann builds N = I - D^-1 A explicitly). Confirms the sign-bug class
// found in the core solver is not present here either.
{
  const A = [[4, -1, 0], [-1, 4, -1], [0, -1, 4]];
  const b = [1, 2, 3];
  const s = new SimpleSublinearSolver(0.1, 200) as unknown as { solveNeumann(m: number[][], v: number[]): number[] };
  const x = s.solveNeumann(A, b);
  const r = denseResidual(A, x, b);
  assert.ok(r < 1e-5, `SimpleSublinearSolver.solveNeumann residual ${r.toExponential(2)} not < 1e-5`);
  passed++;
}

console.log(`Neumann correctness: ${passed} solves reach A x = b (both core and simple-wasm implementations)`);
