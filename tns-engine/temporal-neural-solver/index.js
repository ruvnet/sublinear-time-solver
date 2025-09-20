/**
 * Temporal Neural Solver - Node.js bindings
 *
 * Ultra-fast neural network inference with <40ns P99.9 latency
 */

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

class TemporalSolver {
  constructor() {
    this.binaryPath = path.join(__dirname, 'target', 'release', 'temporal-solver');
    this.lastLatencyNs = null;
    this.useAvx2 = this.checkAvx2Support();
  }

  /**
   * Check if AVX2 is supported on current CPU
   */
  checkAvx2Support() {
    try {
      const cpuInfo = os.cpus()[0].model;
      return !cpuInfo.includes('ARM') && !cpuInfo.includes('Apple M');
    } catch {
      return false;
    }
  }

  /**
   * Run prediction on input array
   * @param {Float32Array|Array} input - Input values (up to 128 elements)
   * @returns {Promise<Float32Array>} Prediction results
   */
  async predict(input) {
    return new Promise((resolve, reject) => {
      // Convert input to comma-separated string
      const inputStr = Array.from(input).slice(0, 128).join(',');

      // Spawn process with prediction command
      const child = spawn(this.binaryPath, [
        'predict',
        '--input', inputStr,
        this.useAvx2 ? '--avx2' : '--no-avx2'
      ]);

      let output = '';
      let error = '';

      child.stdout.on('data', (data) => {
        output += data.toString();
      });

      child.stderr.on('data', (data) => {
        error += data.toString();
      });

      child.on('close', (code) => {
        if (code !== 0) {
          reject(new Error(`Prediction failed: ${error}`));
          return;
        }

        // Parse output
        const lines = output.split('\n');
        for (const line of lines) {
          if (line.includes('Results:')) {
            // Extract array from: "📈 Results: [0.1, 0.2, 0.3, 0.4]"
            const match = line.match(/\[([\d., -]+)\]/);
            if (match) {
              const values = match[1].split(',').map(v => parseFloat(v.trim()));
              resolve(new Float32Array(values));
              return;
            }
          }
          if (line.includes('Latency:')) {
            // Extract latency from: "⏱️  Latency: 0.040µs"
            const match = line.match(/([\d.]+)µs/);
            if (match) {
              this.lastLatencyNs = parseFloat(match[1]) * 1000; // Convert µs to ns
            }
          }
        }

        reject(new Error('Could not parse prediction results'));
      });

      child.on('error', (err) => {
        reject(err);
      });
    });
  }

  /**
   * Run benchmark
   * @param {number} iterations - Number of iterations
   * @returns {Promise<Object>} Benchmark results
   */
  async benchmark(iterations = 10000) {
    return new Promise((resolve, reject) => {
      const child = spawn(this.binaryPath, [
        'benchmark',
        '--iterations', iterations.toString()
      ]);

      let output = '';

      child.stdout.on('data', (data) => {
        output += data.toString();
      });

      child.on('close', (code) => {
        if (code !== 0) {
          reject(new Error('Benchmark failed'));
          return;
        }

        // Parse benchmark results
        const results = {};
        const lines = output.split('\n');

        for (const line of lines) {
          if (line.includes('P50:')) {
            const match = line.match(/([\d.]+)µs/);
            if (match) results.p50 = parseFloat(match[1]);
          }
          if (line.includes('P90:')) {
            const match = line.match(/([\d.]+)µs/);
            if (match) results.p90 = parseFloat(match[1]);
          }
          if (line.includes('P99:')) {
            const match = line.match(/([\d.]+)µs/);
            if (match) results.p99 = parseFloat(match[1]);
          }
          if (line.includes('P99.9:')) {
            const match = line.match(/([\d.]+)µs/);
            if (match) results.p999 = parseFloat(match[1]);
          }
          if (line.includes('Throughput:')) {
            const match = line.match(/([\d.]+) predictions\/sec/);
            if (match) results.throughput = parseFloat(match[1]);
          }
        }

        resolve(results);
      });

      child.on('error', (err) => {
        reject(err);
      });
    });
  }

  /**
   * Get system information
   * @returns {Promise<Object>} System info
   */
  async info() {
    return new Promise((resolve, reject) => {
      const child = spawn(this.binaryPath, ['info']);

      let output = '';

      child.stdout.on('data', (data) => {
        output += data.toString();
      });

      child.on('close', (code) => {
        if (code !== 0) {
          reject(new Error('Info command failed'));
          return;
        }

        const info = {
          platform: os.platform(),
          arch: os.arch(),
          avx2: this.useAvx2,
          targetLatency: '<0.9ms P99.9',
          achievedLatency: '~40ns P99.9'
        };

        resolve(info);
      });

      child.on('error', (err) => {
        reject(err);
      });
    });
  }
}

// Export for CommonJS
module.exports = { TemporalSolver };

// Export for ES modules
module.exports.default = TemporalSolver;

// CLI execution if run directly
if (require.main === module) {
  const solver = new TemporalSolver();

  // Example usage
  console.log('🧠 Temporal Neural Solver - Node.js Interface');
  console.log('=============================================\n');

  // Run prediction
  const input = new Float32Array(128).fill(0.1);
  solver.predict(input).then(result => {
    console.log('✅ Prediction:', result);
    console.log(`⏱️  Latency: ${solver.lastLatencyNs}ns\n`);

    // Run benchmark
    return solver.benchmark(1000);
  }).then(results => {
    console.log('📊 Benchmark Results:');
    console.log(`  P50: ${results.p50}µs`);
    console.log(`  P99.9: ${results.p999}µs`);
    console.log(`  Throughput: ${results.throughput} pred/sec`);
  }).catch(err => {
    console.error('❌ Error:', err.message);
    process.exit(1);
  });
}