/**
 * Iterative solvers for sparse linear systems A x = b.
 */
import {
  CsrMatrix,
  MatrixInput,
  diagonal,
  diagonalDominance,
  isSymmetric,
  toCsr,
} from './csr.js';
import {
  Limits,
  SublinearError,
  assertIterations,
  assertPositive,
  resolveLimits,
  toFloat64,
} from './errors.js';

export type SolveMethod = 'auto' | 'jacobi' | 'gauss-seidel' | 'neumann' | 'conjugate-gradient';

export interface SolveOptions {
  /** Default 'auto': conjugate-gradient for symmetric matrices with a positive diagonal, otherwise gauss-seidel. */
  method?: SolveMethod;
  /** Stop when ||b - A x||_2 / ||b||_2 <= tolerance. Default 1e-10. */
  tolerance?: number;
  /** Default 1000. */
  maxIterations?: number;
  /** Initial guess. Default zeros. */
  x0?: ArrayLike<number>;
  limits?: Partial<Limits>;
}

export interface SolveResult {
  x: Float64Array;
  /** Final relative residual ||b - A x||_2 / ||b||_2, always recomputed from x. */
  residual: number;
  iterations: number;
  converged: boolean;
  method: Exclude<SolveMethod, 'auto'>;
}

/**
 * Solve A x = b.
 *
 * - jacobi / neumann: converge when the Jacobi iteration matrix has spectral
 *   radius < 1 (guaranteed for strictly diagonally dominant A). The Neumann
 *   series x = sum_k (I - D^-1 A)^k D^-1 b has the same iterates as Jacobi
 *   from x0 = 0; it is kept as a named method for API compatibility.
 * - gauss-seidel: converges for strictly diagonally dominant or symmetric
 *   positive definite A, usually in fewer iterations than Jacobi.
 * - conjugate-gradient: symmetric positive definite A.
 *
 * Cost per iteration is O(nnz). The result's `residual` is the true relative
 * residual of the returned x, so callers can check accuracy directly.
 */
export function solve(matrix: MatrixInput, b: ArrayLike<number>, options: SolveOptions = {}): SolveResult {
  const limits = resolveLimits(options.limits);
  const A = toCsr(matrix, { limits });
  if (A.rows !== A.cols) throw new SublinearError('matrix must be square', 'INVALID_DIMENSIONS');
  const n = A.rows;
  const bv = toFloat64('b', b, n);
  const tol = assertPositive('tolerance', options.tolerance ?? 1e-10);
  const maxIt = assertIterations('maxIterations', options.maxIterations ?? 1000, limits);
  const x = options.x0 === undefined ? new Float64Array(n) : toFloat64('x0', options.x0, n);

  let method = options.method ?? 'auto';
  const diag = diagonal(A);
  if (method === 'auto') {
    method = isSymmetric(A) && diag.every((d) => d > 0) ? 'conjugate-gradient' : 'gauss-seidel';
  }
  if (method !== 'conjugate-gradient') {
    for (let i = 0; i < n; i++) {
      if (diag[i] === 0) throw new SublinearError(`zero diagonal at row ${i}`, 'ZERO_DIAGONAL');
    }
  }
  const bNorm = norm2(bv);
  const scale = bNorm > 0 ? bNorm : 1;

  switch (method) {
    case 'jacobi':
    case 'neumann':
      return jacobi(A, bv, x, diag, tol, maxIt, scale, method);
    case 'gauss-seidel':
      return gaussSeidel(A, bv, x, diag, tol, maxIt, scale);
    case 'conjugate-gradient':
      return conjugateGradient(A, bv, x, tol, maxIt, scale);
    default:
      throw new SublinearError(`unknown method '${String(method)}'`, 'INVALID_INPUT');
  }
}

function norm2(v: Float64Array): number {
  let s = 0;
  for (let i = 0; i < v.length; i++) s += v[i] * v[i];
  return Math.sqrt(s);
}

/** r = b - A x; returns ||r||_2. */
function residualInto(A: CsrMatrix, x: Float64Array, b: Float64Array, r: Float64Array): number {
  const { rowPtr, colIdx, values } = A;
  let s2 = 0;
  for (let i = 0; i < A.rows; i++) {
    let s = b[i];
    for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) s -= values[k] * x[colIdx[k]];
    r[i] = s;
    s2 += s * s;
  }
  return Math.sqrt(s2);
}

