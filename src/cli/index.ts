#!/usr/bin/env node

/**
 * CLI for Sublinear-Time Solver MCP Server
 */

import { program } from 'commander';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, join } from 'path';
import { SublinearSolverMCPServer } from '../mcp/server.js';
import { SublinearSolver } from '../core/solver.js';
import { MatrixOperations } from '../core/matrix.js';
import { MatrixTools } from '../mcp/tools/matrix.js';
import { SolverTools } from '../mcp/tools/solver.js';
import { GraphTools } from '../mcp/tools/graph.js';
import {
  Matrix,
  Vector,
  SolverConfig,
  SolverError
} from '../core/types.js';

// Version from package.json
const packagePath = resolve(join(import.meta.url.replace('file://', ''), '../../../package.json'));
const packageJson = JSON.parse(readFileSync(packagePath, 'utf8'));
const VERSION = packageJson.version;

program
  .name('sublinear-solver-mcp')
  .description('Sublinear-time solver for asymmetric diagonally dominant systems with MCP interface')
  .version(VERSION);

// MCP Server command (with multiple aliases)
program
  .command('serve')
  .alias('mcp-server')
  .alias('mcp')
  .alias('server')
  .description('Start the MCP server')
  .option('-p, --port <port>', 'Port number (if using HTTP transport)')
  .option('--transport <type>', 'Transport type (stdio|http)', 'stdio')
  .action(async (options) => {
    try {
      console.error(`Starting Sublinear Solver MCP Server v${VERSION}`);
      console.error(`Transport: ${options.transport}`);

      const server = new SublinearSolverMCPServer();
      await server.run();
    } catch (error) {
      console.error('Failed to start MCP server:', error);
      process.exit(1);
    }
  });

// Solve command for direct CLI usage
program
  .command('solve')
  .description('Solve a linear system from files')
  .requiredOption('-m, --matrix <file>', 'Matrix file (JSON format)')
  .requiredOption('-b, --vector <file>', 'Vector file (JSON format)')
  .option('-o, --output <file>', 'Output file for solution')
  .option('--method <method>', 'Solver method', 'neumann')
  .option('--epsilon <value>', 'Convergence tolerance', '1e-6')
  .option('--max-iterations <value>', 'Maximum iterations', '1000')
  .option('--timeout <ms>', 'Timeout in milliseconds')
  .option('--verbose', 'Verbose output')
  .action(async (options) => {
    try {
      console.log(`Sublinear Solver v${VERSION}`);
      console.log('Loading matrix and vector...');

      // Load matrix
      if (!existsSync(options.matrix)) {
        throw new Error(`Matrix file not found: ${options.matrix}`);
      }
      const matrixData = JSON.parse(readFileSync(options.matrix, 'utf8'));

      // Load vector
      if (!existsSync(options.vector)) {
        throw new Error(`Vector file not found: ${options.vector}`);
      }
      const vectorData = JSON.parse(readFileSync(options.vector, 'utf8'));

      // Validate inputs
      if (!Array.isArray(vectorData)) {
        throw new Error('Vector must be an array of numbers');
      }

      console.log(`Matrix: ${matrixData.rows}x${matrixData.cols} (${matrixData.format})`);
      console.log(`Vector: length ${vectorData.length}`);

      // Analyze matrix
      console.log('Analyzing matrix...');
      const analysis = MatrixTools.analyzeMatrix({ matrix: matrixData });

      if (options.verbose) {
        console.log('Matrix Analysis:');
        console.log(`  Diagonally dominant: ${analysis.isDiagonallyDominant}`);
        console.log(`  Dominance type: ${analysis.dominanceType}`);
        console.log(`  Dominance strength: ${analysis.dominanceStrength.toFixed(4)}`);
        console.log(`  Symmetric: ${analysis.isSymmetric}`);
        console.log(`  Sparsity: ${(analysis.sparsity * 100).toFixed(1)}%`);
        console.log(`  Recommended method: ${analysis.performance.recommendedMethod}`);
      }

      if (!analysis.isDiagonallyDominant) {
        console.warn('Warning: Matrix is not diagonally dominant. Convergence not guaranteed.');
      }

      // Set up solver
      const config: SolverConfig = {
        method: options.method as any,
        epsilon: parseFloat(options.epsilon),
        maxIterations: parseInt(options.maxIterations),
        timeout: options.timeout ? parseInt(options.timeout) : undefined,
        enableProgress: options.verbose
      };

      console.log(`Solving with method: ${config.method}`);
      console.log(`Tolerance: ${config.epsilon}`);

      // Solve
      const startTime = Date.now();
      const result = await SolverTools.solve({
        matrix: matrixData,
        vector: vectorData,
        ...config
      });

      const elapsed = Date.now() - startTime;

      // Display results
      console.log('\\nSolution completed!');
      console.log(`  Converged: ${result.converged}`);
      console.log(`  Iterations: ${result.iterations}`);
      console.log(`  Final residual: ${result.residual.toExponential(3)}`);
      console.log(`  Solve time: ${elapsed}ms`);
      console.log(`  Memory used: ${result.memoryUsed}MB`);

      if (options.verbose && 'efficiency' in result) {
        console.log(`  Convergence rate: ${result.efficiency.convergenceRate.toFixed(6)}`);
        console.log(`  Time per iteration: ${result.efficiency.timePerIteration.toFixed(2)}ms`);
      }

      // Save solution
      if (options.output) {
        const output = {
          solution: result.solution,
          metadata: {
            converged: result.converged,
            iterations: result.iterations,
            residual: result.residual,
            method: result.method,
            solveTime: elapsed,
            timestamp: new Date().toISOString()
          }
        };

        writeFileSync(options.output, JSON.stringify(output, null, 2));
        console.log(`Solution saved to: ${options.output}`);
      } else {
        console.log('\\nSolution vector:');
        console.log(result.solution.slice(0, Math.min(10, result.solution.length)));
        if (result.solution.length > 10) {
          console.log(`... (${result.solution.length - 10} more elements)`);
        }
      }

    } catch (error) {
      console.error('Solve failed:', error instanceof Error ? error.message : error);
      process.exit(1);
    }
  });

