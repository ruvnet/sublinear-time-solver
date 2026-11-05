// Regression tests for the 1.8.0 fixes. Run with: node --test tests/regression-1.8.test.mjs
// (after `npm run build`). Each test checks results against an independent
// dense solve or the measured residual, never against the solver's own claims.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { SublinearSolver, pageRank, graphFromEdges, csrFromCoo, relativeResidual, mulberry32 } from '../dist/index.js';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function denseSolve(A, b) {
  const n = b.length;
  const M = A.map((r, i) => [...r, b[i]]);
  for (let c = 0; c < n; c++) {
    let p = c;
    for (let r = c + 1; r < n; r++) if (Math.abs(M[r][c]) > Math.abs(M[p][c])) p = r;
    [M[c], M[p]] = [M[p], M[c]];
    for (let r = c + 1; r < n; r++) { const f = M[r][c] / M[c][c]; for (let k = c; k <= n; k++) M[r][k] -= f * M[c][k]; }
  }
  const x = new Array(n).fill(0);
  for (let r = n - 1; r >= 0; r--) { let s = M[r][n]; for (let k = r + 1; k < n; k++) s -= M[r][k] * x[k]; x[r] = s / M[r][r]; }
  return x;
}

/** Strictly diagonally dominant system with mixed-sign off-diagonals. */
function system(n, seed) {
  const rand = mulberry32(seed);
  const data = Array.from({ length: n }, () => new Array(n).fill(0));
  for (let i = 0; i < n; i++) {
    let off = 0;
    for (let t = 0; t < 5; t++) {
      const j = Math.floor(rand() * n);
      if (j === i) continue;
      const v = rand() * 2 - 1;
      data[i][j] += v;
      off += Math.abs(v);
    }
    data[i][i] = off * 1.5 + 1;
  }
  const b = Array.from({ length: n }, () => rand() * 2 - 1);
  const ri = [], ci = [], vs = [];
  data.forEach((row, i) => row.forEach((v, j) => { if (v !== 0) { ri.push(i); ci.push(j); vs.push(v); } }));
  return {
    dense: { rows: n, cols: n, format: 'dense', data },
    coo: { rows: n, cols: n, format: 'coo', rowIndices: ri, colIndices: ci, values: vs },
    b,
    exact: denseSolve(data, b),
  };
}
const maxErr = (x, y) => x.reduce((m, v, i) => Math.max(m, Math.abs(v - y[i])), 0);

test('SublinearSolver neumann (default) matches the dense solve', async () => {
  for (const seed of [1, 2, 3]) {
    const s = system(120, seed);
    for (const matrix of [s.dense, s.coo]) {
      const r = await new SublinearSolver({ method: 'neumann', epsilon: 1e-9, maxIterations: 2000 }).solve(matrix, s.b);
      assert.ok(r.converged, 'converged');
      assert.ok(r.residual <= 1e-9, `residual ${r.residual}`);
      assert.ok(maxErr(r.solution, s.exact) < 1e-8, `error ${maxErr(r.solution, s.exact)}`);
      // The reported residual is the real one.
      const A = csrFromCoo(s.coo);
      const norm = Math.sqrt(s.b.reduce((a, v) => a + v * v, 0));
      assert.ok(Math.abs(relativeResidual(A, r.solution, s.b) * norm - r.residual) < 1e-12);
    }
  }
});

test('random-walk returns a statistically accurate solution instead of throwing', async () => {
  const s = system(60, 4);
  const r = await new SublinearSolver({ method: 'random-walk', epsilon: 1e-3, maxIterations: 1000, seed: 7 }).solve(s.coo, s.b);
  assert.equal(r.method, 'random-walk');
  assert.ok(maxErr(r.solution, s.exact) < 0.02, `error ${maxErr(r.solution, s.exact)}`);
});

test('push / bidirectional methods solve to the requested residual', async () => {
  const s = system(80, 5);
  for (const method of ['forward-push', 'backward-push', 'bidirectional']) {
    const r = await new SublinearSolver({ method, epsilon: 1e-9, maxIterations: 2000 }).solve(s.coo, s.b);
    assert.ok(r.converged && r.residual <= 1e-9, method);
    assert.ok(maxErr(r.solution, s.exact) < 1e-8, method);
  }
});

