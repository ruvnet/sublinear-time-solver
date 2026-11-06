// Reproducible benchmark + accuracy harness for @ruvnet/sublinear-core.
//
//   node bench/bench.mjs                 # core only
//   LEGACY=/path/to/sublinear-time-solver node bench/bench.mjs
//       # also times the legacy SublinearSolver from that package directory
//   GRAPH_TRANSFORMER=/path/to/node_modules/@ruvector/graph-transformer node bench/bench.mjs
//       # adds @ruvector/graph-transformer pprScores (native) as a PPR baseline
//
// Inputs are generated from fixed seeds, so every run sees identical systems.
import { performance } from 'node:perf_hooks';
import { pathToFileURL } from 'node:url';
import { join } from 'node:path';
import {
  solve, csrFromCoo, estimateEntry, pageRank, personalizedPageRank, forwardPush,
  graphFromEdges, graphFromAdjacency, topK, mulberry32, relativeResidual,
} from '../dist/esm/index.js';

const rows = [];
const report = (group, name, ms, extra = '') => {
  rows.push({ group, name, ms: +ms.toFixed(2), extra });
  console.log(`${group.padEnd(22)} ${name.padEnd(46)} ${ms.toFixed(2).padStart(10)} ms  ${extra}`);
};
const time = (fn, reps = 3) => {
  let best = Infinity, out;
  for (let r = 0; r < reps; r++) { const t = performance.now(); out = fn(); best = Math.min(best, performance.now() - t); }
  return [best, out];
};

function ddCoo(n, deg, seed, margin = 1.5) {
  const rand = mulberry32(seed);
  const ri = [], ci = [], vs = [];
  const off = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    for (let t = 0; t < deg; t++) {
      const j = Math.floor(rand() * n);
      if (j === i) continue;
      const v = rand() * 2 - 1;
      ri.push(i); ci.push(j); vs.push(v); off[i] += Math.abs(v);
    }
  }
  for (let i = 0; i < n; i++) { ri.push(i); ci.push(i); vs.push(off[i] * margin + 1); }
  return { rows: n, cols: n, rowIndices: ri, colIndices: ci, values: vs };
}
const vec = (n, seed) => { const r = mulberry32(seed); return Array.from({ length: n }, () => r() * 2 - 1); };

function denseSolve(A, b) {
  const n = b.length;
  const M = A.map((row, i) => Float64Array.from([...row, b[i]]));
  for (let c = 0; c < n; c++) {
    let p = c;
    for (let r = c + 1; r < n; r++) if (Math.abs(M[r][c]) > Math.abs(M[p][c])) p = r;
    [M[c], M[p]] = [M[p], M[c]];
    const pc = M[c];
    for (let r = c + 1; r < n; r++) {
      const mr = M[r]; const f = mr[c] / pc[c];
      if (f !== 0) for (let k = c; k <= n; k++) mr[k] -= f * pc[k];
    }
  }
  const x = new Float64Array(n);
  for (let r = n - 1; r >= 0; r--) { let s = M[r][n]; for (let k = r + 1; k < n; k++) s -= M[r][k] * x[k]; x[r] = s / M[r][r]; }
  return x;
}
const toDense = (coo) => {
  const A = Array.from({ length: coo.rows }, () => new Array(coo.cols).fill(0));
  for (let k = 0; k < coo.values.length; k++) A[coo.rowIndices[k]][coo.colIndices[k]] += coo.values[k];
  return A;
};

let legacy = null;
if (process.env.LEGACY) {
  const mod = await import(pathToFileURL(join(process.env.LEGACY, 'dist/core/solver.js')).href);
  legacy = mod.SublinearSolver;
}

console.log(`node ${process.version}  ${process.platform}/${process.arch}`);
// BENCH_ONLY=ppr runs just the personalized PageRank head-to-head (section 5).
const only = process.env.BENCH_ONLY;

// 1. Accuracy against dense Gaussian elimination.
if (!only) {
  const n = 1000;
  const coo = ddCoo(n, 8, 1);
  const b = vec(n, 2);
  const [tDense, exact] = time(() => denseSolve(toDense(coo), b), 1);
  report('accuracy n=1000', 'dense Gaussian elimination (baseline)', tDense);
  for (const method of ['jacobi', 'gauss-seidel', 'neumann']) {
    const [t, r] = time(() => solve(coo, b, { method, tolerance: 1e-12, maxIterations: 10000 }));
    let err = 0; for (let i = 0; i < n; i++) err = Math.max(err, Math.abs(r.x[i] - exact[i]));
    report('accuracy n=1000', `core ${method}`, t, `iters=${r.iterations} relres=${r.residual.toExponential(2)} maxerr=${err.toExponential(2)}`);
  }
  if (legacy) {
    const s = new legacy({ method: 'neumann', epsilon: 1e-12, maxIterations: 10000 });
    const A = { ...coo, format: 'coo' };
    const [t, r] = await (async () => { const t0 = performance.now(); const out = await s.solve(A, b); return [performance.now() - t0, out]; })();
    let err = 0; for (let i = 0; i < n; i++) err = Math.max(err, Math.abs(r.solution[i] - exact[i]));
    const rr = relativeResidual(csrFromCoo(coo), r.solution, b);
    report('accuracy n=1000', 'legacy 1.7.2 SublinearSolver neumann', t, `iters=${r.iterations} relres=${rr.toExponential(2)} maxerr=${err.toExponential(2)}`);
  }
}

