// Security regression tests for 1.8.1. Run with: node --test tests/security-1.8.1.test.mjs
// (after `npm run build`). Every test drives the real MCP server over stdio.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, mkdtempSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

async function connect(t, env = {}) {
  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [join(root, 'dist/cli/index.js'), 'serve'],
    stderr: 'ignore',
    env: { ...process.env, ...env },
  });
  const client = new Client({ name: 'security-1.8.1', version: '1.0.0' });
  await client.connect(transport);
  t.after(() => client.close());
  return client;
}

const call = (client, name, args) =>
  client.callTool({ name, arguments: args }).then((r) => r, (e) => ({ thrown: e }));

async function expectRejected(client, name, args, pattern) {
  const t0 = Date.now();
  const r = await call(client, name, args);
  const ms = Date.now() - t0;
  const msg = r.thrown ? String(r.thrown.message) : r.isError ? JSON.stringify(r.content) : '';
  assert.ok(r.thrown || r.isError, `${name} should reject ${JSON.stringify(args).slice(0, 100)}`);
  if (pattern) assert.match(msg, pattern);
  assert.ok(ms < 2000, `${name} took ${ms} ms to reject`);
}

const coo3 = (rowIndices, colIndices) => ({ rows: 3, cols: 3, format: 'coo', values: [4, 4, 4], rowIndices, colIndices });

