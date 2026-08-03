// baseline retry policy: hammer — retry immediately, no backoff, up to 8
// attempts. Eventually solves, but wastes cost and wall-clock.
export function shouldRetry(attempt: number): { retry: boolean; backoffMs: number } {
  return { retry: attempt < 8, backoffMs: 0 };
}
