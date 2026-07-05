// SPDX-License-Identifier: MIT
//
// retry policy — mutation surface "retryPolicy" (ADR-071). Pure policy: decide
// whether to retry based on an injected, symbolic failure classification.

/** Symbolic classification of a failed attempt, supplied by the caller. */
export type FailureClassification = 'transient' | 'repairable' | 'unknown';

/** The decision about whether and how to retry. */
export interface RetryDecision {
  /** Whether another attempt should be made. */
  retry: boolean;
  /** Backoff to wait before the next attempt, in milliseconds. */
  backoffMs: number;
  /** Why the decision was made. */
  reason: string;
}

/** Maximum number of attempts the baseline policy will allow. */
export const maxAttempts = 3;

/**
 * Decide whether to retry. Transient failures (e.g. a timeout) are retried with
 * exponential backoff; repairable failures (e.g. a type error a patch can fix)
 * get one immediate retry; unknown failures are not retried. The classification
 * is injected — the policy never inspects raw process output.
 */
export function decideRetry(
  attempt: number,
  classification: FailureClassification,
): RetryDecision {
  if (attempt >= maxAttempts) {
    return { retry: false, backoffMs: 0, reason: 'attempt budget exhausted' };
  }
  switch (classification) {
    case 'transient':
      return {
        retry: true,
        backoffMs: 250 * 2 ** attempt,
        reason: 'transient failure — retry with backoff',
      };
    case 'repairable':
      return { retry: true, backoffMs: 0, reason: 'repairable failure — retry once' };
    case 'unknown':
    default:
      return { retry: false, backoffMs: 0, reason: 'unknown failure — do not retry' };
  }
}
