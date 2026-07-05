// SPDX-License-Identifier: MIT
//
// score policy — mutation surface "scorePolicy" (ADR-071). Pure policy: the
// weight vector applied to the positive scoring terms (ADR-072). Weights are
// non-negative and sum to 1.

/** The weights applied to each positive scoring term. */
export interface ScoreWeights {
  taskSuccess: number;
  testPassRate: number;
  traceQuality: number;
  costEfficiency: number;
  latencyEfficiency: number;
  safetyScore: number;
}

/** The baseline weight vector (sums to 1). */
export function scoreWeights(): ScoreWeights {
  return {
    taskSuccess: 0.35,
    testPassRate: 0.2,
    traceQuality: 0.15,
    costEfficiency: 0.1,
    latencyEfficiency: 0.1,
    safetyScore: 0.1,
  };
}
