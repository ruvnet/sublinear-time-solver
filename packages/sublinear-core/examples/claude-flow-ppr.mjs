// Drop-in replacement for claude-flow's `simplePersonalizedPageRank`
// (v3/@claude-flow/cli/src/mcp-tools/agentdb-tools.ts): string node ids,
// [source, target, weight] edges, top-k result.
//
// Note on parameters: claude-flow's `damping` is the RESTART probability
// (it adds `damping` to the seed each step), so pass `damping: 1 - restart`.
import { graphFromEdges, personalizedPageRank, topK } from '@ruvnet/sublinear-core';

export function simplePersonalizedPageRank(seedNodeId, edges, k, restart = 0.15, iterations = 50) {
  const graph = graphFromEdges(edges, { ids: 'map' });
  if (graph.indexOf(seedNodeId) < 0) return [];
  const { scores } = personalizedPageRank(graph, seedNodeId, {
    damping: 1 - restart,
    maxIterations: iterations,
    tolerance: 1e-9,
  });
  return topK(scores, k, graph).map(({ node, score }) => ({ nodeId: node, score }));
}

// Demo
const edges = [
  ['auth', 'jwt', 2], ['auth', 'session', 1], ['jwt', 'crypto', 1],
  ['session', 'redis', 1], ['crypto', 'auth', 1], ['redis', 'session', 1],
];
console.log(simplePersonalizedPageRank('auth', edges, 3));
