#!/usr/bin/env node

/**
 * GOAP MCP CLI
 * Command-line interface for the GOAP MCP server
 */

import { Command } from 'commander';
import { GoapMCPServer } from './mcp/server.js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const program = new Command();

program
  .name('goalie-mcp')
  .description('GOAP (Goal-Oriented Action Planning) MCP Server with Advanced Reasoning Engine')
  .version('1.0.0');

program
  .command('start')
  .description('Start the MCP server')
  .option('--verbose', 'Enable verbose logging')
  .option('--plugins <paths>', 'Comma-separated paths to external plugins')
  .option('--extensions <paths>', 'Comma-separated paths to external extensions')
  .option('--port <number>', 'Port to run HTTP server on (if not stdio)')
  .action(async (options) => {
    try {
      // Set environment variables from options
      if (options.plugins) {
        process.env.GOAP_PLUGINS = options.plugins;
      }
      if (options.extensions) {
        process.env.GOAP_EXTENSIONS = options.extensions;
      }

      if (options.verbose) {
        console.error('🔧 Verbose logging enabled');
        console.error('🌐 Environment:');
        console.error(`  • Perplexity API Key: ${process.env.PERPLEXITY_API_KEY ? '✅ Set' : '❌ Missing'}`);
        console.error(`  • Plugins: ${process.env.GOAP_PLUGINS || 'None'}`);
        console.error(`  • Extensions: ${process.env.GOAP_EXTENSIONS || 'None'}`);
      }

      // Validate required environment variables
      if (!process.env.PERPLEXITY_API_KEY) {
        console.error('❌ ERROR: PERPLEXITY_API_KEY environment variable is required');
        console.error('💡 Get your API key from: https://www.perplexity.ai/settings/api');
        process.exit(1);
      }

      const server = new GoapMCPServer();
      await server.initialize();
      await server.run();

    } catch (error) {
      console.error('💥 Failed to start GOAP MCP server:', error);
      process.exit(1);
    }
  });

program
  .command('test')
  .description('Test the GOAP planner with a sample query')
  .option('--query <string>', 'Test query', 'What are the latest developments in AI?')
  .option('--explain', 'Show plan explanation without executing')
  .action(async (options) => {
    try {
      const { GoapMCPTools } = await import('./mcp/tools.js');
      const tools = new GoapMCPTools();
      await tools.initialize();

      console.log('🧪 Testing GOAP planner...');
      console.log(`📝 Query: ${options.query}`);

      if (options.explain) {
        const explanation = await tools.executePlanExplain({
          query: options.query,
          showSteps: true,
          showReasoning: true
        });

        console.log('📋 Plan Explanation:');
        console.log(JSON.stringify(explanation, null, 2));
      } else {
        const result = await tools.executeGoapSearch({
          query: options.query,
          enableReasoning: true,
          maxResults: 5
        });

        console.log('✅ Test Results:');
        console.log(`📝 Answer: ${result.answer.substring(0, 200)}...`);
        console.log(`📚 Citations: ${result.citations.length}`);
        console.log(`⏱️ Execution time: ${result.metadata.executionTime}ms`);
        console.log(`🔄 Replanned: ${result.metadata.replanned}`);
      }

    } catch (error) {
      console.error('💥 Test failed:', error);
      process.exit(1);
    }
  });

program
  .command('validate')
  .description('Validate configuration and dependencies')
  .action(async () => {
    console.log('🔍 Validating GOAP MCP configuration...');

    // Check environment variables
    const checks = [
      {
        name: 'Perplexity API Key',
        check: () => !!process.env.PERPLEXITY_API_KEY,
        fix: 'Set PERPLEXITY_API_KEY environment variable'
      },
      {
        name: 'Node.js version',
        check: () => {
          const version = process.version;
          const major = parseInt(version.slice(1).split('.')[0]);
          return major >= 18;
        },
        fix: 'Update Node.js to version 18 or higher'
      }
    ];

    let allPassed = true;

    for (const check of checks) {
      const passed = check.check();
      const status = passed ? '✅' : '❌';
      console.log(`${status} ${check.name}`);

      if (!passed) {
        console.log(`   💡 ${check.fix}`);
        allPassed = false;
      }
    }

    // Test Advanced Reasoning Engine WASM
    try {
      const { AdvancedReasoningEngine } = await import('./core/advanced-reasoning-engine');
      const engine = new AdvancedReasoningEngine();
      await engine.initialize();
      console.log('✅ Advanced Reasoning Engine integration');
    } catch (error) {
      console.log('⚠️ Advanced Reasoning Engine (will use fallback)');
    }

    // Test MCP SDK
    try {
      await import('@modelcontextprotocol/sdk/server/index.js');
      console.log('✅ MCP SDK');
    } catch (error) {
      console.log('❌ MCP SDK - npm install required');
      allPassed = false;
    }

    if (allPassed) {
      console.log('🎉 All validations passed! Ready to run GOAP MCP server.');
    } else {
      console.log('⚠️ Some validations failed. Please fix the issues above.');
      process.exit(1);
    }
  });

program
  .command('info')
  .description('Show system information and capabilities')
  .action(async () => {
    console.log('🎯 GOAP MCP Server Information');
    console.log('==============================');
    console.log('');

    console.log('📋 Core Features:');
    console.log('  • STRIPS-style preconditions and effects');
    console.log('  • A* pathfinding for optimal plans');
    console.log('  • Dynamic re-planning on failure');
    console.log('  • Advanced Reasoning Engine enhanced reasoning');
    console.log('  • Perplexity API integration');
    console.log('  • Extensible plugin system');
    console.log('');

    console.log('🔧 Available Tools:');
    console.log('  • goap.search - Intelligent search with planning');
    console.log('  • goap.plan.explain - Plan explanation');
    console.log('  • search.raw - Direct Perplexity search');
    console.log('');

    console.log('🎪 Plugin System:');
    console.log('  • cost-tracker - Track execution costs');
    console.log('  • performance-monitor - Monitor execution performance');
    console.log('  • logger - Comprehensive logging');
    console.log('  • query-diversifier - Enhance search queries');
    console.log('');

    console.log('🧠 Advanced Reasoning Engine:');
    console.log('  • Pattern analysis algorithms');
    console.log('  • Predictive modeling capabilities');
    console.log('  • State-enhanced reasoning');
    console.log('  • Multi-agent coordination');
    console.log('');

    console.log('🌟 Advantages over standard web search:');
    console.log('  • Multi-step planning with dependencies');
    console.log('  • Automatic query optimization');
    console.log('  • Enhanced reasoning with Advanced Reasoning Engine');
    console.log('  • Dynamic re-planning on failures');
    console.log('  • Comprehensive answer verification');
    console.log('  • Cost optimization with A* pathfinding');
    console.log('  • Extensible plugin architecture');
  });

// Default command runs the server
program.parse();

// If no command provided, show help
if (!process.argv.slice(2).length) {
  program.outputHelp();
}