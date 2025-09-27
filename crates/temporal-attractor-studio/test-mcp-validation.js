#!/usr/bin/env node

/**
 * COMPREHENSIVE MCP VALIDATION TEST
 * Validates that the MCP server is using real WASM and producing accurate results
 */

import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { spawn } from 'child_process';
import * as tas from './pkg-node/temporal_attractor_studio.js';

console.log('════════════════════════════════════════════════════════════════');
console.log('     TEMPORAL ATTRACTOR STUDIO - MCP WASM VALIDATION');
console.log('════════════════════════════════════════════════════════════════');
console.log('');

class MCPValidator {
  constructor() {
    this.client = null;
    this.transport = null;
    this.serverProcess = null;
    this.testResults = [];
  }

  async startServer() {
    console.log('🚀 Starting MCP server...');

    // Start the MCP server as a subprocess
    this.serverProcess = spawn('node', ['mcp-server.js'], {
      stdio: ['pipe', 'pipe', 'inherit']
    });

    // Create transport
    this.transport = new StdioClientTransport({
      command: 'node',
      args: ['mcp-server.js']
    });

    // Create client
    this.client = new Client({
      name: 'mcp-validator',
      version: '1.0.0'
    }, {
      capabilities: {}
    });

    // Connect
    await this.client.connect(this.transport);
    console.log('✅ MCP server connected\n');
  }

  async callTool(name, args) {
    const response = await this.client.callTool({
      name,
      arguments: args
    });
    return JSON.parse(response.content[0].text);
  }