function jacobi(
  A: CsrMatrix, b: Float64Array, x: Float64Array, diag: Float64Array,
  tol: number, maxIt: number, scale: number, method: 'jacobi' | 'neumann',
): SolveResult {
  const n = A.rows;
  const r = new Float64Array(n);
  const invD = new Float64Array(n);
  for (let i = 0; i < n; i++) invD[i] = 1 / diag[i];
  let rel = residualInto(A, x, b, r) / scale;
  let it = 0;
  while (rel > tol && it < maxIt) {
    for (let i = 0; i < n; i++) x[i] += r[i] * invD[i];
    rel = residualInto(A, x, b, r) / scale;
    it++;
    if (!Number.isFinite(rel)) break;
  }
  return { x, residual: rel, iterations: it, converged: rel <= tol, method };
}

function gaussSeidel(
  A: CsrMatrix, b: Float64Array, x: Float64Array, diag: Float64Array,
  tol: number, maxIt: number, scale: number,
): SolveResult {
  const n = A.rows;
  const { rowPtr, colIdx, values } = A;
  const r = new Float64Array(n);
  let rel = residualInto(A, x, b, r) / scale;
  let it = 0;
  while (rel > tol && it < maxIt) {
    for (let i = 0; i < n; i++) {
      let s = b[i];
      for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) {
        const j = colIdx[k];
        if (j !== i) s -= values[k] * x[j];
      }
      x[i] = s / diag[i];
    }
    rel = residualInto(A, x, b, r) / scale;
    it++;
    if (!Number.isFinite(rel)) break;
  }
  return { x, residual: rel, iterations: it, converged: rel <= tol, method: 'gauss-seidel' };
}

function conjugateGradient(
  A: CsrMatrix, b: Float64Array, x: Float64Array, tol: number, maxIt: number, scale: number,
): SolveResult {
  const n = A.rows;
  const { rowPtr, colIdx, values } = A;
  const r = new Float64Array(n);
  let rr = residualInto(A, x, b, r) ** 2;
  const p = r.slice();
  const ap = new Float64Array(n);
  let it = 0;
  let rel = Math.sqrt(rr) / scale;
  while (rel > tol && it < maxIt) {
    let pap = 0;
    for (let i = 0; i < n; i++) {
      let s = 0;
      for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) s += values[k] * p[colIdx[k]];
      ap[i] = s;
      pap += p[i] * s;
    }
    if (!(pap > 0)) break; // not positive definite (or breakdown)
    const alpha = rr / pap;
    let rrNew = 0;
    for (let i = 0; i < n; i++) {
      x[i] += alpha * p[i];
      r[i] -= alpha * ap[i];
      rrNew += r[i] * r[i];
    }
    const beta = rrNew / rr;
    rr = rrNew;
    for (let i = 0; i < n; i++) p[i] = r[i] + beta * p[i];
    it++;
    rel = Math.sqrt(rr) / scale;
  }
  // Report the true residual, not the recurrence value.
  rel = residualInto(A, x, b, r) / scale;
  return { x, residual: rel, iterations: it, converged: rel <= tol, method: 'conjugate-gradient' };
}

export interface EstimateEntryOptions {
  /** Number of random walks. Default 10_000. */
  walks?: number;
  /** Seed for the deterministic PRNG. Default 1. */
  seed?: number;
  /** Cap on the length of a single walk. Default 10_000. */
  maxSteps?: number;
  limits?: Partial<Limits>;
}

export interface EstimateEntryResult {
  /** Monte Carlo estimate of x[index]. */
  estimate: number;
  /** Standard error of the estimate (sample std / sqrt(walks)). */
  standardError: number;
  walks: number;
  /** Total walk steps taken; cost is O(steps * average row nnz), independent of n. */
  steps: number;
}

