# Sublinear-Time-Solver — Contributor Guidance

Compiled by the ruflo / `@claude-flow/guidance` control plane into a policy
bundle (constitution + intent-tagged shards + manifest). Rules are bullet
points; `#intent` and `@domain` tags make each shard's routing deterministic.
Section headings avoid the compiler's constitution markers (safety, security,
critical, always, …) so that all domain rules compile to retrievable shards
rather than always-on constitution entries.

## Invariants

- MUST validate and canonicalize every filesystem path in MCP tools to prevent arbitrary file write (CWE-73). #security @security
- NEVER merge a change that fails cargo-audit, cargo-deny, or npm-audit. #security @security

## Dependency hygiene

- MUST run all five supply-chain gates (cargo-audit, cargo-deny, npm-audit, dependency-review, lockfile-integrity) before merging a pull request. #security @security
- ALWAYS reject a dependency whose license or source is outside the cargo-deny allow policy. #security @security

## Runtime speed

- MUST NOT regress the WASM benchmark numbers recorded in BENCHMARK.md. #performance @performance
- Always profile a hot path with the nanosecond scheduler before optimizing it. #performance @performance

## Verification

- MUST run the numerical regression suite on every change to a solver kernel. #testing @testing
- Always assert the measured residual is below tolerance before reporting a system solved. #testing @testing

## Solver design

- Use a Krylov method (Conjugate Gradient or BiCGSTAB) for symmetric-positive-definite systems that are not diagonally dominant. #architecture @architecture
- NEVER rely on plain Jacobi iteration for a system that is not diagonally dominant. #architecture @architecture
