/**
 * Local push algorithms for personalized PageRank. Both touch only the part
 * of the graph near the seed/target, so their work is bounded by the
 * tolerance rather than by the graph size.
 */
import { GraphInput, NodeId, inAdjacency, resolveSeeds, toGraph } from './graph.js';
import { Limits, SublinearError, assertPositive, assertProbability, resolveLimits } from './errors.js';

export interface PushOptions {
  /** Probability of following an out-edge. Default 0.85. */
  damping?: number;
  /** Residual threshold. Smaller = more accurate and more work. Default 1e-6. */
  epsilon?: number;
  /** Optional cap on push operations (also bounded by limits.maxPushes). */
  maxPushes?: number;
  limits?: Partial<Limits>;
}

export interface PushResult {
  /** Sparse estimates: node index -> score. Nodes not present have estimate 0. */
  scores: Map<number, number>;
  /** Residual mass left unpushed (an upper bound on the total L1 error for forward push). */
  residualMass: number;
  pushes: number;
  /** False when stopped by maxPushes before reaching epsilon. */
  converged: boolean;
}

function pushParams(options: PushOptions, limits: Limits) {
  const d = assertProbability('damping', options.damping ?? 0.85);
  const eps = assertPositive('epsilon', options.epsilon ?? 1e-6);
  const cap = options.maxPushes ?? limits.maxPushes;
  if (!Number.isInteger(cap) || cap <= 0 || cap > limits.maxPushes) {
    throw new SublinearError(`maxPushes must be an integer in [1, ${limits.maxPushes}]`, 'LIMIT_EXCEEDED');
  }
  return { d, eps, cap };
}

/**
 * Forward push (Andersen, Chung, Lang 2006): approximates the personalized
 * PageRank vector of `seeds`. Stops when every node u has residual
 * r(u) < epsilon * max(1, outDegree(u)). Total work is O(1 / ((1 - damping) * epsilon))
 * pushes, independent of n. Dangling mass returns to the seeds, matching
 * personalizedPageRank(..., { dangling: 'teleport' }).
 */
export function forwardPush<K extends NodeId>(
  graph: GraphInput<K>,
  seeds: K | readonly K[] | ReadonlyMap<K, number> | Readonly<Record<string, number>>,
  options: PushOptions = {},
): PushResult {
  const limits = resolveLimits(options.limits);
  const g = toGraph(graph);
  const { d, eps, cap } = pushParams(options, limits);
  const seedDist = resolveSeeds(g, seeds);
  const { rowPtr, colIdx, values } = g.adjacency;
  const outW = g.outWeight;

  const p = new Map<number, number>();
  const r = new Map<number, number>(seedDist);
  const queue: number[] = [];
  const queued = new Set<number>();
  const threshold = (u: number) => eps * Math.max(1, rowPtr[u + 1] - rowPtr[u]);
  for (const u of r.keys()) if (r.get(u)! >= threshold(u)) { queue.push(u); queued.add(u); }

  let pushes = 0;
  let head = 0;
  while (head < queue.length && pushes < cap) {
    const u = queue[head++];
    queued.delete(u);
    const ru = r.get(u) ?? 0;
    if (ru < threshold(u)) continue;
    r.set(u, 0);
    p.set(u, (p.get(u) ?? 0) + (1 - d) * ru);
    pushes++;
    const s = rowPtr[u];
    const e = rowPtr[u + 1];
    const spread = d * ru;
    if (s === e) {
      for (const [sd, w] of seedDist) {
        const nv = (r.get(sd) ?? 0) + spread * w;
        r.set(sd, nv);
        if (!queued.has(sd) && nv >= threshold(sd)) { queue.push(sd); queued.add(sd); }
      }
    } else {
      const f = spread / outW[u];
      for (let k = s; k < e; k++) {
        const v = colIdx[k];
        const nv = (r.get(v) ?? 0) + f * values[k];
        r.set(v, nv);
        if (!queued.has(v) && nv >= threshold(v)) { queue.push(v); queued.add(v); }
      }
    }
    if (head > 4096 && head * 2 > queue.length) { queue.splice(0, head); head = 0; }
  }
  let residualMass = 0;
  for (const v of r.values()) residualMass += v;
  return { scores: p, residualMass, pushes, converged: head >= queue.length };
}

/**
 * Backward (reverse) push (Andersen et al. 2007): estimates, for every source
 * s, the personalized PageRank score ppr_s(target), with additive error at
 * most epsilon per source. Dangling nodes are treated as absorbing, so this
 * matches personalizedPageRank(g, s, { dangling: 'drop' })[target].
 * Work is bounded by the in-neighbourhood explored, not by n.
 */
export function backwardPush<K extends NodeId>(graph: GraphInput<K>, target: K, options: PushOptions = {}): PushResult {
  const limits = resolveLimits(options.limits);
  const g = toGraph(graph);
  const { d, eps, cap } = pushParams(options, limits);
  const t = g.indexOf(target);
  if (t < 0) throw new SublinearError(`unknown target node ${String(target)}`, 'UNKNOWN_NODE');
  const T = inAdjacency(g);
  const outW = g.outWeight;

  const p = new Map<number, number>();
  const r = new Map<number, number>([[t, 1]]);
  const queue: number[] = [t];
  const queued = new Set<number>([t]);
  let pushes = 0;
  let head = 0;
  while (head < queue.length && pushes < cap) {
    const v = queue[head++];
    queued.delete(v);
    const rv = r.get(v) ?? 0;
    if (rv < eps) continue;
    r.set(v, 0);
    p.set(v, (p.get(v) ?? 0) + (1 - d) * rv);
    pushes++;
    for (let k = T.rowPtr[v], e = T.rowPtr[v + 1]; k < e; k++) {
      const u = T.colIdx[k];
      const nu = (r.get(u) ?? 0) + (d * rv * T.values[k]) / outW[u];
      r.set(u, nu);
      if (!queued.has(u) && nu >= eps) { queue.push(u); queued.add(u); }
    }
    if (head > 4096 && head * 2 > queue.length) { queue.splice(0, head); head = 0; }
  }
  let residualMass = 0;
  for (const v of r.values()) residualMass += v;
  return { scores: p, residualMass, pushes, converged: head >= queue.length };
}
