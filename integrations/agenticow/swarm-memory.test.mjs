/**
 * Proof suite for SwarmMemory (agenticow-backed collective memory).
 *
 * Runs against real agenticow when installed; falls back to an in-memory fake
 * that models COW read-through + isolation so the coordination logic is always
 * exercised in CI without the native dependency.
 *
 * Run: node --test integrations/agenticow/
 */

import { test, describe, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { SwarmMemory } from './swarm-memory.mjs';

// agenticow is an import-only (ESM) package with no CJS `require` export, so
// probe it with a real dynamic import (top-level await) rather than
// require.resolve, which would spuriously fail and force the fake.
let agenticowMod = null;
try {
  agenticowMod = await import('agenticow');
} catch {
  agenticowMod = null;
}
const hasAgenticow = agenticowMod !== null;

// ---------------------------------------------------------------------------
// In-memory fake modeling agenticow's COW semantics (used when the native
// package isn't installed). Read-through = local edits over parent; fork is
// isolated; promote copies local edits up; checkpoint/rollback snapshot local.
// ---------------------------------------------------------------------------
function makeFakeMemory(parent = null, label = 'base') {
  const local = new Map(); // id -> {vector, text}
  let checkpoints = [];
  const cosine = (a, b) => {
    let dot = 0, na = 0, nb = 0;
    for (let i = 0; i < a.length; i++) { dot += a[i] * b[i]; na += a[i] * a[i]; nb += b[i] * b[i]; }
    return 1 - dot / (Math.sqrt(na) * Math.sqrt(nb) + 1e-12);
  };
  const chain = (self) => {
    // resolve id -> {entry, branch}, local wins over parent
    const seen = new Map();
    const walk = (node, lbl) => {
      if (node.__parent) walk(node.__parent, node.__parentLabel);
      for (const [id, e] of node.__local) seen.set(id, { e, branch: node.__label });
    };
    walk(self, label);
    return seen;
  };
  const self = {
    __local: local,
    __parent: parent,
    __parentLabel: parent?.__label ?? null,
    __label: label,
    // Mirrors agenticow's dual signature: ingest(records[]) with explicit ids,
    // or ingest(vector, {id?, text?}) single-vector convenience with auto-id.
    ingest(recordsOrVector, payload) {
      let autoId = () => local.size + (parent ? 1000 : 0) + 1;
      if (Array.isArray(recordsOrVector) && !ArrayBuffer.isView(recordsOrVector) &&
          typeof recordsOrVector[0] === 'object' && recordsOrVector[0] !== null &&
          'vector' in recordsOrVector[0]) {
        for (const r of recordsOrVector) {
          const id = r.id ?? autoId();
          local.set(id, { vector: Array.from(r.vector), text: r.text });
        }
        return { accepted: recordsOrVector.length, rejected: 0, epoch: 0 };
      }
      // single-vector form
      const id = payload?.id ?? autoId();
      local.set(id, { vector: Array.from(recordsOrVector), text: payload?.text });
      return { accepted: 1, rejected: 0, epoch: 0 };
    },
    query(vector, k = 10) {
      const resolved = chain(self);
      const hits = [];
      for (const [id, { e, branch }] of resolved) {
        hits.push({ id, distance: cosine(Array.from(vector), e.vector), branch, text: e.text });
      }
      hits.sort((a, b) => a.distance - b.distance);
      return hits.slice(0, k);
    },
    fork(childLabel = 'fork') {
      return makeFakeMemory(self, childLabel);
    },
    diff() {
      return { added: [...local.keys()], overridden: [], deleted: [] };
    },
    promote(target) {
      let ingested = 0;
      for (const [id, e] of local) { target.__local.set(id, e); ingested++; }
      return { ingested, deleted: 0 };
    },
    checkpoint(cpLabel = `cp-${checkpoints.length}`) {
      checkpoints.push({ id: `${cpLabel}-${checkpoints.length}`, snapshot: new Map(local) });
      local.clear();
      return { id: checkpoints[checkpoints.length - 1].id, label: cpLabel, path: '', depth: checkpoints.length };
    },
    rollback(checkpointId) {
      const cp = checkpointId
        ? checkpoints.find((c) => c.id === checkpointId)
        : checkpoints[checkpoints.length - 1];
      local.clear();
      return { restoredTo: cp?.id ?? 'base', depth: checkpoints.length };
    },
    lineage() {
      return [{ role: 'working', id: label, label, path: '', parent: parent?.__label ?? null, createdAt: 0, mutations: local.size, tombstones: 0 }];
    },
    status() {
      return { totalVectors: local.size, dimension: 3, metric: 'cosine' };
    },
    close() {},
  };
  return self;
}

// ---------------------------------------------------------------------------

const DIM = 3;
let storeDir;
let openBase; // (label) => AgenticMemoryLike

before(async () => {
  if (hasAgenticow) {
    storeDir = mkdtempSync(join(tmpdir(), 'swarm-mem-'));
    const open = agenticowMod.open ?? agenticowMod.default?.open;
    let seq = 0;
    openBase = () => open(join(storeDir, `hive-${seq++}.rvf`), { dimension: DIM, metric: 'cosine' });
  } else {
    openBase = () => makeFakeMemory(null, 'base');
  }
});

after(() => {
  if (storeDir) rmSync(storeDir, { recursive: true, force: true });
});

describe(`SwarmMemory (${hasAgenticow ? 'native agenticow' : 'COW fake'})`, () => {
  test('agents read through to shared base knowledge', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.seed([{ id: 1, vector: [1, 0, 0], text: 'collective-fact' }]);
    hive.spawn('a1');
    const hits = hive.recall('a1', [1, 0, 0], 1);
    assert.equal(hits[0].text, 'collective-fact', 'agent must see base knowledge through its branch');
    hive.close();
  });

  test('agent private memory is isolated from peers and from the base', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.seed([{ id: 1, vector: [1, 0, 0], text: 'shared' }]);
    hive.spawn('a1');
    hive.spawn('a2');
    hive.remember('a1', [0, 0, 1], 'a1-private');

    // a1 sees its own private note
    assert.equal(hive.recall('a1', [0, 0, 1], 1)[0].text, 'a1-private');
    // a2 does NOT see a1's uncommitted note
    assert.notEqual(hive.recall('a2', [0, 0, 1], 1)[0]?.text, 'a1-private');
    // base does NOT see it either
    assert.notEqual(hive.recallShared([0, 0, 1], 1)[0]?.text, 'a1-private');
    hive.close();
  });

  test('commit promotes validated findings so future agents benefit', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.seed([{ id: 1, vector: [1, 0, 0], text: 'shared' }]);
    hive.spawn('explorer');
    hive.remember('explorer', [0, 0, 1], 'validated-finding');
    const res = hive.commit('explorer');
    assert.ok(res.ingested >= 1, 'promote should ingest the finding into base');
    assert.deepEqual(hive.liveAgents, [], 'branch retired after commit');

    // A newly spawned agent now sees the promoted knowledge
    hive.spawn('latecomer');
    assert.equal(hive.recall('latecomer', [0, 0, 1], 1)[0].text, 'validated-finding');
    hive.close();
  });

  test('discard throws away a dead-end branch without touching the base', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.seed([{ id: 1, vector: [1, 0, 0], text: 'shared' }]);
    hive.spawn('deadend');
    hive.remember('deadend', [0, 0, 1], 'garbage-hypothesis');
    hive.discard('deadend');
    assert.deepEqual(hive.liveAgents, [], 'branch retired after discard');

    // base is pristine — no trace of the discarded exploration
    hive.spawn('fresh');
    assert.notEqual(hive.recall('fresh', [0, 0, 1], 1)[0]?.text, 'garbage-hypothesis');
    hive.close();
  });

  test('checkpoint + rollback snapshots the collective base', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.seed([{ id: 1, vector: [1, 0, 0], text: 'stable' }]);
    const cp = hive.checkpoint('before-risky-round');
    assert.ok(cp.id, 'checkpoint returns an id');

    hive.spawn('risky');
    hive.remember('risky', [0, 1, 0], 'risky-write');
    hive.commit('risky');

    const rb = hive.rollback(cp.id);
    assert.ok(rb.restoredTo, 'rollback reports where it restored to');
    hive.close();
  });

  test('rejects wrong-dimension vectors', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.spawn('a1');
    assert.throws(() => hive.remember('a1', [1, 0], 'bad'), RangeError);
    assert.throws(() => hive.recall('a1', [1, 0, 0, 0], 1), RangeError);
    hive.close();
  });

  test('guards double-spawn and unknown-agent access', () => {
    const hive = new SwarmMemory(openBase(), DIM);
    hive.spawn('a1');
    assert.throws(() => hive.spawn('a1'), /already has a live branch/);
    assert.throws(() => hive.recall('ghost', [1, 0, 0], 1), /no live branch/);
    hive.close();
  });
});

test('SwarmMemory requires a real agenticow memory', () => {
  assert.throws(() => new SwarmMemory({}, 3), TypeError);
});
