/**
 * Main entry point for the Sublinear-Time Solver package
 * Provides both MCP server and direct API access
 */

// Dependency-free core API (solve, pageRank, personalizedPageRank, forwardPush,
// backwardPush, estimateEntry, CSR/graph builders). Also available without the
// MCP server via 'sublinear-time-solver/core' or the @ruvnet/sublinear-core package.
export * from '@ruvnet/sublinear-core';

// 1.x class API
export { SublinearSolver } from './core/solver.js';
export { MatrixOperations } from './core/matrix.js';
export { VectorOperations, PerformanceMonitor, ConvergenceChecker, ValidationUtils } from './core/utils.js';

// MCP exports
export { SublinearSolverMCPServer } from './mcp/server.js';
export { SolverTools } from './mcp/tools/solver.js';
export { MatrixTools } from './mcp/tools/matrix.js';
export { GraphTools } from './mcp/tools/graph.js';

// Temporal Attractor exports
export { temporalAttractorTools, temporalAttractorHandlers } from './mcp/tools/temporal-attractor.js';

// Types
export * from './core/types.js';

// Re-export everything from MCP module
export * from './mcp/index.js';