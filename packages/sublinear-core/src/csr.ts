/**
 * Compressed Sparse Row matrices: construction, products and diagnostics.
 */
import {
  Limits,
  SublinearError,
  assertDimension,
  resolveLimits,
  toFloat64,
} from './errors.js';

/** Immutable CSR matrix. Column indices within a row are sorted and unique. */
export interface CsrMatrix {
  readonly rows: number;
  readonly cols: number;
  /** Length rows + 1. Entries of row i live in [rowPtr[i], rowPtr[i+1]). */
  readonly rowPtr: Uint32Array;
  readonly colIdx: Uint32Array;
  readonly values: Float64Array;
}

/** Coordinate (triplet) input. Duplicate (row, col) entries are summed. */
export interface CooInput {
  rows: number;
  cols: number;
  rowIndices: ArrayLike<number>;
  colIndices: ArrayLike<number>;
  values: ArrayLike<number>;
}

export interface BuildOptions {
  limits?: Partial<Limits>;
  /** Drop explicit zeros after summing duplicates. Default true. */
  dropZeros?: boolean;
}

export function isCsrMatrix(m: unknown): m is CsrMatrix {
  const x = m as CsrMatrix;
  return (
    !!x &&
    typeof x === 'object' &&
    x.rowPtr instanceof Uint32Array &&
    x.colIdx instanceof Uint32Array &&
    x.values instanceof Float64Array &&
    Number.isInteger(x.rows) &&
    Number.isInteger(x.cols)
  );
}

/** Build a CSR matrix from COO triplets. O(nnz + rows) time and memory. */
export function csrFromCoo(input: CooInput, options: BuildOptions = {}): CsrMatrix {
  const limits = resolveLimits(options.limits);
  if (!input || typeof input !== 'object') {
    throw new SublinearError('COO input must be an object', 'INVALID_INPUT');
  }
  const rows = assertDimension('rows', input.rows, limits);
  const cols = assertDimension('cols', input.cols, limits);
  const { rowIndices, colIndices, values } = input;
  for (const [name, arr] of [['rowIndices', rowIndices], ['colIndices', colIndices], ['values', values]] as const) {
    if (!(Array.isArray(arr) || ArrayBuffer.isView(arr))) {
      throw new SublinearError(`${name} must be an array or typed array`, 'INVALID_INPUT');
    }
  }
  const nnz = values.length;
  if (rowIndices.length !== nnz || colIndices.length !== nnz) {
    throw new SublinearError('rowIndices, colIndices and values must have equal length', 'INVALID_DIMENSIONS');
  }
  if (nnz > limits.maxNonZeros) {
    throw new SublinearError(`nnz (${nnz}) exceeds limit ${limits.maxNonZeros}`, 'LIMIT_EXCEEDED');
  }

  // Counting sort by row.
  const counts = new Uint32Array(rows + 1);
  for (let k = 0; k < nnz; k++) {
    const r = rowIndices[k];
    const c = colIndices[k];
    const v = values[k];
    if (!Number.isInteger(r) || r < 0 || r >= rows) {
      throw new SublinearError(`rowIndices[${k}] = ${r} out of range [0, ${rows})`, 'INVALID_DIMENSIONS');
    }
    if (!Number.isInteger(c) || c < 0 || c >= cols) {
      throw new SublinearError(`colIndices[${k}] = ${c} out of range [0, ${cols})`, 'INVALID_DIMENSIONS');
    }
    if (typeof v !== 'number' || !Number.isFinite(v)) {
      throw new SublinearError(`values[${k}] is not a finite number`, 'INVALID_INPUT');
    }
    counts[r + 1]++;
  }
  for (let i = 0; i < rows; i++) counts[i + 1] += counts[i];
  const cursor = counts.slice(0, rows);
  const tmpCol = new Uint32Array(nnz);
  const tmpVal = new Float64Array(nnz);
  for (let k = 0; k < nnz; k++) {
    const dst = cursor[rowIndices[k]]++;
    tmpCol[dst] = colIndices[k];
    tmpVal[dst] = values[k];
  }
  return compactRows(rows, cols, counts, tmpCol, tmpVal, options.dropZeros !== false);
}

/** Sort each row by column, merge duplicates, optionally drop zeros. */
function compactRows(
  rows: number,
  cols: number,
  rowPtrIn: Uint32Array,
  colIn: Uint32Array,
  valIn: Float64Array,
  dropZeros: boolean,
): CsrMatrix {
  const rowPtr = new Uint32Array(rows + 1);
  let out = 0;
  let order: number[] = [];
  for (let i = 0; i < rows; i++) {
    const start = rowPtrIn[i];
    const end = rowPtrIn[i + 1];
    rowPtr[i] = out;
    const len = end - start;
    if (len > 1) {
      let sorted = true;
      for (let k = start + 1; k < end; k++) {
        if (colIn[k] <= colIn[k - 1]) { sorted = false; break; }
      }
      if (!sorted) {
        order.length = 0;
        for (let k = start; k < end; k++) order.push(k);
        order.sort((a, b) => colIn[a] - colIn[b]);
        const c2 = new Uint32Array(len);
        const v2 = new Float64Array(len);
        for (let t = 0; t < len; t++) { c2[t] = colIn[order[t]]; v2[t] = valIn[order[t]]; }
        colIn.set(c2, start);
        valIn.set(v2, start);
      }
    }
    let k = start;
    while (k < end) {
      const c = colIn[k];
      let v = valIn[k];
      k++;
      while (k < end && colIn[k] === c) { v += valIn[k]; k++; }
      if (dropZeros && v === 0) continue;
      colIn[out] = c;
      valIn[out] = v;
      out++;
    }
  }
  rowPtr[rows] = out;
  order = [];
  return {
    rows,
    cols,
    rowPtr,
    colIdx: out === colIn.length ? colIn : colIn.slice(0, out),
    values: out === valIn.length ? valIn : valIn.slice(0, out),
  };
}

