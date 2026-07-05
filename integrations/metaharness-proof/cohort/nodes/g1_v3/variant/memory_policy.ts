// SPDX-License-Identifier: MIT
//
// memory policy — mutation surface "memoryPolicy" (ADR-071). Pure policy:
// decide whether an outcome record is worth keeping.

/** A record of one outcome the harness might choose to remember. */
export interface MemoryRecord {
  /** Whether the associated task ultimately succeeded. */
  success: boolean;
  /** How many attempts the task took. */
  attempts: number;
  /** Whether this outcome was novel relative to what is already known. */
  novel: boolean;
}

/**
 * Decide whether to remember a record. The baseline keeps anything novel, plus
 * any failure that took more than one attempt (a hard case worth recalling).
 */
export function shouldRemember(record: MemoryRecord): boolean {
  if (record.novel) return true;
  if (!record.success && record.attempts > 1) return true;
  return false;
}
