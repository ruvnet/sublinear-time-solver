/**
 * Graph construction for PageRank-style algorithms.
 *
 * A Graph stores out-edges as a CSR matrix (row = source, col = target,
 * value = weight) plus cached out-weight sums. Nodes are dense integers
 * 0..n-1; when edges use other ids (strings, sparse numbers) the builder
 * keeps an id <-> index map.
 */
import { CsrMatrix, csrFromCoo, isCsrMatrix, transpose, validateCsr } from './csr.js';
import { Limits, SublinearError, assertDimension, resolveLimits } from './errors.js';

export type NodeId = string | number;
export type Edge<K extends NodeId = NodeId> = readonly [source: K, target: K, weight?: number];

export interface Graph<K extends NodeId = number> {
  readonly n: number;
  /** Out-adjacency: row u lists targets of u with weights. */
  readonly adjacency: CsrMatrix;
  /** Sum of out-edge weights per node (0 for dangling nodes). */
  readonly outWeight: Float64Array;
  /** Original ids by index, present when the graph was built with an id map. */
  readonly ids?: readonly K[];
  /** Index of an id. Returns -1 when unknown. */
  indexOf(id: K): number;
}

export interface GraphOptions {
  /** Node count when edges use integer indices 0..n-1. Inferred as max index + 1 otherwise. */
  n?: number;
  /** Add the reverse of every edge. Default false. */
  undirected?: boolean;
  /**
   * 'auto' (default): integer endpoints are used as indices directly, anything
   * else gets an id map. 'map': always build an id map.
   */
  ids?: 'auto' | 'map';
  limits?: Partial<Limits>;
}

function makeGraph<K extends NodeId>(adjacency: CsrMatrix, ids?: K[], index?: Map<K, number>): Graph<K> {
  validateCsr(adjacency);
  if (adjacency.rows !== adjacency.cols) throw new SublinearError('adjacency must be square', 'INVALID_DIMENSIONS');
  const n = adjacency.rows;
  const outWeight = new Float64Array(n);
  const { rowPtr, values } = adjacency;
  for (let u = 0; u < n; u++) {
    let s = 0;
    for (let k = rowPtr[u], e = rowPtr[u + 1]; k < e; k++) {
      if (values[k] < 0) throw new SublinearError('edge weights must be non-negative', 'INVALID_INPUT');
      s += values[k];
    }
    outWeight[u] = s;
  }
  return {
    n,
    adjacency,
    outWeight,
    ids,
    indexOf: index
      ? (id: K) => index.get(id) ?? -1
      : (id: K) => (typeof id === 'number' && Number.isInteger(id) && id >= 0 && id < n ? id : -1),
  };
}

/** Build a graph from [source, target, weight?] edges. Weight defaults to 1; duplicates are summed. */
export function graphFromEdges<K extends NodeId>(edges: Iterable<Edge<K>>, options: GraphOptions = {}): Graph<K> {
  const limits = resolveLimits(options.limits);
  if (edges == null || typeof (edges as Iterable<Edge<K>>)[Symbol.iterator] !== 'function') {
    throw new SublinearError('edges must be iterable', 'INVALID_INPUT');
  }
  const list = Array.isArray(edges) ? (edges as Edge<K>[]) : Array.from(edges);
  const m = options.undirected ? list.length * 2 : list.length;
  if (m > limits.maxNonZeros) throw new SublinearError(`edge count ${m} exceeds limit ${limits.maxNonZeros}`, 'LIMIT_EXCEEDED');

  let useMap = options.ids === 'map';
  if (!useMap) {
    for (const e of list) {
      if (!Array.isArray(e) || e.length < 2) throw new SublinearError('each edge must be [source, target, weight?]', 'INVALID_INPUT');
      if (!(Number.isInteger(e[0]) && Number.isInteger(e[1]) && (e[0] as number) >= 0 && (e[1] as number) >= 0)) {
        useMap = true;
        break;
      }
    }
  }

  const src = new Uint32Array(m);
  const dst = new Uint32Array(m);
  const w = new Float64Array(m);
  let ids: K[] | undefined;
  let index: Map<K, number> | undefined;
  let n = 0;
  const idOf = (id: K): number => {
    if (!index) {
      const v = id as number;
      if (v + 1 > n) n = v + 1;
      return v;
    }
    let i = index.get(id);
    if (i === undefined) {
      if (typeof id !== 'string' && typeof id !== 'number') {
        throw new SublinearError('node ids must be strings or numbers', 'INVALID_INPUT');
      }
      i = ids!.length;
      ids!.push(id);
      index.set(id, i);
    }
    return i;
  };
  if (useMap) { ids = []; index = new Map(); }

  let k = 0;
  for (const e of list) {
    if (!Array.isArray(e) || e.length < 2) throw new SublinearError('each edge must be [source, target, weight?]', 'INVALID_INPUT');
    const weight = e[2] ?? 1;
    if (typeof weight !== 'number' || !Number.isFinite(weight) || weight < 0) {
      throw new SublinearError('edge weight must be a finite non-negative number', 'INVALID_INPUT');
    }
    const a = idOf(e[0]);
    const b = idOf(e[1]);
    src[k] = a; dst[k] = b; w[k] = weight; k++;
    if (options.undirected) { src[k] = b; dst[k] = a; w[k] = weight; k++; }
  }
  if (index) n = ids!.length;
  if (options.n !== undefined) {
    const declared = assertDimension('n', options.n, limits);
    if (index) {
      if (declared < n) throw new SublinearError(`n (${declared}) is smaller than the number of distinct ids (${n})`, 'INVALID_DIMENSIONS');
    } else if (declared < n) {
      throw new SublinearError(`edge endpoint ${n - 1} is out of range for n = ${declared}`, 'INVALID_DIMENSIONS');
    }
    n = declared;
  }
  assertDimension('n', n, limits);
  const adjacency = csrFromCoo({ rows: n, cols: n, rowIndices: src, colIndices: dst, values: w }, { limits, dropZeros: true });
  return makeGraph<K>(adjacency, ids, index);
}

