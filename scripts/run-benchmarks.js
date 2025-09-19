#!/usr/bin/env node

const { performance } = require('perf_hooks');
const fs = require('fs');
const path = require('path');

// Import solver if available
const { createSolver } = require('../src/solver.js');

// Generate a sparse diagonally dominant matrix
function generateSparseMatrix(size, density = 0.1) {
  const matrix = [];
  for (let i = 0; i < size; i++) {
    const row = new Array(size).fill(0);

    // Add some off-diagonal elements based on density
    const nonZeros = Math.floor(size * density);
    let rowSum = 0;
    for (let k = 0; k < nonZeros; k++) {
      const j = Math.floor(Math.random() * size);
      if (j !== i) {
        const value = (Math.random() - 0.5) * 0.5;
        row[j] = value;
        rowSum += Math.abs(value);
      }
    }

    // Set diagonal element to ensure strict diagonal dominance
    row[i] = rowSum + 1.0 + Math.random() * size * 0.1;

    matrix.push(row);
  }
  return {
    data: matrix,
    rows: size,
    cols: size,
    format: 'dense'
  };
}

// Generate random RHS vector
function generateVector(size) {
  return Array.from({ length: size }, () => Math.random() * 10);
}

// Benchmark a single configuration
async function benchmarkSolver(method, size, iterations = 10) {
  const results = {
    method,
    size,
    times: [],
    iterations: [],
    residuals: [],
    memory: [],
    convergenceRates: [],
    reductionFactors: [],
    performanceGrades: []
  };

  for (let i = 0; i < iterations; i++) {
    const matrix = generateSparseMatrix(size, 0.1);
    const b = generateVector(size);

    const memBefore = process.memoryUsage().heapUsed;
    const startTime = performance.now();

    try {
      // Create solver with proper configuration
      const solver = await createSolver({
        matrix: matrix,
        method: method,
        tolerance: 1e-6,
        maxIterations: 1000,
        enableVerification: false,
        verbose: false
      });

      const result = await solver.solve(b);

      const endTime = performance.now();
      const memAfter = process.memoryUsage().heapUsed;

      results.times.push(endTime - startTime);
      results.iterations.push(result.iterations || 0);
      results.residuals.push(result.residual || 0);
      results.memory.push((memAfter - memBefore) / 1024 / 1024); // MB
      results.convergenceRates.push(result.convergenceRate || 0);
      results.reductionFactors.push(result.reductionFactor || 1);
      results.performanceGrades.push(result.performanceGrade || 'F');

    } catch (e) {
      // Skip failed iterations but log them
      console.error(`Failed iteration for ${method} size ${size}: ${e.message}`);
    }
  }

  // Calculate statistics
  if (results.times.length > 0) {
    results.avgTime = results.times.reduce((a, b) => a + b, 0) / results.times.length;
    results.minTime = Math.min(...results.times);
    results.maxTime = Math.max(...results.times);
    results.avgIterations = results.iterations.reduce((a, b) => a + b, 0) / results.iterations.length;
    results.avgMemory = results.memory.reduce((a, b) => a + b, 0) / results.memory.length;
    results.avgConvergenceRate = results.convergenceRates.reduce((a, b) => a + b, 0) / results.convergenceRates.length;
    results.avgReductionFactor = results.reductionFactors.reduce((a, b) => a + b, 0) / results.reductionFactors.length;

    // Count performance grades
    const gradeCounts = {};
    results.performanceGrades.forEach(grade => {
      gradeCounts[grade] = (gradeCounts[grade] || 0) + 1;
    });
    results.commonGrade = Object.entries(gradeCounts).sort((a, b) => b[1] - a[1])[0]?.[0] || 'F';
  } else {
    results.avgTime = 0;
    results.minTime = 0;
    results.maxTime = 0;
    results.avgIterations = 0;
    results.avgMemory = 0;
    results.avgConvergenceRate = 0;
    results.avgReductionFactor = 1;
    results.commonGrade = 'F';
  }

  return results;
}