// 2. Scaling of the sparse solvers.
if (!only) for (const n of [10_000, 100_000, 1_000_000]) {
  const coo = ddCoo(n, 8, 3);
  const b = vec(n, 4);
  const [tb, A] = time(() => csrFromCoo(coo), 1);
  report(`solve n=${n}`, 'build CSR from COO', tb, `nnz=${A.rowPtr[n]}`);
  for (const method of ['gauss-seidel', 'jacobi']) {
    const [t, r] = time(() => solve(A, b, { method, tolerance: 1e-10, maxIterations: 1000 }), n >= 1_000_000 ? 1 : 3);
    report(`solve n=${n}`, `core ${method}`, t, `iters=${r.iterations} relres=${r.residual.toExponential(2)}`);
  }
  if (legacy && n <= 10_000) {
    const s = new legacy({ method: 'neumann', epsilon: 1e-10, maxIterations: 1000 });
    const t0 = performance.now();
    const r = await s.solve({ ...coo, format: 'coo' }, b);
    report(`solve n=${n}`, 'legacy 1.7.2 SublinearSolver neumann', performance.now() - t0, `iters=${r.iterations}`);
  }
}

// 3. Single-entry estimate: cost should not grow with n.
if (!only) for (const n of [1_000, 100_000, 1_000_000]) {
  const A = csrFromCoo(ddCoo(n, 8, 5, 2));
  const b = vec(n, 6);
  const [t, e] = time(() => estimateEntry(A, b, 0, { walks: 2000, seed: 7 }));
  report('estimateEntry', `n=${n} walks=2000`, t, `steps=${e.steps} est=${e.estimate.toFixed(5)}±${e.standardError.toFixed(5)}`);
}

// 4. PageRank.
function graphEdges(n, m, seed) {
  const rand = mulberry32(seed);
  const edges = new Array(m);
  for (let k = 0; k < m; k++) {
    // Preferential-ish target choice gives a skewed, web-like in-degree distribution.
    const u = Math.floor(rand() * n);
    const v = Math.floor(Math.pow(rand(), 2) * n);
    edges[k] = [u, v === u ? (v + 1) % n : v, 1];
  }
  return edges;
}
function naivePageRank(n, edges, seed, damping, iterations) {
  // The Map-of-arrays fallback pattern used in downstream projects.
  const out = new Map();
  for (const [s, t, w] of edges) { if (!out.has(s)) out.set(s, []); out.get(s).push([t, w]); }
  let scores = new Float64Array(n); scores[seed] = 1;
  for (let it = 0; it < iterations; it++) {
    const next = new Float64Array(n);
    for (let i = 0; i < n; i++) {
      const o = out.get(i) ?? [];
      if (o.length === 0) { next[seed] += damping * scores[i]; continue; }
      const tw = o.reduce((a, [, w]) => a + w, 0);
      for (const [t, w] of o) next[t] += scores[i] * (w / tw) * damping;
    }
    next[seed] += 1 - damping;
    scores = next;
  }
  return scores;
}
if (!only) for (const [n, m] of [[100_000, 1_000_000], [1_000_000, 10_000_000]]) {
  const edges = graphEdges(n, m, 11);
  const [tg, g] = time(() => graphFromEdges(edges, { n }), 1);
  report(`pagerank n=${n}`, `graphFromEdges m=${m}`, tg);
  const [tp, pr] = time(() => pageRank(g, { tolerance: 1e-9 }), 1);
  report(`pagerank n=${n}`, 'core pageRank (global)', tp, `iters=${pr.iterations} delta=${pr.delta.toExponential(2)}`);
  const [tppr, ppr] = time(() => personalizedPageRank(g, 0, { tolerance: 1e-9 }), 1);
  report(`pagerank n=${n}`, 'core personalizedPageRank (power)', tppr, `iters=${ppr.iterations}`);
  const iters = ppr.iterations;
  if (n <= 100_000) {
    const [tn, naive] = time(() => naivePageRank(n, edges, 0, 0.85, iters), 1);
    let diff = 0; for (let i = 0; i < n; i++) diff = Math.max(diff, Math.abs(naive[i] - ppr.scores[i]));
    report(`pagerank n=${n}`, `naive Map-of-arrays PPR (${iters} iters, baseline)`, tn, `maxdiff vs core=${diff.toExponential(2)}`);
  }
  for (const eps of [1e-4, 1e-6]) {
    const [tf, fp] = time(() => forwardPush(g, 0, { epsilon: eps }));
    const exactTop = topK(ppr.scores, 10).map((r) => r.index);
    const pushTop = new Set(topK(fp.scores, 10).map((r) => r.index));
    const overlap = exactTop.filter((i) => pushTop.has(i)).length;
    let l1 = 0; for (let i = 0; i < n; i++) l1 += Math.abs((fp.scores.get(i) ?? 0) - ppr.scores[i]);
    report(`pagerank n=${n}`, `core forwardPush eps=${eps}`, tf, `pushes=${fp.pushes} touched=${fp.scores.size} L1=${l1.toExponential(2)} top10=${overlap}/10`);
  }
}

