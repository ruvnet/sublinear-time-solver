const test = require('node:test');
const assert = require('node:assert/strict');
const core = require('../dist/cjs/index.js');

test('CommonJS build exposes the same API', async () => {
  const esm = await import('../dist/esm/index.js');
  assert.deepEqual(Object.keys(core).sort(), Object.keys(esm).sort());
  const g = core.graphFromEdges([[0, 1], [1, 2], [2, 0]]);
  const r = core.pageRank(g, { tolerance: 1e-12 });
  for (const s of r.scores) assert.ok(Math.abs(s - 1 / 3) < 1e-9);
});
