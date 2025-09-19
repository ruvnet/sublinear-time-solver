#!/usr/bin/env node

const { createSolver } = require('../src/solver');
const { SolverServer } = require('../server');
const { FlowNexusIntegration } = require('../integrations/flow-nexus');

// Example 1: Basic CLI-style solving
async function basicSolveExample() {
  console.log('🔧 Example 1: Basic Matrix Solving\n');

  try {
    // Create a simple 3x3 system
    const matrix = {
      rows: 3,
      cols: 3,
      format: 'dense',
      data: [
        [2, -1, 0],
        [-1, 2, -1],
        [0, -1, 2]
      ]
    };

    const vector = [1, 2, 3];

    console.log('Matrix:');
    matrix.data.forEach(row => console.log(row.join('\t')));
    console.log('Vector:', vector);
    console.log();

    // Create solver
    const solver = await createSolver({
      matrix,
      method: 'jacobi',
      tolerance: 1e-8,
      maxIterations: 1000
    });

    console.log('🔄 Solving system...');
    const startTime = Date.now();

    const solution = await solver.solve(vector, {
      onProgress: (update) => {
        if (update.iteration % 100 === 0) {
          console.log(`  Iteration ${update.iteration}: residual = ${update.residual.toExponential(2)}`);
        }
      }
    });

    const elapsed = Date.now() - startTime;

    console.log('\n✅ Solution found:');
    console.log('x =', solution.values.map(x => x.toFixed(6)));
    console.log(`Iterations: ${solution.iterations}`);
    console.log(`Time: ${elapsed}ms`);
    console.log(`Final residual: ${solution.residual.toExponential(2)}`);

    // Verify solution
    console.log('\n🔍 Verification:');
    const verification = await verifySolution(matrix, solution.values, vector);
    console.log(`Max error: ${verification.maxError.toExponential(2)}`);
    console.log(`Verified: ${verification.verified ? '✅' : '❌'}`);

  } catch (error) {
    console.error('❌ Error:', error.message);
  }
}

// Example 2: Streaming solve with real-time updates
async function streamingSolveExample() {
  console.log('\n🌊 Example 2: Streaming Solve\n');

  try {
    // Create a larger sparse system
    const matrix = generateRandomSparseMatrix(100, 0.05);
    const vector = generateRandomVector(100);

    console.log(`Matrix: ${matrix.rows}×${matrix.cols}, ${matrix.entries} non-zeros`);
    console.log(`Sparsity: ${(matrix.entries / (matrix.rows * matrix.cols) * 100).toFixed(2)}%`);

    const solver = await createSolver({
      matrix,
      method: 'conjugate-gradient',
      tolerance: 1e-10,
      maxIterations: 500,
      enableVerification: true
    });

    console.log('\n🔄 Starting streaming solve...');

    let lastUpdate = Date.now();
    for await (const update of solver.streamSolve(vector)) {
      const now = Date.now();
      const deltaTime = now - lastUpdate;
      lastUpdate = now;

      console.log(
        `[${update.iteration.toString().padStart(3)}] ` +
        `Residual: ${update.residual.toExponential(2)} ` +
        `Rate: ${update.convergenceRate?.toFixed(4) || 'N/A'} ` +
        `Δt: ${deltaTime}ms ` +
        `${update.verified ? '✅' : '⏳'}`
      );

      if (update.converged) {
        console.log('\n🎯 Converged!');
        console.log(`Final solution norm: ${vectorNorm(update.solution).toFixed(6)}`);
        break;
      }
    }

  } catch (error) {
    console.error('❌ Streaming error:', error.message);
  }
}

// Example 3: HTTP server mode
async function serverModeExample() {
  console.log('\n🌐 Example 3: HTTP Server Mode\n');

  try {
    const server = new SolverServer({
      port: 3001,
      cors: true,
      workers: 2,
      maxSessions: 50
    });

    console.log('🚀 Starting server...');
    await server.start();

    console.log('Server running on http://localhost:3001');
    console.log('API endpoints:');
    console.log('  POST /api/v1/solve-stream - Streaming solve');
    console.log('  POST /api/v1/verify - Solution verification');
    console.log('  GET  /health - Health check');
    console.log('  WS   /ws - WebSocket streaming');

    // Simulate some API calls
    console.log('\n📡 Testing API...');
    await testServerAPI('http://localhost:3001');

    // Keep server running for demo
    console.log('\n⏱️  Server will run for 30 seconds...');
    setTimeout(async () => {
      await server.stop();
      console.log('🛑 Server stopped');
    }, 30000);

  } catch (error) {
    console.error('❌ Server error:', error.message);
  }
}

