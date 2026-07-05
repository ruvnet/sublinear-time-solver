/**
 * Improved solver genome.
 *
 * Jacobi-preconditioned BiCGStab. Unlike plain Jacobi/Gauss-Seidel, BiCGStab
 * is a Krylov-subspace method that converges on general (possibly
 * non-diagonally-dominant) SPD and non-symmetric systems, not just
 * diagonally-dominant ones. Falls back gracefully via residual checks so it
 * never does worse than returning the best iterate found.
 *
 * Contract: pure ESM module exporting solve(matrix, b) -> number[]
 * where matrix is COO {rows, cols, values, rowIndices, colIndices}.
 * No imports, no I/O.
 */
export function solve(matrix, b) {
  const { rows, values, rowIndices, colIndices } = matrix;
  const n = rows;
  const nnz = values.length;

  const diag = new Array(n).fill(0);
  for (let k = 0; k < nnz; k++) {
    if (rowIndices[k] === colIndices[k]) diag[rowIndices[k]] += values[k];
  }
  for (let i = 0; i < n; i++) if (diag[i] === 0) diag[i] = 1;

  function matVec(x) {
    const y = new Array(n).fill(0);
    for (let k = 0; k < nnz; k++) {
      y[rowIndices[k]] += values[k] * x[colIndices[k]];
    }
    return y;
  }
  function dot(u, v) {
    let s = 0;
    for (let i = 0; i < n; i++) s += u[i] * v[i];
    return s;
  }
  function precond(r) {
    const z = new Array(n);
    for (let i = 0; i < n; i++) z[i] = r[i] / diag[i];
    return z;
  }

  let x = new Array(n).fill(0);
  let r = b.slice();
  const rHat = r.slice();
  const bNorm = Math.sqrt(dot(b, b)) || 1;

  let rho = 1, alpha = 1, omega = 1;
  let p = new Array(n).fill(0);
  let v = new Array(n).fill(0);

  let bestX = x.slice();
  let bestNorm = Math.sqrt(dot(r, r)) / bNorm;

  const maxIter = Math.min(2000, n * 20 + 100);
  for (let iter = 0; iter < maxIter; iter++) {
    const rhoNew = dot(rHat, r);
    if (Math.abs(rhoNew) < 1e-300) break;

    if (iter === 0) {
      p = r.slice();
    } else {
      const beta = (rhoNew / rho) * (alpha / omega);
      for (let i = 0; i < n; i++) p[i] = r[i] + beta * (p[i] - omega * v[i]);
    }

    const pHat = precond(p);
    v = matVec(pHat);
    const denom = dot(rHat, v);
    if (Math.abs(denom) < 1e-300) break;
    alpha = rhoNew / denom;

    const s = new Array(n);
    for (let i = 0; i < n; i++) s[i] = r[i] - alpha * v[i];
    const sNorm = Math.sqrt(dot(s, s)) / bNorm;

    if (sNorm < 1e-10) {
      for (let i = 0; i < n; i++) x[i] += alpha * pHat[i];
      bestX = x.slice();
      bestNorm = sNorm;
      break;
    }

    const sHat = precond(s);
    const t = matVec(sHat);
    const tt = dot(t, t);
    omega = tt < 1e-300 ? 0 : dot(t, s) / tt;

    for (let i = 0; i < n; i++) x[i] += alpha * pHat[i] + omega * sHat[i];
    for (let i = 0; i < n; i++) r[i] = s[i] - omega * t[i];

    const rNorm = Math.sqrt(dot(r, r)) / bNorm;
    if (rNorm < bestNorm) {
      bestNorm = rNorm;
      bestX = x.slice();
    }
    if (rNorm < 1e-10) break;
    if (Math.abs(omega) < 1e-300) break;

    rho = rhoNew;
  }

  return bestNorm <= Math.sqrt(dot(matVecResidual(bestX), matVecResidual(bestX))) / bNorm
    ? bestX
    : bestX;

  function matVecResidual(xv) {
    const Ax = matVec(xv);
    const res = new Array(n);
    for (let i = 0; i < n; i++) res[i] = b[i] - Ax[i];
    return res;
  }
}