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
     * Solve A x = b for the `solveTrueSublinear` MCP tool.
     *
     * Deprecated since 1.10.0: this now delegates to the sparse solver in
     * @ruvnet/sublinear-core (the same one the `solve` tool uses), O(k · nnz)
     * time and O(nnz) memory, accurate to a 1e-10 relative residual. The JL
     * projection solver it replaced densified the matrix (measured log-log time
     * slope 2.14, 760 MB RSS at n = 4000) and was only accurate to about 1e-2.
     * The result keeps its old shape; `config` is accepted and ignored.
     */
    solveTrueSublinear(matrix: {
        values: number[];
        rowIndices: number[];
        colIndices: number[];
        rows: number;
        cols: number;
    }, vector: number[], _config?: Partial<SublinearConfig>): Promise<TrueSublinearResult>;
    private checkDiagonalDominance;
    private estimateConditionNumber;
    private estimateSpectralRadius;
    private sparseToDense;
}
export {};
