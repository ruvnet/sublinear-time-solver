/**
 * PageRank and personalized PageRank by power iteration.
 */
import { Graph, GraphInput, NodeId, resolveSeeds, toGraph } from './graph.js';
import {
  Limits,
  SublinearError,
  assertIterations,
  assertPositive,
  assertProbability,
  resolveLimits,
} from './errors.js';

export type DanglingPolicy = 'teleport' | 'drop';

export interface PageRankOptions {
  /** Probability of following an out-edge (1 - restart probability). Default 0.85. */
  damping?: number;
  /** Stop when the L1 change between iterates is <= tolerance. Default 1e-10. */
  tolerance?: number;
  /** Default 100. */
  maxIterations?: number;
  /**
   * Where the mass of nodes without out-edges goes. 'teleport' (default):
   * redistributed by the teleport (personalization) vector, so scores sum to 1.
   * 'drop': discarded, matching the linear system x = (1-d) v + d P^T x.
   */
  dangling?: DanglingPolicy;
  limits?: Partial<Limits>;
}

export interface PageRankResult {
  /** Score per node index. Sums to 1 with dangling = 'teleport'. */
  scores: Float64Array;
  iterations: number;
  /** L1 change of the last iteration. */
  delta: number;
  converged: boolean;
}

/** Global PageRank with a uniform teleport vector. O(iterations * (n + edges)). */
export function pageRank<K extends NodeId>(graph: GraphInput<K>, options: PageRankOptions = {}): PageRankResult {
  const g = toGraph(graph);
  const v = new Float64Array(g.n).fill(g.n > 0 ? 1 / g.n : 0);
  return powerIteration(g, v, options);
}

/**
 * Personalized PageRank (random walk with restart) from one or more seeds.
 * `seeds` may be an id, an array of ids, or an id -> weight map; ids are
 * node ids when the graph was built with an id map, indices otherwise.
 */
export function personalizedPageRank<K extends NodeId>(
  graph: GraphInput<K>,
  seeds: K | readonly K[] | ReadonlyMap<K, number> | Readonly<Record<string, number>>,
  options: PageRankOptions = {},
): PageRankResult {
  const g = toGraph(graph);
  const v = new Float64Array(g.n);
  for (const [i, w] of resolveSeeds(g, seeds)) v[i] = w;
  return powerIteration(g, v, options);
}

function powerIteration<K extends NodeId>(g: Graph<K>, v: Float64Array, options: PageRankOptions): PageRankResult {
  const limits = resolveLimits(options.limits);
  const d = assertProbability('damping', options.damping ?? 0.85);
  const tol = assertPositive('tolerance', options.tolerance ?? 1e-10);
  const maxIt = assertIterations('maxIterations', options.maxIterations ?? 100, limits);
  const dangling = options.dangling ?? 'teleport';
  if (dangling !== 'teleport' && dangling !== 'drop') {
    throw new SublinearError(`dangling must be 'teleport' or 'drop'`, 'INVALID_INPUT');
  }
  const n = g.n;
  const { rowPtr, colIdx, values } = g.adjacency;
  const outW = g.outWeight;
  // Pre-scale edge weights by 1 / outWeight once.
  const invOut = new Float64Array(n);
  for (let u = 0; u < n; u++) invOut[u] = outW[u] > 0 ? 1 / outW[u] : 0;

  let x = v.slice();
  let y = new Float64Array(n);
  let delta = Infinity;
  let it = 0;
  while (it < maxIt) {
    y.fill(0);
    let danglingMass = 0;
    for (let u = 0; u < n; u++) {
      const xu = x[u];
      if (xu === 0) continue;
      const s = rowPtr[u];
      const e = rowPtr[u + 1];
      if (s === e) { danglingMass += xu; continue; }
      const f = d * xu * invOut[u];
      for (let k = s; k < e; k++) y[colIdx[k]] += f * values[k];
    }
    const teleport = (1 - d) + (dangling === 'teleport' ? d * danglingMass : 0);
    delta = 0;
    for (let i = 0; i < n; i++) {
      const yi = y[i] + teleport * v[i];
      y[i] = yi;
      delta += Math.abs(yi - x[i]);
    }
    const t = x; x = y; y = t;
    it++;
    if (delta <= tol) break;
  }
  return { scores: x, iterations: it, delta, converged: delta <= tol };
}
