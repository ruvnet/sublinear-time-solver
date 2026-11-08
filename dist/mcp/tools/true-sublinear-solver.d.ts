/**
 * Approximate solver behind the `solveTrueSublinear` MCP tool: random (JL)
 * projections of the system, a small reduced solve, reconstruction, then a
 * few Jacobi correction sweeps.
 *
 * Despite the tool name this is NOT sublinear. It densifies the matrix and
 * builds dense projection matrices, so time and memory are O(n^2) (measured
 * log-log time slope 2.14 for n = 500..4000), and the result is approximate
 * (relative residual ~1e-2 on well-conditioned DD systems). The reported
 * complexity_bound says so since 1.8.1. Use the `solve` tool for an accurate
 * O(k nnz) solve.
 */
interface SublinearConfig {
    /** Target dimension after JL reduction */
    target_dimension: number;
    /** Sparsification parameter (0 < eps < 1) */
    sparsification_eps: number;
    /** Johnson-Lindenstrauss distortion parameter */
    jl_distortion: number;
    /** Sampling probability for sketching */
    sampling_probability: number;
    /** Maximum recursion depth */
    max_recursion_depth: number;
    /** Base case threshold for recursion */
    base_case_threshold: number;
}
export declare class NotDiagonallyDominantError extends Error {
    constructor();
}
interface ComplexityBound {
    type: 'logarithmic' | 'square_root' | 'sublinear' | 'linear' | 'polynomial';
    n: number;
    eps?: number;
    description: string;
}
interface TrueSublinearResult {
    solution: number[] | {
        first_elements: number[];
        total_elements: number;
        truncated: boolean;
        sample_statistics: {
            min: number;
            max: number;
            mean: number;
            norm: number;
        };
    };
    iterations: number;
    residual_norm: number;
    complexity_bound: ComplexityBound;
    dimension_reduction_ratio: number;
    series_terms_used: number;
    reconstruction_error: number;
    actual_complexity: string;
    method_used: string;
}
interface MatrixAnalysis {
    is_diagonally_dominant: boolean;
    condition_number_estimate: number;
    sparsity_ratio: number;
    spectral_radius_estimate: number;
    recommended_method: string;
    complexity_guarantee: ComplexityBound;
}
export declare class TrueSublinearSolverTools {
    private initialized;
    private wasmModule;
    constructor();
    /**
     * Generate test vectors for matrix solving
     */
    generateTestVector(size: number, pattern?: 'unit' | 'random' | 'sparse' | 'ones' | 'alternating', seed?: number): {
        vector: number[];
        description: string;
    };
    /**
     * Initialize connection to TRUE sublinear WASM algorithms
     */
    private initializeWasm;
    /**
     * Analyze matrix for sublinear solvability
     */
    analyzeMatrix(matrix: {
        values: number[];
        rowIndices: number[];
        colIndices: number[];
        rows: number;
        cols: number;
    }): Promise<MatrixAnalysis>;
    /**
     * Solve with TRUE O(log n) algorithms
     */
    solveTrueSublinear(matrix: {
        values: number[];
        rowIndices: number[];
        colIndices: number[];
        rows: number;
        cols: number;
    }, vector: number[], config?: Partial<SublinearConfig>): Promise<TrueSublinearResult>;
    /**
     * TRUE O(log n) Algorithm - Genuine Sublinear Complexity
     */
    private solveWithTrueOLogN;
    /**
     * DEPRECATED: Old method that was incorrectly returning O(sqrt n)
     */
    private solveWithSublinearNeumann;
    /**
     * Apply Johnson-Lindenstrauss dimension reduction
     */
    private applyJohnsonLindenstrauss;
    /**
     * Solve reduced system with O(log k) Neumann terms
     */
    private solveReducedNeumann;
    /**
     * Reconstruct solution in original space
     */
    private reconstructSolution;
    /**
     * Apply error correction using Richardson iteration
     */
    private applyErrorCorrection;
    /**
     * Solve base case directly for small matrices
     */
    private solveBaseCaseDirect;
    /**
     * Solve using dimension reduction for non-diagonally-dominant matrices
     */
    private solveWithDimensionReduction;
    private checkDiagonalDominance;
    private estimateConditionNumber;
    private estimateSpectralRadius;
    private sparseToDense;
    private applySpectralSparsification;
    private solveReducedIterative;
    private computeResidual;
    /**
     * Convert dense matrix to sparse format for recursive reduction
     */
    private sparseToSparseReduction;
    /**
     * Solve base case with O(log k) complexity where k = O(log n)
     */
    private solveBaseWithLogComplexity;
    /**
     * Apply O(log n) error correction - each iteration improves by constant factor
     */
    private applyLogNErrorCorrection;
    private gaussianRandom;
}
export {};
