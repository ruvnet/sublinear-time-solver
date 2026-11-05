import { mulberry32 } from '../dist/esm/index.js';

/** Dense Gaussian elimination with partial pivoting: the accuracy reference. */
export function denseSolve(A, b) {
  const n = b.length;
  const M = A.map((row, i) => [...row, b[i]]);
  for (let c = 0; c < n; c++) {
    let p = c;
    for (let r = c + 1; r < n; r++) if (Math.abs(M[r][c]) > Math.abs(M[p][c])) p = r;
    [M[c], M[p]] = [M[p], M[c]];
    for (let r = c + 1; r < n; r++) {
      const f = M[r][c] / M[c][c];
      if (f === 0) continue;
      for (let k = c; k <= n; k++) M[r][k] -= f * M[c][k];
    }
  }
  const x = new Array(n).fill(0);
  for (let r = n - 1; r >= 0; r--) {
    let s = M[r][n];
    for (let k = r + 1; k < n; k++) s -= M[r][k] * x[k];
    x[r] = s / M[r][r];
  }
  return x;
}

/** Random strictly diagonally dominant matrix (dense rows) with ~deg off-diagonals per row. */
export function randomDD(n, deg, seed, { symmetric = false, margin = 1.5 } = {}) {
  const rand = mulberry32(seed);
  const A = Array.from({ length: n }, () => new Array(n).fill(0));
  for (let i = 0; i < n; i++) {
    for (let t = 0; t < deg; t++) {
      const j = Math.floor(rand() * n);
      if (j === i) continue;
      const v = rand() * 2 - 1;
      A[i][j] += v;
      if (symmetric) A[j][i] += v;
    }
  }
  for (let i = 0; i < n; i++) {
    let off = 0;
    for (let j = 0; j < n; j++) if (j !== i) off += Math.abs(A[i][j]);
    A[i][i] = off * margin + 1;
  }
  return A;
}

export function randomVector(n, seed) {
  const rand = mulberry32(seed);
  return Array.from({ length: n }, () => rand() * 2 - 1);
}

export function randomGraphEdges(n, m, seed) {
  const rand = mulberry32(seed);
  const edges = [];
  for (let k = 0; k < m; k++) {
    const u = Math.floor(rand() * n);
    const v = Math.floor(rand() * n);
    if (u !== v) edges.push([u, v, 1 + Math.floor(rand() * 3)]);
  }
  return edges;
}

export function maxAbsDiff(a, b) {
  let m = 0;
  for (let i = 0; i < a.length; i++) m = Math.max(m, Math.abs(a[i] - b[i]));
  return m;
}

/** Exact PPR by dense solve of (I - d P'^T) x = (1 - d) v, P' with dangling policy applied. */
export function densePageRank(n, edges, v, d, dangling = 'teleport') {
  const outW = new Array(n).fill(0);
  for (const [u, , w = 1] of edges) outW[u] += w;
  const PT = Array.from({ length: n }, () => new Array(n).fill(0));
  for (const [u, t, w = 1] of edges) PT[t][u] += w / outW[u];
  if (dangling === 'teleport') {
    for (let u = 0; u < n; u++) if (outW[u] === 0) for (let t = 0; t < n; t++) PT[t][u] += v[t];
  }
  const A = PT.map((row, i) => row.map((x, j) => (i === j ? 1 : 0) - d * x));
  return denseSolve(A, v.map((x) => (1 - d) * x));
}
