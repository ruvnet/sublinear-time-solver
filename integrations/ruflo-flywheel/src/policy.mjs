/**
 * Two bounded retrieval POLICIES and a held-out intent-precision metric.
 *
 * Both policies call the SAME real ShardRetriever.retrieve() on the SAME
 * compiled bundle. The only difference is one bounded knob — whether the
 * policy supplies the task's intent to the retriever:
 *
 *   baseline  — intent-blind. Passes `intent: 'general'`, which no shard
 *               matches, so the retriever's intent boost (0.15 per matching
 *               shard, see scoreShards) never fires. Ranking is driven only by
 *               the default HashEmbeddingProvider similarity, which the package
 *               documents as test-only with "no real semantic meaning".
 *   candidate — intent-routed. Passes the task's intent (as an agent would
 *               from an issue label / task category), so the retriever applies
 *               its real 0.15 intent boost to matching shards.
 *
 * Metric: intent-precision@k — of the k shards the policy injects, the fraction
 * whose rule intent matches the task's author-labeled ground-truth intent. This
 * is a narrow, deterministic quality proxy (does the injected guidance match
 * the task's intent), NOT a human relevance judgment. It is fully reproducible:
 * re-running the real retriever on the same bundle + tasks yields identical
 * numbers. The 0.15 boost competes with embedding similarity, so the gain is
 * empirical and measured — not a tautology.
 */

/** Held-out task suite. `intent` is the author-assigned ground-truth label. */
export const HELD_OUT_TASKS = [
  { id: 't1', intent: 'security', task: 'Fix an arbitrary file write (CWE-73) in the MCP path handler' },
  { id: 't2', intent: 'security', task: 'Reject a dependency whose license violates the cargo-deny allow policy' },
  { id: 't3', intent: 'performance', task: 'Reduce allocations in the WASM matvec hot loop without regressing benchmarks' },
  { id: 't4', intent: 'performance', task: 'Profile a slow solver iteration with the nanosecond scheduler' },
  { id: 't5', intent: 'testing', task: 'Add a regression fixture that asserts the residual tolerance check' },
  { id: 't6', intent: 'testing', task: 'Run the numerical regression suite after changing a solver kernel' },
  { id: 't7', intent: 'architecture', task: 'Choose a Krylov method for an SPD system that is not diagonally dominant' },
  { id: 't8', intent: 'architecture', task: 'Replace Jacobi iteration on a non-diagonally-dominant system' },
];

/** Extract the rule intents attached to a retrieved shard entry. */
function shardIntents(entry) {
  const rule = entry?.shard?.rule ?? entry?.rule ?? {};
  return Array.isArray(rule.intents) ? rule.intents.map((s) => String(s).toLowerCase()) : [];
}

/** Retrieval request builder per policy. `candidate` supplies the task intent. */
export function requestFor(policy, task, k) {
  const base = { taskDescription: task.task, maxShards: k };
  if (policy === 'baseline') return { ...base, intent: 'general' };
  if (policy === 'candidate') return { ...base, intent: task.intent };
  throw new Error(`unknown policy: ${policy}`);
}

/**
 * Run one policy over the held-out suite with the REAL retriever. Returns the
 * per-task selections and the intent-precision@k metric (fraction of injected
 * shards whose rule intent includes the task's ground-truth intent).
 */
export async function scorePolicy(retriever, policy, k = 2, tasks = HELD_OUT_TASKS) {
  const perTask = [];
  let hits = 0, total = 0;
  for (const t of tasks) {
    const result = await retriever.retrieve(requestFor(policy, t, k));
    const selected = result.shards ?? [];
    const ids = selected.map((e) => (e.shard?.rule?.id ?? e.rule?.id ?? null));
    const matched = selected.filter((e) => shardIntents(e).includes(t.intent)).length;
    hits += matched; total += selected.length;
    perTask.push({ id: t.id, intent: t.intent, selectedRuleIds: ids, matched, of: selected.length });
  }
  return { policy, k, intentPrecision: total ? hits / total : 0, hits, total, perTask };
}