// Run comprehensive benchmarks
async function runBenchmarks() {
  console.log('🚀 Running Comprehensive Solver Benchmarks\n');
  console.log('=' .repeat(80));

  const methods = ['jacobi', 'gauss_seidel', 'conjugate_gradient', 'hybrid'];
  const sizes = [10, 50, 100, 200]; // Reduced for faster testing
  const allResults = [];

  for (const method of methods) {
    console.log(`\n📊 Benchmarking ${method.toUpperCase()} method:`);
    console.log('-'.repeat(60));

    const methodResults = [];

    for (const size of sizes) {
      process.stdout.write(`  Size ${size}x${size}... `);
      try {
        const results = await benchmarkSolver(method, size, 3); // Fewer iterations for speed
        methodResults.push(results);

        if (results.avgTime > 0) {
          console.log(`✓ ${results.avgTime.toFixed(2)}ms (${results.avgIterations.toFixed(0)} iters, ${results.avgConvergenceRate.toFixed(1)}%)`);
        } else {
          console.log('✗ Failed');
        }
      } catch (error) {
        console.log(`✗ Error: ${error.message}`);
        methodResults.push({
          method,
          size,
          avgTime: 0,
          avgIterations: 0,
          avgConvergenceRate: 0,
          commonGrade: 'F'
        });
      }
    }

    allResults.push({ method, results: methodResults });
  }

  // Generate summary table
  console.log('\n' + '='.repeat(80));
  console.log('\n📈 ENHANCED BENCHMARK SUMMARY\n');
  console.log('| Method            | Size  | Time (ms) | Iters | Conv% | Grade | Reduction |');
  console.log('|-------------------|-------|-----------|-------|-------|-------|-----------|');

  for (const { method, results } of allResults) {
    for (const result of results) {
      if (result.avgTime > 0) {
        console.log(
          `| ${method.padEnd(17)} | ${String(result.size).padEnd(5)} | ${
            result.avgTime.toFixed(1).padStart(9)
          } | ${String(Math.round(result.avgIterations)).padStart(5)} | ${
            result.avgConvergenceRate.toFixed(1).padStart(5)
          } | ${result.commonGrade.padEnd(5)} | ${
            result.avgReductionFactor.toExponential(1).padStart(9)
          } |`
        );
      }
    }
  }

  console.log('\n' + '='.repeat(80));

  // Performance analysis
  console.log('\n🔬 PERFORMANCE ANALYSIS\n');

  // Find best performing method for each size
  for (const size of sizes) {
    const sizeResults = allResults
      .map(({ method, results }) => ({
        method,
        result: results.find(r => r.size === size)
      }))
      .filter(({ result }) => result && result.avgTime > 0)
      .sort((a, b) => a.result.avgTime - b.result.avgTime);

    if (sizeResults.length > 0) {
      const best = sizeResults[0];
      const ratio = sizeResults.length > 1
        ? (sizeResults[sizeResults.length - 1].result.avgTime / best.result.avgTime).toFixed(1)
        : 'N/A';

      console.log(
        `Size ${size}: ${best.method} is fastest (${best.result.avgTime.toFixed(2)}ms), ` +
        `${ratio}x faster than slowest`
      );
    }
  }

  // Time complexity analysis
  console.log('\n📊 TIME COMPLEXITY ANALYSIS\n');
  console.log('Analyzing scaling behavior (time vs problem size):');

  for (const { method, results } of allResults) {
    const validResults = results.filter(r => r.avgTime > 0 && r.size >= 50);
    if (validResults.length >= 3) {
      // Calculate scaling factor
      const first = validResults[0];
      const last = validResults[validResults.length - 1];
      const sizeRatio = last.size / first.size;
      const timeRatio = last.avgTime / first.avgTime;
      const scalingExponent = Math.log(timeRatio) / Math.log(sizeRatio);

      let complexity;
      if (scalingExponent < 0.5) complexity = 'O(log n) - Sublinear! 🎉';
      else if (scalingExponent < 1.2) complexity = 'O(n) - Linear';
      else if (scalingExponent < 1.8) complexity = 'O(n log n) - Linearithmic';
      else if (scalingExponent < 2.2) complexity = 'O(n²) - Quadratic';
      else complexity = 'O(n³) or worse - Cubic+';

      console.log(`  ${method}: scaling exponent ≈ ${scalingExponent.toFixed(2)} → ${complexity}`);
    }
  }

  // Memory usage analysis
  console.log('\n💾 MEMORY EFFICIENCY\n');
  for (const { method, results } of allResults) {
    const validResults = results.filter(r => r.avgMemory > 0);
    if (validResults.length > 0) {
      const avgMem = validResults.reduce((a, r) => a + r.avgMemory, 0) / validResults.length;
      const memPerElement = avgMem / validResults.reduce((a, r) => a + r.size * r.size, 0) * validResults.length * 1000000;
      console.log(`  ${method}: ${avgMem.toFixed(2)} MB avg, ${memPerElement.toFixed(3)} bytes/element`);
    }
  }

  // Save results to file
  const timestamp = new Date().toISOString().replace(/:/g, '-').slice(0, -5);
  const outputPath = path.join(__dirname, '..', 'docs', 'benchmarks', `benchmark-${timestamp}.json`);

  fs.writeFileSync(outputPath, JSON.stringify({ timestamp, results: allResults }, null, 2));
  console.log(`\n✅ Results saved to ${outputPath}`);

  return allResults;
}

// Main execution
if (require.main === module) {
  runBenchmarks();
}

module.exports = { benchmarkSolver, runBenchmarks };