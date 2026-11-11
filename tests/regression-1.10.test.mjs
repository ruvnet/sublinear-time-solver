// Regression tests for 1.10.0: MCP error codes, solveTrueSublinear on the
// core sparse solver, and the dropped strange_loop wasm.
// Run with: node --test tests/regression-1.10.test.mjs (after `npm run build`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { ErrorCode, McpError } from '@modelcontextprotocol/sdk/types.js';

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

async function connect(t) {
  const transport = new StdioClientTransport({ command: process.execPath, args: [join(root, 'dist/cli/index.js'), 'serve'], stderr: 'pipe' });
  let stderr = '';
  transport.stderr?.on('data', (d) => { stderr += d; });
  const client = new Client({ name: 'regression-1.10', version: '1.0.0' });
  await client.connect(transport);
  t.after(() => client.close());
  return { client, stderr: () => stderr };
}

const rejection = (p) => p.then(() => assert.fail('expected the call to be rejected'), (e) => e);

test('InvalidParams reaches the client as -32602, not -32603', async (t) => {
  const { client } = await connect(t);
  const m = tridiagCoo(3, 4);
  const e = await rejection(client.callTool({ name: 'solve', arguments: { matrix: { ...m, format: 'coo' }, vector: 'not-an-array' } }));
  assert.ok(e instanceof McpError, String(e));
  assert.equal(e.code, ErrorCode.InvalidParams, e.message);
  assert.doesNotMatch(e.message, /-32603/);

  const bounds = await rejection(client.callTool({ name: 'solveTrueSublinear', arguments: { matrix: m, vector: [1, 1] } }));
  assert.equal(bounds.code, ErrorCode.InvalidParams, bounds.message);
});

test('unknown tools reach the client as MethodNotFound', async (t) => {
  const { client } = await connect(t);
  const e = await rejection(client.callTool({ name: 'noSuchTool', arguments: {} }));
  assert.equal(e.code, ErrorCode.MethodNotFound, e.message);
});

test('solveTrueSublinear runs the core sparse solve: accurate, sparse, and deprecated', async (t) => {
  const { client, stderr } = await connect(t);
  // n = 20000 would need a 3.2 GB dense copy under the old JL solver.
  for (const n of [50, 20000]) {
    const m = tridiagCoo(n, 4);
    const b = Array.from({ length: n }, (_, i) => Math.sin(i));
    const started = Date.now();
    const r = await client.callTool({ name: 'solveTrueSublinear', arguments: { matrix: m, vector: b } });
    const out = JSON.parse(r.content[0].text);
    assert.ok(Date.now() - started < 15000, `n=${n} took ${Date.now() - started} ms`);
    assert.ok(out.verified_relative_residual < 1e-8, `n=${n}: residual ${out.verified_relative_residual}`);
    assert.equal(out.complexity_bound.type, 'linear');
    assert.match(out.method_used, /^sublinear-core /);
    if (n > 1000) assert.equal(out.solution.truncated, true);
    else assert.equal(out.solution.length, n);
  }
  // Non-DD input is still rejected with InvalidParams.
  const nonDd = { rows: 3, cols: 3, rowIndices: [0, 0, 1, 1, 1, 2, 2], colIndices: [0, 1, 0, 1, 2, 1, 2], values: [1, 2, 2, 1, 2, 2, 1] };
  const e = await rejection(client.callTool({ name: 'solveTrueSublinear', arguments: { matrix: nonDd, vector: [1, 1, 1] } }));
  assert.equal(e.code, ErrorCode.InvalidParams, e.message);
  assert.match(stderr(), /solveTrueSublinear is deprecated/);
});

test('the complete solver works without dist/wasm/strange_loop_bg.wasm', async () => {
  assert.equal(existsSync(join(root, 'dist/wasm/strange_loop_bg.wasm')), false, 'strange_loop wasm should not be built into dist');
  const { CompleteWasmSublinearSolverTools } = await import('../dist/mcp/tools/wasm-sublinear-complete.js');
  const tools = new CompleteWasmSublinearSolverTools();
  const A = [[4, -1, 0], [-1, 4, -1], [0, -1, 4]];
  const b = [1, 2, 3];
  const out = await tools.solveComplete(A, b);
  const x = out.solution;
  // Same behaviour as 1.9.0 with the file present (accuracy is not this test's concern:
  // the class is constructed by the server but no tool routes to it).
  assert.ok(Array.isArray(x) && x.length === 3 && x.every(Number.isFinite), JSON.stringify(out).slice(0, 200));
});

test('the npm tarball does not ship the strange_loop wasm', () => {
  const [pack] = JSON.parse(execFileSync('npm', ['pack', '--dry-run', '--json', '--ignore-scripts'], { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }));
  const files = pack.files.map((f) => f.path);
  assert.deepEqual(files.filter((f) => /strange_loop/.test(f)), []);
  assert.ok(files.includes('dist/mcp/tools/wasm-sublinear-complete.js'));
});
