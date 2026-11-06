/**
 * Core matrix operations for sublinear-time solvers
 */
import { Matrix, SparseMatrix, DenseMatrix, Vector, MatrixAnalysis } from './types.js';
/**
 * Upper bounds applied to every matrix that crosses an API boundary (MCP tool
 * arguments, CLI files, library calls). They stop a few bytes of JSON from
 * declaring a matrix whose dense form or index arrays would exhaust memory.
 */
export declare const MATRIX_LIMITS: Readonly<{
    /** Max rows or cols of any matrix. */
    maxDimension: 10000000;
    /** Max stored entries of a sparse matrix. */
    maxNonZeros: 50000000;
    /** Max rows * cols for anything materialized densely. */
    maxDenseElements: 25000000;
}>;
/** Throws unless a rows x cols dense allocation is within MATRIX_LIMITS. */
export declare function assertDenseAllocation(rows: number, cols: number): void;
export declare class MatrixOperations {
    /**
     * Matrices that have already passed validateMatrix, so per-entry accessors
     * don't re-run an O(nnz) validation on every call. Assumes matrices are
     * treated as immutable after first use (true for the analysis/solve paths).
     */
    private static validatedMatrices;
    /**
     * Lazily-built O(1) entry lookup for COO matrices, keyed per matrix object.
     * Key is `row * cols + col`; value is the FIRST value seen for that
     * coordinate, matching the original linear-scan first-match-wins semantics.
     */
    private static cooIndexCache;
    private static ensureValidated;
    private static getCooIndex;
    /**
     * Validates matrix format and properties
     */
    static validateMatrix(matrix: Matrix): void;
    /**
     * Matrix-vector multiplication: result = matrix * vector
     */
    static multiplyMatrixVector(matrix: Matrix, vector: Vector): Vector;
    /**
     * Get matrix entry at (row, col)
     */
    static getEntry(matrix: Matrix, row: number, col: number): number;
    /**
     * Get diagonal entry at position i
     */
    static getDiagonal(matrix: Matrix, i: number): number;
    /**
     * Extract diagonal as vector
     */
    static getDiagonalVector(matrix: Matrix): Vector;
    /**
     * Get row sum for diagonal dominance check
     */
    static getRowSum(matrix: Matrix, row: number, excludeDiagonal?: boolean): number;
    /**
     * Get column sum for diagonal dominance check
     */
    static getColumnSum(matrix: Matrix, col: number, excludeDiagonal?: boolean): number;
    /**
     * Compute, in a single pass, the diagonal and off-diagonal absolute row/col
     * sums used for diagonal-dominance. O(nnz) for COO (vs O(rows*nnz) when each
     * row/col sum re-scanned all entries). Semantics match getDiagonal (first
     * stored value at (i,i)) and getRowSum/getColumnSum (sum of |value| over all
     * stored off-diagonal entries, duplicates included).
     */
    private static computeDominanceComponents;
    /**
     * Check if matrix is diagonally dominant
     */
    static checkDiagonalDominance(matrix: Matrix): {
        isRowDD: boolean;
        isColDD: boolean;
        strength: number;
    };
    /**
     * Check if matrix is symmetric
     */
    static isSymmetric(matrix: Matrix, tolerance?: number): boolean;
    /**
     * Calculate sparsity ratio (fraction of zero entries)
     */
    static calculateSparsity(matrix: Matrix): number;
    /**
     * Analyze matrix properties
     */
    static analyzeMatrix(matrix: Matrix): MatrixAnalysis;
    /**
     * Convert dense matrix to COO sparse format
     */
    static denseToSparse(dense: DenseMatrix, tolerance?: number): SparseMatrix;
    /**
     * Convert COO sparse matrix to dense format
     */
    static sparseToDense(sparse: SparseMatrix): DenseMatrix;
}
