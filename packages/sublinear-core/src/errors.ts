/**
 * Error type and input limits shared by every entry point.
 *
 * All public functions validate their inputs before allocating, so a caller
 * that forwards untrusted data (an MCP tool argument, a JSON request body)
 * cannot make the library allocate unbounded memory or loop forever.
 */

export type SublinearErrorCode =
  | 'INVALID_INPUT'
  | 'INVALID_DIMENSIONS'
  | 'LIMIT_EXCEEDED'
  | 'NOT_DIAGONALLY_DOMINANT'
  | 'ZERO_DIAGONAL'
  | 'UNKNOWN_NODE';

export class SublinearError extends Error {
  readonly code: SublinearErrorCode;
  constructor(message: string, code: SublinearErrorCode) {
    super(message);
    this.name = 'SublinearError';
    this.code = code;
  }
}

/** Hard ceilings. Override per call via `limits` options where offered. */
export interface Limits {
  /** Max rows/cols of a matrix or nodes of a graph. */
  maxDimension: number;
  /** Max stored non-zeros / edges. */
  maxNonZeros: number;
  /** Max rows*cols accepted from a dense (array-of-rows) input. */
  maxDenseElements: number;
  /** Max iterations any iterative method may be asked to run. */
  maxIterations: number;
  /** Max push operations for local push methods. */
  maxPushes: number;
  /** Max random walks for Monte Carlo estimation. */
  maxWalks: number;
}

export const DEFAULT_LIMITS: Readonly<Limits> = Object.freeze({
  maxDimension: 100_000_000,
  maxNonZeros: 1_000_000_000,
  maxDenseElements: 25_000_000,
  maxIterations: 1_000_000,
  maxPushes: 100_000_000,
  maxWalks: 10_000_000,
});

export function resolveLimits(partial?: Partial<Limits>): Limits {
  if (!partial) return DEFAULT_LIMITS;
  const out: Limits = { ...DEFAULT_LIMITS };
  for (const key of Object.keys(DEFAULT_LIMITS) as (keyof Limits)[]) {
    const v = partial[key];
    if (v !== undefined) {
      if (!Number.isFinite(v) || v <= 0) {
        throw new SublinearError(`limits.${key} must be a positive number`, 'INVALID_INPUT');
      }
      out[key] = v;
    }
  }
  return out;
}

export function assertDimension(name: string, v: unknown, limits: Limits): number {
  if (typeof v !== 'number' || !Number.isInteger(v) || v < 0) {
    throw new SublinearError(`${name} must be a non-negative integer`, 'INVALID_DIMENSIONS');
  }
  if (v > limits.maxDimension) {
    throw new SublinearError(`${name} (${v}) exceeds limit ${limits.maxDimension}`, 'LIMIT_EXCEEDED');
  }
  return v;
}

export function assertIterations(name: string, v: unknown, limits: Limits): number {
  if (typeof v !== 'number' || !Number.isInteger(v) || v < 0) {
    throw new SublinearError(`${name} must be a non-negative integer`, 'INVALID_INPUT');
  }
  if (v > limits.maxIterations) {
    throw new SublinearError(`${name} (${v}) exceeds limit ${limits.maxIterations}`, 'LIMIT_EXCEEDED');
  }
  return v;
}

export function assertPositive(name: string, v: unknown): number {
  if (typeof v !== 'number' || !Number.isFinite(v) || v <= 0) {
    throw new SublinearError(`${name} must be a positive finite number`, 'INVALID_INPUT');
  }
  return v;
}

export function assertProbability(name: string, v: unknown): number {
  if (typeof v !== 'number' || !Number.isFinite(v) || v < 0 || v >= 1) {
    throw new SublinearError(`${name} must be in [0, 1)`, 'INVALID_INPUT');
  }
  return v;
}

export function toFloat64(name: string, v: unknown, expectedLength: number): Float64Array {
  if (!(Array.isArray(v) || ArrayBuffer.isView(v))) {
    throw new SublinearError(`${name} must be an array or typed array`, 'INVALID_INPUT');
  }
  const arr = v as ArrayLike<number>;
  if (arr.length !== expectedLength) {
    throw new SublinearError(`${name} length ${arr.length} does not match ${expectedLength}`, 'INVALID_DIMENSIONS');
  }
  const out = new Float64Array(expectedLength);
  for (let i = 0; i < expectedLength; i++) {
    const x = arr[i];
    if (typeof x !== 'number' || !Number.isFinite(x)) {
      throw new SublinearError(`${name}[${i}] is not a finite number`, 'INVALID_INPUT');
    }
    out[i] = x;
  }
  return out;
}
