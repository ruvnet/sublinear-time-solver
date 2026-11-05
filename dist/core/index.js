/**
 * `sublinear-time-solver/core`: the solver API without the MCP server, CLI or
 * WASM loaders. Importing this entry loads no third-party modules other than
 * the dependency-free @ruvnet/sublinear-core.
 *
 * For the smallest install, depend on @ruvnet/sublinear-core directly; the
 * functions below are the same objects.
 */
export * from '@ruvnet/sublinear-core';
export { SublinearSolver } from './solver.js';
export { SolverError, ErrorCodes } from './types.js';
