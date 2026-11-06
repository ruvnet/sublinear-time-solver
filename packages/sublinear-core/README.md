# @ruvnet/sublinear-core

Zero-dependency sparse linear solvers and PageRank for JavaScript and TypeScript. It provides CSR matrices, Jacobi, Gauss-Seidel, conjugate gradient and Neumann-series solves, global and personalized PageRank, local forward and backward push, and single-entry random-walk estimates. The package works in Node.js 16+, Bun, Deno and browsers, with ESM and CommonJS builds and bundled type definitions.

This is the embeddable core of [`sublinear-time-solver`](https://www.npmjs.com/package/sublinear-time-solver). Use it when you need the math without the MCP server, CLI, Express or the MCP SDK.

```bash
npm install @ruvnet/sublinear-core
```

- **0 runtime dependencies.** The npm tarball is under 30 kB. Importing it loads no third-party module.
- **Typed, validated inputs.** Indices must be in-range integers, values must be finite, and dimensions, nnz, iterations and pushes are all capped (see `DEFAULT_LIMITS`). Untrusted JSON cannot make it allocate unbounded memory or loop forever.
- **Honest results.** `solve()` returns the true residual of the vector it hands back. The tests check every method against a dense Gaussian-elimination reference.

## API at a glance

```ts
import {
  solve, estimateEntry,                       // linear systems A x = b
  pageRank, personalizedPageRank,             // power iteration
  forwardPush, backwardPush,                  // local (sublinear) PPR
  graphFromEdges, graphFromAdjacency, topK,   // graph builders + ranking
  csrFromCoo, csrFromDense, spmv, relativeResidual, diagonalDominance,
} from '@ruvnet/sublinear-core';
```

| Function | What it computes | Cost |
|---|---|---|
| `solve(A, b, { method })` | x with `‖b − A x‖₂ / ‖b‖₂ ≤ tolerance` | O(iterations · nnz) |
| `estimateEntry(A, b, i)` | Monte Carlo estimate of `x[i]` with a standard error | O(walks · row nnz / (1 − ‖M‖∞)), independent of n after a one-time O(nnz) check per matrix |
| `pageRank(g)` | Global PageRank (scores sum to 1) | O(iterations · (n + m)) |
| `personalizedPageRank(g, seeds)` | Random walk with restart from seeds | O(iterations · (n + m)) |
| `forwardPush(g, seeds, { epsilon })` | Sparse PPR approximation from seeds | O(1 / ((1 − damping) · epsilon)) pushes, independent of n |
| `backwardPush(g, target, { epsilon })` | `ppr_s(target)` for every source s, error ≤ epsilon | Bounded by the in-neighbourhood explored |

Methods for `solve`: `'auto'` (the default) uses conjugate gradient when the matrix is symmetric with a positive diagonal, and Gauss-Seidel otherwise. The explicit choices are `'conjugate-gradient'`, `'gauss-seidel'`, `'jacobi'` and `'neumann'`. Jacobi and Neumann have identical iterates from x0 = 0. Jacobi, Neumann and Gauss-Seidel converge for strictly diagonally dominant matrices. Conjugate gradient requires a symmetric positive definite matrix.

## How-to

### Solve a sparse system

```js
import { solve } from '@ruvnet/sublinear-core';

const A = { rows: 3, cols: 3, rowIndices: [0, 0, 1, 1, 2], colIndices: [0, 1, 1, 2, 2], values: [4, -1, 4, -1, 4] };
const { x, residual, converged, iterations, method } = solve(A, [1, 2, 3], { tolerance: 1e-12 });
```

`A` can be a COO object (duplicates are summed), a dense array of rows, or a `CsrMatrix` from `csrFromCoo`/`csrFromDense`. Build the CSR once and reuse it when you solve repeatedly.

### PageRank and personalized PageRank with string ids

```js
import { graphFromEdges, pageRank, personalizedPageRank, topK } from '@ruvnet/sublinear-core';

const g = graphFromEdges([['a', 'b'], ['b', 'c', 2], ['c', 'a'], ['a', 'c']]); // [source, target, weight?]
topK(pageRank(g).scores, 2, g);                    // [{ node: 'c', index, score }, ...]
personalizedPageRank(g, 'a', { damping: 0.85 });   // seeds: id | id[] | Map<id, weight>
```

`damping` is the probability of following an edge. A restart (teleport) probability of `alpha` corresponds to `damping = 1 - alpha`. By default, mass that reaches a node with no out-edges is redistributed through the teleport vector, so scores sum to 1. Pass `dangling: 'drop'` to get the linear-system form `x = (1 − d) v + d Pᵀ x` instead.

### Local top-k on a huge graph

```js
import { forwardPush, topK } from '@ruvnet/sublinear-core';

const { scores, pushes, residualMass } = forwardPush(g, seed, { epsilon: 1e-5 });
topK(scores, 10, g); // scores is a sparse Map<index, score>
```

Forward push stops once every node u has residual below `epsilon · max(1, outDegree(u))`. The total unpushed mass `residualMass` bounds the L1 error. Each node's error is bounded by its degree-scaled threshold, which keeps the top of the ranking accurate even when the L1 error is large. The `forwardPush` rows in the benchmarks below show this.

### Estimate a single solution entry

```js
import { csrFromCoo, estimateEntry } from '@ruvnet/sublinear-core';
const A = csrFromCoo(coo);                        // reuse the CSR across calls
const { estimate, standardError } = estimateEntry(A, b, 42, { walks: 20000, seed: 1 });
```

This requires strict row diagonal dominance.

## Ecosystem drop-ins

Runnable versions live in [`examples/`](https://github.com/ruvnet/sublinear-time-solver/tree/main/packages/sublinear-core/examples):

- **claude-flow** `simplePersonalizedPageRank(seed, edges, topK, damping, iterations)`: see `claude-flow-ppr.mjs`. claude-flow's `damping` is the restart probability, so pass `damping: 1 - restart`.
- **agentdb** `SparsificationService` JS PPR fallback: see `agentdb-sparsification.mjs`. It takes adjacency lists and `alpha` (teleport) and returns `Float32Array` scores, with a `forwardPush` option for large graphs.
- **ruvector** similarity or code graphs: see `ruvector-graph-rank.mjs`. It covers PageRank, backward push, and Laplacian smoothing with conjugate gradient.

Make it an optional dependency and fall back to your own code when it is missing:

```js
let core = null;
try { core = await import('@ruvnet/sublinear-core'); } catch { /* optional */ }
```

## Benchmarks

Run `npm run bench` in this package. The harness generates the same inputs from fixed seeds on every run and compares against dense Gaussian elimination. It can also time the legacy 1.7.2 solver (`LEGACY=<dir>`) and `@ruvector/graph-transformer` `pprScores` (`GRAPH_TRANSFORMER=<dir>`). Results are in the [main README](https://github.com/ruvnet/sublinear-time-solver#benchmarks).

## License

MIT OR Apache-2.0
