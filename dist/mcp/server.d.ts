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
    /**
     * Twelve-tier complexity-class ranking, matched to the Rust
     * `ComplexityClass::rank()` in `src/complexity.rs`. Lower = cheaper.
     * Used by `enforceComplexityBudget` to compare a solver's worst-case
     * class against a caller-supplied `max_complexity_class` budget.
     */
    private static readonly COMPLEXITY_RANK;
    /**
     * Per-method worst-case complexity class. Single source of truth for
     * both `estimateComplexityClass` and the `max_complexity_class` budget
     * gate. Mirrors the Rust `Complexity` impls in `src/complexity.rs`.
     *
     * For `Adaptive` solvers we use the **worst-case** bound so callers
     * always see safe behaviour — a Cognitum reflex loop with a
     * `SubLinear` budget won't accidentally invoke a solver that can
     * degrade to `Linear` on hard inputs.
     */
    private static readonly METHOD_WORST_CASE;
    /**
     * Reject the call with a structured McpError if the chosen `method`'s
     * worst-case complexity class exceeds the caller's `max_complexity_class`
     * budget. Returns silently otherwise. No-op when the budget arg is
     * absent (the default — preserves wire compatibility with pre-1.7.1
     * clients).
     *
     * ADR-001 item #4 phase-2 — the "bounded-planning kernel" promise.
     */
    private enforceComplexityBudget;
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
