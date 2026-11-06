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

  // Invariant: a node is in the queue exactly when its residual is at or
  // above its threshold (residuals only grow until the node is pushed), so
  // a node is enqueued when its residual crosses the threshold.
  //
  // Storage starts sparse (Maps), so small queries never touch O(n) memory.
  // Once the explored region exceeds n / 8 nodes the state moves to dense
  // typed arrays, which are several times faster per push.
  const n = g.n;
  const denseAt = n >>> 3;
  let pMap: Map<number, number> | null = new Map<number, number>();
  let rMap: Map<number, number> | null = new Map<number, number>(seedDist);
  let pArr: Float64Array | null = null;
  let rArr: Float64Array | null = null;
  const queue: number[] = [];
  const threshold = (u: number) => eps * Math.max(1, rowPtr[u + 1] - rowPtr[u]);
  for (const [u, ru] of rMap) if (ru >= threshold(u)) queue.push(u);

  const getR = (u: number) => (rArr ? rArr[u] : rMap!.get(u) ?? 0);
  const setR = (u: number, v: number) => { if (rArr) rArr[u] = v; else rMap!.set(u, v); };
  const addR = (u: number, delta: number) => {
    const old = getR(u);
    const nv = old + delta;
    setR(u, nv);
    const th = eps * Math.max(1, rowPtr[u + 1] - rowPtr[u]);
    if (old < th && nv >= th) queue.push(u);
  };

  let pushes = 0;
  let head = 0;
  while (head < queue.length && pushes < cap) {
    if (!rArr && rMap!.size > denseAt && denseAt > 0) {
      rArr = new Float64Array(n);
      pArr = new Float64Array(n);
      for (const [k, v] of rMap!) rArr[k] = v;
      for (const [k, v] of pMap!) pArr[k] = v;
      rMap = null;
      pMap = null;
    }
    const u = queue[head++];
    const ru = getR(u);
    setR(u, 0);
    if (pArr) pArr[u] += (1 - d) * ru; else pMap!.set(u, (pMap!.get(u) ?? 0) + (1 - d) * ru);
    pushes++;
    const s = rowPtr[u];
    const e = rowPtr[u + 1];
    const spread = d * ru;
    if (s === e) {
      for (const [sd, w] of seedDist) addR(sd, spread * w);
    } else if (rArr) {
      const f = spread / outW[u];
      for (let k = s; k < e; k++) {
        const v = colIdx[k];
        const old = rArr[v];
        const nv = old + f * values[k];
        rArr[v] = nv;
        const th = eps * Math.max(1, rowPtr[v + 1] - rowPtr[v]);
        if (old < th && nv >= th) queue.push(v);
      }
    } else {
      const f = spread / outW[u];
      for (let k = s; k < e; k++) addR(colIdx[k], f * values[k]);
    }
    if (head > 4096 && head * 2 > queue.length) { queue.splice(0, head); head = 0; }
  }
  let residualMass = 0;
  let scores: Map<number, number>;
  if (rArr) {
    for (let i = 0; i < n; i++) residualMass += rArr[i];
    scores = new Map<number, number>();
    for (let i = 0; i < n; i++) if (pArr![i] !== 0) scores.set(i, pArr![i]);
  } else {
    for (const v of rMap!.values()) residualMass += v;
    scores = pMap!;
  }
  return { scores, residualMass, pushes, converged: head >= queue.length };
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
  const queue: number[] = 1 >= eps ? [t] : [];
  let pushes = 0;
  let head = 0;
  while (head < queue.length && pushes < cap) {
    const v = queue[head++];
    const rv = r.get(v) ?? 0;
    r.set(v, 0);
    p.set(v, (p.get(v) ?? 0) + (1 - d) * rv);
    pushes++;
    for (let k = T.rowPtr[v], e = T.rowPtr[v + 1]; k < e; k++) {
      const u = T.colIdx[k];
      const old = r.get(u) ?? 0;
      const nu = old + (d * rv * T.values[k]) / outW[u];
      r.set(u, nu);
      if (old < eps && nu >= eps) queue.push(u);
    }
    if (head > 4096 && head * 2 > queue.length) { queue.splice(0, head); head = 0; }
  }
  let residualMass = 0;
  for (const v of r.values()) residualMass += v;
  return { scores: p, residualMass, pushes, converged: head >= queue.length };
}
