const { createSolver, JSSolver } = require('../../src/solver.js');

async function testSolver() {
  console.log('=== Testing JavaScript Solver Implementation ===\n');
  
  // Test 1: Small system
  const A = [[4, 1, 0], [1, 3, -1], [0, -1, 2]];
  const b = [1, 2, 3];
  
  console.log('Test 1: Small 3x3 system');
  console.log('Matrix A:', A);
  console.log('Vector b:', b);
  
  // Convert simple array format to expected matrix format
  const matrixFormat = {
    rows: A.length,
    cols: A[0].length,
    format: 'dense',
    data: A
  };

  const solver = new JSSolver({
    matrix: matrixFormat,
    tolerance: 1e-6,
    maxIterations: 100
  });
  
  // Test Jacobi method
  console.log('\nJacobi Method:');
  solver.config.method = 'jacobi';
  const jacobiResult = await solver.solve(b);
  console.log('Solution:', jacobiResult.values);
  console.log('Iterations:', jacobiResult.iterations);
  console.log('Residual:', jacobiResult.residual);

  // Test Conjugate Gradient
  console.log('\nConjugate Gradient:');
  const cgSolver = new JSSolver({
    matrix: matrixFormat,
    tolerance: 1e-6,
    maxIterations: 100,
    method: 'conjugate-gradient'
  });
  const cgResult = await cgSolver.solve(b);
  console.log('Solution:', cgResult.values);
  console.log('Iterations:', cgResult.iterations);
  console.log('Residual:', cgResult.residual);
  
  // Verify solution
  console.log('\n=== Verification ===');
  const verify = (A, x, b) => {
    const Ax = A.map(row => row.reduce((sum, val, i) => sum + val * x[i], 0));
    const error = Math.max(...b.map((val, i) => Math.abs(val - Ax[i])));
    return { Ax, error };
  };
  
  const verification = verify(A, cgResult.values, b);
  console.log('Ax =', verification.Ax);
  console.log('b =', b);
  console.log('Max error:', verification.error);
  console.log('Success:', verification.error < 1e-5 ? '✓' : '✗');
}

testSolver().catch(console.error);