/** Build a CSR matrix from a dense array of rows. Zeros are not stored. */
export function csrFromDense(data: ArrayLike<ArrayLike<number>>, options: BuildOptions = {}): CsrMatrix {
  const limits = resolveLimits(options.limits);
  if (!(Array.isArray(data) || ArrayBuffer.isView(data))) {
    throw new SublinearError('dense matrix must be an array of rows', 'INVALID_INPUT');
  }
  const rows = assertDimension('rows', data.length, limits);
  const cols = rows === 0 ? 0 : assertDimension('cols', (data[0] as ArrayLike<number>)?.length, limits);
  if (rows * cols > limits.maxDenseElements) {
    throw new SublinearError(`dense input ${rows}x${cols} exceeds limit ${limits.maxDenseElements} elements`, 'LIMIT_EXCEEDED');
  }
  const ri: number[] = [];
  const ci: number[] = [];
  const vs: number[] = [];
  for (let i = 0; i < rows; i++) {
    const row = data[i];
    if (!row || row.length !== cols) {
      throw new SublinearError(`row ${i} must have length ${cols}`, 'INVALID_DIMENSIONS');
    }
    for (let j = 0; j < cols; j++) {
      const v = row[j];
      if (typeof v !== 'number' || !Number.isFinite(v)) {
        throw new SublinearError(`entry (${i}, ${j}) is not a finite number`, 'INVALID_INPUT');
      }
      if (v !== 0) { ri.push(i); ci.push(j); vs.push(v); }
    }
  }
  return csrFromCoo({ rows, cols, rowIndices: ri, colIndices: ci, values: vs }, options);
}

/** Accepts a CsrMatrix, a COO object, or a dense array of rows. */
export type MatrixInput = CsrMatrix | CooInput | ArrayLike<ArrayLike<number>>;

export function toCsr(m: MatrixInput, options: BuildOptions = {}): CsrMatrix {
  if (isCsrMatrix(m)) {
    validateCsr(m, resolveLimits(options.limits));
    return m;
  }
  if (Array.isArray(m) || ArrayBuffer.isView(m)) return csrFromDense(m as ArrayLike<ArrayLike<number>>, options);
  if (m && typeof m === 'object' && 'rowIndices' in m) return csrFromCoo(m as CooInput, options);
  throw new SublinearError('matrix must be a CsrMatrix, a COO object or a dense array of rows', 'INVALID_INPUT');
}

const validated = new WeakSet<CsrMatrix>();

/** Structural check for externally constructed CSR objects. Cached per object. */
export function validateCsr(m: CsrMatrix, limits: Limits = resolveLimits()): void {
  if (validated.has(m)) return;
  assertDimension('rows', m.rows, limits);
  assertDimension('cols', m.cols, limits);
  const { rowPtr, colIdx, values, rows, cols } = m;
  if (rowPtr.length !== rows + 1 || rowPtr[0] !== 0) {
    throw new SublinearError('rowPtr must have length rows + 1 and start at 0', 'INVALID_DIMENSIONS');
  }
  const nnz = rowPtr[rows];
  if (colIdx.length < nnz || values.length < nnz) {
    throw new SublinearError('colIdx/values shorter than rowPtr[rows]', 'INVALID_DIMENSIONS');
  }
  if (nnz > limits.maxNonZeros) {
    throw new SublinearError(`nnz (${nnz}) exceeds limit ${limits.maxNonZeros}`, 'LIMIT_EXCEEDED');
  }
  for (let i = 0; i < rows; i++) {
    const s = rowPtr[i];
    const e = rowPtr[i + 1];
    if (e < s) throw new SublinearError(`rowPtr decreases at row ${i}`, 'INVALID_DIMENSIONS');
    for (let k = s; k < e; k++) {
      if (colIdx[k] >= cols) throw new SublinearError(`column index out of range in row ${i}`, 'INVALID_DIMENSIONS');
      if (k > s && colIdx[k] <= colIdx[k - 1]) {
        throw new SublinearError(`row ${i} columns must be strictly increasing`, 'INVALID_INPUT');
      }
      if (!Number.isFinite(values[k])) throw new SublinearError(`non-finite value in row ${i}`, 'INVALID_INPUT');
    }
  }
  validated.add(m);
}