// Example 4: Flow-Nexus integration
async function flowNexusExample() {
  console.log('\n☁️  Example 4: Flow-Nexus Integration\n');

  try {
    const integration = new FlowNexusIntegration({
      endpoint: process.env.FLOW_NEXUS_ENDPOINT || 'https://api.flow-nexus.ruv.io',
      token: process.env.FLOW_NEXUS_TOKEN
    });

    if (!process.env.FLOW_NEXUS_TOKEN) {
      console.log('⚠️  FLOW_NEXUS_TOKEN not set, skipping integration example');
      return;
    }

    console.log('🔗 Registering solver with Flow-Nexus...');
    const registration = await integration.registerSolver({
      capabilities: ['high-performance', 'real-time', 'verification'],
      metadata: {
        description: 'Example sublinear time solver',
        max_matrix_size: 10000
      }
    });

    console.log('✅ Registered:', registration.solver_id);

    // Join a test swarm (if available)
    console.log('🤝 Attempting to join test swarm...');
    try {
      await integration.joinSwarm('test-swarm', {
        nodeId: 'example-node',
        capabilities: ['solver', 'verifier']
      });
      console.log('✅ Joined swarm successfully');
    } catch (error) {
      console.log('⚠️  Could not join swarm:', error.message);
    }

    // Demonstrate cost propagation
    console.log('📊 Broadcasting cost update...');
    await integration.broadcastCostUpdate({
      session_id: 'example-session',
      delta_costs: {
        indices: [1, 5, 10],
        values: [0.01, -0.005, 0.02]
      }
    });

    console.log('📈 Flow-Nexus status:');
    const status = await integration.getStatus();
    console.log(JSON.stringify(status, null, 2));

    // Clean up
    setTimeout(async () => {
      await integration.disconnect();
      console.log('🔌 Disconnected from Flow-Nexus');
    }, 10000);

  } catch (error) {
    console.error('❌ Flow-Nexus error:', error.message);
  }
}

// Example 5: Verification loops and accuracy testing
async function verificationExample() {
  console.log('\n🔍 Example 5: Solution Verification\n');

  try {
    const matrix = {
      rows: 5,
      cols: 5,
      format: 'coo',
      entries: 13,
      data: {
        rowIndices: [0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 4],
        colIndices: [0, 1, 0, 1, 2, 1, 2, 3, 2, 3, 2, 3, 4],
        values: [2, -1, -1, 2, -1, -1, 2, -1, -1, 2, -1, -1, 2]
      }
    };

    const exactSolution = [1, 1, 1, 1, 1];
    const vector = multiplyMatrixVector(matrix, exactSolution);

    console.log('Known exact solution:', exactSolution);
    console.log('Computed RHS:', vector.map(x => x.toFixed(3)));

    // Solve with different methods
    const methods = ['jacobi', 'gauss-seidel', 'conjugate-gradient'];

    for (const method of methods) {
      console.log(`\n📐 Testing ${method} method:`);

      const solver = await createSolver({
        matrix,
        method,
        tolerance: 1e-12,
        maxIterations: 1000
      });

      const solution = await solver.solve(vector);

      // Compare with exact solution
      const errors = solution.values.map((x, i) => Math.abs(x - exactSolution[i]));
      const maxError = Math.max(...errors);
      const meanError = errors.reduce((a, b) => a + b) / errors.length;

      console.log(`  Solution: [${solution.values.map(x => x.toFixed(6)).join(', ')}]`);
      console.log(`  Max error: ${maxError.toExponential(2)}`);
      console.log(`  Mean error: ${meanError.toExponential(2)}`);
      console.log(`  Iterations: ${solution.iterations}`);
      console.log(`  Converged: ${solution.converged ? '✅' : '❌'}`);

      // Verification with random probes
      const verification = await verifySolution(matrix, solution.values, vector, {
        tolerance: 1e-8,
        probes: 20
      });

      console.log(`  Verification: ${verification.verified ? '✅' : '❌'} (${verification.probeCount} probes)`);
    }

  } catch (error) {
    console.error('❌ Verification error:', error.message);
  }
}

// Utility functions
function generateRandomSparseMatrix(size, sparsity) {
  // Use the improved matrix utilities
  const { MatrixUtils } = require('../src/utils/matrix-utils');

  return MatrixUtils.generateWellConditionedSparseMatrix(size, sparsity, {
    diagonalStrategy: 'rowsum_plus_one',
    offDiagonalRange: [-0.5, 0.5],
    ensureDominance: true
  });
}

function generateRandomVector(size) {
  return Array.from({ length: size }, () => Math.random() * 10 - 5);
}

