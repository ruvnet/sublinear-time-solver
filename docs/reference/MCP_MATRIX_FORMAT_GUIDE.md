# MCP Matrix Format Guide

## Issue: "Dense matrix data must be array of rows" Error

### Problem
When using MCP with large dense matrices (e.g., 1000×1000), you may encounter:
```
Error: MCP error -32603: Dense matrix data must be array of rows
```

This happens because:
1. Dense matrices require sending ALL rows of data
2. MCP may truncate large payloads for performance
3. A 1000×1000 dense matrix = 1 million elements = ~8MB of JSON

### Example of the Problem
```javascript
// ❌ This fails with large matrices
const matrix = {
  rows: 1000,
  cols: 1000,
  format: 'dense',
  data: [[10, -1, ...], [...], ...] // 1000 arrays of 1000 elements each!
  // MCP might truncate this to only first 5 rows, causing validation error
}
```

## Solution: Use Sparse COO Format

For matrices larger than 100×100, **always use sparse COO format**:

### Correct Usage
```javascript
// ✅ Use sparse COO format for large matrices
const matrix = {
  rows: 1000,
  cols: 1000,
  format: 'coo',
  values: [10, -1, -0.5, 10, -1, ...],        // Only non-zero values
  rowIndices: [0, 0, 0, 1, 1, ...],          // Row index for each value
  colIndices: [0, 1, 2, 1, 2, ...]           // Column index for each value
}

// Call MCP solver
const result = await mcp__sublinear-solver__solve({
  matrix: matrix,  // Sparse format works perfectly!
  vector: b
});
```

## Converting Dense to Sparse

### Helper Function
```javascript
function denseToSparse(denseMatrix) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  for (let i = 0; i < denseMatrix.rows; i++) {
    for (let j = 0; j < denseMatrix.cols; j++) {
      const val = denseMatrix.data[i][j];
      if (Math.abs(val) > 1e-10) {  // Only non-zeros
        values.push(val);
        rowIndices.push(i);
        colIndices.push(j);
      }
    }
  }

  return {
    rows: denseMatrix.rows,
    cols: denseMatrix.cols,
    format: 'coo',
    values,
    rowIndices,
    colIndices
  };
}
```

## Matrix Format Comparison

| Format | Best For | Memory Usage | MCP Performance |
|--------|----------|--------------|-----------------|
| Dense | Small matrices (<100×100) | O(n²) | ⚠️ Slow for large |
| COO Sparse | Large sparse matrices | O(nnz) | ✅ Fast & efficient |

### Dense Format
```javascript
{
  format: 'dense',
  rows: 5,
  cols: 5,
  data: [
    [10, -1, 0, 0, 0],
    [-1, 10, -1, 0, 0],
    [0, -1, 10, -1, 0],
    [0, 0, -1, 10, -1],
    [0, 0, 0, -1, 10]
  ]
}
```

### Sparse COO Format (Recommended)
```javascript
{
  format: 'coo',
  rows: 5,
  cols: 5,
  values: [10, -1, -1, 10, -1, -1, 10, -1, -1, 10, -1, -1, 10],
  rowIndices: [0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4],
  colIndices: [0, 1, 0, 1, 2, 1, 2, 3, 2, 3, 4, 3, 4]
}
```

## Performance Benefits

### Memory Savings
- Dense 1000×1000: 8MB (1M elements × 8 bytes)
- Sparse 1000×1000 (0.3% density): 24KB (3K elements × 8 bytes)
- **333x memory reduction!**

### MCP Transfer
- Dense: Sends 1,000,000 numbers
- Sparse: Sends only ~3,000 numbers
- **333x faster transfer!**

## Common Matrix Patterns

### Tridiagonal Matrix (Sparse)
```javascript
function createTridiagonal(n) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  for (let i = 0; i < n; i++) {
    // Diagonal
    values.push(2);
    rowIndices.push(i);
    colIndices.push(i);

    // Lower diagonal
    if (i > 0) {
      values.push(-1);
      rowIndices.push(i);
      colIndices.push(i - 1);
    }

    // Upper diagonal
    if (i < n - 1) {
      values.push(-1);
      rowIndices.push(i);
      colIndices.push(i + 1);
    }
  }

  return { rows: n, cols: n, format: 'coo', values, rowIndices, colIndices };
}
```

### Laplacian Matrix (Sparse)
```javascript
function createLaplacian(n) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  for (let i = 0; i < n; i++) {
    let degree = 0;

    // Add random connections (10% connectivity)
    for (let j = 0; j < n; j++) {
      if (i !== j && Math.random() < 0.1) {
        values.push(-1);
        rowIndices.push(i);
        colIndices.push(j);
        degree++;
      }
    }

    // Diagonal = degree
    values.push(degree);
    rowIndices.push(i);
    colIndices.push(i);
  }

  return { rows: n, cols: n, format: 'coo', values, rowIndices, colIndices };
}
```

## Recommendations

1. **Always use sparse format for matrices > 100×100**
2. **Pre-convert dense to sparse before MCP calls**
3. **Only send non-zero elements**
4. **Use helper functions for common patterns**

## Example: Complete MCP Call

```javascript
// Create a large sparse matrix
const matrix = createTridiagonal(1000);

// Create vector
const b = new Array(1000).fill(1);

// Solve via MCP (fast & efficient!)
const result = await mcp__sublinear-solver__solve({
  matrix,  // Sparse COO format
  vector: b,
  epsilon: 1e-6,
  maxIterations: 1000
});

console.log('Solution computed in', result.computeTime, 'ms');
console.log('Converged:', result.converged);
console.log('Iterations:', result.iterations);
```

## Summary

The "Dense matrix data must be array of rows" error occurs when:
- Using dense format for large matrices
- MCP truncates the data during serialization
- Validation expects all rows but only receives partial data

**Solution: Always use sparse COO format for large matrices!**