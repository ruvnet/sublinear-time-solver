#!/usr/bin/env node

/**
 * Temporal Attractor Studio CLI
 * Command-line interface for chaos analysis and Lyapunov exponent calculation
 */

import { Command } from 'commander';
import chalk from 'chalk';
import * as tas from './pkg-node/temporal_attractor_studio.js';
import fs from 'fs';
import { spawn } from 'child_process';

const program = new Command();

// Initialize studio
const studio = new tas.TemporalAttractorStudio();

program
  .name('temporal-attractor-studio')
  .description('High-performance chaos analysis and Lyapunov exponent calculation')
  .version(tas.version())
  .alias('tas');

// MCP server command
program
  .command('mcp')
  .description('Start MCP server for Model Context Protocol integration')
  .action(() => {
    console.log(chalk.cyan('🚀 Starting Temporal Attractor Studio MCP server...'));
    const child = spawn('node', ['mcp-server.js'], {
      stdio: 'inherit',
      cwd: process.cwd()
    });

    child.on('error', (err) => {
      console.error(chalk.red('Failed to start MCP server:'), err);
      process.exit(1);
    });
  });

// Analyze command
program
  .command('analyze <file>')
  .description('Calculate Lyapunov exponent from time series data')
  .option('-d, --dimensions <n>', 'dimensions per point', (v) => parseInt(v, 10), 3)
  .option('-t, --dt <value>', 'time step', (v) => parseFloat(v), 0.01)
  .option('-k, --kfit <n>', 'points for linear fitting', (v) => parseInt(v, 10), 12)
  .option('--theiler <n>', 'Theiler window', (v) => parseInt(v, 10), 20)
  .option('--max-pairs <n>', 'maximum trajectory pairs', (v) => parseInt(v, 10), 1000)
  .option('-o, --output <file>', 'output file for results')
  .action((file, options) => {
    try {
      console.log(chalk.cyan(`📊 Analyzing ${file}...`));

      // Read data file
      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.trim().split('\n');
      const data = [];

      for (const line of lines) {
        if (line.trim() && !line.startsWith('#')) {
          const values = line.split(/[\s,]+/).map(parseFloat);
          data.push(...values);
        }
      }

      console.log(chalk.gray(`Loaded ${data.length / options.dimensions} points`));

      // Calculate Lyapunov exponent
      const result = studio.calculate_lyapunov(
        data,
        options.dimensions,
        options.dt,
        options.kfit,
        options.theiler,
        options.maxPairs,
        1e-10
      );

      // Display results
      console.log(chalk.green('\n✅ Analysis Complete:\n'));
      console.log(chalk.white(`  Lyapunov Exponent (λ): ${chalk.yellow(result.lambda.toFixed(4))}`));
      console.log(chalk.white(`  System Type: ${chalk.magenta(result.chaos_level)}`));
      console.log(chalk.white(`  Is Chaotic: ${result.is_chaotic ? chalk.red('Yes') : chalk.green('No')}`));
      console.log(chalk.white(`  Lyapunov Time: ${chalk.cyan(result.lyapunov_time.toFixed(2))} time units`));
      console.log(chalk.white(`  Doubling Time: ${chalk.cyan(result.doubling_time.toFixed(2))} time units`));
      console.log(chalk.white(`  Safe Prediction: ${chalk.blue(result.safe_prediction_steps)} steps`));
      console.log(chalk.white(`  Pairs Found: ${result.pairs_found}`));

      // Save results if requested
      if (options.output) {
        const output = {
          file: file,
          timestamp: new Date().toISOString(),
          parameters: options,
          results: result
        };
        fs.writeFileSync(options.output, JSON.stringify(output, null, 2));
        console.log(chalk.green(`\n📁 Results saved to ${options.output}`));
      }

    } catch (error) {
      console.error(chalk.red('Error:'), error.message);
      process.exit(1);
    }
  });

// Embed command
program
  .command('embed <file>')
  .description('Perform delay embedding on univariate time series')
  .option('-d, --dim <n>', 'embedding dimension', parseInt, 3)
  .option('-t, --tau <n>', 'time delay', parseInt, 1)
  .option('-o, --output <file>', 'output file')
  .action((file, options) => {
    try {
      console.log(chalk.cyan(`🔄 Embedding ${file}...`));

      // Read data
      const content = fs.readFileSync(file, 'utf-8');
      const series = content.trim().split(/[\s,\n]+/)
        .filter(v => v && !v.startsWith('#'))
        .map(parseFloat);

      console.log(chalk.gray(`Input: ${series.length} points`));

      // Perform embedding
      const embedded = studio.delay_embedding(series, options.dim, options.tau);

      console.log(chalk.green(`✅ Embedded into ${embedded.length / options.dim} vectors`));

      // Save if requested
      if (options.output) {
        // Format as rows of embedded vectors
        const vectors = [];
        for (let i = 0; i < embedded.length; i += options.dim) {
          vectors.push(embedded.slice(i, i + options.dim).join(' '));
        }
        fs.writeFileSync(options.output, vectors.join('\n'));
        console.log(chalk.green(`📁 Saved to ${options.output}`));
      }

    } catch (error) {
      console.error(chalk.red('Error:'), error.message);
      process.exit(1);
    }
  });

