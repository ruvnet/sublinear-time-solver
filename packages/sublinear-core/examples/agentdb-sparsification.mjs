// JS fallback for agentdb's SparsificationService.pprSparsification
// (src/controllers/SparsificationService.ts): numeric adjacency lists,
// `alpha` = teleport probability, returns Float32Array scores + top-k indices.
//
// For large graphs use forwardPush: its cost depends on epsilon, not on n,
// so a top-k query on a million-node graph touches only a few hundred nodes.
import { graphFromAdjacency, personalizedPageRank, forwardPush, topK } from '@ruvnet/sublinear-core';

export function pprSparsification(sourceNode, adjacency, k, alpha = 0.15, { local = false } = {}) {
  const graph = graphFromAdjacency(adjacency);
  if (local) {
    const push = forwardPush(graph, sourceNode, { damping: 1 - alpha, epsilon: 1e-6 });
    const top = topK(push.scores, k);
    return { topKIndices: top.map((r) => r.index), scores: push.scores, method: 'ppr-forward-push', pushes: push.pushes };
  }
  const r = personalizedPageRank(graph, sourceNode, { damping: 1 - alpha, maxIterations: 50, tolerance: 1e-8 });
  return {
    topKIndices: topK(r.scores, k).map((x) => x.index),
    scores: Float32Array.from(r.scores),
    method: 'ppr-power-iteration',
    iterations: r.iterations,
  };
}

// Demo: adjacency[u] lists the neighbours of u
const adjacency = [[1, 2], [2, 3], [0], [4], [0, 1], []];
console.log(pprSparsification(0, adjacency, 3));
console.log(pprSparsification(0, adjacency, 3, 0.15, { local: true }));
