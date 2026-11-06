// Ranking nodes of a similarity graph (for example a ruvector HNSW neighbour
// graph or a code-graph) and solving a graph-Laplacian system for smoothing.
import { graphFromEdges, pageRank, backwardPush, csrFromCoo, solve, topK } from '@ruvnet/sublinear-core';

// Weighted, undirected similarity edges between vector ids.
const sim = [['v1', 'v2', 0.9], ['v2', 'v3', 0.7], ['v3', 'v1', 0.4], ['v3', 'v4', 0.8], ['v4', 'v5', 0.6]];
const graph = graphFromEdges(sim, { undirected: true });

// 1. Global importance.
console.log('pageRank', topK(pageRank(graph).scores, 3, graph));

// 2. Which nodes would reach v5 most often? (one backward push, all sources at once)
const toV5 = backwardPush(graph, 'v5', { epsilon: 1e-8 });
console.log('ppr(s -> v5)', topK(toV5.scores, 3, graph));

// 3. Laplacian smoothing: solve (I + L) x = y, symmetric positive definite -> conjugate gradient.
const n = graph.n;
const { rowPtr, colIdx, values } = graph.adjacency;
const ri = [], ci = [], vs = [];
for (let i = 0; i < n; i++) {
  let deg = 0;
  for (let k = rowPtr[i]; k < rowPtr[i + 1]; k++) { ri.push(i); ci.push(colIdx[k]); vs.push(-values[k]); deg += values[k]; }
  ri.push(i); ci.push(i); vs.push(1 + deg);
}
const A = csrFromCoo({ rows: n, cols: n, rowIndices: ri, colIndices: ci, values: vs });
const y = [1, 0, 0, 0, 5];
const r = solve(A, y);
console.log('smoothed', Array.from(r.x).map((v) => +v.toFixed(4)), r.method, 'residual', r.residual.toExponential(2));