// Generate command
program
  .command('generate <system>')
  .description('Generate test data (lorenz, henon)')
  .option('-n, --points <n>', 'number of points', (v) => parseInt(v, 10), 1000)
  .option('-t, --dt <value>', 'time step (Lorenz only)', parseFloat, 0.01)
  .option('-o, --output <file>', 'output file')
  .action((system, options) => {
    try {
      let data;
      let dimensions;

      switch (system.toLowerCase()) {
        case 'lorenz':
          console.log(chalk.cyan(`🌀 Generating Lorenz attractor...`));
          data = tas.generate_lorenz_data(options.points, options.dt);
          dimensions = 3;
          break;

        case 'henon':
          console.log(chalk.cyan(`📍 Generating Hénon map...`));
          data = tas.generate_henon_data(options.points);
          dimensions = 2;
          break;

        default:
          console.error(chalk.red(`Unknown system: ${system}`));
          console.log('Available: lorenz, henon');
          process.exit(1);
      }

      console.log(chalk.green(`✅ Generated ${options.points} points`));

      // Format and save data
      const lines = [];
      for (let i = 0; i < data.length; i += dimensions) {
        lines.push(data.slice(i, i + dimensions).join(' '));
      }

      if (options.output) {
        fs.writeFileSync(options.output, lines.join('\n'));
        console.log(chalk.green(`📁 Saved to ${options.output}`));
      } else {
        // Print to stdout
        console.log(lines.slice(0, 10).join('\n'));
        if (lines.length > 10) {
          console.log(chalk.gray(`... (${lines.length - 10} more lines)`));
        }
      }

    } catch (error) {
      console.error(chalk.red('Error:'), error.message);
      process.exit(1);
    }
  });

// Fractal command
program
  .command('fractal <file>')
  .description('Estimate fractal dimension')
  .option('-d, --dimensions <n>', 'dimensions per point', parseInt, 3)
  .action((file, options) => {
    try {
      console.log(chalk.cyan(`📐 Calculating fractal dimension...`));

      // Read data
      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.trim().split('\n');
      const data = [];

      for (const line of lines) {
        if (line.trim() && !line.startsWith('#')) {
          const values = line.split(/[\s,]+/).map(parseFloat);
          data.push(...values);
        }
      }

      // Calculate dimension
      const dimension = studio.estimate_fractal_dimension(data, options.dimensions);

      console.log(chalk.green('\n✅ Fractal Analysis:'));
      console.log(chalk.white(`  Dimension: ${chalk.yellow(dimension.toFixed(3))}`));
      console.log(chalk.white(`  Type: ${
        dimension > 2 ? chalk.magenta('Complex attractor') :
        dimension > 1 ? chalk.cyan('Fractal structure') :
        chalk.green('Simple dynamics')
      }`));

    } catch (error) {
      console.error(chalk.red('Error:'), error.message);
      process.exit(1);
    }
  });

// Interpret command
program
  .command('interpret <lambda>')
  .description('Interpret a Lyapunov exponent value')
  .action((lambda) => {
    try {
      const value = parseFloat(lambda);
      const interpretation = studio.interpret_chaos(value);

      console.log(chalk.cyan('\n📊 Chaos Interpretation:\n'));
      console.log(interpretation);

    } catch (error) {
      console.error(chalk.red('Error:'), error.message);
      process.exit(1);
    }
  });

// Recommend command
program
  .command('recommend')
  .description('Get parameter recommendations')
  .option('-n, --points <n>', 'number of data points', parseInt, 1000)
  .option('-d, --dims <n>', 'dimensions', parseInt, 3)
  .option('-r, --rate <n>', 'sampling rate (Hz)', parseFloat, 100)
  .action((options) => {
    const recommendations = studio.recommend_parameters(
      options.points || 1000,
      options.dims || 3,
      options.rate || 100
    );

    console.log(chalk.cyan('\n🎯 Recommended Parameters:\n'));
    console.log(recommendations);
  });

// Info command
program
  .command('info')
  .description('Show information about the tool')
  .action(() => {
    console.log(chalk.cyan('\n🌌 Temporal Attractor Studio\n'));
    console.log(chalk.white(`Version: ${tas.version()}`));
    console.log(chalk.gray('High-performance chaos analysis in WebAssembly\n'));

    console.log(chalk.yellow('Features:'));
    console.log('  • Lyapunov exponent calculation (FTLE)');
    console.log('  • Delay embedding (Takens theorem)');
    console.log('  • Echo-State Networks for prediction');
    console.log('  • Fractal dimension estimation');
    console.log('  • Regime change detection');
    console.log('  • MCP server for AI integration\n');

    console.log(chalk.yellow('Commands:'));
    console.log('  analyze   - Calculate Lyapunov exponent');
    console.log('  embed     - Perform delay embedding');
    console.log('  generate  - Generate test data');
    console.log('  fractal   - Estimate fractal dimension');
    console.log('  interpret - Interpret chaos level');
    console.log('  recommend - Get parameter recommendations');
    console.log('  mcp       - Start MCP server\n');

    console.log(chalk.gray('GitHub: https://github.com/ruvnet/sublinear-time-solver'));
  });

program.parse();