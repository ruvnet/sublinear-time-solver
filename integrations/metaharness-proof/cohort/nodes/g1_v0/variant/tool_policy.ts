// SPDX-License-Identifier: MIT
//
// tool policy — mutation surface "toolPolicy" (ADR-071). Pure policy over
// SYMBOLIC command kinds. No raw shell strings are embedded.

/** The symbolic kinds of command the harness may schedule. */
export type CommandKind = 'test' | 'build' | 'lint';

/** The allow-listed kinds, in canonical order. */
export const allowedKinds: CommandKind[] = ['test', 'build', 'lint'];

/** True iff the given kind is permitted. */
export function isKindAllowed(k: CommandKind): boolean {
  return allowedKinds.includes(k);
}

/** Preferred execution order: lint first (cheap), then build, then test. */
const ORDER: Record<CommandKind, number> = { lint: 0, build: 1, test: 2 };

/**
 * Order a set of requested kinds deterministically (cheapest-first), dropping
 * any kind that is not allow-listed.
 */
export function orderKinds(kinds: CommandKind[]): CommandKind[] {
  return kinds
    .filter(isKindAllowed)
    .slice()
    .sort((a, b) => ORDER[a] - ORDER[b]);
}
// darwin: toolPolicy perturbation v4 (seed 0)
