/**
 * SublinearSolver: the 1.x class API, now a thin adapter over the
 * dependency-free @ruvnet/sublinear-core package.
 *
 * Behavioural notes (1.8.0):
 * - Every method returns a solution whose `residual` is the true
 *   ||A x - b||_2 recomputed from the returned x.
 * - Non-convergence is reported with `converged: false` instead of a throw.
 * - 'forward-push', 'backward-push' and 'bidirectional' run Gauss-Seidel for
 *   full linear solves (the earlier implementations reported convergence at
 *   several percent error). For local PageRank push use forwardPush /
 *   backwardPush from the core API.
 * - Nothing is written to stdout; set SUBLINEAR_DEBUG=1 for stderr diagnostics.
 */
import { solve as coreSolve, estimateEntry as coreEstimateEntry, csrFromCoo, csrFromDense, diagonalDominance, transpose, graphFromCsr, pageRank as corePageRank, personalizedPageRank as corePersonalizedPageRank, relativeResidual, SublinearError, } from '@ruvnet/sublinear-core';
import { SolverError, ErrorCodes, } from './types.js';
import { ValidationUtils } from './utils.js';
const debug = (...args) => {
    if (typeof process !== 'undefined' && process.env?.SUBLINEAR_DEBUG)
        console.error('[sublinear]', ...args);
};
/** Map core errors onto the legacy SolverError codes. */
function wrap(err) {
    if (err instanceof SolverError)
        throw err;
    if (err instanceof SublinearError) {
        const code = err.code === 'NOT_DIAGONALLY_DOMINANT' ? ErrorCodes.NOT_DIAGONALLY_DOMINANT
            : err.code === 'INVALID_DIMENSIONS' ? ErrorCodes.INVALID_DIMENSIONS
                : err.code === 'LIMIT_EXCEEDED' ? ErrorCodes.MEMORY_LIMIT_EXCEEDED
                    : err.code === 'ZERO_DIAGONAL' ? ErrorCodes.NUMERICAL_INSTABILITY
                        : ErrorCodes.INVALID_PARAMETERS;
        throw new SolverError(err.message, code);
    }
    throw new SolverError(`Solver failed: ${err instanceof Error ? err.message : String(err)}`, ErrorCodes.CONVERGENCE_FAILED);
}
/** Convert a legacy dense/COO matrix into a validated core CSR matrix. */
export function toCoreMatrix(matrix) {
    if (!matrix || typeof matrix !== 'object') {
        throw new SolverError('Matrix must be an object', ErrorCodes.INVALID_MATRIX);
    }
    try {
        if (matrix.format === 'dense') {
            const dense = matrix;
            if (!Array.isArray(dense.data) || dense.data.length !== dense.rows) {
                throw new SolverError(`Dense matrix data must have ${dense.rows} rows`, ErrorCodes.INVALID_DIMENSIONS);
            }
            const A = csrFromDense(dense.data);
            if (A.cols !== dense.cols) {
                throw new SolverError(`Dense matrix data must have ${dense.cols} columns`, ErrorCodes.INVALID_DIMENSIONS);
            }
            return A;
        }
        if (matrix.format === 'coo' || matrix.format === 'csr' || matrix.format === 'csc') {
            const s = matrix;
            return csrFromCoo({ rows: s.rows, cols: s.cols, rowIndices: s.rowIndices, colIndices: s.colIndices, values: s.values });
        }
    }
    catch (e) {
        wrap(e);
    }
    throw new SolverError(`Unsupported matrix format: ${matrix.format}`, ErrorCodes.INVALID_MATRIX);
}
function norm2(v) {
    let s = 0;
    for (let i = 0; i < v.length; i++)
        s += v[i] * v[i];
    return Math.sqrt(s);
}
export class SublinearSolver {
    config;
    constructor(config) {
        this.config = config;
        ValidationUtils.validatePositiveNumber(config.epsilon, 'epsilon');
        ValidationUtils.validateIntegerRange(config.maxIterations, 1, 1e6, 'maxIterations');
        if (config.timeout)
            ValidationUtils.validatePositiveNumber(config.timeout, 'timeout');
    }
    /**
     * Solve A x = b. `epsilon` bounds the absolute residual ||A x - b||_2.
     */
    async solve(matrix, vector, progressCallback) {
        const start = Date.now();
        const A = toCoreMatrix(matrix);
        if (A.rows !== A.cols) {
            throw new SolverError('Matrix must be square', ErrorCodes.INVALID_DIMENSIONS);
        }
        if (!Array.isArray(vector) || vector.length !== A.rows) {
            throw new SolverError(`Vector length ${Array.isArray(vector) ? vector.length : 'n/a'} does not match matrix dimension ${A.rows}`, ErrorCodes.INVALID_DIMENSIONS);
        }
        const rowDom = diagonalDominance(A);
        if (!rowDom.weaklyRowDominant && !diagonalDominance(transpose(A)).weaklyRowDominant) {
            throw new SolverError('Matrix is not diagonally dominant', ErrorCodes.NOT_DIAGONALLY_DOMINANT, { minMargin: rowDom.minMargin });
        }
        const requested = this.config.method;
        let x;
        let iterations;
        let method;
        try {
            if (requested === 'random-walk') {
                ({ x, iterations } = this.randomWalkSolve(A, vector));
                method = 'random-walk';
            }
            else {
                const coreMethod = requested === 'neumann' ? 'neumann'
                    : requested === 'forward-push' || requested === 'backward-push' || requested === 'bidirectional' ? 'gauss-seidel'
                        : (() => { throw new SolverError(`Unknown method: ${String(requested)}`, ErrorCodes.INVALID_PARAMETERS); })();
                const bNorm = norm2(vector);
                const tolerance = bNorm > 0 ? Math.max(this.config.epsilon / bNorm, Number.EPSILON) : this.config.epsilon;
                const r = coreSolve(A, vector, { method: coreMethod, tolerance, maxIterations: this.config.maxIterations });
                x = Array.from(r.x);
                iterations = r.iterations;
                method = r.method;
            }
        }
        catch (e) {
            wrap(e);
        }
        const bNorm = norm2(vector);
        const residual = relativeResidual(A, x, vector) * (bNorm > 0 ? bNorm : 1);
        const result = {
            solution: x,
            iterations: iterations,
            residual,
            converged: residual <= this.config.epsilon,
            method: method,
            computeTime: Date.now() - start,
            memoryUsed: Math.round((A.rowPtr.byteLength + A.colIdx.byteLength + A.values.byteLength + 16 * A.rows) / 1024),
        };
        debug(`solve ${requested} -> ${result.method}: n=${A.rows} iters=${result.iterations} residual=${residual.toExponential(3)}`);
        progressCallback?.({ iteration: result.iterations, residual, elapsed: result.computeTime });
        return result;
    }
    /**
     * Monte Carlo solve: every entry is estimated independently by random walks.
     * Accuracy is statistical (standard error ~ 1/sqrt(walks)); use 'neumann'
     * when you need the full vector to high precision.
     */
    randomWalkSolve(A, b) {
        const n = A.rows;
        // Walks per entry: ~1/epsilon, bounded so the total work stays reasonable.
        const budget = 2_000_000;
        const walks = Math.max(16, Math.min(Math.ceil(1 / this.config.epsilon), 4000, Math.floor(budget / Math.max(1, n))));
        const seed = this.config.seed ?? 1;
        const x = new Array(n);
        for (let i = 0; i < n; i++) {
            x[i] = coreEstimateEntry(A, b, i, { walks, seed: (seed + i * 2654435761) >>> 0 }).estimate;
        }
        return { x, iterations: walks };
    }
    /**
     * Estimate one entry. method 'random-walk' / 'monte-carlo' estimates
     * x[row] of A x = b by random walks; otherwise returns (A^-1)[row][column]
     * via a solve against the unit vector e_column.
     */
    async estimateEntry(matrix, vector, config) {
        const A = toCoreMatrix(matrix);
        if (!Number.isInteger(config.row) || config.row < 0 || config.row >= A.rows) {
            throw new SolverError(`Row index ${config.row} out of bounds (0-${A.rows - 1})`, ErrorCodes.INVALID_PARAMETERS);
        }
        if (!Number.isInteger(config.column) || config.column < 0 || config.column >= A.cols) {
            throw new SolverError(`Column index ${config.column} out of bounds (0-${A.cols - 1})`, ErrorCodes.INVALID_PARAMETERS);
        }
        if (!Array.isArray(vector) || vector.length !== A.rows) {
            throw new SolverError(`Vector length does not match matrix rows ${A.rows}`, ErrorCodes.INVALID_DIMENSIONS);
        }
        ValidationUtils.validatePositiveNumber(config.epsilon, 'epsilon');
        ValidationUtils.validateRange(config.confidence, 0, 1, 'confidence');
        try {
            if (config.method === 'random-walk' || config.method === 'monte-carlo') {
                const walks = Math.max(100, Math.min(Math.ceil(1 / (config.epsilon * config.epsilon)), 1_000_000));
                const r = coreEstimateEntry(A, vector, config.row, { walks, seed: this.config.seed ?? 1 });
                return { estimate: r.estimate, variance: r.standardError * r.standardError * r.walks, confidence: config.confidence };
            }
            const e = new Array(A.cols).fill(0);
            e[config.column] = 1;
            const r = coreSolve(A, e, { method: 'gauss-seidel', tolerance: Math.max(config.epsilon, 1e-14), maxIterations: this.config.maxIterations });
            return { estimate: r.x[config.row], variance: 0, confidence: r.converged ? 1.0 : 0.5 };
        }
        catch (err) {
            wrap(err);
        }
    }
    /**
     * PageRank of a weighted adjacency matrix (entry [i][j] = weight of edge i -> j).
     * Scores sum to 1: the mass of dangling nodes is redistributed by the
     * teleport vector. `personalized` (non-negative, any scale) selects
     * personalized PageRank; it is normalized internally.
     */
    async computePageRank(adjacency, config) {
        ValidationUtils.validateRange(config.damping, 0, 1, 'damping');
        ValidationUtils.validatePositiveNumber(config.epsilon, 'epsilon');
        const A = toCoreMatrix(adjacency);
        if (A.rows !== A.cols) {
            throw new SolverError('Adjacency matrix must be square', ErrorCodes.INVALID_DIMENSIONS);
        }
        const damping = config.damping >= 1 ? 1 - 1e-12 : config.damping;
        const opts = { damping, tolerance: config.epsilon, maxIterations: Math.min(config.maxIterations ?? 100, 1_000_000) };
        try {
            const g = graphFromCsr(A);
            if (config.personalized) {
                if (!Array.isArray(config.personalized) || config.personalized.length !== A.rows) {
                    throw new SolverError('personalized must be a vector of length n', ErrorCodes.INVALID_DIMENSIONS);
                }
                const seeds = new Map();
                config.personalized.forEach((w, i) => { if (w !== 0)
                    seeds.set(i, w); });
                return Array.from(corePersonalizedPageRank(g, seeds, opts).scores);
            }
            return Array.from(corePageRank(g, opts).scores);
        }
        catch (err) {
            wrap(err);
        }
    }
}