/** Build a graph from an adjacency list: adjacency[u] = targets of u (unit weights). */
export function graphFromAdjacency(adjacency: ArrayLike<ArrayLike<number>>, options: Omit<GraphOptions, 'n' | 'ids'> = {}): Graph<number> {
  if (!(Array.isArray(adjacency) || ArrayBuffer.isView(adjacency))) {
    throw new SublinearError('adjacency must be an array of neighbour lists', 'INVALID_INPUT');
  }
  const edges: [number, number][] = [];
  const n = adjacency.length;
  for (let u = 0; u < n; u++) {
    const row = adjacency[u];
    if (row == null) continue;
    for (let k = 0; k < row.length; k++) {
      const v = row[k];
      if (!Number.isInteger(v) || v < 0 || v >= n) {
        throw new SublinearError(`adjacency[${u}][${k}] = ${v} out of range [0, ${n})`, 'INVALID_DIMENSIONS');
      }
      edges.push([u, v]);
    }
  }
  return graphFromEdges<number>(edges, { ...options, n, ids: 'auto' });
}

/** Wrap an existing square CSR adjacency matrix (row = source). */
export function graphFromCsr(adjacency: CsrMatrix): Graph<number> {
  return makeGraph<number>(adjacency);
}

export type GraphInput<K extends NodeId = number> = Graph<K> | CsrMatrix;

export function toGraph<K extends NodeId>(g: GraphInput<K>): Graph<K> {
  if (isCsrMatrix(g)) return graphFromCsr(g) as unknown as Graph<K>;
  if (g && typeof g === 'object' && isCsrMatrix((g as Graph<K>).adjacency) && (g as Graph<K>).outWeight instanceof Float64Array) {
    return g as Graph<K>;
  }
  throw new SublinearError('expected a Graph (from graphFromEdges/graphFromAdjacency) or a CSR adjacency matrix', 'INVALID_INPUT');
}

const inCache = new WeakMap<CsrMatrix, CsrMatrix>();

/** In-adjacency (transpose), cached per graph. */
export function inAdjacency(g: Graph<NodeId>): CsrMatrix {
  let t = inCache.get(g.adjacency);
  if (!t) { t = transpose(g.adjacency); inCache.set(g.adjacency, t); }
  return t;
}

/** Resolve seed ids (or a weight map) into a normalized sparse distribution. */
export function resolveSeeds<K extends NodeId>(
  g: Graph<K>, seeds: K | readonly K[] | ReadonlyMap<K, number> | Readonly<Record<string, number>>,
): Map<number, number> {
  const out = new Map<number, number>();
  const add = (id: K, w: number) => {
    if (typeof w !== 'number' || !Number.isFinite(w) || w < 0) {
      throw new SublinearError('seed weights must be finite and non-negative', 'INVALID_INPUT');
    }
    const i = g.indexOf(id);
    if (i < 0) throw new SublinearError(`unknown seed node ${String(id)}`, 'UNKNOWN_NODE');
    out.set(i, (out.get(i) ?? 0) + w);
  };
  if (typeof seeds === 'string' || typeof seeds === 'number') add(seeds as K, 1);
  else if (Array.isArray(seeds)) for (const s of seeds as readonly K[]) add(s, 1);
  else if (seeds instanceof Map) for (const [s, w] of seeds) add(s, w);
  else if (seeds && typeof seeds === 'object') {
    for (const key of Object.keys(seeds)) {
      const id = (g.ids ? key : Number(key)) as K;
      add(id, (seeds as Record<string, number>)[key]);
    }
  } else throw new SublinearError('seeds must be an id, an array of ids, or an id -> weight map', 'INVALID_INPUT');
  let total = 0;
  for (const v of out.values()) total += v;
  if (!(total > 0)) throw new SublinearError('seed weights must not all be zero', 'INVALID_INPUT');
  for (const [k, v] of out) out.set(k, v / total);
  return out;
}

export interface RankedNode<K extends NodeId = number> {
  node: K;
  index: number;
  score: number;
}

/** Top-k nodes by score, ties broken by index. Accepts dense or sparse scores. */
export function topK<K extends NodeId = number>(
  scores: ArrayLike<number> | ReadonlyMap<number, number>, k: number, graph?: Graph<K>,
): RankedNode<K>[] {
  if (!Number.isInteger(k) || k < 0) throw new SublinearError('k must be a non-negative integer', 'INVALID_INPUT');
  const entries: [number, number][] = [];
  if (scores instanceof Map) for (const [i, s] of scores) entries.push([i, s]);
  else for (let i = 0; i < (scores as ArrayLike<number>).length; i++) entries.push([i, (scores as ArrayLike<number>)[i]]);
  entries.sort((a, b) => b[1] - a[1] || a[0] - b[0]);
  return entries.slice(0, k).map(([index, score]) => ({
    node: (graph?.ids ? graph.ids[index] : index) as K,
    index,
    score,
  }));
}