/**
 * Estimate a single entry x[i] of the solution of A x = b by random walks on
 * the Jacobi iteration matrix M = I - D^-1 A, without touching the rest of x.
 *
 * Requires strict row diagonal dominance (so every row of |M| sums to < 1,
 * which makes each walk terminate). Expected cost is
 * walks * (average row nnz) / (1 - ||M||_inf): it does not depend on n,
 * which is the sense in which this is sublinear. Pass a CsrMatrix (not a
 * dense/COO input) to amortize the one-time O(nnz) validation and
 * diagonal-dominance check across calls; they are cached per matrix object.
 */
export function estimateEntry(
  matrix: MatrixInput, b: ArrayLike<number>, index: number, options: EstimateEntryOptions = {},
): EstimateEntryResult {
  const limits = resolveLimits(options.limits);
  const A = toCsr(matrix, { limits });
  if (A.rows !== A.cols) throw new SublinearError('matrix must be square', 'INVALID_DIMENSIONS');
  const n = A.rows;
  if (!Number.isInteger(index) || index < 0 || index >= n) {
    throw new SublinearError(`index ${index} out of range [0, ${n})`, 'INVALID_DIMENSIONS');
  }
  if (!(Array.isArray(b) || ArrayBuffer.isView(b)) || b.length !== n) {
    throw new SublinearError(`b must be an array of length ${n}`, 'INVALID_DIMENSIONS');
  }
  const walks = options.walks ?? 10_000;
  if (!Number.isInteger(walks) || walks <= 0 || walks > limits.maxWalks) {
    throw new SublinearError(`walks must be an integer in [1, ${limits.maxWalks}]`, 'LIMIT_EXCEEDED');
  }
  const maxSteps = assertIterations('maxSteps', options.maxSteps ?? 10_000, limits);
  const { strict, diag } = walkPrep(A);
  if (!strict) {
    throw new SublinearError('estimateEntry requires a strictly row diagonally dominant matrix', 'NOT_DIAGONALLY_DOMINANT');
  }
  // b is read only at visited rows, so validate lazily to keep the per-call cost independent of n.
  const bAt = (u: number): number => {
    const v = b[u];
    if (typeof v !== 'number' || !Number.isFinite(v)) {
      throw new SublinearError(`b[${u}] is not a finite number`, 'INVALID_INPUT');
    }
    return v;
  };
  const rand = mulberry32(options.seed ?? 1);
  const { rowPtr, colIdx, values } = A;

  let sum = 0;
  let sumSq = 0;
  let steps = 0;
  for (let w = 0; w < walks; w++) {
    let u = index;
    let sign = 1;
    let acc = 0;
    for (let s = 0; s <= maxSteps; s++) {
      acc += sign * (bAt(u) / diag[u]);
      steps++;
      // Sample the next state with P(u -> j) = |M_uj| = |a_uj| / |a_uu|; stop otherwise.
      const du = Math.abs(diag[u]);
      let t = rand() * du;
      let next = -1;
      let nextSign = 1;
      for (let k = rowPtr[u], e = rowPtr[u + 1]; k < e; k++) {
        const j = colIdx[k];
        if (j === u) continue;
        const a = values[k];
        t -= Math.abs(a);
        if (t < 0) {
          next = j;
          // M_uj = -a_uj / a_uu
          nextSign = -a / diag[u] < 0 ? -1 : 1;
          break;
        }
      }
      if (next < 0) break;
      sign *= nextSign;
      u = next;
    }
    sum += acc;
    sumSq += acc * acc;
  }
  const mean = sum / walks;
  const variance = walks > 1 ? Math.max(0, (sumSq - walks * mean * mean) / (walks - 1)) : 0;
  return { estimate: mean, standardError: Math.sqrt(variance / walks), walks, steps };
}

const prepCache = new WeakMap<CsrMatrix, { strict: boolean; diag: Float64Array }>();

/** One-time O(nnz) preprocessing per matrix object, reused by later estimateEntry calls. */
function walkPrep(A: CsrMatrix): { strict: boolean; diag: Float64Array } {
  let p = prepCache.get(A);
  if (!p) {
    p = { strict: diagonalDominance(A).strictlyRowDominant, diag: diagonal(A) };
    prepCache.set(A, p);
  }
  return p;
}

/** Small, fast, deterministic PRNG (mulberry32). */
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
