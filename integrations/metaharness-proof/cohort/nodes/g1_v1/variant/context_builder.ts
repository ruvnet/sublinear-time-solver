// SPDX-License-Identifier: MIT
//
// context builder — mutation surface "contextBuilder" (ADR-071). Pure policy:
// rank candidate files by overlap with the task's terms.

/** A ranked piece of context offered to the worker. */
export interface ContextItem {
  /** Relative path of the file. */
  path: string;
  /** Overlap score (count of shared terms). Higher is more relevant. */
  score: number;
}

/** Split a string into lowercased alphanumeric terms of length >= 2. */
function terms(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter((t) => t.length >= 2);
}

/**
 * Rank `files` by how many task terms appear in each file path, returning the
 * top 30 items in descending relevance. Ties keep the original path order.
 */
export function buildContext(task: string, files: string[]): ContextItem[] {
  const wanted = new Set(terms(task));
  const scored = files.map((path, index) => {
    const pathTerms = terms(path);
    let score = 0;
    for (const t of pathTerms) if (wanted.has(t)) score += 1;
    return { path, score, index };
  });
  scored.sort((a, b) => (b.score - a.score) || (a.index - b.index));
  return scored.slice(0, 50).map(({ path, score }) => ({ path, score }));
}
