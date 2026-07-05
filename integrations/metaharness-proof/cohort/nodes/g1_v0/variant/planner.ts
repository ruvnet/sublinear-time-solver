// SPDX-License-Identifier: MIT
//
// planner — mutation surface "planner" (ADR-071). Pure policy: task -> steps.

/** One ordered step in a plan. */
export interface PlanStep {
  /** Stable ordering index, 0-based. */
  order: number;
  /** Short symbolic kind of work this step performs. */
  kind: 'map' | 'inspect' | 'patch' | 'verify';
  /** Human-readable description of the step. */
  description: string;
}

/** A one-line summary of the repository this harness was generated for. */
export const repoSummary = "cohort";

/**
 * Build an ordered plan for a task. The baseline strategy is deliberately
 * conservative: locate the relevant files, inspect the existing tests, apply a
 * minimal patch, then verify by running the test command.
 */
export function createPlan(task: string): PlanStep[] {
  const trimmed = task.trim();
  const label = trimmed.length > 0 ? trimmed : 'the requested change';
  return [
    { order: 0, kind: 'map', description: `Map files relevant to: ${label}` },
    { order: 1, kind: 'inspect', description: 'Inspect existing tests and surrounding code' },
    { order: 2, kind: 'patch', description: 'Apply the smallest patch that satisfies the task' },
    { order: 3, kind: 'verify', description: 'Verify by running the project test command' },
  ];
}
