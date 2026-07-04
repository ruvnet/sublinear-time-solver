# SwarmMemory — copy-on-write collective memory (agenticow)

Copy-on-write collective memory for the repo's multi-agent / hive-mind
coordination layer, backed by [`agenticow`](https://www.npmjs.com/package/agenticow)
(ruvnet's *"Git for Agent Memory"*, built on the same `rvf`/`ruvector` stack this
repo already uses).

## Why

The repo has an extensive swarm/hive-mind coordination surface
(`swarm-memory-manager`, `collective-intelligence-coordinator`, `hive-mind`
memory commands) but no cheap, branchable shared memory primitive underneath it.
`agenticow` provides exactly that: fork a per-agent branch of collective memory
in ~constant time regardless of base size, explore speculatively, then either
**commit** the validated findings up to the shared base or **discard** the
branch with zero cost to everyone else.

This maps the COW model onto swarm operations:

| Swarm concept | SwarmMemory | agenticow |
|---|---|---|
| Hive collective knowledge | shared base | `open()` base |
| Spawn a speculative agent | `spawn(id)` | `fork()` |
| Agent's private note | `remember(id, vec, text)` | branch `ingest()` |
| Agent recall (base ∪ own edits) | `recall(id, vec, k)` | branch `query()` |
| Commit validated findings | `commit(id)` | `promote()` + close |
| Abandon a dead end | `discard(id)` | close branch |
| Snapshot before a risky round | `checkpoint()` | `checkpoint()` |
| Undo a round | `rollback(cpId)` | `rollback()` |

## Proven properties

`swarm-memory.test.mjs` asserts, **against the native agenticow backend**:

- **Read-through** — an agent's `recall()` sees base ∪ its own edits.
- **Isolation** — one agent's uncommitted notes are invisible to peers and to
  the base.
- **Commit propagates** — after `commit()`, newly spawned agents see the
  promoted knowledge.
- **Discard is clean** — an abandoned branch leaves the base byte-for-byte as it
  was; no trace of the dead-end exploration.
- **Checkpoint/rollback** — the whole hive base can be snapshotted and restored.
- Dimension and lifecycle guards (wrong-size vectors, double-spawn,
  unknown-agent access).

The suite runs everywhere: when `agenticow` isn't installed it falls back to an
in-memory fake that models COW read-through + isolation, so CI exercises the
coordination logic without the native binary. (Detection uses a real dynamic
import, since agenticow is an import-only ESM package with no CJS `require`
export.)

## Fork cost is O(1) in base size

`benchmark.mjs` (measured here, file-backed):

```
base vectors |  median fork  |  mean fork
-------------+---------------+------------
        100 |    3.01 ms    |   3.17 ms
       1000 |    2.93 ms    |   2.95 ms
      10000 |    3.00 ms    |   3.03 ms
      50000 |    2.98 ms    |   3.06 ms
```

Fork time stays flat as the collective memory grows 500× — spawning speculative
agents does not scale with corpus size, which is what makes wide swarm fan-out
affordable.

## Usage

```js
import { SwarmMemory } from './swarm-memory.mjs';

const hive = await SwarmMemory.open('hive.rvf', { dimension: 384 });
hive.seed([{ id: 1, vector: embed('known fact'), text: 'known fact' }]);

hive.spawn('researcher-1');
hive.remember('researcher-1', embed('hypothesis'), 'hypothesis');
const context = hive.recall('researcher-1', embed('query'), 10); // base ∪ own edits

if (validated) hive.commit('researcher-1'); // promote to base for future agents
else hive.discard('researcher-1');          // drop it; base untouched

hive.close();
```

## Run

```bash
cd integrations/agenticow
node --test .        # coordination proof (COW fake, no deps)
npm install          # add native agenticow
node --test .        # same suite, native backend
node benchmark.mjs   # fork-cost-vs-base-size benchmark
```

## Files

- `swarm-memory.mjs` — the `SwarmMemory` coordinator
- `swarm-memory.test.mjs` — proof suite (native + fake fallback)
- `benchmark.mjs` — fork-cost benchmark
