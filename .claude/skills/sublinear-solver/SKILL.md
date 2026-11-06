---
name: sublinear-solver
description: Solve sparse diagonally dominant linear systems, rank graph nodes with PageRank / personalized PageRank, or get local top-k scores with forward/backward push, using @ruvnet/sublinear-core (zero dependencies) or the sublinear-time-solver MCP server. Use when a task needs A x = b on a sparse matrix, node importance in a graph, "related items" from a seed node, or a single entry of a large solution.
---

# Sublinear solver

## Pick the entry point

- Library code: `npm i @ruvnet/sublinear-core`. It has 0 dependencies and provides ESM, CJS and types.
- Agent tools: the MCP server `npx -y sublinear-time-solver serve`. The tools to use are `solve`, `pageRank`, `estimateEntry` and `analyzeMatrix`.
- Rust: `cargo add sublinear`.

## Recipes

```js
import { solve, graphFromEdges, pageRank, personalizedPageRank, forwardPush, backwardPush, estimateEntry, topK } from '@ruvnet/sublinear-core';

// A x = b. A is COO {rows, cols, rowIndices, colIndices, values}, dense rows, or a CsrMatrix.
const { x, residual, converged, method } = solve(A, b, { tolerance: 1e-10 });

// Graph ranking with string ids; edges are [source, target, weight?].
const g = graphFromEdges(edges);
topK(pageRank(g).scores, 10, g);
topK(personalizedPageRank(g, seedId, { damping: 0.85 }).scores, 10, g);

// Large graph, local answer: the cost depends on epsilon, not on n.
topK(forwardPush(g, seedId, { epsilon: 1e-6 }).scores, 10, g);
backwardPush(g, targetId);              // ppr_s(target) for every source s

// One entry of x without solving for all of x (requires strict row diagonal dominance).
estimateEntry(A, b, i, { walks: 20000 }); // { estimate, standardError }
```

## Rules

- `damping` is the probability of following an edge. A restart probability α means `damping = 1 - α`.
- Check `converged` and `residual` before relying on `x`. The residual is recomputed from the returned vector.
- Jacobi, Neumann and Gauss-Seidel need diagonal dominance. Conjugate gradient needs a symmetric positive definite matrix. `method: 'auto'` picks between them.
- A full solution vector costs time linear in nnz per iteration. Only `forwardPush`, `backwardPush` and `estimateEntry` have a cost independent of n.
- The MCP `solveTrueSublinear` tool is approximate. Check its `verified_relative_residual`, or use `solve` when you need accuracy.
- The consciousness, psycho-symbolic, emergence and temporal-lead MCP tools are experimental simulations, not solvers.
