# API Documentation

## Table of Contents

### API Reference
- [**api.md**](api.md) - Complete API reference for the sublinear-time solver
  - MCP tool interfaces
  - JavaScript/TypeScript API
  - Rust crate API
  - WebAssembly bindings

## Quick Links

### MCP Tools
- `solve` - Solve diagonally dominant linear systems
- `estimateEntry` - Estimate single matrix entry
- `analyzeMatrix` - Analyze matrix properties
- `pageRank` - Compute PageRank using sublinear solver

### JavaScript API
- Matrix class with sparse/dense support
- Solver class with multiple algorithms
- Async/await interface
- Error handling and validation

### Rust API
- Core solver implementation
- WASM compilation target
- Performance optimizations
- Memory management

## Usage Examples

```javascript
// JavaScript
const solver = new SublinearSolver();
const result = await solver.solve(matrix, vector, { epsilon: 1e-6 });

// MCP
{
  "tool": "solve",
  "arguments": {
    "matrix": {...},
    "vector": [...],
    "epsilon": 1e-6
  }
}
```

## Integration
- Node.js 16+ required
- TypeScript support included
- WASM module for performance
- MCP server for tool integration