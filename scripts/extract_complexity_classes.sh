#!/usr/bin/env bash
#
# Extract the (type → ComplexityClass) pairs declared via the
# `impl Complexity for ...` blocks across the crate. Output is one
# line per impl, sorted, of the form:
#
#     crate::path::Type    ComplexityClass::Variant
#
# Used by the CI `complexity-baseline` regression-guard job in
# `.github/workflows/ci.yml`. Drive direct as:
#
#     scripts/extract_complexity_classes.sh > .github/complexity-baseline.txt
#
# ADR-001 item #1 + phase-2 enforcement: this script turns the
# type-level Complexity contract into a textual snapshot that CI
# can diff between PRs. A silent downgrade (SubLinear → Linear etc.)
# now fails the build the same way a regression in tests does.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)" 2>/dev/null || cd "$(dirname "$0")/.."

# `awk` walks each .rs file under src/, tracks the most-recent
# `impl Complexity for X { ... }` block, and emits a single line
# per impl pairing the type with the right-hand side of `const CLASS`.
#
# Excludes the test-only `Dummy` type and inside `#[cfg(test)]` mods
# (heuristic: skip lines under `mod tests`).

find src -name '*.rs' -print0 |
  xargs -0 awk '
    /^[[:space:]]*#\[cfg\(test\)\]/ { in_test = NR + 1 }
    /^mod tests \{/                 { in_mod_tests = 1 }
    /^\}/ && in_mod_tests           { in_mod_tests = 0 }
    in_mod_tests                    { next }

    /^impl Complexity for/ {
      # capture everything after "impl Complexity for " up to `{`
      line = $0
      sub(/^impl Complexity for /, "", line)
      sub(/[[:space:]]*\{.*$/, "", line)
      gsub(/[[:space:]]+/, " ", line)
      type_name = line
      next
    }
    /^[[:space:]]*const CLASS: ComplexityClass = / && type_name != "" {
      rhs = $0
      sub(/^[[:space:]]*const CLASS: ComplexityClass = /, "", rhs)
      sub(/;.*$/, "", rhs)
      # Adaptive { default, worst } may span lines; the common case is
      # one-liner so we extract whatever ends at the semicolon on the
      # same line. Multi-line variants are summarised as "Adaptive".
      if (rhs ~ /^ComplexityClass::Adaptive/) rhs = "ComplexityClass::Adaptive"
      printf "%-90s %s\n", type_name, rhs
      type_name = ""
    }
  ' |
  sort -u
