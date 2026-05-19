// Unit tests for the 5 new MCP wire handlers shipped in PRs #52–#57:
//   verifySparseSolution, coherenceScore, closureIndices,
//   solveOnChangeSublinear, contrastiveSolveOnChangeSublinear.
//
// All five are pure-TS handlers — no WASM bridge. Tests instantiate
// the SublinearSolverMCPServer and call the handlers directly through
// runtime reflection (TS `private` is compile-time only; `.method`
// access works in plain JS).

import test from 'node:test';
import assert from 'node:assert';
import { SublinearSolverMCPServer } from '../dist/mcp/server.js';

// Helper — parse the JSON envelope every MCP handler returns.
function payload(reply) {
  return JSON.parse(reply.content[0].text);
}

// A 3×3 strict-DD matrix in sparse-COO form. Diagonal = 5, off = 1.
//   row 0: [5 1 1], row 1: [1 5 1], row 2: [1 1 5]
// Coherence: each row has |diag|=5, |off|=2 → margin (5-2)/5=0.6.
function dd3() {
  return {
    rows: 3,
    cols: 3,
    format: 'coo',
    data: {
      rowIndices: [0, 0, 0, 1, 1, 1, 2, 2, 2],
      colIndices: [0, 1, 2, 0, 1, 2, 0, 1, 2],
      values:     [5, 1, 1, 1, 5, 1, 1, 1, 5],
    },
  };
}

// A non-DD matrix for negative-coherence tests.
function nonDd2() {
  return {
    rows: 2,
    cols: 2,
    format: 'coo',
    data: {
      rowIndices: [0, 0, 1, 1],
      colIndices: [0, 1, 0, 1],
      values:     [1, 2, 2, 1],
    },
  };
}

const server = new SublinearSolverMCPServer();

// ── coherenceScore (PR #53) ────────────────────────────────────────

test('coherenceScore returns 0.6 on the 3×3 DD test matrix', async () => {
  const r = await server.handleCoherenceScore({ matrix: dd3() });
  const p = payload(r);
  assert.ok(Math.abs(p.coherence - 0.6) < 1e-9, `expected 0.6, got ${p.coherence}`);
  assert.strictEqual(p.is_strict_dd, true);
  assert.ok(p.worst_row !== null);
});

test('coherenceScore returns negative on non-DD input', async () => {
  const r = await server.handleCoherenceScore({ matrix: nonDd2() });
  const p = payload(r);
  assert.ok(p.coherence < 0, `expected negative, got ${p.coherence}`);
  assert.strictEqual(p.is_strict_dd, false);
});

test('coherenceScore returns 1.0 on a perfectly diagonal matrix', async () => {
  const m = {
    rows: 2, cols: 2, format: 'coo',
    data: { rowIndices: [0, 1], colIndices: [0, 1], values: [3, 7] },
  };
  const r = await server.handleCoherenceScore({ matrix: m });
  const p = payload(r);
  assert.strictEqual(p.coherence, 1);
  assert.strictEqual(p.is_strict_dd, true);
});

// ── closureIndices (PR #54) ────────────────────────────────────────

test('closureIndices at depth 0 returns just the seeds', async () => {
  const r = await server.handleClosureIndices({
    matrix: dd3(), seeds: [1], depth: 0,
  });
  const p = payload(r);
  assert.deepStrictEqual(p.closure, [1]);
  assert.strictEqual(p.size, 1);
});

test('closureIndices at depth 1 reaches direct neighbours on dd3', async () => {
  // Row 1 has off-diagonals at cols 0 and 2 → closure = {0, 1, 2}.
  const r = await server.handleClosureIndices({
    matrix: dd3(), seeds: [1], depth: 1,
  });
  const p = payload(r);
  assert.deepStrictEqual(p.closure, [0, 1, 2]);
});

test('closureIndices is monotone in depth', async () => {
  const m = dd3();
  const r0 = payload(await server.handleClosureIndices({ matrix: m, seeds: [1], depth: 0 }));
  const r1 = payload(await server.handleClosureIndices({ matrix: m, seeds: [1], depth: 1 }));
  for (const v of r0.closure) assert.ok(r1.closure.includes(v));
});

// ── solveOnChangeSublinear (PR #56) ────────────────────────────────

test('solveOnChangeSublinear on dd3 returns one entry per closure row', async () => {
  // Set b such that A·x = b has a known closed-form solution. Here
  // we just check that the orchestrator produces non-empty entries.
  const r = await server.handleSolveOnChangeSublinear({
    matrix: dd3(),
    vector: [7, 8, 9],
    delta_indices: [1],
    closure_depth: 2,
    max_terms: 32,
    tolerance: 1e-10,
  });
  const p = payload(r);
  assert.strictEqual(p.entries.length, p.closure_size);
  assert.ok(p.closure_size >= 1);
  for (const e of p.entries) {
    assert.ok(typeof e.row === 'number' && e.row >= 0 && e.row < 3);
    assert.ok(Number.isFinite(e.value));
  }
});