test('computePageRank sums to 1 with dangling nodes and matches the core pageRank', async () => {
  // 0 -> 1 -> 2, 2 dangles; 3 -> 0
  const data = [[0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 0], [1, 0, 0, 0]];
  const solver = new SublinearSolver({ method: 'neumann', epsilon: 1e-10, maxIterations: 1000 });
  const pr = await solver.computePageRank({ rows: 4, cols: 4, format: 'dense', data }, { damping: 0.85, epsilon: 1e-12, maxIterations: 1000 });
  assert.ok(Math.abs(pr.reduce((a, v) => a + v, 0) - 1) < 1e-9, `sum ${pr.reduce((a, v) => a + v, 0)}`);
  const core = pageRank(graphFromEdges([[0, 1], [1, 2], [3, 0]], { n: 4 }), { tolerance: 1e-12, maxIterations: 1000 });
  assert.ok(maxErr(pr, Array.from(core.scores)) < 1e-10);
  const ppr = await solver.computePageRank({ rows: 4, cols: 4, format: 'dense', data }, { damping: 0.85, epsilon: 1e-12, maxIterations: 1000, personalized: [0, 0, 0, 5] });
  assert.ok(Math.abs(ppr.reduce((a, v) => a + v, 0) - 1) < 1e-9);
  assert.ok(ppr[3] > 0.15 - 1e-9);
});

test('importing and using the library writes nothing to stdout', () => {
  const code = `
    const m = await import(${JSON.stringify(join(root, 'dist/index.js'))});
    const s = new m.SublinearSolver({ method: 'neumann', epsilon: 1e-8, maxIterations: 100 });
    await s.solve({ rows: 2, cols: 2, format: 'dense', data: [[4, 1], [1, 3]] }, [1, 2]);
    await s.computePageRank({ rows: 2, cols: 2, format: 'dense', data: [[0, 1], [1, 0]] }, { damping: 0.85, epsilon: 1e-8, maxIterations: 100 });
    m.pageRank(m.graphFromEdges([[0, 1]]));`;
  const r = spawnSync(process.execPath, ['--input-type=module', '-e', code], { encoding: 'utf8' });
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.stdout, '');
});

test('the core entry loads no third-party module except @ruvnet/sublinear-core', () => {
  // A resolve hook records every bare specifier the core entry pulls in.
  const hooks = 'export async function resolve(s, c, next) { if (!s.startsWith(".") && !s.startsWith("/") && !s.startsWith("node:") && !s.startsWith("file:")) process.stderr.write("BARE:" + s + "\\n"); return next(s, c); }';
  const register = `import { register } from 'node:module'; register('data:text/javascript,' + encodeURIComponent(${JSON.stringify(hooks)}));`;
  const r = spawnSync(process.execPath, ['--import', 'data:text/javascript,' + encodeURIComponent(register), '--input-type=module', '-e',
    `await import(${JSON.stringify(join(root, 'dist/core/index.js'))});`], { encoding: 'utf8' });
  assert.equal(r.status, 0, r.stderr);
  const bare = [...new Set(r.stderr.split('\n').filter((l) => l.startsWith('BARE:')).map((l) => l.slice(5)))];
  assert.deepEqual(bare, ['@ruvnet/sublinear-core']);
});

test('MCP server rejects hostile matrix arguments and stays healthy', async (t) => {
  const transport = new StdioClientTransport({ command: process.execPath, args: [join(root, 'dist/cli/index.js'), 'serve'], stderr: 'ignore' });
  const client = new Client({ name: 'regression', version: '1.0.0' });
  await client.connect(transport);
  t.after(() => client.close());

  const expectRejected = async (name, args) => {
    const r = await client.callTool({ name, arguments: args }).then((x) => x, (e) => ({ thrown: e }));
    assert.ok(r.thrown || r.isError, `${name} should reject ${JSON.stringify(args).slice(0, 80)}`);
  };
  // Prototype-pollution shaped indices.
  await expectRejected('analyzeTrueSublinearMatrix', { matrix: { rows: 2, cols: 2, values: [0], rowIndices: ['__proto__'], colIndices: ['map'] } });
  await expectRejected('solve', { matrix: { rows: 2, cols: 2, format: 'coo', values: [7], rowIndices: ['__proto__'], colIndices: ['push'] }, vector: [1, 1] });
  // Tiny body declaring a huge dense matrix.
  await expectRejected('analyzeTrueSublinearMatrix', { matrix: { rows: 40000, cols: 40000, values: [1], rowIndices: [0], colIndices: [0] } });
  await expectRejected('solve', { matrix: { rows: 2, cols: 2, format: 'dense', data: [[1, 0], [0, 1]] }, vector: [1, 'x'] });

  // A clean call afterwards still works and is accurate.
  const s = system(50, 9);
  const r = await client.callTool({ name: 'solve', arguments: { matrix: s.coo, vector: s.b, epsilon: 1e-9 } });
  const out = JSON.parse(r.content[0].text);
  assert.ok(maxErr(out.solution, s.exact) < 1e-8);
  const pr = await client.callTool({ name: 'pageRank', arguments: { adjacency: { rows: 3, cols: 3, format: 'dense', data: [[0, 1, 0], [0, 0, 1], [0, 0, 0]] } } });
  assert.ok(Math.abs(JSON.parse(pr.content[0].text).totalScore - 1) < 1e-6);
});
