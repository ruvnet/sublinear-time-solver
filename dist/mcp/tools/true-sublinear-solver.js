/**
 * Backing class for the `solveTrueSublinear`, `analyzeTrueSublinearMatrix`
 * and `generateTestVector` MCP tools.
 *
 * `solveTrueSublinear` is deprecated. Since 1.10.0 it runs the sparse solver
 * from @ruvnet/sublinear-core (O(k · nnz), accurate), the same one behind the
 * `solve` tool. Until 1.9.0 it ran a Johnson-Lindenstrauss projection solver
 * that densified the matrix: O(n^2) time and memory (log-log slope 2.14,
 * 760 MB RSS at n = 4000) and only ~1e-2 relative accuracy.
 */
import * as fs from 'fs';
import * as path from 'path';
import { csrFromCoo, diagonalDominance, solve as coreSolve } from '@ruvnet/sublinear-core';
import { MatrixOperations, assertDenseAllocation } from '../../core/matrix.js';
let deprecationWarned = false;
/** One stderr line per process; stdout is the MCP stdio transport. */
function warnSolveTrueSublinearDeprecated() {
    if (deprecationWarned)
        return;
    deprecationWarned = true;
    process.stderr.write('[sublinear-time-solver] solveTrueSublinear is deprecated and now runs the same sparse solver as `solve` (@ruvnet/sublinear-core). Call `solve` instead.\n');
}
export class NotDiagonallyDominantError extends Error {
    constructor() {
        super('Matrix is not diagonally dominant; this approximate solver requires it. Use the solve tool (method conjugate-gradient for SPD matrices).');
        this.name = 'NotDiagonallyDominantError';
    }
}
export class TrueSublinearSolverTools {
    initialized = false;
    wasmModule = null;
    constructor() {
        this.initializeWasm();
    }
    /**
     * Generate test vectors for matrix solving
     */
    generateTestVector(size, pattern = 'sparse', seed) {
        if (seed !== undefined) {
            // Simple seeded random number generator
            let currentSeed = seed;
            const seedRandom = () => {
                const x = Math.sin(currentSeed++) * 10000;
                return x - Math.floor(x);
            };
            const vector = new Array(size).fill(0);
            let description = '';
            switch (pattern) {
                case 'unit':
                    if (size > 0)
                        vector[0] = 1;
                    description = `Unit vector e_1 of size ${size}`;
                    break;
                case 'random':
                    for (let i = 0; i < size; i++) {
                        vector[i] = seedRandom() * 2 - 1; // Random values in [-1, 1]
                    }
                    description = `Seeded random vector of size ${size} with values in [-1, 1]`;
                    break;
                case 'sparse':
                    const sparsity = Math.min(10, Math.ceil(size * 0.01)); // 1% or at least 10 elements
                    for (let i = 0; i < sparsity; i++) {
                        vector[i] = 1;
                    }
                    description = `Sparse vector of size ${size} with ${sparsity} leading ones`;
                    break;
                case 'ones':
                    vector.fill(1);
                    description = `All-ones vector of size ${size}`;
                    break;
                case 'alternating':
                    for (let i = 0; i < size; i++) {
                        vector[i] = i % 2 === 0 ? 1 : -1;
                    }
                    description = `Alternating +1/-1 vector of size ${size}`;
                    break;
                default:
                    const defaultSparsity = Math.min(10, Math.ceil(size * 0.01));
                    for (let i = 0; i < defaultSparsity; i++) {
                        vector[i] = 1;
                    }
                    description = `Default sparse vector of size ${size} with ${defaultSparsity} leading ones`;
            }
            return { vector, description };
        }
        else {
            // Use Math.random for non-seeded generation
            const vector = new Array(size).fill(0);
            let description = '';
            switch (pattern) {
                case 'unit':
                    if (size > 0)
                        vector[0] = 1;
                    description = `Unit vector e_1 of size ${size}`;
                    break;
                case 'random':
                    for (let i = 0; i < size; i++) {
                        vector[i] = Math.random() * 2 - 1; // Random values in [-1, 1]
                    }
                    description = `Random vector of size ${size} with values in [-1, 1]`;
                    break;
                case 'sparse':
                    const sparsity = Math.min(10, Math.ceil(size * 0.01)); // 1% or at least 10 elements
                    for (let i = 0; i < sparsity; i++) {
                        vector[i] = 1;
                    }
                    description = `Sparse vector of size ${size} with ${sparsity} leading ones`;
                    break;
                case 'ones':
                    vector.fill(1);
                    description = `All-ones vector of size ${size}`;
                    break;
                case 'alternating':
                    for (let i = 0; i < size; i++) {
                        vector[i] = i % 2 === 0 ? 1 : -1;
                    }
                    description = `Alternating +1/-1 vector of size ${size}`;
                    break;
                default:
                    const defaultSparsity = Math.min(10, Math.ceil(size * 0.01));
                    for (let i = 0; i < defaultSparsity; i++) {
                        vector[i] = 1;
                    }
                    description = `Default sparse vector of size ${size} with ${defaultSparsity} leading ones`;
            }
            return { vector, description };
        }
    }
    /**
     * Initialize connection to TRUE sublinear WASM algorithms
     */
    async initializeWasm() {
        try {
            // Check if TRUE sublinear WASM module exists
            const wasmPath = path.join(process.cwd(), 'dist', 'wasm', 'sublinear_true_bg.wasm');
            if (!fs.existsSync(wasmPath)) {
                console.warn('TRUE sublinear WASM not found, using TypeScript fallback');
                this.initialized = true;
                return;
            }
            // In a real implementation, load the WASM module
            // For now, use TypeScript implementation
            this.initialized = true;
        }
        catch (error) {
            console.error('Failed to initialize TRUE sublinear WASM:', error);
            this.initialized = true; // Continue with fallback
        }
    }
    /**
     * Analyze matrix for sublinear solvability
     */
    async analyzeMatrix(matrix) {
        if (!this.initialized) {
            await this.initializeWasm();
        }
        // Check diagonal dominance (required for O(log n) complexity)
        const isDiagonallyDominant = this.checkDiagonalDominance(matrix);
        // Estimate condition number using Gershgorin circles
        const conditionEstimate = this.estimateConditionNumber(matrix);
        // Calculate sparsity
        const sparsity = matrix.values.length / (matrix.rows * matrix.cols);
        // Estimate spectral radius
        const spectralRadius = this.estimateSpectralRadius(matrix);
        // Determine recommended method and complexity guarantee
        let recommendedMethod;
        let complexityGuarantee;
        // Every path below densifies the matrix (sparseToDense) and builds dense
        // random projections, so the cost is O(n^2) time and memory. Measured
        // (1.8.1 review): log-log time slope 2.14 over n = 500..4000, 760 MB RSS at
        // n = 4000. It was labelled "TRUE O(log n)" before 1.8.1.
        recommendedMethod = isDiagonallyDominant && conditionEstimate < 1e6
            ? 'sublinear_neumann'
            : 'recursive_dimension_reduction';
        complexityGuarantee = {
            type: 'polynomial',
            n: matrix.rows,
            description: `O(n^2) for n = ${matrix.rows}: the matrix is densified and projected with dense random matrices`
        };
        return {
            is_diagonally_dominant: isDiagonallyDominant,
            condition_number_estimate: conditionEstimate,
            sparsity_ratio: sparsity,
            spectral_radius_estimate: spectralRadius,
            recommended_method: recommendedMethod,
            complexity_guarantee: complexityGuarantee
        };
    }
    /**
     * Solve A x = b for the `solveTrueSublinear` MCP tool.
     *
     * Deprecated since 1.10.0: this now delegates to the sparse solver in
     * @ruvnet/sublinear-core (the same one the `solve` tool uses), O(k · nnz)
     * time and O(nnz) memory, accurate to a 1e-10 relative residual. The JL
     * projection solver it replaced densified the matrix (measured log-log time
     * slope 2.14, 760 MB RSS at n = 4000) and was only accurate to about 1e-2.
     * The result keeps its old shape; `config` is accepted and ignored.
     */
    async solveTrueSublinear(matrix, vector, _config = {}) {
        warnSolveTrueSublinearDeprecated();
        const A = csrFromCoo({
            rows: matrix.rows,
            cols: matrix.cols,
            rowIndices: matrix.rowIndices,
            colIndices: matrix.colIndices,
            values: matrix.values,
        });
        // The old solver required strict row dominance; keep that contract so
        // callers that relied on the rejection still get it.
        if (!diagonalDominance(A).strictlyRowDominant) {
            throw new NotDiagonallyDominantError();
        }
        const res = coreSolve(A, vector, { method: 'auto', tolerance: 1e-10, maxIterations: 1000 });
        const solution = Array.from(res.x);
        const bNorm = Math.sqrt(vector.reduce((s, v) => s + v * v, 0));
        const residualNorm = res.residual * (bNorm > 0 ? bNorm : 1);
        const nnz = A.values.length;
        // Truncate large solutions to keep the MCP reply under the token limit.
        const maxSolutionElements = 1000;
        let solutionSummary = solution;
        if (solution.length > maxSolutionElements) {
            let min = Infinity, max = -Infinity, sum = 0, sq = 0;
            for (const v of solution) {
                if (v < min)
                    min = v;
                if (v > max)
                    max = v;
                sum += v;
                sq += v * v;
            }
            solutionSummary = {
                first_elements: solution.slice(0, maxSolutionElements),
                total_elements: solution.length,
                truncated: true,
                sample_statistics: { min, max, mean: sum / solution.length, norm: Math.sqrt(sq) },
            };
        }
        return {
            solution: solutionSummary,
            iterations: res.iterations,
            residual_norm: residualNorm,
            complexity_bound: {
                type: 'linear',
                n: matrix.rows,
                description: `O(k · nnz): ${res.iterations} ${res.method} iterations over nnz = ${nnz} (sparse, no densification)`,
            },
            dimension_reduction_ratio: 1,
            series_terms_used: res.iterations,
            reconstruction_error: 0,
            actual_complexity: `O(k · nnz), k = ${res.iterations}, nnz = ${nnz}`,
            method_used: `sublinear-core ${res.method}${res.converged ? '' : ' (not converged)'}`,
        };
    }
    // Helper methods
    checkDiagonalDominance(matrix) {
        const dense = this.sparseToDense(matrix);
        for (let i = 0; i < matrix.rows; i++) {
            const diagonal = Math.abs(dense[i][i]);
            const offDiagonalSum = dense[i].reduce((sum, val, j) => {
                return i === j ? sum : sum + Math.abs(val);
            }, 0);
            if (diagonal <= offDiagonalSum) {
                return false;
            }
        }
        return true;
    }
    estimateConditionNumber(matrix) {
        // Simplified estimate using Gershgorin circles
        const dense = this.sparseToDense(matrix);
        let maxRadius = 0;
        let minDiag = Infinity;
        for (let i = 0; i < matrix.rows; i++) {
            const diagonal = Math.abs(dense[i][i]);
            const offDiagSum = dense[i].reduce((sum, val, j) => {
                return i === j ? sum : sum + Math.abs(val);
            }, 0);
            maxRadius = Math.max(maxRadius, diagonal + offDiagSum);
            minDiag = Math.min(minDiag, Math.max(1e-14, diagonal - offDiagSum));
        }
        return maxRadius / minDiag;
    }
    estimateSpectralRadius(matrix) {
        // Power iteration estimate
        const dense = this.sparseToDense(matrix);
        let v = new Array(matrix.rows).fill(1.0 / Math.sqrt(matrix.rows));
        for (let iter = 0; iter < 10; iter++) {
            const w = new Array(matrix.rows).fill(0);
            for (let i = 0; i < matrix.rows; i++) {
                for (let j = 0; j < matrix.cols; j++) {
                    w[i] += dense[i][j] * v[j];
                }
            }
            const norm = Math.sqrt(w.reduce((sum, x) => sum + x * x, 0));
            v = w.map(x => x / norm);
        }
        // Rayleigh quotient
        let num = 0, den = 0;
        for (let i = 0; i < matrix.rows; i++) {
            let Av_i = 0;
            for (let j = 0; j < matrix.cols; j++) {
                Av_i += dense[i][j] * v[j];
            }
            num += v[i] * Av_i;
            den += v[i] * v[i];
        }
        return Math.abs(num / den);
    }
    sparseToDense(matrix) {
        // Validate before materializing: integer in-range indices (no property-key
        // injection) and a bounded rows x cols allocation.
        MatrixOperations.validateMatrix({ ...matrix, format: 'coo' });
        assertDenseAllocation(matrix.rows, matrix.cols);
        const dense = Array(matrix.rows).fill(0).map(() => Array(matrix.cols).fill(0));
        for (let i = 0; i < matrix.values.length; i++) {
            const row = matrix.rowIndices[i];
            const col = matrix.colIndices[i];
            const val = matrix.values[i];
            dense[row][col] = val;
        }
        return dense;
    }
}
