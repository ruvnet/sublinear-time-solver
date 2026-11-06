import test from 'node:test';
import assert from 'node:assert/strict';
import {
  pageRank, personalizedPageRank, forwardPush, backwardPush, graphFromEdges, graphFromAdjacency, topK, SublinearError,
} from '../dist/esm/index.js';
import { densePageRank, randomGraphEdges, maxAbsDiff } from './helpers.mjs';

test('pageRank equals the exact dense solution, with and without dangling nodes', () => {
  for (const seed of [1, 2, 3]) {
    const n = 50;
    const edges = randomGraphEdges(n, 150, seed); // sparse: some nodes dangle
    const g = graphFromEdges(edges, { n });
    const uniform = new Array(n).fill(1 / n);
    for (const dangling of ['teleport', 'drop']) {
      const r = pageRank(g, { tolerance: 1e-14, maxIterations: 1000, dangling });
      assert.ok(r.converged);
      const exact = densePageRank(n, edges, uniform, 0.85, dangling);
      assert.ok(maxAbsDiff(r.scores, exact) < 1e-11, `${dangling}: ${maxAbsDiff(r.scores, exact)}`);
    }
    const sum = pageRank(g).scores.reduce((a, b) => a + b, 0);
    assert.ok(Math.abs(sum - 1) < 1e-9);
  }
});

test('personalizedPageRank matches dense solve; forwardPush matches within its residual bound', () => {
  const n = 80;
  const edges = randomGraphEdges(n, 320, 9);
  const g = graphFromEdges(edges, { n });
  const v = new Array(n).fill(0); v[3] = 0.5; v[10] = 0.5;
  const exact = densePageRank(n, edges, v, 0.85);
  const ppr = personalizedPageRank(g, [3, 10], { tolerance: 1e-14, maxIterations: 1000 });
  assert.ok(maxAbsDiff(ppr.scores, exact) < 1e-11);
  const fp = forwardPush(g, [3, 10], { epsilon: 1e-9 });
  assert.ok(fp.converged);
  let l1 = 0;
  for (let i = 0; i < n; i++) l1 += Math.abs((fp.scores.get(i) ?? 0) - exact[i]);
  assert.ok(l1 <= fp.residualMass + 1e-12, `L1 ${l1} > residual ${fp.residualMass}`);
  assert.ok(l1 < 1e-6);
});

test('backwardPush matches single-source PPR (dangling = drop) for every source', () => {
  const n = 60;
  const edges = randomGraphEdges(n, 240, 4);
  const g = graphFromEdges(edges, { n });
  const target = 7;
  const eps = 1e-9;
  const bp = backwardPush(g, target, { epsilon: eps });
  for (const s of [0, 7, 13, 59]) {
    const v = new Array(n).fill(0); v[s] = 1;
    const exact = densePageRank(n, edges, v, 0.85, 'drop')[target];
    assert.ok(Math.abs((bp.scores.get(s) ?? 0) - exact) <= eps + 1e-12, `source ${s}`);
  }
});

test('forwardPush work is independent of graph size (local)', () => {
  // A ring of n nodes with one long chord: push from node 0 only explores a neighbourhood.
  const pushesFor = (n) => {
    const edges = [];
    for (let i = 0; i < n; i++) edges.push([i, (i + 1) % n], [i, (i + 7) % n]);
    return forwardPush(graphFromEdges(edges, { n }), 0, { epsilon: 1e-4 }).pushes;
  };
  const small = pushesFor(1_000);
  const large = pushesFor(200_000);
  assert.equal(small, large);
});

test('string ids, adjacency lists and topK', () => {
  const g = graphFromEdges([['a', 'b', 2], ['b', 'c'], ['c', 'a'], ['a', 'c']]);
  assert.deepEqual(g.ids, ['a', 'b', 'c']);
  const r = personalizedPageRank(g, 'a');
  const top = topK(r.scores, 2, g);
  assert.equal(top[0].node, 'a');
  assert.equal(top.length, 2);
  const weighted = personalizedPageRank(g, new Map([['a', 3], ['b', 1]]));
  assert.ok(Math.abs(weighted.scores.reduce((x, y) => x + y, 0) - 1) < 1e-9);
  const fpTop = topK(forwardPush(g, ['a']).scores, 1, g);
  assert.equal(fpTop[0].node, 'a');
  const adj = graphFromAdjacency([[1, 2], [2], [0], []]);
  assert.equal(adj.n, 4);
  assert.ok(pageRank(adj).converged);
  assert.throws(() => personalizedPageRank(g, 'zzz'), (e) => e.code === 'UNKNOWN_NODE');
});

test('graph input validation', () => {
  const bad = (fn, code) => assert.throws(fn, (e) => e instanceof SublinearError && e.code === code);
  bad(() => graphFromEdges([[0, 1, -1]]), 'INVALID_INPUT');
  bad(() => graphFromEdges([[0, 1, NaN]]), 'INVALID_INPUT');
  bad(() => graphFromEdges([[0, 5]], { n: 3 }), 'INVALID_DIMENSIONS');
  bad(() => graphFromEdges([[0, 1e12]]), 'LIMIT_EXCEEDED');
  bad(() => graphFromAdjacency([[3]]), 'INVALID_DIMENSIONS');
  bad(() => pageRank(graphFromEdges([[0, 1]]), { damping: 1 }), 'INVALID_INPUT');
  bad(() => pageRank(graphFromEdges([[0, 1]]), { maxIterations: 1e12 }), 'LIMIT_EXCEEDED');
  bad(() => forwardPush(graphFromEdges([[0, 1]]), 0, { maxPushes: 1e12 }), 'LIMIT_EXCEEDED');
  bad(() => pageRank({ adjacency: 1 }), 'INVALID_INPUT');
  // '__proto__' is just another id, never a prototype write.
  const g = graphFromEdges([['__proto__', 'constructor'], ['constructor', '__proto__']]);
  assert.equal(g.n, 2);
  assert.equal({}.polluted, undefined);
});

test('empty graph', () => {
  const g = graphFromEdges([], { n: 0 });
  assert.equal(pageRank(g).scores.length, 0);
});
