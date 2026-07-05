/**
 * The synthetic single-round fixture.
 *
 * THIS IS SYNTHETIC. The candidate, its mutation, and its per-task outcomes are
 * hand-authored fixtures — NOT the product of a real agent solving real tasks.
 * Their only job is to exercise the real gate wiring end-to-end. The mechanism
 * (hashTasks, scoreBenchmark, decidePromotion, receipt sealing, shadow/no-serve)
 * is real; the inputs are a fixture. See README — "single-round
 * proof-of-mechanism", not learning.
 *
 * The seed is fixed so the sealed bundle is byte-for-byte reproducible.
 */

export const SEED = 42;
const COMMIT = '0000000000000000000000000000000000000000';

/** A small frozen holdout suite of BenchmarkTask objects. */
export const HOLDOUT_TASKS = [
  mkTask('retry-transient-1', 'Recover from a transient failure', 1),
  mkTask('retry-transient-2', 'Recover after two transient failures', 2),
  mkTask('ctx-window-1', 'Assemble enough context to pass', 1),
  mkTask('verify-loop-1', 'Fix then re-verify', 2),
  mkTask('no-regress-1', 'Change without breaking the suite', 1),
];

function mkTask(id, title, difficulty) {
  return {
    id,
    repo: 'proof-of-mechanism',
    commit: COMMIT,
    title,
    prompt: title,
    publicTestCommand: 'true',
    hiddenTestCommand: 'true',
    regressionTestCommand: 'true',
    timeoutMs: 5000,
    maxCostUsd: 0.05,
    allowedMutationFiles: ['retry_policy.ts'],
    blockedFiles: ['.env', 'package-lock.json'],
    successCriteria: [`solve ${id}`],
    difficulty,
  };
}

/**
 * Real policy source for two variants. The candidate mutates the retry policy
 * from give-up-immediately to bounded exponential backoff — a plausible,
 * self-contained improvement. These files are content-hashed as the baseline /
 * candidate manifests.
 */
export const VARIANT_FILES = {
  baseline: {
    'retry_policy.ts':
      `// baseline retry policy: hammer — retry immediately, no backoff, up to 8\n` +
      `// attempts. Eventually solves, but wastes cost and wall-clock.\n` +
      `export function shouldRetry(attempt: number): { retry: boolean; backoffMs: number } {\n` +
      `  return { retry: attempt < 8, backoffMs: 0 };\n` +
      `}\n`,
  },
  candidate: {
    'retry_policy.ts':
      `// candidate retry policy: bounded exponential backoff (up to 3 attempts).\n` +
      `// Solves the same tasks with far fewer, better-spaced attempts.\n` +
      `export function shouldRetry(attempt: number): { retry: boolean; backoffMs: number } {\n` +
      `  if (attempt >= 3) return { retry: false, backoffMs: 0 };\n` +
      `  return { retry: true, backoffMs: 20 * 2 ** attempt };\n` +
      `}\n`,
  },
};

/**
 * Synthetic per-task raw outcomes. Both variants SOLVE every task, but the
 * baseline's hammer-retry is uniformly expensive and slow while the candidate's
 * backoff is cheap and fast — a genuine, low-variance efficiency win. Every
 * field feeds the real scoreBenchmark; nothing is a pre-computed score.
 * @param {'baseline'|'candidate'} variant
 */
export function rawOutcomes(variant) {
  const candidate = variant === 'candidate';
  return HOLDOUT_TASKS.map((task) => ({
    taskId: task.id,
    publicTestPassed: true,
    hiddenTestPassed: true,
    regressionPassed: true,
    safetyViolations: [],
    blockedFileTouches: [],
    hallucinatedFileRefs: false,
    // baseline hammers (near cost/timeout budget); candidate backs off (cheap/fast).
    costUsd: candidate ? 0.004 : 0.045,
    maxCostUsd: task.maxCostUsd,
    durationMs: candidate ? 120 : 4500,
    timeoutMs: task.timeoutMs,
  }));
}
