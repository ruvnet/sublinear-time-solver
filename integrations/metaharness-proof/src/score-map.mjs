/**
 * The deterministic scoring pipeline: raw per-task outcomes -> BenchmarkResult[]
 * via the REAL scoreBenchmark. Shared by the producer and the verifier so both
 * apply the identical mapping; the verifier still sources its raw outcomes from
 * the sealed bundle, not from the fixture.
 */
import { bench } from '@metaharness/darwin';

const COMMIT = '0000000000000000000000000000000000000000';
const { scoreBenchmark } = bench;

export function scoreRawOutcomes(variantId, parentId, rawList) {
  return rawList.map((raw) => {
    const score = scoreBenchmark(raw);
    return {
      taskId: raw.taskId,
      variantId,
      parentId,
      repoCommit: COMMIT,
      solved: score.verifiedSolve,
      publicTestPassed: raw.publicTestPassed,
      hiddenTestPassed: raw.hiddenTestPassed,
      regressionPassed: raw.regressionPassed,
      durationMs: raw.durationMs,
      costUsd: raw.costUsd,
      changedFiles: variantId === 'baseline' ? [] : ['retry_policy.ts'],
      blockedFileTouches: raw.blockedFileTouches,
      safetyViolations: raw.safetyViolations,
      hallucinatedFileRefs: raw.hallucinatedFileRefs,
      traceQuality: 0.9,
      patchPath: `patches/${variantId}/${raw.taskId}.patch`,
      tracePath: `traces/${variantId}/${raw.taskId}.json`,
      baseScore: score.baseScore,
      finalScore: score.finalScore,
    };
  });
}
