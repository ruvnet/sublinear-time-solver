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
import { type CsrMatrix } from '@ruvnet/sublinear-core';
import { Matrix, Vector, SolverConfig, SolverResult, EstimationConfig, PageRankConfig, ProgressCallback } from './types.js';
/** Convert a legacy dense/COO matrix into a validated core CSR matrix. */
export declare function toCoreMatrix(matrix: Matrix): CsrMatrix;
export declare class SublinearSolver {
    private config;
    constructor(config: SolverConfig);
    /**
     * Solve A x = b. `epsilon` bounds the absolute residual ||A x - b||_2.
     */
    solve(matrix: Matrix, vector: Vector, progressCallback?: ProgressCallback): Promise<SolverResult>;
    /**
     * Monte Carlo solve: every entry is estimated independently by random walks.
     * Accuracy is statistical (standard error ~ 1/sqrt(walks)); use 'neumann'
     * when you need the full vector to high precision.
     */
    private randomWalkSolve;
    /**
     * Estimate one entry. method 'random-walk' / 'monte-carlo' estimates
     * x[row] of A x = b by random walks; otherwise returns (A^-1)[row][column]
     * via a solve against the unit vector e_column.
     */
    estimateEntry(matrix: Matrix, vector: Vector, config: EstimationConfig): Promise<{
        estimate: number;
        variance: number;
        confidence: number;
    }>;
    /**
     * PageRank of a weighted adjacency matrix (entry [i][j] = weight of edge i -> j).
     * Scores sum to 1: the mass of dangling nodes is redistributed by the
     * teleport vector. `personalized` (non-negative, any scale) selects
     * personalized PageRank; it is normalized internally.
     */
    computePageRank(adjacency: Matrix, config: PageRankConfig): Promise<Vector>;
}
