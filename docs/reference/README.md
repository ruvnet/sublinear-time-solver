# Reference Documentation

## Table of Contents

### MCP Documentation
- [**MCP_DENSE_FIX_COMPLETE.md**](MCP_DENSE_FIX_COMPLETE.md) - Dense matrix format implementation details
- [**MCP_MATRIX_FORMAT_GUIDE.md**](MCP_MATRIX_FORMAT_GUIDE.md) - Complete guide to matrix formats (dense, COO sparse)
- [**MCP_TOOL_TEST_RESULTS.md**](MCP_TOOL_TEST_RESULTS.md) - MCP tool testing results and validation

### CLI & Tools
- [**cli.md**](cli.md) - Command-line interface documentation
  - Commands and options
  - Usage examples
  - Configuration

### WebAssembly
- [**WASM_STATUS.md**](WASM_STATUS.md) - WebAssembly module status and integration
  - Build instructions
  - Performance characteristics
  - Browser compatibility

## Quick Reference

### MCP Tool Signatures

```typescript
// Solve linear system
solve(matrix: Matrix, vector: Vector, options?: SolverOptions): Solution

// Estimate single entry
estimateEntry(matrix: Matrix, vector: Vector, row: number, column: number): number

// Analyze matrix properties
analyzeMatrix(matrix: Matrix): MatrixAnalysis

// Compute PageRank
pageRank(adjacency: Matrix, options?: PageRankOptions): Vector
```

### Matrix Format Examples

**Dense Format:**
```json
{
  "rows": 3,
  "cols": 3,
  "format": "dense",
  "data": [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
}
```

**COO Sparse Format:**
```json
{
  "rows": 3,
  "cols": 3,
  "format": "coo",
  "data": {
    "values": [1, 1, 1],
    "rowIndices": [0, 1, 2],
    "colIndices": [0, 1, 2]
  }
}
```

## Integration Guidelines

### Node.js/JavaScript
```javascript
const solver = require('sublinear-time-solver');
const result = await solver.solve(matrix, vector);
```

### MCP Server
```bash
npx @modelcontextprotocol/server sublinear-solver
```

### WebAssembly
```javascript
import init, { solve } from './pkg/sublinear_solver.js';
await init();
const result = solve(matrix, vector);
```