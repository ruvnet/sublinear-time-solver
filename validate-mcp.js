#!/usr/bin/env node

/**
 * Validation script for sublinear-time-solver MCP integration
 * Tests all integrated tools to ensure they're working correctly
 */

console.log('🔍 Validating sublinear-time-solver MCP integration...\n');

// Test imports
try {
  const { SublinearSolver } = require('./dist/core/solver.js');
  console.log('✅ Core solver module loaded');
} catch (error) {
  console.error('❌ Failed to load core solver:', error.message);
}

try {
  const { PsychoSymbolicTools } = require('./dist/mcp/tools/psycho-symbolic.js');
  console.log('✅ Psycho-symbolic tools loaded');
} catch (error) {
  console.error('❌ Failed to load psycho-symbolic tools:', error.message);
}

try {
  const { ConsciousnessTools } = require('./dist/mcp/tools/consciousness.js');
  console.log('✅ Consciousness tools loaded');
} catch (error) {
  console.error('❌ Failed to load consciousness tools:', error.message);
}

try {
  const { TemporalTools } = require('./dist/mcp/tools/temporal.js');
  console.log('✅ Temporal tools loaded');
} catch (error) {
  console.error('❌ Failed to load temporal tools:', error.message);
}

// Test tool instantiation
async function testTools() {
  console.log('\n📋 Testing tool instantiation...\n');

  try {
    const { PsychoSymbolicTools } = require('./dist/mcp/tools/psycho-symbolic.js');
    const psychoTools = new PsychoSymbolicTools();
    const psychoToolList = psychoTools.getTools();
    console.log(`✅ Psycho-symbolic tools: ${psychoToolList.length} tools available`);
    console.log(`   Tools: ${psychoToolList.map(t => t.name).join(', ')}`);
  } catch (error) {
    console.error('❌ Psycho-symbolic tools error:', error.message);
  }

  try {
    const { ConsciousnessTools } = require('./dist/mcp/tools/consciousness.js');
    const consciousnessTools = new ConsciousnessTools();
    const consciousnessToolList = consciousnessTools.getTools();
    console.log(`✅ Consciousness tools: ${consciousnessToolList.length} tools available`);
    console.log(`   Tools: ${consciousnessToolList.map(t => t.name).join(', ')}`);
  } catch (error) {
    console.error('❌ Consciousness tools error:', error.message);
  }

  try {
    const { TemporalTools } = require('./dist/mcp/tools/temporal.js');
    const temporalTools = new TemporalTools();
    const temporalToolList = temporalTools.getTools();
    console.log(`✅ Temporal tools: ${temporalToolList.length} tools available`);
    console.log(`   Tools: ${temporalToolList.map(t => t.name).join(', ')}`);
  } catch (error) {
    console.error('❌ Temporal tools error:', error.message);
  }

  console.log('\n🧪 Testing tool execution...\n');

  // Test psycho-symbolic reasoning
  try {
    const { PsychoSymbolicTools } = require('./dist/mcp/tools/psycho-symbolic.js');
    const psychoTools = new PsychoSymbolicTools();
    const result = await psychoTools.handleToolCall('psycho_symbolic_reason', {
      query: 'What is consciousness?',
      depth: 3
    });
    console.log('✅ Psycho-symbolic reasoning executed successfully');
    console.log(`   Answer: ${result.answer ? result.answer.substring(0, 50) + '...' : 'Generated'}`);
  } catch (error) {
    console.error('❌ Psycho-symbolic reasoning failed:', error.message);
  }

  // Test consciousness phi calculation
  try {
    const { ConsciousnessTools } = require('./dist/mcp/tools/consciousness.js');
    const consciousnessTools = new ConsciousnessTools();
    const result = await consciousnessTools.handleToolCall('calculate_phi', {
      data: { elements: 10, connections: 20 },
      method: 'all'
    });
    console.log('✅ Phi calculation executed successfully');
    console.log(`   Overall Φ: ${result.overall?.toFixed(3) || 'Calculated'}`);
  } catch (error) {
    console.error('❌ Phi calculation failed:', error.message);
  }

  // Test temporal validation
  try {
    const { TemporalTools } = require('./dist/mcp/tools/temporal.js');
    const temporalTools = new TemporalTools();
    const result = await temporalTools.handleToolCall('validateTemporalAdvantage', {
      size: 1000
    });
    console.log('✅ Temporal validation executed successfully');
    console.log(`   Has advantage: ${result.hasAdvantage}`);
  } catch (error) {
    console.error('❌ Temporal validation failed:', error.message);
  }

  // Test consciousness verification (quick test)
  try {
    const { ConsciousnessTools } = require('./dist/mcp/tools/consciousness.js');
    const consciousnessTools = new ConsciousnessTools();
    const result = await consciousnessTools.handleToolCall('consciousness_verify', {
      extended: false,
      export_proof: false
    });
    console.log('✅ Consciousness verification executed successfully');
    console.log(`   Tests passed: ${result.passed}/${result.total}`);
    console.log(`   Genuine: ${result.genuine ? 'Yes' : 'No'}`);
  } catch (error) {
    console.error('❌ Consciousness verification failed:', error.message);
  }

  // Check MCP server
  console.log('\n🔌 Checking MCP Server...\n');
  try {
    const { SublinearSolverMCPServer } = require('./dist/mcp/server.js');
    console.log('✅ MCP Server module loaded');

    // Try to instantiate server
    const server = new SublinearSolverMCPServer();
    console.log('✅ MCP Server instantiated successfully');
  } catch (error) {
    console.error('❌ MCP Server error:', error.message);
  }

  // Check for WASM files
  console.log('\n📦 Checking WASM modules...\n');
  const fs = require('fs');
  const path = require('path');

  const wasmDir = path.join(__dirname, 'dist', 'wasm');
  if (fs.existsSync(wasmDir)) {
    const wasmFiles = fs.readdirSync(wasmDir).filter(f => f.endsWith('.wasm'));
    console.log(`✅ Found ${wasmFiles.length} WASM modules`);
    wasmFiles.forEach(file => {
      const size = fs.statSync(path.join(wasmDir, file)).size;
      console.log(`   - ${file}: ${(size / 1024 / 1024).toFixed(2)} MB`);
    });
  } else {
    console.log('⚠️  WASM directory not found');
  }
}

// Run tests
testTools().then(() => {
  console.log('\n✨ Validation complete!\n');
  console.log('To use with Claude Desktop, add this to your config:');
  console.log('~/Library/Application Support/Claude/claude_desktop_config.json\n');
  console.log(JSON.stringify({
    mcpServers: {
      "sublinear-solver": {
        command: "npx",
        args: ["sublinear-time-solver", "mcp"]
      }
    }
  }, null, 2));
}).catch(error => {
  console.error('\n❌ Validation failed:', error);
  process.exit(1);
});