// 5. Personalized PageRank head-to-head on unweighted graphs with no dangling
// nodes (so every implementation's dangling-node convention agrees).
// Reference: core power iteration to tolerance 1e-14. graph-transformer's
// pprScores runs a fixed 20 iterations (no convergence check), so its error
// is bounded by roughly damping^20 = 0.85^20 = 0.039 of the mass.
{
  let gt = null;
  if (process.env.GRAPH_TRANSFORMER) {
    const req = (await import('node:module')).createRequire(import.meta.url);
    const { GraphTransformer } = req(process.env.GRAPH_TRANSFORMER);
    gt = new GraphTransformer();
  }
  const noDangling = (n, extra, seed) => {
    const rand = mulberry32(seed);
    const adj = Array.from({ length: n }, () => []);
    for (let u = 0; u < n; u++) {
      const deg = 1 + Math.floor(rand() * extra);
      for (let t = 0; t < deg; t++) {
        let v = Math.floor(Math.pow(rand(), 2) * n);
        if (v === u) v = (v + 1) % n;
        if (!adj[u].includes(v)) adj[u].push(v);
      }
    }
    return adj;
  };
  const maxDiff = (a, b) => { let m = 0; for (let i = 0; i < a.length; i++) m = Math.max(m, Math.abs(a[i] - (b.get ? (b.get(i) ?? 0) : b[i]))); return m; };
  for (const n of [1_000, 100_000, 1_000_000]) {
    const adj = noDangling(n, 19, 21); // average out-degree ~10
    const m = adj.reduce((a, r) => a + r.length, 0);
    const [tb, g] = time(() => graphFromAdjacency(adj), 1);
    const exact = personalizedPageRank(g, 0, { tolerance: 1e-14, maxIterations: 1000 }).scores;
    const group = `ppr n=${n} m=${m}`;
    report(group, 'core graphFromAdjacency (one-time build)', tb);
    const [t1, r1] = time(() => personalizedPageRank(g, 0, { tolerance: 1e-9 }), n >= 1_000_000 ? 1 : 3);
    report(group, 'core personalizedPageRank tol=1e-9', t1, `iters=${r1.iterations} maxerr=${maxDiff(exact, r1.scores).toExponential(2)}`);
    const [t2, r2] = time(() => personalizedPageRank(g, 0, { tolerance: 1e-300, maxIterations: 20 }), n >= 1_000_000 ? 1 : 3);
    report(group, 'core personalizedPageRank 20 iters', t2, `maxerr=${maxDiff(exact, r2.scores).toExponential(2)}`);
    for (const eps of [1e-4, 1e-7]) {
      const [t3, r3] = time(() => forwardPush(g, 0, { epsilon: eps }), 3);
      report(group, `core forwardPush eps=${eps}`, t3, `pushes=${r3.pushes} maxerr=${maxDiff(exact, r3.scores).toExponential(2)}`);
    }
    if (gt) {
      const [t4, r4] = time(() => gt.pprScores(0, adj, 0.15), n >= 1_000_000 ? 1 : 3);
      report(group, 'graph-transformer pprScores (native, 20 iters)', t4, `maxerr=${maxDiff(exact, r4).toExponential(2)}`);
    }
    if (legacy && n <= 1_000) {
      const dense = Array.from({ length: n }, (_, i) => { const row = new Array(n).fill(0); for (const v of adj[i]) row[v] = 1; return row; });
      const s = new legacy({ method: 'neumann', epsilon: 1e-9, maxIterations: 1000 });
      const personalized = new Array(n).fill(0); personalized[0] = 1;
      const t0 = performance.now();
      const r5 = await s.computePageRank({ rows: n, cols: n, format: 'dense', data: dense }, { damping: 0.85, epsilon: 1e-9, maxIterations: 1000, personalized });
      const sum = r5.reduce((a, v) => a + v, 0);
      report(group, 'legacy 1.7.2 computePageRank (personalized)', performance.now() - t0, `maxerr=${maxDiff(exact, r5).toExponential(2)} sum=${sum.toFixed(3)}`);
    }
  }
}

if (process.env.BENCH_JSON) {
  const { writeFileSync } = await import('node:fs');
  writeFileSync(process.env.BENCH_JSON, JSON.stringify({ node: process.version, rows }, null, 2));
}
