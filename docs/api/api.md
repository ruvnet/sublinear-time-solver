# API Reference

## JavaScript/TypeScript API

### Installation

```bash
npm install sublinear-time-solver
```

### Basic Usage

```javascript
import { createSolver, Matrix, Vector } from 'sublinear-time-solver';

const solver = await createSolver(options);
```

## Core Classes

### `Solver`

Main solver interface for linear systems.

#### Constructor Options

```typescript
interface SolverOptions {
  tolerance?: number;        // Convergence tolerance (default: 1e-6)
  maxIterations?: number;    // Maximum iterations (default: 1000)
  method?: SolverMethod;     // Default solving method
  parallel?: boolean;        // Enable parallel execution
  memoryLimit?: number;      // Memory limit in MB
}
```

#### Methods

##### `solve(A, b, method?)`

Solve linear system Ax = b.

```typescript
solve(
  A: Matrix | number[][],
  b: Vector | number[],
  method?: 'jacobi' | 'gauss_seidel' | 'conjugate_gradient' | 'hybrid'
): Promise<Solution>
```

**Returns:**
```typescript
interface Solution {
  solution: number[];      // Solution vector x
  iterations: number;      // Number of iterations
  residual: number;        // Final residual norm
  converged: boolean;      // Convergence status
  time: number;           // Solve time in ms
}
```

##### `solveStream(A, b, method?)`

Solve with streaming progress updates.

```typescript
async function* solveStream(
  A: Matrix,
  b: Vector,
  method?: string
): AsyncIterator<SolutionStep>
```

**Yields:**
```typescript
interface SolutionStep {
  iteration: number;
  residual: number;
  solution?: number[];    // Partial solution
  converged: boolean;
}
```

##### `verify(A, x, b)`

Verify solution accuracy.

```typescript
verify(
  A: Matrix,
  x: Vector,
  b: Vector
): VerificationResult
```

**Returns:**
```typescript
interface VerificationResult {
  maxError: number;
  avgError: number;
  residualNorm: number;
  valid: boolean;
}
```

### `Matrix` Class

Sparse matrix representation.

```typescript
class Matrix {
  constructor(data: number[][] | SparseFormat);

  // Properties
  readonly rows: number;
  readonly cols: number;
  readonly nnz: number;    // Non-zero entries

  // Methods
  multiply(x: Vector): Vector;
  transpose(): Matrix;
  toDense(): number[][];
  toSparse(): SparseFormat;
}
```

### `Vector` Class

Vector operations.

```typescript
class Vector {
  constructor(data: number[] | Float64Array);

  // Properties
  readonly length: number;

  // Methods
  norm(): number;
  dot(other: Vector): number;
  scale(alpha: number): Vector;
  add(other: Vector): Vector;
  subtract(other: Vector): Vector;
}
```

## Streaming API

### AsyncIterator Pattern

```javascript
// Stream solution progress
for await (const step of solver.solveStream(A, b)) {
  console.log(`Progress: ${step.iteration}/${maxIterations}`);
  console.log(`Residual: ${step.residual}`);

  if (step.residual < targetAccuracy) {
    break; // Early termination
  }
}
```

### Event-Based Pattern

```javascript
const stream = solver.createStream(A, b);

stream.on('progress', (step) => {
  updateProgressBar(step.iteration);
});

stream.on('complete', (solution) => {
  console.log('Final solution:', solution);
});

stream.on('error', (err) => {
  console.error('Solver error:', err);
});

stream.start();
```

## Batch Operations

### `solveBatch()`

Solve multiple systems efficiently.

```typescript
solveBatch(
  problems: Array<{A: Matrix, b: Vector}>,
  options?: BatchOptions
): Promise<Solution[]>
```

**Options:**
```typescript
interface BatchOptions {
  parallel?: boolean;      // Process in parallel
  batchSize?: number;      // Batch size for processing
  progressCallback?: (completed: number, total: number) => void;
}
```

## Error Handling

All methods may throw these errors:

```typescript
class SolverError extends Error {
  code: string;
  details: any;
}

// Error codes
'INVALID_MATRIX'      // Matrix is not valid
'SINGULAR_MATRIX'     // Matrix is singular
'DIVERGED'            // Solution diverged
'MEMORY_EXCEEDED'     // Memory limit exceeded
'TIMEOUT'             // Computation timeout
```

## TypeScript Support

Full TypeScript definitions included:

```typescript
import {
  Solver,
  Matrix,
  Vector,
  Solution,
  SolverOptions,
  SolverMethod,
  SparseFormat
} from 'sublinear-time-solver';
```

## Performance Tips

1. **Use sparse formats** for large matrices
2. **Enable streaming** for real-time feedback
3. **Choose appropriate method** based on system properties
4. **Reuse solver instances** for multiple solves
5. **Consider memory limits** for very large problems

## Examples

### Complete Example

```javascript
import { createSolver } from 'sublinear-time-solver';

async function example() {
  // Create solver with options
  const solver = await createSolver({
    tolerance: 1e-8,
    maxIterations: 500,
    parallel: true
  });

  // Define problem
  const A = [
    [4, -1, 0],
    [-1, 4, -1],
    [0, -1, 3]
  ];
  const b = [15, 10, 10];

  // Solve with progress
  for await (const step of solver.solveStream(A, b, 'conjugate_gradient')) {
    if (step.iteration % 10 === 0) {
      console.log(`Iteration ${step.iteration}: ${step.residual.toExponential(2)}`);
    }
  }

  // Get final solution
  const result = await solver.solve(A, b);
  console.log('Solution:', result.solution);
  console.log('Verified:', solver.verify(A, result.solution, b).valid);
}
```