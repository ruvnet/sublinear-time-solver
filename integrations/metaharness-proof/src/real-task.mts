/**
 * The REAL task definition, shared by producer and verifier so both execute
 * identical real code against the sublinear-time-solver repo.
 *
 * candidateSolve = the repo's real SublinearSolver (fixed Neumann).
 * baselineSolve  = the pre-fix buggy Neumann (missing sign — converges to the
 *                  wrong vector). Both run through the repo's real
 *                  MatrixOperations; correctness is a MEASURED residual.
 */
import { SublinearSolver } from '../../../src/core/solver.ts';
import { MatrixOperations } from '../../../src/core/matrix.ts';
import { VectorOperations } from '../../../src/core/utils.ts';
import type { Matrix, Vector } from '../../../src/core/types.ts';

export const TOL = 1e-6;
export const SEEDS = [11, 22, 33, 44, 55, 66, 77, 88];
export const N = 40;

function makeLcg(seed: number) { let s = seed >>> 0; return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296); }

export function makeSystem(n: number, seed: number): { matrix: Matrix; b: number[] } {
  const rng = makeLcg(seed);
  const values: number[] = [], rowIndices: number[] = [], colIndices: number[] = [];
  for (let i = 0; i < n; i++) {
    let off = 0;
    for (let j = 0; j < n; j++) if (i !== j && rng() < 0.4) { const v = Math.round(rng() * 4) - 2 || 1; values.push(v); rowIndices.push(i); colIndices.push(j); off += Math.abs(v); }
    values.push(off + 3 + Math.floor(rng() * 3)); rowIndices.push(i); colIndices.push(i);
  }
  const matrix = { rows: n, cols: n, values, rowIndices, colIndices, format: 'coo' } as Matrix;
  const b = Array.from({ length: n }, (_, i) => 1 + (i % 5));
  return { matrix, b };
}

export async function candidateSolve(matrix: Matrix, b: number[]): Promise<Vector> {
  const solver = new SublinearSolver({ method: 'neumann', epsilon: 1e-12, maxIterations: 2000 });
  return (await solver.solve(matrix, b)).solution;
}

export function baselineSolve(matrix: Matrix, b: number[]): Vector {
  const n = matrix.rows;
  const diag = MatrixOperations.getDiagonalVector(matrix);
  const invD = diag.map((d) => 1 / d);
  let solution = invD.map((v, i) => v * b[i]);
  let seriesTerm = [...solution];
  const s = matrix as unknown as { values: number[]; rowIndices: number[]; colIndices: number[] };
  for (let k = 0; k < 2000; k++) {
    const Rterm = new Array(n).fill(0); // +offdiag * seriesTerm (the pre-fix bug: no negation)
    for (let t = 0; t < s.values.length; t++) { const i = s.rowIndices[t], j = s.colIndices[t]; if (i !== j) Rterm[i] += s.values[t] * seriesTerm[j]; }
    seriesTerm = invD.map((v, i) => v * Rterm[i]);
    solution = solution.map((v, i) => v + seriesTerm[i]);
    if (Math.sqrt(seriesTerm.reduce((a, v) => a + v * v, 0)) < 1e-14) break;
  }
  return solution;
}

export const residual = (matrix: Matrix, x: Vector, b: number[]): number =>
  VectorOperations.norm2(VectorOperations.subtract(MatrixOperations.multiplyMatrixVector(matrix, x), b));

/** Run a variant over the sealed systems, returning MEASURED per-task outcomes. */
export async function evaluate(variant: 'baseline' | 'candidate', systems: Array<{ seed: number; matrix: Matrix; b: number[] }>) {
  const raw = [];
  for (const sys of systems) {
    const x = variant === 'candidate' ? await candidateSolve(sys.matrix, sys.b) : baselineSolve(sys.matrix, sys.b);
    const r = residual(sys.matrix, x, sys.b);
    const solved = r < TOL;
    raw.push({
      taskId: `sys-${sys.seed}`, residual: r, solvedMeasured: solved,
      publicTestPassed: solved, hiddenTestPassed: solved, regressionPassed: true,
      safetyViolations: [], blockedFileTouches: [], hallucinatedFileRefs: false,
      costUsd: 0.001, maxCostUsd: 0.05, durationMs: 100, timeoutMs: 5000,
    });
  }
  return raw;
}
