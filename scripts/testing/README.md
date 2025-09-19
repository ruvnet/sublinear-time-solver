# Testing Utilities

This directory contains testing and validation utilities for the sublinear-time-solver project.

## Files

### `test-solver.js`
Tests the JavaScript solver implementation with small test systems using both Jacobi and Conjugate Gradient methods.

**Usage:**
```bash
cd scripts/testing
node test-solver.js
```

### `test-wasm-build.sh`
Validates that the WASM build environment is properly set up, including Rust toolchain and wasm-pack.

**Usage:**
```bash
cd scripts/testing
./test-wasm-build.sh
```

### `validate-api.js`
Tests the HTTP API server by starting a server instance and sending test requests to validate API functionality.

**Usage:**
```bash
cd scripts/testing
node validate-api.js
```

### `validate-full.js`
Comprehensive validation script that checks:
- File structure integrity
- Package.json validity
- CLI executable functionality
- Solver module exports
- Dependencies installation
- Rust/WASM environment

**Usage:**
```bash
cd scripts/testing
node validate-full.js
```

## Notes

- All utilities run from the `scripts/testing` directory
- Path references are configured to work relative to the project root (`../../`)
- These are development/testing utilities and should not be included in production builds