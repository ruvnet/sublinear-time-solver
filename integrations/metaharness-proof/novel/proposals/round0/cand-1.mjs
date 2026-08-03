/**
 * Preconditioned BiCGSTAB with a Gauss-Seidel/SOR fallback refinement.
 *
 * BiCGSTAB (Bi-Conjugate Gradient Stabilized) converges on general sparse
 * linear systems including symmetric positive-definite matrices that are
 * NOT diagonally dominant (where plain Jacobi diverges), as well as on
 * diagonally-dominant systems. A Jacobi (diagonal) preconditioner speeds
 * convergence. If BiCGSTAB stalls or breaks down numerically, Gauss-Seidel
 * sweeps refine the best solution found so far.
 */
export function solve(matrix, b) {
  const { rows, values, rowIndices, colIndices } = matrix;
  const n = rows;
  const nnz = values.length;

  const diag = new Array(n).fill(0);
  for (let k = 0; k < nnz; k++) if (rowIndices[k] === colIndices[k]) diag[rowIndices[k]] = values[k];
  const diagInv = new Array(n);
  for (let i = 0; i < n; i++) diagInv[i] = Math.abs(diag[i]) > 1e-300 ? 1 / diag[i] : 1;

  function matvec(x) {
    const y = new Array(n).fill(0);
    for (let k = 0; k < nnz; k++) y[rowIndices[k]] += values[k] * x[colIndices[k]];
    return y;
  }
  function dot(a, c) {
    let s = 0;
    for (let i = 0; i < n; i++) s += a[i] * c[i];
    return s;
  }
  function norm(a) { return Math.sqrt(dot(a, a)); }
  function precond(r) {
    const z = new Array(n);
    for (let i = 0; i < n; i++) z[i] = r[i] * diagInv[i];
    return z;
  }

  let x = new Array(n).fill(0);
  let r = b.slice();
  const bnorm = norm(b) || 1;

  const rhat = r.slice();
  let rho = 1, alpha = 1, omega = 1;
  let v = new Array(n).fill(0);
  let p = new Array(n).fill(0);

  const maxIter = Math.min(2000, 20 * n + 100);
  let bestX = x.slice();
  let bestRes = norm(r);

  for (let iter = 0; iter < maxIter; iter++) {
    const rhoNew = dot(rhat, r);
    if (Math.abs(rhoNew) < 1e-300) break;
    const beta = (rhoNew / rho) * (alpha / omega);
    for (let i = 0; i < n; i++) p[i] = r[i] + beta * (p[i] - omega * v[i]);
    const phat = precond(p);
    v = matvec(phat);
    const denom = dot(rhat, v);
    if (Math.abs(denom) < 1e-300) break;
    alpha = rhoNew / denom;
    const s = new Array(n);
    for (let i = 0; i < n; i++) s[i] = r[i] - alpha * v[i];
    const sNorm = norm(s);
    if (sNorm < 1e-10 * bnorm) {
      const xt = x.slice();
      for (let i = 0; i < n; i++) xt[i] += alpha * phat[i];
      if (sNorm < bestRes) { bestRes = sNorm; bestX = xt; }
      x = xt; r = s;
      break;
    }
    const shat = precond(s);
    const t = matvec(shat);
    const tt = dot(t, t);
    omega = tt > 1e-300 ? dot(t, s) / tt : 0;
    for (let i = 0; i < n; i++) x[i] += alpha * phat[i] + omega * shat[i];
    const rnew = new Array(n);
    for (let i = 0; i < n; i++) rnew[i] = s[i] - omega * t[i];
    r = rnew;
    rho = rhoNew;

    const rn = norm(r);
    if (rn < bestRes) { bestRes = rn; bestX = x.slice(); }
    if (rn < 1e-10 * bnorm) break;
    if (Math.abs(omega) < 1e-300) break;
  }

  let xr = bestX;
  let curRes = norm(matvec(xr).map((val, i) => b[i] - val));

  if (curRes > 1e-6 * bnorm) {
    const rowsList = new Array(n);
    for (let i = 0; i < n; i++) rowsList[i] = [];
    for (let k = 0; k < nnz; k++) rowsList[rowIndices[k]].push(k);

    let xg = xr.slice();
    for (let sweep = 0; sweep < 300; sweep++) {
      for (let i = 0; i < n; i++) {
        let sum = b[i];
        const idxs = rowsList[i];
        for (let t = 0; t < idxs.length; t++) {
          const k = idxs[t];
          const j = colIndices[k];
          if (j !== i) sum -= values[k] * xg[j];
        }
        if (Math.abs(diag[i]) > 1e-300) xg[i] = sum / diag[i];
      }
      const rg = matvec(xg).map((val, i) => b[i] - val);
      const rgn = norm(rg);
      if (rgn < curRes) { curRes = rgn; xr = xg.slice(); }
      if (rgn < 1e-8 * bnorm) { xr = xg.slice(); curRes = rgn; break; }
    }
  }

  return xr;
}