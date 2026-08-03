/**
 * ruvector integration proof (ruvnet/ruvector).
 *
 * Indexes simulated zeptosecond measurement events in a ruvector VectorDb and
 * verifies that nearest-neighbor retrieval recovers orientation-similar events
 * — i.e. events whose birth-time delays agree to zeptosecond precision.
 *
 * Skips gracefully when ruvector is not installed:
 *   cd validation/zeptosecond && npm install
 */

import { test, describe, after } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { simulateEventVectors, makeLcg } from './zeptosecond-physics.mjs';

const require = createRequire(import.meta.url);

let VectorDb = null;
try {
  ({ VectorDb } = require('ruvector'));
} catch {
  // not installed — suite will be skipped
}

// VectorDb persists to ./ruvector.db by default and shares that store across
// instances; give every test its own storagePath in a throwaway directory.
const storeDir = mkdtempSync(join(tmpdir(), 'zepto-ruvector-'));
let storeSeq = 0;
const freshDb = () =>
  new VectorDb({ dimensions: 3, storagePath: join(storeDir, `store-${storeSeq++}.db`) });

after(() => rmSync(storeDir, { recursive: true, force: true }));

describe('ruvector event indexing', { skip: VectorDb === null && 'ruvector not installed' }, () => {
  test('kNN over event vectors retrieves zeptosecond-consistent neighbors', async () => {
    const db = freshDb();

    const events = simulateEventVectors(2000, makeLcg(42));
    const byId = new Map();
    for (const e of events) {
      const id = String(e.id);
      byId.set(id, e);
      await db.insert({ id, vector: Array.from(e.vector) });
    }

    // Probe with a fresh event and check its neighbors have similar delays.
    const [probe] = simulateEventVectors(1, makeLcg(7));
    const results = await db.search({ vector: Array.from(probe.vector), k: 10 });

    assert.ok(results.length >= 5, `expected >=5 neighbors, got ${results.length}`);
    for (const hit of results) {
      const neighbor = byId.get(String(hit.id));
      assert.ok(neighbor, `unknown neighbor id ${hit.id}`);
      const deltaZs = Math.abs(neighbor.delayZs - probe.delayZs);
      // Neighbors in feature space must agree in physics space: within 25 zs
      // of the probe's delay (full range is 0..247 zs).
      assert.ok(
        deltaZs < 25,
        `neighbor ${hit.id} delay differs by ${deltaZs.toFixed(2)} zs from probe`,
      );
    }
  });

  test('retrieval separates aligned from perpendicular molecules', async () => {
    const db = freshDb();

    // Two labeled populations: aligned (|cos| ~ 1, delay ~ 247 zs) and
    // perpendicular (|cos| ~ 0, delay ~ 0 zs).
    const insert = (id, cos) =>
      db.insert({ id, vector: [cos, Math.sqrt(1 - cos * cos), Math.abs(cos)] });

    for (let i = 0; i < 50; i++) {
      await insert(`aligned-${i}`, 0.98 + 0.02 * (i / 50));
      await insert(`perp-${i}`, 0.02 * (i / 50));
    }

    const nearAligned = await db.search({ vector: [1, 0, 1], k: 10 });
    const nearPerp = await db.search({ vector: [0, 1, 0], k: 10 });

    assert.ok(nearAligned.every((r) => String(r.id).startsWith('aligned-')));
    assert.ok(nearPerp.every((r) => String(r.id).startsWith('perp-')));
  });
});
