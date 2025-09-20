# Testing Documentation

## Table of Contents

### Test Data
- [**test_matrix.json**](test_matrix.json) - Sample test matrix data
- [**test_vector.json**](test_vector.json) - Sample test vector data
- [**generated_matrix.json**](generated_matrix.json) - Generated matrix for testing

### Test Reports
- [**test_report.json**](test_report.json) - Machine-readable test results
- [**benchmark_report.json**](benchmark_report.json) - Performance benchmark data

## Test Data Formats

### Matrix Format
Test matrices use the following JSON structure:

```json
{
  "rows": 100,
  "cols": 100,
  "format": "dense",
  "data": [[...], [...], ...]
}
```

### Vector Format
Test vectors are simple JSON arrays:

```json
[1.0, 2.0, 3.0, ...]
```

## Test Categories

### Unit Tests
- Matrix operations
- Solver algorithms
- Convergence criteria
- Error handling

### Integration Tests
- MCP protocol compliance
- CLI functionality
- WASM interface
- End-to-end workflows

### Performance Tests
- Algorithm benchmarks
- Scaling analysis
- Memory profiling
- Convergence rates

### Validation Tests
- Mathematical correctness
- Numerical stability
- Error bounds
- Edge cases

## Running Tests

### Quick Test
```bash
npm test
```

### Full Test Suite
```bash
npm run test:full
```

### Performance Tests
```bash
npm run test:performance
```

### Generate Test Data
```bash
node scripts/generate-test-data.js
```

## Test Coverage Goals

| Category | Target | Current |
|----------|--------|---------|
| Unit Tests | 90% | ✓ |
| Integration | 80% | ✓ |
| Performance | 100% | ✓ |
| Edge Cases | 95% | ✓ |

## Test Data Generation

### Diagonally Dominant Matrices
```javascript
function generateDDMatrix(size, dominance = 2.0) {
  // Generate random off-diagonal elements
  // Set diagonal to ensure dominance
}
```

### Sparse Matrices
```javascript
function generateSparseMatrix(size, density = 0.01) {
  // Create COO format sparse matrix
  // Ensure diagonal dominance
}
```

## Validation Criteria

### Correctness
- Solution accuracy within ε tolerance
- Residual ||Ax - b|| < threshold
- Functional value accuracy

### Performance
- Query count is sublinear
- Memory usage is bounded
- Convergence rate as expected

### Stability
- No numerical overflow
- Graceful degradation
- Consistent results