function multiplyMatrixVector(matrix, vector) {
  if (matrix.format === 'dense') {
    return matrix.data.map(row =>
      row.reduce((sum, val, i) => sum + val * vector[i], 0)
    );
  } else if (matrix.format === 'coo') {
    const result = new Array(matrix.rows).fill(0);
    for (let i = 0; i < matrix.data.values.length; i++) {
      const row = matrix.data.rowIndices[i];
      const col = matrix.data.colIndices[i];
      const val = matrix.data.values[i];
      result[row] += val * vector[col];
    }
    return result;
  }
  throw new Error(`Unsupported matrix format: ${matrix.format}`);
}

function vectorNorm(vector) {
  return Math.sqrt(vector.reduce((sum, val) => sum + val * val, 0));
}

async function verifySolution(matrix, solution, vector, options = {}) {
  const tolerance = options.tolerance || 1e-8;
  const probes = options.probes || 10;

  // Compute residual: ||Ax - b||
  const Ax = multiplyMatrixVector(matrix, solution);
  const residual = Ax.map((val, i) => val - vector[i]);
  const residualNorm = vectorNorm(residual);

  // Random probe verification
  const errors = [];
  for (let i = 0; i < probes; i++) {
    const idx = Math.floor(Math.random() * matrix.rows);
    const computed = multiplyMatrixRow(matrix, idx, solution);
    const error = Math.abs(computed - vector[idx]);
    errors.push(error);
  }

  const maxError = Math.max(...errors);
  const meanError = errors.reduce((a, b) => a + b) / errors.length;

  return {
    verified: maxError < tolerance && residualNorm < tolerance,
    maxError,
    meanError,
    residualNorm,
    probeCount: probes
  };
}

function multiplyMatrixRow(matrix, rowIndex, vector) {
  if (matrix.format === 'dense') {
    return matrix.data[rowIndex].reduce((sum, val, i) => sum + val * vector[i], 0);
  } else if (matrix.format === 'coo') {
    let result = 0;
    for (let i = 0; i < matrix.data.values.length; i++) {
      if (matrix.data.rowIndices[i] === rowIndex) {
        const col = matrix.data.colIndices[i];
        const val = matrix.data.values[i];
        result += val * vector[col];
      }
    }
    return result;
  }
  throw new Error(`Unsupported matrix format: ${matrix.format}`);
}

async function testServerAPI(baseUrl) {
  const fetch = require('node-fetch');

  try {
    // Test health endpoint
    console.log('📋 Testing health endpoint...');
    const healthResponse = await fetch(`${baseUrl}/health`);
    const health = await healthResponse.json();
    console.log(`   Status: ${health.status}`);

    // Test solve endpoint
    console.log('🧮 Testing solve endpoint...');
    const matrix = {
      rows: 3,
      cols: 3,
      format: 'dense',
      data: [[2, -1, 0], [-1, 2, -1], [0, -1, 2]]
    };
    const vector = [1, 2, 3];

    const solveResponse = await fetch(`${baseUrl}/api/v1/solve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ matrix, vector, method: 'jacobi' })
    });

    const solveResult = await solveResponse.json();
    console.log(`   Job ID: ${solveResult.job_id}`);
    console.log(`   Status: ${solveResult.status}`);

  } catch (error) {
    console.error('   API test failed:', error.message);
  }
}

// Main execution
async function main() {
  console.log('🚀 Sublinear Time Solver - Usage Examples\n');
  console.log('This demonstrates various usage patterns of the solver.\n');

  const examples = [
    { name: 'Basic Solving', fn: basicSolveExample },
    { name: 'Streaming Solve', fn: streamingSolveExample },
    { name: 'Server Mode', fn: serverModeExample },
    { name: 'Flow-Nexus Integration', fn: flowNexusExample },
    { name: 'Verification', fn: verificationExample }
  ];

  for (const example of examples) {
    try {
      await example.fn();
    } catch (error) {
      console.error(`\n❌ ${example.name} failed:`, error.message);
    }

    // Pause between examples
    if (example !== examples[examples.length - 1]) {
      console.log('\n' + '='.repeat(60));
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }

  console.log('\n✅ All examples completed!');
  console.log('\nFor more information:');
  console.log('  CLI help: npx sublinear-time-solver --help');
  console.log('  Server:   npx sublinear-time-solver serve --help');
  console.log('  Docs:     https://github.com/your-org/sublinear-time-solver');
}

// Run examples if called directly
if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = {
  basicSolveExample,
  streamingSolveExample,
  serverModeExample,
  flowNexusExample,
  verificationExample
};