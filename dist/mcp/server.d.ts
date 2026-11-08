/**
 * MCP Server for Sublinear-Time Solver
 * Provides MCP interface to the core solver algorithms
 */
/**
 * stdout carries JSON-RPC frames when serving over stdio. Route every console
 * diagnostic (ours and any dependency's) to stderr so a stray log line cannot
 * corrupt the stream. Call before constructing the server.
 */
export declare function routeConsoleToStderr(): void;
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
    private toolSchemas?;
    private listTools;
    private callTool;
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
    /**
     * ADR-001 open Q#3 / PR #41 (Rust). Wire-callable witness for
     * SubLinear orchestrator outputs. Computes the closure-restricted
     * residual `r[i] = b[i] - Σ_j A[i,j]·x_new[j]` for every supplied
     * `(row, value)` entry. `x_new[j]` uses the entry's value if j
     * appears in the entries list; otherwise falls back to
     * `prev_solution[j]` (the closure-boundary contract).
     *
     * Pure-TS implementation — does NOT cross into Rust/WASM. The
     * residual math is straightforward enough that duplicating it
     * here is cheaper than plumbing a new WASM binding. Matches
     * Rust's `verify_sparse_solution` semantics exactly.
     *
     * Cost: O(|entries| · avg_row_nnz). SubLinear in n for sparse
     * DD matrices — same complexity class as the orchestrator
     * whose output it verifies.
     */
    private handleVerifySparseSolution;
    /**
     * Wire-callable coherence-score primitive. Mirrors Rust's
     * `coherence::coherence_score(matrix)`:
     *
     *   margin(i) = (|A[i,i]| - Σ_{j ≠ i} |A[i,j]|) / |A[i,i]|
     *   coherence(A) = min_i margin(i)
     *
     * Strictly DD matrices score in (0, 1]; the boundary case scores 0;
     * non-DD scores negative; rows with zero diagonal score -Infinity.
     *
     * Pure-TS — no WASM bridge needed. Cost O(nnz(A)).
     */
    private handleCoherenceScore;
    /**
     * Wire-callable bounded-depth row-graph BFS. Mirrors Rust's
     * `closure::closure_indices(matrix, seeds, depth)`:
     *
     *   closure_0 = seeds
     *   closure_{d+1} = closure_d ∪ {j : ∃ i ∈ closure_d, A[i,j] ≠ 0}
     *
     * Pure-TS — no WASM bridge. Cost O(depth · branch · |closure|).
     */
    private handleClosureIndices;
    /**
     * Wire-callable SubLinear orchestrator. Mirrors Rust's
     * `incremental::solve_on_change_sublinear(matrix, prev, b_new,
     * delta, closure_depth, max_terms, tolerance)`. Pure-TS — no
     * WASM bridge. Composes:
     *
     *   1. closure_indices(matrix, delta.indices, closure_depth)
     *   2. for each closure entry: per-entry Neumann iteration
     *      restricted to the closure (matches src/entry.rs math)
     *   3. return Vec<{row, value}>
     *
     * Cost O(|closure| · max_terms · branch). End-to-end SubLinear
     * in n for sparse DD matrices with bounded depth + max_terms.
     */
    private handleSolveOnChangeSublinear;
    /**
     * Wire-callable contrastive top-k orchestrator. Mirrors Rust's
     * `contrastive::contrastive_solve_on_change_sublinear`. Delegates
     * the SubLinear solve to handleSolveOnChangeSublinear, then ranks
     * the resulting entries by `|current - prev_solution[row]|` and
     * returns the top-k.
     *
     * Pure-TS — no WASM bridge. Same complexity class as the
     * underlying orchestrator: end-to-end SubLinear in n.
     */
    private handleContrastiveSolveOnChangeSublinear;
    private handleSaveVectorToFile;
    private loadVectorFromFile;
    private saveVectorToFile;
    private getFileFormat;
    private generateRecommendations;
    run(): Promise<void>;
}
