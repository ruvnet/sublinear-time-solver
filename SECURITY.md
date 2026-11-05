# Security policy

## Reporting

Check the repository's [Security tab](https://github.com/ruvnet/sublinear-time-solver/security) for private vulnerability reporting. If that option is unavailable, contact a maintainer privately. Share the affected version, a minimal reproduction, and the impact.

## MCP file handling

Starting with npm version 1.6.0, MCP state and vector tools accept a file name within their configured directory. Pass a name such as `state.json` and set `CONSCIOUSNESS_EXPLORER_STATE_DIR` or `SUBLINEAR_SOLVER_VECTOR_DIR` if you need a different directory. See the [1.6.0 upgrade notes](CHANGELOG.md#160--rust-crate-020--2026-05-18) and [issue 19](https://github.com/ruvnet/sublinear-time-solver/issues/19) for the change history.

The [CI workflow](.github/workflows/ci.yml) runs file handling regression tests. The [supply chain workflow](.github/workflows/supply-chain.yml) runs Rust and npm audits, dependency policy checks, and lockfile verification. It also runs GitHub dependency review as an informational check on pull requests.