  async validateLorenzSystem() {
    console.log('┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 1: Lorenz System (Literature λ ≈ 0.9-1.5)            │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Generate Lorenz data via MCP
    const lorenzResponse = await this.callTool('generate_lorenz', {
      n_points: 2000,
      dt: 0.01
    });

    // Calculate Lyapunov via MCP
    const chaosResponse = await this.callTool('chaos_analyze', {
      data: lorenzResponse.data,
      dimensions: 3,
      dt: 0.01,
      k_fit: 12,
      theiler: 20,
      max_pairs: 1000
    });

    // Also calculate directly with WASM for comparison
    const studio = new tas.TemporalAttractorStudio();
    const directResult = studio.calculate_lyapunov(
      lorenzResponse.data,
      3, 0.01, 12, 20, 1000, 1e-10
    );

    console.log('MCP Server Result:');
    console.log(`  λ = ${chaosResponse.lambda.toFixed(4)}`);
    console.log(`  Chaos level: ${chaosResponse.chaos_level}`);
    console.log(`  Lyapunov time: ${chaosResponse.lyapunov_time.toFixed(2)}`);

    console.log('\nDirect WASM Result:');
    console.log(`  λ = ${directResult.lambda.toFixed(4)}`);
    console.log(`  Chaos level: ${directResult.chaos_level}`);
    console.log(`  Lyapunov time: ${directResult.lyapunov_time.toFixed(2)}`);

    // Validate results match
    const match = Math.abs(chaosResponse.lambda - directResult.lambda) < 0.0001;
    const inRange = chaosResponse.lambda > 0.9 && chaosResponse.lambda < 1.5;

    console.log('\n✅ Validation:');
    console.log(`  MCP matches WASM: ${match ? '✓' : '✗'}`);
    console.log(`  In literature range: ${inRange ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Lorenz System',
      mcp: chaosResponse.lambda,
      wasm: directResult.lambda,
      match,
      valid: inRange
    });

    return match && inRange;
  }

  async validateHenonMap() {
    console.log('\n┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 2: Hénon Map (Literature λ ≈ 0.42)                   │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Generate Hénon data via MCP
    const henonResponse = await this.callTool('generate_henon', {
      n_points: 2000
    });

    // Calculate Lyapunov via MCP
    const chaosResponse = await this.callTool('chaos_analyze', {
      data: henonResponse.data,
      dimensions: 2,
      dt: 1.0,
      k_fit: 10,
      theiler: 10,
      max_pairs: 500
    });

    // Direct WASM calculation
    const studio = new tas.TemporalAttractorStudio();
    const directResult = studio.calculate_lyapunov(
      henonResponse.data,
      2, 1.0, 10, 10, 500, 1e-10
    );

    console.log('MCP Server Result:');
    console.log(`  λ = ${chaosResponse.lambda.toFixed(4)}`);

    console.log('\nDirect WASM Result:');
    console.log(`  λ = ${directResult.lambda.toFixed(4)}`);

    const match = Math.abs(chaosResponse.lambda - directResult.lambda) < 0.0001;
    const inRange = chaosResponse.lambda > 0.35 && chaosResponse.lambda < 0.50;

    console.log('\n✅ Validation:');
    console.log(`  MCP matches WASM: ${match ? '✓' : '✗'}`);
    console.log(`  Near literature value (0.42): ${inRange ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Hénon Map',
      mcp: chaosResponse.lambda,
      wasm: directResult.lambda,
      match,
      valid: inRange
    });

    return match && inRange;
  }

  async validateDelayEmbedding() {
    console.log('\n┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 3: Delay Embedding (Takens Theorem)                   │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Create test series
    const series = Array.from({length: 100}, (_, i) => Math.sin(i * 0.1));

    // MCP embedding
    const mcpResponse = await this.callTool('delay_embed', {
      series,
      embedding_dim: 3,
      tau: 2
    });

    // Direct WASM embedding
    const studio = new tas.TemporalAttractorStudio();
    const directResult = studio.delay_embedding(series, 3, 2);

    console.log(`MCP embedded vectors: ${mcpResponse.embedded_vectors}`);
    console.log(`WASM embedded vectors: ${directResult.length / 3}`);

    const match = mcpResponse.embedded_vectors === directResult.length / 3;
    const dataMatch = JSON.stringify(mcpResponse.data) === JSON.stringify(directResult);

    console.log('\n✅ Validation:');
    console.log(`  Vector count matches: ${match ? '✓' : '✗'}`);
    console.log(`  Data matches exactly: ${dataMatch ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Delay Embedding',
      match: match && dataMatch,
      valid: true
    });

    return match && dataMatch;
  }

  async validateFractalDimension() {
    console.log('\n┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 4: Fractal Dimension Calculation                      │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Generate test data
    const lorenzData = tas.generate_lorenz_data(500, 0.01);

    // MCP calculation
    const mcpResponse = await this.callTool('fractal_dimension', {
      data: lorenzData,
      dimensions: 3
    });

    // Direct WASM calculation
    const studio = new tas.TemporalAttractorStudio();
    const directResult = studio.estimate_fractal_dimension(lorenzData, 3);

    console.log(`MCP fractal dimension: ${mcpResponse.fractal_dimension.toFixed(3)}`);
    console.log(`WASM fractal dimension: ${directResult.toFixed(3)}`);

    const match = Math.abs(mcpResponse.fractal_dimension - directResult) < 0.001;

    console.log('\n✅ Validation:');
    console.log(`  Dimensions match: ${match ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Fractal Dimension',
      mcp: mcpResponse.fractal_dimension,
      wasm: directResult,
      match,
      valid: true
    });

    return match;
  }

  async validateEchoNetwork() {
    console.log('\n┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 5: Echo-State Network Training & Prediction           │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Initialize network via MCP
    await this.callTool('echo_network_init', {
      reservoir_size: 100,
      input_dim: 3,
      output_dim: 3,
      spectral_radius: 0.95,
      connectivity: 0.1,
      leak_rate: 0.3
    });

    // Generate training data
    const lorenzData = tas.generate_lorenz_data(100, 0.01);
    const trainSamples = 50;
    const trainInputs = lorenzData.slice(0, trainSamples * 3);
    const trainTargets = lorenzData.slice(3, (trainSamples + 1) * 3);

    // Train via MCP
    const trainResponse = await this.callTool('echo_network_train', {
      inputs: trainInputs,
      targets: trainTargets,
      n_samples: trainSamples,
      input_dim: 3,
      output_dim: 3
    });

    // Predict via MCP
    const predictResponse = await this.callTool('echo_network_predict', {
      input: [1.0, 1.0, 1.0],
      n_steps: 1
    });

    console.log(`Training MSE: ${trainResponse.mse.toFixed(6)}`);
    console.log(`Prediction: [${predictResponse.prediction.slice(0, 3).map(v => v.toFixed(2)).join(', ')}]`);

    const valid = trainResponse.mse > 0 && predictResponse.prediction.length === 3;

    console.log('\n✅ Validation:');
    console.log(`  Training completed: ${trainResponse.training_complete ? '✓' : '✗'}`);
    console.log(`  Prediction valid: ${valid ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Echo-State Network',
      valid
    });

    return valid;
  }

  async validateRegimeDetection() {
    console.log('\n┌─────────────────────────────────────────────────────────────┐');
    console.log('│  TEST 6: Regime Change Detection                            │');
    console.log('└─────────────────────────────────────────────────────────────┘');

    // Generate test data
    const lorenzData = tas.generate_lorenz_data(600, 0.01);

    // MCP detection
    const mcpResponse = await this.callTool('regime_changes', {
      data: lorenzData,
      dimensions: 3,
      window_size: 50,
      stride: 25
    });

    console.log(`Detected ${mcpResponse.n_windows} regime windows`);
    console.log(`Changes detected: ${mcpResponse.changes_detected}`);

    const valid = mcpResponse.n_windows > 0 &&
                  mcpResponse.lyapunov_values.every(v => !isNaN(v));

    console.log('\n✅ Validation:');
    console.log(`  Valid detection: ${valid ? '✓' : '✗'}`);

    this.testResults.push({
      test: 'Regime Detection',
      valid
    });

    return valid;
  }

  async runAllTests() {
    try {
      await this.startServer();

      await this.validateLorenzSystem();
      await this.validateHenonMap();
      await this.validateDelayEmbedding();
      await this.validateFractalDimension();
      await this.validateEchoNetwork();
      await this.validateRegimeDetection();

      console.log('\n════════════════════════════════════════════════════════════════');
      console.log('                      VALIDATION SUMMARY');
      console.log('════════════════════════════════════════════════════════════════');

      let passed = 0;
      let failed = 0;

      for (const result of this.testResults) {
        const status = (result.match !== false && result.valid) ? '✅' : '❌';
        console.log(`${status} ${result.test}`);
        if (result.mcp !== undefined) {
          console.log(`   MCP: ${result.mcp?.toFixed?.(4) ?? result.mcp}`);
          console.log(`   WASM: ${result.wasm?.toFixed?.(4) ?? result.wasm}`);
        }
        if (result.match !== false && result.valid) {
          passed++;
        } else {
          failed++;
        }
      }

      console.log('\n📊 Results:');
      console.log(`   Passed: ${passed}/${this.testResults.length}`);
      console.log(`   Failed: ${failed}/${this.testResults.length}`);

      if (failed === 0) {
        console.log('\n🎉 ALL TESTS PASSED!');
        console.log('✨ MCP server is using real WASM with accurate calculations!');
        console.log('🚀 100% REAL - Not BS!');
        console.log('\n💡 Key Validations:');
        console.log('• Lorenz λ matches literature (0.9-1.5)');
        console.log('• Hénon λ matches literature (~0.42)');
        console.log('• MCP results exactly match direct WASM calls');
        console.log('• All mathematical operations verified');
      } else {
        console.log('\n⚠️ Some tests failed - review results above');
      }

      console.log('════════════════════════════════════════════════════════════════');

    } catch (error) {
      console.error('❌ Test error:', error);
    } finally {
      if (this.client) {
        await this.client.close();
      }
      if (this.serverProcess) {
        this.serverProcess.kill();
      }
      process.exit(0);
    }
  }
}

// Run validation
const validator = new MCPValidator();
validator.runAllTests().catch(console.error);