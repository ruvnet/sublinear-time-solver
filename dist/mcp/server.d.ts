/**
 * MCP Server for Sublinear-Time Solver
 * Provides MCP interface to the core solver algorithms
 */
export declare class SublinearSolverMCPServer {
    private server;
    private solvers;
    private temporalTools;
    private psychoSymbolicTools;
    private dynamicPsychoSymbolicTools;
    private domainManagementTools;
    private domainValidationTools;
    private consciousnessTools;
    private emergenceTools;
    private schedulerTools;
    private wasmSolver;
    private trueSublinearSolver;
    constructor();
    private setupToolHandlers;
    private setupErrorHandling;
    private handleSolve;
    private handleEstimateEntry;
    private handleAnalyzeMatrix;
    private handlePageRank;
    private handleSolveTrueSublinear;
    private handleAnalyzeTrueSublinearMatrix;
    private handleGenerateTestVector;
    /**
     * ADR-001 item #4 — estimate the worst-case complexity class for a
     * candidate solve without running it. Used by agents to decide whether
     * to spend the J/decision budget on a given method, or fall back to a
     * cached / cheaper answer.
     *
     * The class table mirrors the `Complexity` impls in `src/complexity.rs`
     * so the wire contract matches the Rust contract. Keep them in sync —
     * a CI guard for this is on the ADR roadmap (phase 2).
     */
    private handleEstimateComplexityClass;
    private handleSaveVectorToFile;
    private loadVectorFromFile;
    private saveVectorToFile;
    private getFileFormat;
    private generateRecommendations;
    run(): Promise<void>;
}
