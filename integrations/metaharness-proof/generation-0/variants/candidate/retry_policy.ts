// candidate retry policy: bounded exponential backoff (up to 3 attempts).
// Solves the same tasks with far fewer, better-spaced attempts.
export function shouldRetry(attempt: number): { retry: boolean; backoffMs: number } {
  if (attempt >= 3) return { retry: false, backoffMs: 0 };
  return { retry: true, backoffMs: 20 * 2 ** attempt };
}
