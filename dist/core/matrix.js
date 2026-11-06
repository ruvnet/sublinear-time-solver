/**
 * Core matrix operations for sublinear-time solvers
 */
import { SolverError, ErrorCodes } from './types.js';
/**
 * Upper bounds applied to every matrix that crosses an API boundary (MCP tool
 * arguments, CLI files, library calls). They stop a few bytes of JSON from
 * declaring a matrix whose dense form or index arrays would exhaust memory.
 */
export const MATRIX_LIMITS = Object.freeze({
    /** Max rows or cols of any matrix. */
    maxDimension: 10_000_000,
    /** Max stored entries of a sparse matrix. */
    maxNonZeros: 50_000_000,
    /** Max rows * cols for anything materialized densely. */
    maxDenseElements: 25_000_000,
});
/** Throws unless a rows x cols dense allocation is within MATRIX_LIMITS. */
export function assertDenseAllocation(rows, cols) {
    if (!Number.isInteger(rows) || !Number.isInteger(cols) || rows < 0 || cols < 0) {
        throw new SolverError('Matrix dimensions must be non-negative integers', ErrorCodes.INVALID_DIMENSIONS);
    }
    if (rows * cols > MATRIX_LIMITS.maxDenseElements) {
        throw new SolverError(`Dense ${rows}x${cols} matrix exceeds the ${MATRIX_LIMITS.maxDenseElements}-element limit; use a sparse (coo) matrix`, ErrorCodes.MEMORY_LIMIT_EXCEEDED);
    }
}
export class MatrixOperations {
    /**
     * Matrices that have already passed validateMatrix, so per-entry accessors
     * don't re-run an O(nnz) validation on every call. Assumes matrices are
     * treated as immutable after first use (true for the analysis/solve paths).
     */
    static validatedMatrices = new WeakSet();
    /**
     * Lazily-built O(1) entry lookup for COO matrices, keyed per matrix object.
     * Key is `row * cols + col`; value is the FIRST value seen for that
     * coordinate, matching the original linear-scan first-match-wins semantics.
     */
    static cooIndexCache = new WeakMap();
    static ensureValidated(matrix) {
        if (!this.validatedMatrices.has(matrix)) {
            this.validateMatrix(matrix);
        }
    }
    static getCooIndex(matrix) {
        let index = this.cooIndexCache.get(matrix);
        if (!index) {
            index = new Map();
            const { values, rowIndices, colIndices, cols } = matrix;
            for (let k = 0; k < values.length; k++) {
                const key = rowIndices[k] * cols + colIndices[k];
                // Keep the first occurrence to preserve legacy first-match-wins.
                if (!index.has(key))
                    index.set(key, values[k]);
            }
            this.cooIndexCache.set(matrix, index);
        }
        return index;
    }
    /**
     * Validates matrix format and properties
     */
    static validateMatrix(matrix) {
        if (!matrix) {
            throw new SolverError('Matrix is required', ErrorCodes.INVALID_MATRIX);
        }
        if (typeof matrix !== 'object') {
            throw new SolverError('Matrix must be an object', ErrorCodes.INVALID_MATRIX);
        }
        // Integer checks matter: a non-number index such as "__proto__" passes
        // `idx < 0 || idx >= rows` and would be used as a property key later.
        if (!Number.isInteger(matrix.rows) || !Number.isInteger(matrix.cols) || matrix.rows <= 0 || matrix.cols <= 0) {
            throw new SolverError('Matrix dimensions must be positive integers', ErrorCodes.INVALID_DIMENSIONS);
        }
        if (matrix.rows > MATRIX_LIMITS.maxDimension || matrix.cols > MATRIX_LIMITS.maxDimension) {
            throw new SolverError(`Matrix dimension exceeds limit of ${MATRIX_LIMITS.maxDimension}`, ErrorCodes.MEMORY_LIMIT_EXCEEDED);
        }
        if (matrix.format === 'dense') {
            const dense = matrix;
            assertDenseAllocation(dense.rows, dense.cols);
            if (!Array.isArray(dense.data) || dense.data.length !== dense.rows) {
                throw new SolverError('Dense matrix data must be array of rows', ErrorCodes.INVALID_MATRIX);
            }
            for (let i = 0; i < dense.rows; i++) {
                const row = dense.data[i];
                if (!Array.isArray(row) || row.length !== dense.cols) {
                    throw new SolverError(`Row ${i} has invalid length`, ErrorCodes.INVALID_MATRIX);
                }
                for (let j = 0; j < row.length; j++) {
                    if (typeof row[j] !== 'number' || !Number.isFinite(row[j])) {
                        throw new SolverError(`Entry (${i}, ${j}) is not a finite number`, ErrorCodes.INVALID_MATRIX);
                    }
                }
            }
        }
        else if (matrix.format === 'coo' || matrix.format === 'csr' || matrix.format === 'csc') {
            const sparse = matrix;
            const { values, rowIndices, colIndices } = sparse;
            if (!Array.isArray(values) || !Array.isArray(rowIndices) || !Array.isArray(colIndices)) {
                throw new SolverError('COO matrix must have values, rowIndices, and colIndices arrays', ErrorCodes.INVALID_MATRIX);
            }
            if (values.length !== rowIndices.length || values.length !== colIndices.length) {
                throw new SolverError('COO matrix arrays must have same length', ErrorCodes.INVALID_MATRIX);
            }
            if (values.length > MATRIX_LIMITS.maxNonZeros) {
                throw new SolverError(`Matrix has more than ${MATRIX_LIMITS.maxNonZeros} stored entries`, ErrorCodes.MEMORY_LIMIT_EXCEEDED);
            }
            // Check indices are valid integers in range and values are finite numbers
            for (let i = 0; i < rowIndices.length; i++) {
                const r = rowIndices[i];
                const c = colIndices[i];
                if (!Number.isInteger(r) || r < 0 || r >= sparse.rows) {
                    throw new SolverError(`Invalid row index ${String(r)}`, ErrorCodes.INVALID_MATRIX);
                }
                if (!Number.isInteger(c) || c < 0 || c >= sparse.cols) {
                    throw new SolverError(`Invalid column index ${String(c)}`, ErrorCodes.INVALID_MATRIX);
                }
                if (typeof values[i] !== 'number' || !Number.isFinite(values[i])) {
                    throw new SolverError(`Value at position ${i} is not a finite number`, ErrorCodes.INVALID_MATRIX);
                }
            }
        }
        else {
            throw new SolverError(`Unsupported matrix format: ${matrix.format}`, ErrorCodes.INVALID_MATRIX);
        }
        this.validatedMatrices.add(matrix);
    }
    /**
     * Matrix-vector multiplication: result = matrix * vector
     */
    static multiplyMatrixVector(matrix, vector) {
        this.validateMatrix(matrix);
        if (vector.length !== matrix.cols) {
            throw new SolverError(`Vector length ${vector.length} does not match matrix columns ${matrix.cols}`, ErrorCodes.INVALID_DIMENSIONS);
        }
        const result = new Array(matrix.rows).fill(0);
        if (matrix.format === 'dense') {
            const dense = matrix;
            for (let i = 0; i < matrix.rows; i++) {
                for (let j = 0; j < matrix.cols; j++) {
                    result[i] += dense.data[i][j] * vector[j];
                }
            }
        }
        else if (matrix.format === 'coo') {
            const sparse = matrix;
            for (let k = 0; k < sparse.values.length; k++) {
                const row = sparse.rowIndices[k];
                const col = sparse.colIndices[k];
                const val = sparse.values[k];
                result[row] += val * vector[col];
            }
        }
        return result;
    }
    /**
     * Get matrix entry at (row, col)
     */
    static getEntry(matrix, row, col) {
        this.ensureValidated(matrix);
        if (row < 0 || row >= matrix.rows || col < 0 || col >= matrix.cols) {
            throw new SolverError(`Index (${row}, ${col}) out of bounds`, ErrorCodes.INVALID_DIMENSIONS);
        }
        if (matrix.format === 'dense') {
            const dense = matrix;
            return dense.data[row][col];
        }
        else if (matrix.format === 'coo') {
            const sparse = matrix;
            // O(1) lookup via the per-matrix index instead of an O(nnz) scan.
            return this.getCooIndex(sparse).get(row * matrix.cols + col) ?? 0;
        }
        return 0;
    }
    /**
     * Get diagonal entry at position i
     */
    static getDiagonal(matrix, i) {
        return this.getEntry(matrix, i, i);
    }
    /**
     * Extract diagonal as vector
     */
    static getDiagonalVector(matrix) {
        if (matrix.rows !== matrix.cols) {
            throw new SolverError('Matrix must be square to extract diagonal', ErrorCodes.INVALID_DIMENSIONS);
        }
        const diagonal = new Array(matrix.rows);
        for (let i = 0; i < matrix.rows; i++) {
            diagonal[i] = this.getDiagonal(matrix, i);
        }
        return diagonal;
    }
    /**
     * Get row sum for diagonal dominance check
     */
    static getRowSum(matrix, row, excludeDiagonal = false) {
        this.validateMatrix(matrix);
        if (row < 0 || row >= matrix.rows) {
            throw new SolverError(`Row index ${row} out of bounds`, ErrorCodes.INVALID_DIMENSIONS);
        }
        let sum = 0;
        if (matrix.format === 'dense') {
            const dense = matrix;
            for (let j = 0; j < matrix.cols; j++) {
                if (!excludeDiagonal || j !== row) {
                    sum += Math.abs(dense.data[row][j]);
                }
            }
        }
        else if (matrix.format === 'coo') {
            const sparse = matrix;
            for (let k = 0; k < sparse.values.length; k++) {
                if (sparse.rowIndices[k] === row) {
                    const col = sparse.colIndices[k];
                    if (!excludeDiagonal || col !== row) {
                        sum += Math.abs(sparse.values[k]);
                    }
                }
            }
        }
        return sum;
    }
    /**
     * Get column sum for diagonal dominance check
     */
    static getColumnSum(matrix, col, excludeDiagonal = false) {
        this.validateMatrix(matrix);
        if (col < 0 || col >= matrix.cols) {
            throw new SolverError(`Column index ${col} out of bounds`, ErrorCodes.INVALID_DIMENSIONS);
        }
        let sum = 0;
        if (matrix.format === 'dense') {
            const dense = matrix;
            for (let i = 0; i < matrix.rows; i++) {
                if (!excludeDiagonal || i !== col) {
                    sum += Math.abs(dense.data[i][col]);
                }
            }
        }
        else if (matrix.format === 'coo') {
            const sparse = matrix;
            for (let k = 0; k < sparse.values.length; k++) {
                if (sparse.colIndices[k] === col) {
                    const row = sparse.rowIndices[k];
                    if (!excludeDiagonal || row !== col) {
                        sum += Math.abs(sparse.values[k]);
                    }
                }
            }
        }
        return sum;
    }
    /**
     * Compute, in a single pass, the diagonal and off-diagonal absolute row/col
     * sums used for diagonal-dominance. O(nnz) for COO (vs O(rows*nnz) when each
     * row/col sum re-scanned all entries). Semantics match getDiagonal (first
     * stored value at (i,i)) and getRowSum/getColumnSum (sum of |value| over all
     * stored off-diagonal entries, duplicates included).
     */
    static computeDominanceComponents(matrix) {
        const n = matrix.rows;
        const diag = new Float64Array(n);
        const rowOff = new Float64Array(n);
        const colOff = new Float64Array(n);
        if (matrix.format === 'dense') {
            const dense = matrix;
            for (let i = 0; i < n; i++) {
                const rowData = dense.data[i];
                for (let j = 0; j < matrix.cols; j++) {
                    const v = rowData[j];
                    if (i === j)
                        diag[i] = v;
                    else {
                        rowOff[i] += Math.abs(v);
                        colOff[j] += Math.abs(v);
                    }
                }
            }
        }
        else {
            const sparse = matrix;
            const diagSet = new Uint8Array(n);
            for (let k = 0; k < sparse.values.length; k++) {
                const i = sparse.rowIndices[k];
                const j = sparse.colIndices[k];
                const v = sparse.values[k];
                if (i === j) {
                    if (!diagSet[i]) {
                        diag[i] = v;
                        diagSet[i] = 1;
                    } // first-match wins
                }
                else {
                    rowOff[i] += Math.abs(v);
                    colOff[j] += Math.abs(v);
                }
            }
        }
        return { diag, rowOff, colOff };
    }
    /**
     * Check if matrix is diagonally dominant
     */
    static checkDiagonalDominance(matrix) {
        this.validateMatrix(matrix);
        if (matrix.rows !== matrix.cols) {
            return { isRowDD: false, isColDD: false, strength: 0 };
        }
        const { diag, rowOff, colOff } = this.computeDominanceComponents(matrix);
        let isRowDD = true;
        let isColDD = true;
        let minRowStrength = Infinity;
        let minColStrength = Infinity;
        for (let i = 0; i < matrix.rows; i++) {
            const diagonal = Math.abs(diag[i]);
            const rowOffDiagonalSum = rowOff[i];
            const colOffDiagonalSum = colOff[i];
            if (diagonal === 0) {
                isRowDD = false;
                isColDD = false;
                minRowStrength = 0;
                minColStrength = 0;
                break;
            }
            const rowStrength = diagonal - rowOffDiagonalSum;
            const colStrength = diagonal - colOffDiagonalSum;
            if (rowStrength < 0) {
                isRowDD = false;
            }
            else {
                minRowStrength = Math.min(minRowStrength, rowStrength / diagonal);
            }
            if (colStrength < 0) {
                isColDD = false;
            }
            else {
                minColStrength = Math.min(minColStrength, colStrength / diagonal);
            }
        }
        const strength = Math.max(isRowDD ? minRowStrength : 0, isColDD ? minColStrength : 0);
        return { isRowDD, isColDD, strength };
    }
    /**
     * Check if matrix is symmetric
     */
    static isSymmetric(matrix, tolerance = 1e-10) {
        this.validateMatrix(matrix);
        if (matrix.rows !== matrix.cols) {
            return false;
        }
        // For sparse matrices, this is more complex - we'd need to compare all entries
        if (matrix.format === 'dense') {
            const dense = matrix;
            for (let i = 0; i < matrix.rows; i++) {
                for (let j = i + 1; j < matrix.cols; j++) {
                    if (Math.abs(dense.data[i][j] - dense.data[j][i]) > tolerance) {
                        return false;
                    }
                }
            }
            return true;
        }
        // For sparse matrices, only stored off-diagonal coordinates can break
        // symmetry: a pair where both sides are absent is 0 == 0. Iterating the
        // stored entries (O(nnz)) and comparing each against its mirror via the
        // O(1) index catches every asymmetric pair, matching the O(V^2) probe.
        const sparse = matrix;
        for (let k = 0; k < sparse.values.length; k++) {
            const i = sparse.rowIndices[k];
            const j = sparse.colIndices[k];
            if (i === j)
                continue;
            if (Math.abs(this.getEntry(matrix, i, j) - this.getEntry(matrix, j, i)) > tolerance) {
                return false;
            }
        }
        return true;
    }
    /**
     * Calculate sparsity ratio (fraction of zero entries)
     */
    static calculateSparsity(matrix) {
        this.validateMatrix(matrix);
        const totalEntries = matrix.rows * matrix.cols;
        if (matrix.format === 'dense') {
            const dense = matrix;
            let nonZeros = 0;
            for (let i = 0; i < matrix.rows; i++) {
                for (let j = 0; j < matrix.cols; j++) {
                    if (Math.abs(dense.data[i][j]) > 1e-15) {
                        nonZeros++;
                    }
                }
            }
            return 1 - (nonZeros / totalEntries);
        }
        else if (matrix.format === 'coo') {
            const sparse = matrix;
            return 1 - (sparse.values.length / totalEntries);
        }
        return 0;
    }
    /**
     * Analyze matrix properties
     */
    static analyzeMatrix(matrix) {
        this.validateMatrix(matrix);
        const dominance = this.checkDiagonalDominance(matrix);
        const isSymmetric = this.isSymmetric(matrix);
        const sparsity = this.calculateSparsity(matrix);
        let dominanceType = 'none';
        if (dominance.isRowDD && dominance.isColDD) {
            dominanceType = 'row'; // Prefer row if both
        }
        else if (dominance.isRowDD) {
            dominanceType = 'row';
        }
        else if (dominance.isColDD) {
            dominanceType = 'column';
        }
        return {
            isDiagonallyDominant: dominance.isRowDD || dominance.isColDD,
            dominanceType,
            dominanceStrength: dominance.strength,
            isSymmetric,
            sparsity,
            size: { rows: matrix.rows, cols: matrix.cols }
        };
    }
    /**
     * Convert dense matrix to COO sparse format
     */
    static denseToSparse(dense, tolerance = 1e-15) {
        const values = [];
        const rowIndices = [];
        const colIndices = [];
        for (let i = 0; i < dense.rows; i++) {
            for (let j = 0; j < dense.cols; j++) {
                const value = dense.data[i][j];
                if (Math.abs(value) > tolerance) {
                    values.push(value);
                    rowIndices.push(i);
                    colIndices.push(j);
                }
            }
        }
        return {
            rows: dense.rows,
            cols: dense.cols,
            values,
            rowIndices,
            colIndices,
            format: 'coo'
        };
    }
    /**
     * Convert COO sparse matrix to dense format
     */
    static sparseToDense(sparse) {
        this.validateMatrix(sparse);
        assertDenseAllocation(sparse.rows, sparse.cols);
        const data = Array(sparse.rows).fill(null).map(() => Array(sparse.cols).fill(0));
        for (let k = 0; k < sparse.values.length; k++) {
            const row = sparse.rowIndices[k];
            const col = sparse.colIndices[k];
            const val = sparse.values[k];
            data[row][col] = val;
        }
        return {
            rows: sparse.rows,
            cols: sparse.cols,
            data,
            format: 'dense'
        };
    }
}