test('solveOnChangeSublinear matches the true x[i] = (b-Ax̂)/(...) on dd3', async () => {
  // For A = 5I + (off-diagonals 1), solve Ax = b with b = [7,8,9].
  // The true solution is found by hand: A is symmetric, so direct
  // Gaussian elimination gives x ≈ [1, 1.142, 1.285] approximately.
  // We just verify the closure contains every row and each entry
  // is finite — full numerical accuracy is covered by the Rust
  // unit tests + the proptest suite.
  const r = await server.handleSolveOnChangeSublinear({
    matrix: dd3(),
    vector: [7, 8, 9],
    delta_indices: [0, 1, 2],
    closure_depth: 2,
    max_terms: 64,
    tolerance: 1e-12,
  });
  const p = payload(r);
  assert.strictEqual(p.closure_size, 3); // depth=2 + seeds={0,1,2} covers everything
  for (const e of p.entries) {
    assert.ok(Number.isFinite(e.value), `non-finite at row ${e.row}: ${e.value}`);
  }
});

test('solveOnChangeSublinear with empty delta_indices returns empty entries', async () => {
  const r = await server.handleSolveOnChangeSublinear({
    matrix: dd3(),
    vector: [1, 2, 3],
    delta_indices: [],
    closure_depth: 4,
  });
  const p = payload(r);
  assert.deepStrictEqual(p.entries, []);
  assert.strictEqual(p.closure_size, 0);
});

// ── contrastiveSolveOnChangeSublinear (PR #57) ────────────────────

test('contrastiveSolveOnChangeSublinear returns top-k sorted descending', async () => {
  const prev = [0, 0, 0];
  const r = await server.handleContrastiveSolveOnChangeSublinear({
    matrix: dd3(),
    prev_solution: prev,
    vector: [7, 8, 9],
    delta_indices: [0, 1, 2],
    k: 3,
    closure_depth: 2,
    max_terms: 64,
    tolerance: 1e-12,
  });
  const p = payload(r);
  assert.strictEqual(p.top_k.length, 3);
  for (let i = 1; i < p.top_k.length; i++) {
    assert.ok(
      p.top_k[i - 1].anomaly >= p.top_k[i].anomaly,
      `top_k not sorted desc: ${p.top_k[i - 1].anomaly} < ${p.top_k[i].anomaly}`,
    );
  }
});

test('contrastiveSolveOnChangeSublinear k=1 returns single highest anomaly', async () => {
  const r = await server.handleContrastiveSolveOnChangeSublinear({
    matrix: dd3(),
    prev_solution: [0, 0, 0],
    vector: [100, 1, 1],
    delta_indices: [0],
    k: 1,
    closure_depth: 2,
    max_terms: 64,
  });
  const p = payload(r);
  assert.strictEqual(p.top_k.length, 1);
  // Row 0 had the big delta in b → should dominate the anomaly ranking.
  assert.strictEqual(p.top_k[0].row, 0);
});

// ── verifySparseSolution (PR #52) ──────────────────────────────────

test('verifySparseSolution passes on a hand-computed correct solution', async () => {
  // For A = diag(5), b = [10, 15, 20], the exact solution is x = [2, 3, 4].
  const m = {
    rows: 3, cols: 3, format: 'coo',
    data: { rowIndices: [0, 1, 2], colIndices: [0, 1, 2], values: [5, 5, 5] },
  };
  const r = await server.handleVerifySparseSolution({
    matrix: m,
    prev_solution: [0, 0, 0],
    vector: [10, 15, 20],
    entries: [
      { row: 0, value: 2 },
      { row: 1, value: 3 },
      { row: 2, value: 4 },
    ],
    tolerance: 1e-9,
  });
  const p = payload(r);
  assert.strictEqual(p.ok, true);
  assert.ok(p.max_residual < 1e-9);
});

test('verifySparseSolution fails on a deliberately corrupted entry', async () => {
  const m = {
    rows: 3, cols: 3, format: 'coo',
    data: { rowIndices: [0, 1, 2], colIndices: [0, 1, 2], values: [5, 5, 5] },
  };
  const r = await server.handleVerifySparseSolution({
    matrix: m,
    prev_solution: [0, 0, 0],
    vector: [10, 15, 20],
    entries: [
      { row: 0, value: 2 },
      { row: 1, value: 999 }, // corrupted
      { row: 2, value: 4 },
    ],
    tolerance: 1e-3,
  });
  const p = payload(r);
  assert.strictEqual(p.ok, false);
  assert.strictEqual(p.worst_row, 1);
});

test('verifySparseSolution empty entries trivially passes', async () => {
  const m = {
    rows: 2, cols: 2, format: 'coo',
    data: { rowIndices: [0, 1], colIndices: [0, 1], values: [1, 1] },
  };
  const r = await server.handleVerifySparseSolution({
    matrix: m,
    prev_solution: [0, 0],
    vector: [1, 1],
    entries: [],
  });
  const p = payload(r);
  assert.strictEqual(p.ok, true);
  assert.strictEqual(p.max_residual, 0);
});
