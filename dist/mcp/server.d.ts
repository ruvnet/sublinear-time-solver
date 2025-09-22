/**
 * MCP Server for Sublinear-Time Solver
 * Provides MCP interface to the core solver algorithms
 */
export declare class SublinearSolverMCPServer {
    private server;
    private solvers;
    private temporalTools;
    private psychoSymbolicTools;
    private consciousnessTools;
    private schedulerTools;
    constructor();
    private setupToolHandlers;
    private setupErrorHandling;
    private handleSolve;
    private handleEstimateEntry;
    private handleAnalyzeMatrix;
    private handlePageRank;
    private generateRecommendations;
    run(): Promise<void>;
}
