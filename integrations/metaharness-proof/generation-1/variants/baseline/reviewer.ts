// SPDX-License-Identifier: MIT
//
// reviewer — mutation surface "reviewer" (ADR-071). Pure policy: judge a patch
// against an injected risk-file list and the test outcome.

/** A single review finding for one changed file. */
export interface ReviewFinding {
  /** The changed file the finding refers to. */
  file: string;
  /** Severity of the finding. */
  severity: 'blocker' | 'warning' | 'info';
  /** Why the finding was raised. */
  reason: string;
}

/**
 * Review a patch. A changed file is flagged when it appears in the injected
 * `riskFiles` list (the reviewer does not itself decide what is risky — that
 * judgement is supplied as data). When the test suite failed, findings are
 * escalated to 'blocker'; otherwise risk-file edits are 'warning' and all other
 * changes are 'info'.
 */
export function reviewPatch(
  changedFiles: string[],
  testPassed: boolean,
  riskFiles: string[],
): ReviewFinding[] {
  const risky = new Set(riskFiles);
  const findings: ReviewFinding[] = [];
  for (const file of changedFiles) {
    const isRisky = risky.has(file);
    if (!testPassed) {
      findings.push({
        file,
        severity: 'blocker',
        reason: isRisky
          ? 'tests failed and the change touches a protected file'
          : 'tests failed for this change',
      });
    } else if (isRisky) {
      findings.push({
        file,
        severity: 'warning',
        reason: 'change touches a protected file',
      });
    } else {
      findings.push({ file, severity: 'info', reason: 'routine change' });
    }
  }
  return findings;
}
