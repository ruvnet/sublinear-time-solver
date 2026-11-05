/**
 * @ruvnet/sublinear-core: zero-dependency sparse linear solvers and
 * PageRank for Node.js, Deno, Bun and browsers.
 */
export {
  SublinearError,
  DEFAULT_LIMITS,
} from './errors.js';
export type { Limits, SublinearErrorCode } from './errors.js';

export {
  csrFromCoo,
  csrFromDense,
  toCsr,
  isCsrMatrix,
  validateCsr,
  spmv,
  transpose,
  diagonal,
  diagonalDominance,
  isSymmetric,
  relativeResidual,
} from './csr.js';
export type { CsrMatrix, CooInput, MatrixInput, BuildOptions, DominanceReport } from './csr.js';

export { solve, estimateEntry, mulberry32 } from './solve.js';
export type { SolveMethod, SolveOptions, SolveResult, EstimateEntryOptions, EstimateEntryResult } from './solve.js';

export {
  graphFromEdges,
  graphFromAdjacency,
  graphFromCsr,
  toGraph,
  topK,
} from './graph.js';
export type { Graph, GraphInput, GraphOptions, Edge, NodeId, RankedNode } from './graph.js';

export { pageRank, personalizedPageRank } from './pagerank.js';
export type { PageRankOptions, PageRankResult, DanglingPolicy } from './pagerank.js';

export { forwardPush, backwardPush } from './push.js';
export type { PushOptions, PushResult } from './push.js';
