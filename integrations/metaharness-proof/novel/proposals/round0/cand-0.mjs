/**
 * Preconditioned BiCGSTAB solver.
 *
 * BiCGSTAB is a Krylov-subspace method that converges for general (possibly
 * non-symmetric) matrices, and in particular for SPD matrices whether or not
 * they are diagonally dominant -- unlike plain Jacobi, which only converges
 * under diagonal dominance. A Jacobi (diagonal) preconditioner speeds
 * convergence on both system classes. The best iterate (lowest residual) is
 * tracked throughout so a stray divergent step near the end can't discard a
 * good earlier solution.
 */
export function solve(matrix, b) {
  const { rows, values, rowIndices, colIndices } = matrix;
  const n = rows;
  const nnz = values.length;

  const rowStart = new Array(n + 1).fill(0);
  for (let k = 0; k < nnz; k++) rowStart[rowIndices[k] + 1]++;
  for (let i = 0; i < n; i++) rowStart[i + 1] += rowStart[i];
  const colIdx = new Array(nnz);
  const vals = new Array(nnz);
  const cursor = rowStart.slice();
  for (let k = 0; k < nnz; k++) {
    const r = rowIndices[k];
    const pos = cursor[r]++;
    colIdx[pos] = colIndices[k];
    vals[pos] = values[k];
  }

  const diag = new Array(n).fill(1);
  for (let i = 0; i < n; i++) {
    for (let p = rowStart[i]; p < rowStart[i + 1]; p++) {
      if (colIdx[p] === i) diag[i] = vals[p];
    }
  }
  for (let i = 0; i < n; i++) if (Math.abs(diag[i]) < 1e-300) diag[i] = 1e-12;

  function matvec(x) {
    const y = new Array(n).fill(0);
    for (let i = 0; i < n; i++) {
      let s = 0;
      for (let p = rowStart[i]; p < rowStart[i + 1]; p++) s += vals[p] * x[colIdx[p]];
      y[i] = s;
    }
    return y;
  }
  function dot(a, c) {
    let s = 0;
    for (let i = 0; i < n; i++) s += a[i] * c[i];
    return s;
  }
  function norm(v) { return Math.sqrt(dot(v, v)); }
  function precond(r) {
    const z = new Array(n);
    for (let i = 0; i < n; i++) z[i] = r[i] / diag[i];
    return z;
  }
  function residual(x) {
    const ax = matvec(x);
    const r = new Array(n);
    for (let i = 0; i < n; i++) r[i] = b[i] - ax[i];
    return r;
  }

  const bnorm = norm(b) || 1;
  let x = new Array(n).fill(0);
  let r = residual(x);
  let bestX = x.slice();
  let bestNorm = norm(r);

  const rHat0 = r.slice();
  let rho = 1, alpha = 1, omega = 1;
  let v = new Array(n).fill(0);
  let p = new Array(n).fill(0);

  const maxIter = Math.min(2000, Math.max(300, n * 3));

  for (let it = 0; it < maxIter; it++) {
    const rhoNew = dot(rHat0, r);
    if (Math.abs(rhoNew) < 1e-300 || Math.abs(omega) < 1e-300) {
      // breakdown: restart with current residual as new shadow vector
      r = residual(bestX);
      x = bestX.slice();
      for (let i = 0; i < n; i++) { p[i] = 0; v[i] = 0; }
      rHat0.splice(0, n, ...r);
      rho = 1; alpha = 1; omega = 1;
      continue;
    }
    const beta = (rhoNew / rho) * (alpha / omega);
    for (let i = 0; i < n; i++) p[i] = r[i] + beta * (p[i] - omega * v[i]);
    const pHat = precond(p);
    v = matvec(pHat);
    const denom = dot(rHat0, v);
    if (Math.abs(denom) < 1e-300) break;
    alpha = rhoNew / denom;
    const s = new Array(n);
    for (let i = 0; i < n; i++) s[i] = r[i] - alpha * v[i];
    const sNorm = norm(s);
    if (sNorm / bnorm < 1e-9) {
      for (let i = 0; i < n; i++) x[i] += alpha * pHat[i];
      if (sNorm < bestNorm) { bestNorm = sNorm; bestX = x.slice(); }
      break;
    }
    const sHat = precond(s);
    const t = matvec(sHat);
    const tDot = dot(t, t);
    omega = tDot < 1e-300 ? 0 : dot(t, s) / tDot;
    for (let i = 0; i < n; i++) x[i] += alpha * pHat[i] + omega * sHat[i];
    for (let i = 0; i < n; i++) r[i] = s[i] - omega * t[i];
    rho = rhoNew;

    const rn = norm(r);
    if (isFinite(rn) && rn < bestNorm) { bestNorm = rn; bestX = x.slice(); }
    if (rn / bnorm < 1e-9) break;
    if (!isFinite(rn)) break;
  }

  return bestX;
}