/**
 * Statistical plateau detector (design item #2).
 *
 * A plateau is declared only when ALL three hold over a rolling window of the
 * last W generations — this separates a genuine local optimum from a noisy
 * benchmark or an optimizer failure, without relying on intuition:
 *
 *   1. median per-generation improvement (bestDelta) < epsilon
 *   2. promotion rate over the window < maxPromotionRate
 *   3. candidate-score variance is shrinking (last < first in the window)
 *
 * Pure and deterministic: same history in -> same verdict out. The verifier
 * recomputes it from the sealed generation history.
 */

export const DEFAULT_PLATEAU_CONFIG = Object.freeze({
  window: 3,
  epsilon: 0.02,
  maxPromotionRate: 0.34,
});

function median(xs) {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
}

/**
 * @param {Array<{generation:number,bestDelta:number,attempts:number,promotions:number,scoreVariance:number}>} history
 * @param {typeof DEFAULT_PLATEAU_CONFIG} config
 */
export function detectPlateau(history, config = DEFAULT_PLATEAU_CONFIG) {
  const { window, epsilon, maxPromotionRate } = config;
  if (history.length < window) {
    return { plateau: false, reason: `insufficient history: ${history.length} < window ${window}`, config };
  }
  const w = history.slice(-window);
  const medianImprovement = Number(median(w.map((g) => g.bestDelta)).toFixed(6));
  const attempts = w.reduce((s, g) => s + g.attempts, 0);
  const promotions = w.reduce((s, g) => s + g.promotions, 0);
  const promotionRate = attempts ? Number((promotions / attempts).toFixed(6)) : 0;
  const varianceShrinking = w[w.length - 1].scoreVariance < w[0].scoreVariance;

  const clauses = {
    medianImprovementBelowEpsilon: medianImprovement < epsilon,
    promotionRateBelowMax: promotionRate < maxPromotionRate,
    varianceShrinking,
  };
  const plateau = clauses.medianImprovementBelowEpsilon && clauses.promotionRateBelowMax && clauses.varianceShrinking;

  return {
    plateau,
    // What KIND of state this is — the point of the rigorous rule.
    classification: plateau
      ? 'local-optimum'
      : clauses.medianImprovementBelowEpsilon && !clauses.varianceShrinking
        ? 'noisy-benchmark'
        : !clauses.promotionRateBelowMax
          ? 'still-improving'
          : 'inconclusive',
    window,
    medianImprovement,
    promotionRate,
    varianceShrinking,
    clauses,
    config,
  };
}