// Analyze command
program
  .command('analyze')
  .description('Analyze a matrix for solvability')
  .requiredOption('-m, --matrix <file>', 'Matrix file (JSON format)')
  .option('-o, --output <file>', 'Output file for analysis')
  .option('--full', 'Perform full analysis including condition estimation')
  .action(async (options) => {
    try {
      console.log(`Matrix Analyzer v${VERSION}`);

      // Load matrix
      if (!existsSync(options.matrix)) {
        throw new Error(`Matrix file not found: ${options.matrix}`);
      }
      const matrixData = JSON.parse(readFileSync(options.matrix, 'utf8'));

      console.log(`Analyzing matrix: ${matrixData.rows}x${matrixData.cols} (${matrixData.format})`);

      // Perform analysis
      const analysis = MatrixTools.analyzeMatrix({
        matrix: matrixData,
        checkDominance: true,
        computeGap: options.full,
        estimateCondition: options.full,
        checkSymmetry: true
      });

      // Display results
      console.log('\\n=== Matrix Analysis ===');
      console.log(`Size: ${analysis.size.rows} x ${analysis.size.cols}`);
      console.log(`Format: ${matrixData.format}`);
      console.log(`Sparsity: ${(analysis.sparsity * 100).toFixed(1)}%`);
      console.log(`Symmetric: ${analysis.isSymmetric}`);
      console.log();

      console.log('=== Diagonal Dominance ===');
      console.log(`Diagonally dominant: ${analysis.isDiagonallyDominant}`);
      console.log(`Dominance type: ${analysis.dominanceType}`);
      console.log(`Dominance strength: ${analysis.dominanceStrength.toFixed(4)}`);
      console.log();

      console.log('=== Performance Predictions ===');
      console.log(`Expected complexity: ${analysis.performance.expectedComplexity}`);
      console.log(`Memory usage: ${analysis.performance.memoryUsage}`);
      console.log(`Recommended method: ${analysis.performance.recommendedMethod}`);
      console.log();

      console.log('=== Visual Metrics ===');
      console.log(`Bandwidth: ${analysis.visualMetrics.bandwidth}`);
      console.log(`Profile metric: ${analysis.visualMetrics.profileMetric}`);
      console.log(`Fill ratio: ${(analysis.visualMetrics.fillRatio * 100).toFixed(1)}%`);
      console.log();

      if (analysis.recommendations.length > 0) {
        console.log('=== Recommendations ===');
        analysis.recommendations.forEach((rec, i) => {
          console.log(`${i + 1}. ${rec}`);
        });
        console.log();
      }

      // Save analysis
      if (options.output) {
        writeFileSync(options.output, JSON.stringify(analysis, null, 2));
        console.log(`Analysis saved to: ${options.output}`);
      }

    } catch (error) {
      console.error('Analysis failed:', error instanceof Error ? error.message : error);
      process.exit(1);
    }
  });

