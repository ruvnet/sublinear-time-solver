// Regression tests for the 1.8.1 complexity-label and solveTrueSublinear fixes.
// Run with: node --test tests/regression-1.8.1.test.mjs (after `npm run build`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { spawn } from 'node:child_process';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function tridiagCoo(n, diag) {
  const rowIndices = [], colIndices = [], values = [];
  for (let i = 0; i < n; i++) {
    rowIndices.push(i); colIndices.push(i); values.push(diag);
    if (i > 0) { rowIndices.push(i); colIndices.push(i - 1); values.push(-1); }
    if (i < n - 1) { rowIndices.push(i); colIndices.push(i + 1); values.push(-1); }
  }
  return { rows: n, cols: n, rowIndices, colIndices, values };
}

async function rawToolsList(t) {
  const p = spawn(process.execPath, [join(root, 'dist/cli/index.js'), 'serve'], { stdio: ['pipe', 'pipe', 'ignore'] });
  t.after(() => p.kill());
  let buf = '';
  const pending = new Map();
  p.stdout.on('data', (d) => {
    buf += d;
    let i;
    while ((i = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, i);
      buf = buf.slice(i + 1);
      try { const m = JSON.parse(line); pending.get(m.id)?.(m); } catch { /* not JSON-RPC */ }
    }
  });
  const call = (id, method, params) => new Promise((r) => { pending.set(id, r); p.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n'); });
  await call(1, 'initialize', { protocolVersion: '2024-11-05', capabilities: {}, clientInfo: { name: 'raw', version: '0' } });
  const list = await call(2, 'tools/list', {});
  p.kill();
  return list.result.tools;
}

async function connect(t) {
  const transport = new StdioClientTransport({ command: process.execPath, args: [join(root, 'dist/cli/index.js'), 'serve'], stderr: 'ignore' });
  const client = new Client({ name: 'regression-1.8.1', version: '1.0.0' });
  await client.connect(transport);
  t.after(() => client.close());
  return client;
}

test('no tool or method that returns a full solution vector claims sublinear cost', async (t) => {
  const client = await connect(t);
  // Raw JSON-RPC: the SDK Client strips non-standard tool keys such as x-complexity.
  const tools = await rawToolsList(t);
  const byName = Object.fromEntries(tools.map((x) => [x.name, x]));
  const cheap = new Set(['Logarithmic', 'PolyLogarithmic', 'SubLinear']);
  for (const name of ['solve', 'solveTrueSublinear']) {
    const c = byName[name]['x-complexity'];
    for (const k of ['class', 'default', 'worst']) {
      if (c[k] !== undefined) assert.ok(!cheap.has(c[k]), `${name} x-complexity.${k} = ${c[k]}`);
    }
  }
  // These method names run a full solve in this package.
  for (const method of ['forward-push', 'backward-push', 'bidirectional', 'sublinear-neumann', 'neumann']) {
    const r = await client.callTool({ name: 'estimateComplexityClass', arguments: { method } });
    const out = JSON.parse(r.content[0].text);
    const cls = out.class ?? out.complexity?.class ?? out.worst_case_class;
    assert.ok(cls && !cheap.has(cls), `${method}: ${JSON.stringify(out).slice(0, 160)}`);
  }
  // The budget gate follows: a SubLinear budget must reject a full solve.
  const s = tridiagCoo(10, 4);
  const res = await client.callTool({ name: 'solve', arguments: { matrix: { ...s, format: 'coo' }, vector: new Array(10).fill(1), method: 'forward-push', max_complexity_class: 'SubLinear' } }).then((x) => x, (e) => ({ thrown: e }));
  assert.ok(res.thrown || res.isError, 'SubLinear budget must reject forward-push (a full Gauss-Seidel solve)');
});

test('solveTrueSublinear rejects non-diagonally-dominant input instead of returning garbage', async (t) => {
  const client = await connect(t);
  // Symmetric, not DD (diag 1 < off-diagonal sum 2).
  const m = { rows: 3, cols: 3, rowIndices: [0, 0, 1, 1, 1, 2, 2], colIndices: [0, 1, 0, 1, 2, 1, 2], values: [1, 2, 2, 1, 2, 2, 1] };
  const r = await client.callTool({ name: 'solveTrueSublinear', arguments: { matrix: m, vector: [1, 1, 1] } }).then((x) => x, (e) => ({ thrown: e }));
  assert.ok(r.thrown || r.isError, 'non-DD input must be rejected');
});

test('solveTrueSublinear labels itself honestly and reports a residual even when truncated', async (t) => {
  const client = await connect(t);
  for (const n of [60, 1200]) {
    const m = tridiagCoo(n, 4);
    const b = Array.from({ length: n }, (_, i) => Math.sin(i));
    const r = await client.callTool({ name: 'solveTrueSublinear', arguments: { matrix: m, vector: b } });
    const out = JSON.parse(r.content[0].text);
    assert.notEqual(out.complexity_bound.type, 'logarithmic', `n=${n}`);
    assert.doesNotMatch(out.actual_complexity, /log/i, `n=${n}`);
    assert.equal(typeof out.verified_relative_residual, 'number', `n=${n}: verified_relative_residual missing`);
    assert.ok(Number.isFinite(out.verified_relative_residual));
  }
});