/** y = A x. Writes into `out` when provided. */
export function spmv(A: CsrMatrix, x: ArrayLike<number>, out?: Float64Array): Float64Array {
  if (x.length !== A.cols) {
    throw new SublinearError(`vector length ${x.length} does not match cols ${A.cols}`, 'INVALID_DIMENSIONS');
  }
  const y = out ?? new Float64Array(A.rows);
  const { rowPtr, colIdx, values } = A;
  for (let i = 0; i < A.rows; i++) {
    let s = 0;
    for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) s += values[k] * x[colIdx[k]];
    y[i] = s;
  }
  return y;
}

export function transpose(A: CsrMatrix): CsrMatrix {
  const { rows, cols, rowPtr, colIdx, values } = A;
  const nnz = rowPtr[rows];
  const tPtr = new Uint32Array(cols + 1);
  for (let k = 0; k < nnz; k++) tPtr[colIdx[k] + 1]++;
  for (let j = 0; j < cols; j++) tPtr[j + 1] += tPtr[j];
  const cursor = tPtr.slice(0, cols);
  const tCol = new Uint32Array(nnz);
  const tVal = new Float64Array(nnz);
  for (let i = 0; i < rows; i++) {
    for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) {
      const d = cursor[colIdx[k]]++;
      tCol[d] = i;
      tVal[d] = values[k];
    }
  }
  return { rows: cols, cols: rows, rowPtr: tPtr, colIdx: tCol, values: tVal };
}

export function diagonal(A: CsrMatrix): Float64Array {
  const n = Math.min(A.rows, A.cols);
  const d = new Float64Array(n);
  const { rowPtr, colIdx, values } = A;
  for (let i = 0; i < n; i++) {
    for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) {
      if (colIdx[k] === i) { d[i] = values[k]; break; }
      if (colIdx[k] > i) break;
    }
  }
  return d;
}

export interface DominanceReport {
  /** Every row satisfies |a_ii| > sum_{j != i} |a_ij|. */
  strictlyRowDominant: boolean;
  /** Every row satisfies |a_ii| >= sum_{j != i} |a_ij|. */
  weaklyRowDominant: boolean;
  /** min_i (|a_ii| - sum_{j != i} |a_ij|) / |a_ii|; -Infinity for a zero diagonal. */
  minMargin: number;
  /** max_i sum_{j != i} |a_ij| / |a_ii|: the infinity norm of the Jacobi iteration matrix. */
  jacobiNorm: number;
}

/** O(nnz) diagonal-dominance diagnostics. */
export function diagonalDominance(A: CsrMatrix): DominanceReport {
  if (A.rows !== A.cols) throw new SublinearError('matrix must be square', 'INVALID_DIMENSIONS');
  const { rowPtr, colIdx, values } = A;
  let strict = true;
  let weak = true;
  let minMargin = Infinity;
  let jacobiNorm = 0;
  for (let i = 0; i < A.rows; i++) {
    let diag = 0;
    let off = 0;
    for (let k = rowPtr[i], e = rowPtr[i + 1]; k < e; k++) {
      if (colIdx[k] === i) diag = Math.abs(values[k]);
      else off += Math.abs(values[k]);
    }
    if (diag === 0) {
      strict = false;
      if (off !== 0) weak = false;
      minMargin = -Infinity;
      jacobiNorm = Infinity;
      continue;
    }
    if (!(diag > off)) strict = false;
    if (!(diag >= off)) weak = false;
    minMargin = Math.min(minMargin, (diag - off) / diag);
    jacobiNorm = Math.max(jacobiNorm, off / diag);
  }
  if (A.rows === 0) minMargin = 0;
  return { strictlyRowDominant: strict, weaklyRowDominant: weak, minMargin, jacobiNorm };
}

/** True when A equals its transpose (exact comparison). O(nnz). */
export function isSymmetric(A: CsrMatrix): boolean {
  if (A.rows !== A.cols) return false;
  const T = transpose(A);
  const n = A.rowPtr[A.rows];
  if (T.rowPtr[A.rows] !== n) return false;
  for (let i = 0; i <= A.rows; i++) if (A.rowPtr[i] !== T.rowPtr[i]) return false;
  for (let k = 0; k < n; k++) {
    if (A.colIdx[k] !== T.colIdx[k] || A.values[k] !== T.values[k]) return false;
  }
  return true;
}

/** Relative residual ||b - A x||_2 / ||b||_2 (absolute when b = 0). */
export function relativeResidual(A: CsrMatrix, x: ArrayLike<number>, b: ArrayLike<number>): number {
  const bv = b instanceof Float64Array ? b : toFloat64('b', b, A.rows);
  const ax = spmv(A, x);
  let rr = 0;
  let bb = 0;
  for (let i = 0; i < A.rows; i++) {
    const r = bv[i] - ax[i];
    rr += r * r;
    bb += bv[i] * bv[i];
  }
  return bb > 0 ? Math.sqrt(rr / bb) : Math.sqrt(rr);
}