test('prototype-key indices are rejected and leave Object/Array prototypes untouched', async (t) => {
  // A preload in the server process records any new own property on
  // Object.prototype or Array.prototype (including numeric indices).
  const dir = mkdtempSync(join(tmpdir(), 'sublinear-proto-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const out = join(dir, 'pollution.log');
  const watcher = join(dir, 'watch.cjs');
  writeFileSync(watcher, `
    const fs = require('fs');
    const o0 = new Set(Object.getOwnPropertyNames(Object.prototype));
    const a0 = new Set(Object.getOwnPropertyNames(Array.prototype));
    setInterval(() => {
      const o = Object.getOwnPropertyNames(Object.prototype).filter((k) => !o0.has(k));
      const a = Object.getOwnPropertyNames(Array.prototype).filter((k) => !a0.has(k));
      if (o.length || a.length) fs.appendFileSync(${JSON.stringify(out)}, JSON.stringify({ o, a }) + '\\n');
    }, 20).unref();`);
  const client = await connect(t, { NODE_OPTIONS: `--require ${watcher}` });

  for (const key of ['__proto__', 'constructor', 'prototype', 'toString', 'length']) {
    await expectRejected(client, 'solve', { matrix: coo3([0, 1, key], [0, 1, 2]), vector: [1, 1, 1] });
    await expectRejected(client, 'solve', { matrix: coo3([0, 1, 2], [0, 1, key]), vector: [1, 1, 1] });
    await expectRejected(client, 'pageRank', { adjacency: coo3([key, 1, 2], [1, 2, 0]) });
    await expectRejected(client, 'coherenceScore', { matrix: { rows: 3, cols: 3, format: 'coo', data: { values: [1], rowIndices: [0], colIndices: [key] } } });
    await expectRejected(client, 'estimateComplexityClass', { method: key });
    await expectRejected(client, 'solve', { matrix: coo3([0, 1, 2], [0, 1, 2]), vector: [1, 1, 1], max_complexity_class: key });
  }
  // Prototype-named ids/keys in stores must not reach the prototype either.
  await call(client, 'scheduler_create', { id: '__proto__' });
  await call(client, 'add_knowledge', { subject: '__proto__', predicate: 'constructor', object: 'polluted' });

  const ok = await call(client, 'solve', { matrix: coo3([0, 1, 2], [0, 1, 2]), vector: [1, 1, 1] });
  assert.deepEqual(JSON.parse(ok.content[0].text).solution, [0.25, 0.25, 0.25]);
  await new Promise((r) => setTimeout(r, 100));
  assert.equal(existsSync(out) ? readFileSync(out, 'utf8') : '', '', 'prototype was modified');
});

test('declared-but-absent dense matrices are rejected without allocating', async (t) => {
  const client = await connect(t);
  const huge = { rows: 40000, cols: 40000, format: 'dense' };
  await expectRejected(client, 'solve', { matrix: huge, vector: [1] });
  await expectRejected(client, 'analyzeMatrix', { matrix: huge });
  await expectRejected(client, 'pageRank', { adjacency: huge });
  await expectRejected(client, 'predictWithTemporalAdvantage', { matrix: { rows: 30000, cols: 30000, data: [] }, vector: [1], distanceKm: 1 });
  await expectRejected(client, 'analyzeTrueSublinearMatrix', { matrix: { rows: 40000, cols: 40000, values: [1], rowIndices: [0], colIndices: [0] } });
});

test('schema numeric bounds are enforced (one request cannot pin or OOM the server)', async (t) => {
  const client = await connect(t);
  // Each of these crashed the server with an OOM or blocked it indefinitely in 1.8.0.
  await expectRejected(client, 'validateTemporalAdvantage', { size: 1e9 }, /size must be <= 5000/);
  await expectRejected(client, 'scheduler_benchmark', { numTasks: 1e10 }, /numTasks must be <= 100000/);
  await expectRejected(client, 'scheduler_consciousness', { iterations: 1e10 }, /iterations must be <= 10000/);
  await expectRejected(client, 'consciousness_evolve', { iterations: 1e12 }, /iterations must be <= 10000/);
  await expectRejected(client, 'emergence_generate_diverse', { input: 'x', count: 1e9 }, /count must be <= 10/);
  await expectRejected(client, 'scheduler_benchmark', { numTasks: '10' }, /finite number/);
  const s = { rows: 3, cols: 3, format: 'coo', data: { values: [4, 4, 4], rowIndices: [0, 1, 2], colIndices: [0, 1, 2] } };
  await expectRejected(client, 'solveOnChangeSublinear', { matrix: s, vector: [1, 1, 1], delta_indices: [0], max_terms: 1e12 }, /max_terms must be <= 10000/);
  // In-range values still work.
  const r = await call(client, 'validateTemporalAdvantage', { size: 100 });
  assert.ok(!r.thrown && !r.isError);
});

function ring(n, diag) {
  const values = [], rowIndices = [], colIndices = [];
  for (let i = 0; i < n; i++) {
    values.push(diag, 1, 1); rowIndices.push(i, i, i); colIndices.push(i, (i + 1) % n, (i + 7) % n);
  }
  return { rows: n, cols: n, format: 'coo', data: { values, rowIndices, colIndices } };
}

test('solveOnChangeSublinear: worst-case in-bounds request finishes and is accurate', async (t) => {
  const client = await connect(t);
  const n = 2000;
  const t0 = Date.now();
  const r = await call(client, 'solveOnChangeSublinear', {
    matrix: ring(n, 3), vector: new Array(n).fill(1), delta_indices: [...Array(n).keys()],
    max_terms: 10000, tolerance: 1e-300, closure_depth: 1000,
  });
  const ms = Date.now() - t0;
  assert.ok(!r.thrown && !r.isError, String(r.thrown?.message));
  // 1.8.0 recomputed the series once per closure row and did not finish in 60 s.
  assert.ok(ms < 5000, `took ${ms} ms`);
  const { entries } = JSON.parse(r.content[0].text);
  assert.equal(entries.length, n);
  // Row sums are all 5, so the exact solution of A x = 1 is x = 1/5.
  for (const e of entries) assert.ok(Math.abs(e.value - 0.2) < 1e-12, `row ${e.row}: ${e.value}`);
});

test('solveOnChangeSublinear: hard work limit stops oversized requests', async (t) => {
  // Diagonal barely above the off-diagonal sum: the series decays by ~0.99995
  // per term, so 10000 terms over 80000 non-zeros would be 8e8 multiply-adds.
  const client = await connect(t);
  const n = 20000;
  await expectRejected(client, 'solveOnChangeSublinear', {
    matrix: ring(n, 2.0001), vector: new Array(n).fill(1), delta_indices: [...Array(n).keys()],
    max_terms: 10000, tolerance: 1e-300, closure_depth: 0,
  }, /work limit exceeded/);
});

test('verifySparseSolution and the contrastive tool reject non-numeric prev_solution', async (t) => {
  const client = await connect(t);
  const s = { rows: 3, cols: 3, format: 'coo', data: { values: [4, 1, 4, 1, 4], rowIndices: [0, 0, 1, 1, 2], colIndices: [0, 1, 1, 2, 2] } };
  // 1.8.0 computed NaN residuals here and reported { ok: true, max_residual: 0 }.
  await expectRejected(client, 'verifySparseSolution', { matrix: s, prev_solution: ['a', 'a', 'a'], vector: [1, 1, 1], entries: [{ row: 0, value: 1 }] }, /prev_solution\[0\]/);
  await expectRejected(client, 'contrastiveSolveOnChangeSublinear', { matrix: s, prev_solution: ['__proto__', 1, 1], vector: [1, 1, 1], delta_indices: [0] }, /prev_solution\[0\]/);
  // A genuinely wrong solution still fails the audit.
  const bad = await call(client, 'verifySparseSolution', { matrix: s, prev_solution: [0, 0, 0], vector: [1, 1, 1], entries: [{ row: 0, value: 5 }] });
  assert.equal(JSON.parse(bad.content[0].text).ok, false);
});

test('the package does not claim the strange-loops bin of the separate strange-loops package', () => {
  const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
  assert.equal(pkg.bin['strange-loops'], undefined);
});