// PageRank command
program
  .command('pagerank')
  .description('Compute PageRank for a graph')
  .requiredOption('-g, --graph <file>', 'Adjacency matrix file (JSON format)')
  .option('-o, --output <file>', 'Output file for PageRank results')
  .option('--damping <value>', 'Damping factor', '0.85')
  .option('--epsilon <value>', 'Convergence tolerance', '1e-6')
  .option('--max-iterations <value>', 'Maximum iterations', '1000')
  .option('--top <n>', 'Show top N nodes', '10')
  .action(async (options) => {
    try {
      console.log(`PageRank Calculator v${VERSION}`);

      // Load graph
      if (!existsSync(options.graph)) {
        throw new Error(`Graph file not found: ${options.graph}`);
      }
      const graphData = JSON.parse(readFileSync(options.graph, 'utf8'));

      console.log(`Computing PageRank for graph: ${graphData.rows}x${graphData.cols}`);

      // Compute PageRank
      const result = await GraphTools.pageRank({
        adjacency: graphData,
        damping: parseFloat(options.damping),
        epsilon: parseFloat(options.epsilon),
        maxIterations: parseInt(options.maxIterations)
      });

      // Display results
      console.log('\\n=== PageRank Results ===');
      console.log(`Total score: ${result.statistics.totalScore.toFixed(6)}`);
      console.log(`Max score: ${result.statistics.maxScore.toExponential(3)}`);
      console.log(`Min score: ${result.statistics.minScore.toExponential(3)}`);
      console.log(`Mean: ${result.statistics.mean.toExponential(3)}`);
      console.log(`Standard deviation: ${result.statistics.standardDeviation.toExponential(3)}`);
      console.log(`Entropy: ${result.statistics.entropy.toFixed(4)}`);
      console.log();

      const topN = parseInt(options.top);
      console.log(`=== Top ${topN} Nodes ===`);
      result.topNodes.slice(0, topN).forEach((item, i) => {
        console.log(`${i + 1}. Node ${item.node}: ${item.score.toExponential(4)}`);
      });

      // Save results
      if (options.output) {
        writeFileSync(options.output, JSON.stringify(result, null, 2));
        console.log(`\\nPageRank results saved to: ${options.output}`);
      }

    } catch (error) {
      console.error('PageRank computation failed:', error instanceof Error ? error.message : error);
      process.exit(1);
    }
  });

// Generate test matrix command
program
  .command('generate')
  .description('Generate test matrices')
  .requiredOption('-t, --type <type>', 'Matrix type (diagonally-dominant|laplacian|random-sparse|tridiagonal)')
  .requiredOption('-s, --size <size>', 'Matrix size')
  .option('-o, --output <file>', 'Output file for matrix')
  .option('--strength <value>', 'Diagonal dominance strength', '2.0')
  .option('--density <value>', 'Sparsity density', '0.1')
  .option('--connectivity <value>', 'Graph connectivity', '0.1')
  .action(async (options) => {
    try {
      console.log(`Matrix Generator v${VERSION}`);

      const size = parseInt(options.size);
      if (size <= 0 || size > 100000) {
        throw new Error('Size must be between 1 and 100000');
      }

      console.log(`Generating ${options.type} matrix of size ${size}x${size}`);

      const params = {
        strength: parseFloat(options.strength),
        density: parseFloat(options.density),
        connectivity: parseFloat(options.connectivity)
      };

      const matrix = MatrixTools.generateTestMatrix(options.type, size, params);

      console.log(`Generated matrix: ${matrix.rows}x${matrix.cols} (${matrix.format})`);

      // Quick analysis
      const analysis = MatrixTools.analyzeMatrix({ matrix });
      console.log(`Diagonally dominant: ${analysis.isDiagonallyDominant}`);
      console.log(`Sparsity: ${(analysis.sparsity * 100).toFixed(1)}%`);

      // Save matrix
      const outputFile = options.output || `${options.type}_${size}x${size}.json`;
      writeFileSync(outputFile, JSON.stringify(matrix, null, 2));
      console.log(`Matrix saved to: ${outputFile}`);

    } catch (error) {
      console.error('Matrix generation failed:', error instanceof Error ? error.message : error);
      process.exit(1);
    }
  });

// Help command
program
  .command('help-examples')
  .description('Show usage examples')
  .action(() => {
    console.log(`
Sublinear Solver MCP - Usage Examples

1. Start MCP Server:
   npx sublinear-solver-mcp serve

2. Solve a linear system:
   npx sublinear-solver-mcp solve -m matrix.json -b vector.json -o solution.json

3. Analyze a matrix:
   npx sublinear-solver-mcp analyze -m matrix.json --full

4. Compute PageRank:
   npx sublinear-solver-mcp pagerank -g graph.json --top 20

5. Generate test matrices:
   npx sublinear-solver-mcp generate -t diagonally-dominant -s 1000 -o test_matrix.json

Matrix File Format (JSON):
{
  "rows": 3,
  "cols": 3,
  "format": "dense",
  "data": [
    [4, -1, 0],
    [-1, 4, -1],
    [0, -1, 4]
  ]
}

Vector File Format (JSON):
[1, 2, 1]

For MCP integration with Claude Desktop, add to your config:
{
  "mcpServers": {
    "sublinear-solver": {
      "command": "npx",
      "args": ["sublinear-solver-mcp", "serve"]
    }
  }
}
`);
  });

// Parse command line arguments
program.parse();

// Default action - show help
if (!process.argv.slice(2).length) {
  program.outputHelp();
}