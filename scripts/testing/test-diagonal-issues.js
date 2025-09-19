#!/usr/bin/env node

const { createSolver, JSSolver } = require('../../src/solver.js');

// Test various matrix configurations that might lead to zero diagonal errors
async function testDiagonalIssues() {
  console.log('=== Testing Diagonal Element Issues ===\n');

  const testCases = [
    {
      name: "Random sparse matrix with poor conditioning",
      matrixFactory: () => generatePoorlyConditionedMatrix(50, 0.1),
      vectorFactory: (size) => Array.from({length: size}, () => Math.random() * 10 - 5)
    },
    {
      name: "Random sparse matrix with zero diagonal possibility",
      matrixFactory: () => generateUnsafeMatrix(30, 0.15),
      vectorFactory: (size) => Array.from({length: size}, () => Math.random() * 5)
    },
    {
      name: "Large sparse matrix like in examples",
      matrixFactory: () => generateRandomSparseMatrix(100, 0.05),
      vectorFactory: (size) => Array.from({length: size}, () => Math.random() * 10 - 5)
    },
    {
      name: "Matrix with explicit zero diagonal element",
      matrixFactory: () => ({
        rows: 3,
        cols: 3,
        format: 'coo',
        entries: 5,
        data: {
          rowIndices: [0, 0, 1, 2, 2],
          colIndices: [1, 2, 2, 0, 1],
          values: [1, -1, 2, -1, 1]
          // Note: no diagonal element for row 0!
        }
      }),
      vectorFactory: () => [1, 2, 3]
    }
  ];

  for (const testCase of testCases) {
    console.log(`\n🧪 ${testCase.name}`);
    console.log('='.repeat(50));

    try {
      const matrix = testCase.matrixFactory();
      const vector = testCase.vectorFactory(matrix.rows);

      console.log(`Matrix: ${matrix.rows}×${matrix.cols}`);
      if (matrix.format === 'coo') {
        console.log(`Entries: ${matrix.data.values.length}`);

        // Check for missing diagonal elements
        const diagonalElements = new Set();
        for (let i = 0; i < matrix.data.values.length; i++) {
          const row = matrix.data.rowIndices[i];
          const col = matrix.data.colIndices[i];
          if (row === col) {
            diagonalElements.add(row);
          }
        }

        const missingDiagonals = [];
        for (let i = 0; i < matrix.rows; i++) {
          if (!diagonalElements.has(i)) {
            missingDiagonals.push(i);
          }
        }

        if (missingDiagonals.length > 0) {
          console.log(`⚠️  Missing diagonal elements at rows: ${missingDiagonals.join(', ')}`);
        } else {
          console.log(`✅ All diagonal elements present`);
        }
      }

      // Test each solver method
      const methods = ['jacobi', 'gauss-seidel', 'conjugate-gradient'];

      for (const method of methods) {
        console.log(`\n  🔧 Testing ${method}:`);

        try {
          const solver = await createSolver({
            matrix,
            method,
            tolerance: 1e-8,
            maxIterations: 100
          });

          const result = await solver.solve(vector);

          console.log(`    ✅ Success: ${result.iterations} iterations, residual: ${result.residual.toExponential(2)}`);

        } catch (error) {
          console.log(`    ❌ Error: ${error.message}`);

          // If it's a zero diagonal error, let's diagnose it
          if (error.message.includes('Zero diagonal element')) {
            const position = error.message.match(/position (\d+)/);
            if (position) {
              const pos = parseInt(position[1]);
              console.log(`    🔍 Analyzing position ${pos}:`);
              analyzeDiagonalElement(matrix, pos);
            }
          }
        }
      }

    } catch (error) {
      console.log(`  ❌ Matrix generation failed: ${error.message}`);
    }
  }
}

function generatePoorlyConditionedMatrix(size, sparsity) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  // Add very small diagonal entries (poorly conditioned)
  for (let i = 0; i < size; i++) {
    rowIndices.push(i);
    colIndices.push(i);
    values.push(Math.random() * 0.01 + 0.001); // Very small diagonal
  }

  // Add random off-diagonal entries
  const numEntries = Math.floor(size * size * sparsity);
  for (let i = 0; i < numEntries; i++) {
    const row = Math.floor(Math.random() * size);
    const col = Math.floor(Math.random() * size);

    if (row !== col) {
      rowIndices.push(row);
      colIndices.push(col);
      values.push((Math.random() - 0.5) * 2); // Larger off-diagonal entries
    }
  }

  return {
    rows: size,
    cols: size,
    entries: values.length,
    format: 'coo',
    data: { values, rowIndices, colIndices }
  };
}

function generateUnsafeMatrix(size, sparsity) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  // Randomly skip some diagonal entries
  for (let i = 0; i < size; i++) {
    if (Math.random() > 0.1) { // 10% chance of missing diagonal
      rowIndices.push(i);
      colIndices.push(i);
      values.push(Math.random() + 0.5);
    }
  }

  // Add random off-diagonal entries
  const numEntries = Math.floor(size * size * sparsity);
  for (let i = 0; i < numEntries; i++) {
    const row = Math.floor(Math.random() * size);
    const col = Math.floor(Math.random() * size);

    if (row !== col) {
      rowIndices.push(row);
      colIndices.push(col);
      values.push((Math.random() - 0.5) * 0.5);
    }
  }

  return {
    rows: size,
    cols: size,
    entries: values.length,
    format: 'coo',
    data: { values, rowIndices, colIndices }
  };
}

function generateRandomSparseMatrix(size, sparsity) {
  const values = [];
  const rowIndices = [];
  const colIndices = [];

  const numEntries = Math.floor(size * size * sparsity);

  // Add diagonal entries for better conditioning
  for (let i = 0; i < size; i++) {
    rowIndices.push(i);
    colIndices.push(i);
    values.push(2 + Math.random());
  }

  // Add random off-diagonal entries
  for (let i = 0; i < numEntries - size; i++) {
    const row = Math.floor(Math.random() * size);
    const col = Math.floor(Math.random() * size);

    if (row !== col) {
      rowIndices.push(row);
      colIndices.push(col);
      values.push((Math.random() - 0.5) * 0.5);
    }
  }

  return {
    rows: size,
    cols: size,
    entries: values.length,
    format: 'coo',
    data: { values, rowIndices, colIndices }
  };
}

function analyzeDiagonalElement(matrix, position) {
  if (matrix.format === 'dense') {
    const diag = matrix.data[position][position];
    console.log(`      Dense matrix diagonal[${position}] = ${diag}`);
  } else if (matrix.format === 'coo') {
    let diagonalValue = 0;
    let found = false;

    for (let i = 0; i < matrix.data.values.length; i++) {
      const row = matrix.data.rowIndices[i];
      const col = matrix.data.colIndices[i];

      if (row === position && col === position) {
        diagonalValue = matrix.data.values[i];
        found = true;
        break;
      }
    }

    if (found) {
      console.log(`      COO matrix diagonal[${position}] = ${diagonalValue} (magnitude: ${Math.abs(diagonalValue)})`);
      if (Math.abs(diagonalValue) < 1e-14) {
        console.log(`      ⚠️  Diagonal element is effectively zero!`);
      }
    } else {
      console.log(`      ❌ No diagonal element found at position ${position}`);
    }
  }
}

// Run the tests
if (require.main === module) {
  testDiagonalIssues().catch(console.error);
}

module.exports = { testDiagonalIssues };