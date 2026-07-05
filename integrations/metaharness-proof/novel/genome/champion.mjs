/**
 * Champion solver genome (generation 0 of the novel-discovery engine).
 *
 * A deliberately-modest Jacobi iteration with a fixed, small iteration budget —
 * it converges on strongly-dominant systems but not weakly-dominant ones, so
 * there is real room for an open-ended proposer to improve it (acceleration,
 * Gauss–Seidel sweeps, adaptive stopping, preconditioning, …).
 *
 * Contract (all genomes must honour it): a pure ESM module exporting
 *   solve(matrix, b) -> number[]
 * where matrix is COO {rows, cols, values, rowIndices, colIndices} and the
 * return is the solution vector. No imports, no I/O.
 */
export function solve(matrix, b) {
  const { rows, values, rowIndices, colIndices } = matrix;
  const n = rows;
  const diag = new Array(n).fill(0);
  for (let k = 0; k < values.length; k++) if (rowIndices[k] === colIndices[k]) diag[rowIndices[k]] = values[k];

  let x = new Array(n).fill(0);
  for (let it = 0; it < 20; it++) {
    const sum = b.slice();
    for (let k = 0; k < values.length; k++) {
      const i = rowIndices[k], j = colIndices[k];
      if (i !== j) sum[i] -= values[k] * x[j];
    }
    const xn = new Array(n);
    for (let i = 0; i < n; i++) xn[i] = sum[i] / diag[i];
    x = xn;
  }
  return x;
}
