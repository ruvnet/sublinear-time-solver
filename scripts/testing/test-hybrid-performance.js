#!/usr/bin/env node

const { createSolver } = require('../../src/solver');

/**
 * Test script to demonstrate the hybrid solver's performance compared to individual methods
 */
async function testHybridPerformance() {
  console.log('🔬 Hybrid Solver Performance Test\n');

  // Create a challenging test matrix
  const matrix = {
    rows: 100,
    cols: 100,
    format: 'dense',
    data: createTestMatrix(100)
  };

  const vector = Array.from({ length: 100 }, (_, i) => Math.sin(i * 0.1) + 1);

  const methods = ['jacobi', 'gauss-seidel', 'conjugate-gradient', 'hybrid'];
  const results = {};

  console.log('Testing matrix: 100x100 dense system');
  console.log('Target tolerance: 1e-8');
  console.log('Max iterations: 1000\n');

  for (const method of methods) {
    console.log(`\n🔧 Testing ${method.toUpperCase()} method:`);

    try {
      const startTime = Date.now();

      const solver = await createSolver({
        matrix,
        method,
        tolerance: 1e-8,
        maxIterations: 1000,
        verbose: method === 'hybrid' // Verbose output for hybrid to show phases
      });

      const solution = await solver.solve(vector);
      const elapsed = Date.now() - startTime;

      results[method] = {
        iterations: solution.iterations,
        residual: solution.residual,
        converged: solution.converged,
        time: elapsed,
        memoryUsage: solution.memoryUsage
      };

      console.log(`  ✅ Converged: ${solution.converged}`);
      console.log(`  📊 Iterations: ${solution.iterations}`);
      console.log(`  📈 Final residual: ${solution.residual.toExponential(2)}`);
      console.log(`  ⏱️  Time: ${elapsed}ms`);
      console.log(`  💾 Memory: ${solution.memoryUsage}MB`);

      // Show hybrid-specific metrics
      if (method === 'hybrid' && solution.convergenceReport) {
        const report = solution.convergenceReport;
        console.log(`  🎯 Convergence rate: ${report.convergence.convergenceRatePercent}%`);
        console.log(`  🔄 Reduction factor: ${report.convergence.reductionFactor}`);
        console.log(`  📝 Performance grade: ${report.performance.grade}`);
      }

    } catch (error) {
      console.error(`  ❌ Error: ${error.message}`);
      results[method] = { error: error.message };
    }
  }

  // Performance comparison
  console.log('\n📈 PERFORMANCE COMPARISON\n');

  const convergedMethods = Object.entries(results)
    .filter(([_, result]) => result.converged)
    .sort((a, b) => a[1].time - b[1].time);

  if (convergedMethods.length > 0) {
    console.log('🏆 Methods ranked by speed (converged only):');
    convergedMethods.forEach(([method, result], index) => {
      const rank = index + 1;
      const medal = rank === 1 ? '🥇' : rank === 2 ? '🥈' : rank === 3 ? '🥉' : '📍';
      console.log(`  ${medal} ${rank}. ${method}: ${result.time}ms (${result.iterations} iterations)`);
    });

    // Efficiency analysis
    console.log('\n⚡ Efficiency Analysis:');
    convergedMethods.forEach(([method, result]) => {
      const efficiency = result.iterations / result.time; // iterations per ms
      console.log(`  ${method}: ${efficiency.toFixed(3)} iterations/ms`);
    });

    // Quality analysis
    console.log('\n🎯 Solution Quality:');
    convergedMethods.forEach(([method, result]) => {
      console.log(`  ${method}: ${result.residual.toExponential(2)} residual`);
    });

    // Highlight hybrid advantages
    const hybridResult = results.hybrid;
    if (hybridResult && hybridResult.converged) {
      console.log('\n🔬 Hybrid Solver Analysis:');
      console.log(`  The hybrid solver combines multiple approaches for:`)
      console.log(`  • Fast initial convergence (forward push simulation)`);
      console.log(`  • Global accuracy improvement (random walk simulation)`);
      console.log(`  • Final polish (conjugate gradient)`);
      console.log(`  • Adaptive switching based on convergence rate`);

      const bestSingle = convergedMethods.find(([method]) => method !== 'hybrid');
      if (bestSingle) {
        const improvement = (bestSingle[1].time - hybridResult.time) / bestSingle[1].time * 100;
        if (improvement > 0) {
          console.log(`  🚀 ${improvement.toFixed(1)}% faster than best single method (${bestSingle[0]})`);
        }
      }
    }
  }

  console.log('\n✅ Performance test completed!');
}

// Create a test matrix with interesting convergence properties
function createTestMatrix(n) {
  const matrix = Array(n).fill(null).map(() => Array(n).fill(0));

  // Create a diagonally dominant matrix with some off-diagonal structure
  for (let i = 0; i < n; i++) {
    let rowSum = 0;

    // Add off-diagonal elements
    for (let j = 0; j < n; j++) {
      if (i !== j) {
        if (Math.abs(i - j) === 1) {
          // Tridiagonal structure
          matrix[i][j] = -0.25 + Math.random() * 0.1;
        } else if (Math.abs(i - j) <= 3) {
          // Some additional coupling
          matrix[i][j] = (Math.random() - 0.5) * 0.1;
        }
        rowSum += Math.abs(matrix[i][j]);
      }
    }

    // Make diagonally dominant
    matrix[i][i] = rowSum + 1 + Math.random() * 0.5;
  }

  return matrix;
}

// Run the test
if (require.main === module) {
  testHybridPerformance().catch(error => {
    console.error('Test failed:', error);
    process.exit(1);
  });
}

module.exports = { testHybridPerformance };