/**
 * Benchmark suite for Temporal Neural Solver WASM
 */

const { TemporalNeuralSolver, benchmark, version } = require('./temporal-neural-solver.js');

console.log('⚡ Temporal Neural Solver Benchmark Suite');
console.log('   Version:', version());
console.log('   Platform: Node.js WASM');
console.log('   Date:', new Date().toISOString());
console.log('');

function formatNumber(num) {
    return Math.round(num).toLocaleString();
}

function percentile(values, p) {
    const sorted = values.slice().sort((a, b) => a - b);
    const index = Math.floor(sorted.length * p / 100);
    return sorted[index];
}

async function runBenchmark(name, iterations, fn) {
    console.log(`📊 ${name}`);
    console.log(`   Iterations: ${formatNumber(iterations)}`);

    // Warm up
    for (let i = 0; i < 10; i++) {
        await fn();
    }

    // Measure
    const timings = [];
    const start = Date.now();

    for (let i = 0; i < iterations; i++) {
        const iterStart = performance.now();
        await fn();
        const iterEnd = performance.now();
        timings.push(iterEnd - iterStart);
    }

    const end = Date.now();
    const totalTime = end - start;

    // Calculate statistics
    const min = Math.min(...timings);
    const max = Math.max(...timings);
    const avg = timings.reduce((a, b) => a + b, 0) / timings.length;
    const p50 = percentile(timings, 50);
    const p90 = percentile(timings, 90);
    const p99 = percentile(timings, 99);
    const p999 = percentile(timings, 99.9);

    console.log(`   Total time: ${totalTime.toFixed(2)} ms`);
    console.log(`   Min latency: ${(min * 1000).toFixed(2)} µs`);
    console.log(`   Avg latency: ${(avg * 1000).toFixed(2)} µs`);
    console.log(`   P50 latency: ${(p50 * 1000).toFixed(2)} µs`);
    console.log(`   P90 latency: ${(p90 * 1000).toFixed(2)} µs`);
    console.log(`   P99 latency: ${(p99 * 1000).toFixed(2)} µs`);
    console.log(`   P99.9 latency: ${(p999 * 1000).toFixed(2)} µs`);
    console.log(`   Max latency: ${(max * 1000).toFixed(2)} µs`);
    console.log(`   Throughput: ${formatNumber(1000 / avg)} ops/sec`);
    console.log('');

    return { min, max, avg, p50, p90, p99, p999 };
}

async function main() {
    const solver = new TemporalNeuralSolver();

    // Benchmark 1: Single prediction
    const input = new Float32Array(128).fill(0.5);
    await runBenchmark('Single Prediction', 10000, () => {
        solver.predict(input);
    });

    // Benchmark 2: Varied inputs
    const inputs = [];
    for (let i = 0; i < 100; i++) {
        const arr = new Float32Array(128);
        for (let j = 0; j < 128; j++) {
            arr[j] = Math.sin(i * 0.1 + j * 0.01);
        }
        inputs.push(arr);
    }

    let inputIndex = 0;
    await runBenchmark('Varied Input Prediction', 10000, () => {
        solver.predict(inputs[inputIndex % 100]);
        inputIndex++;
    });

    // Benchmark 3: Batch processing
    const batchSizes = [10, 100, 1000];
    for (const batchSize of batchSizes) {
        const batchInput = new Float32Array(128 * batchSize);
        for (let i = 0; i < batchSize * 128; i++) {
            batchInput[i] = Math.random();
        }

        await runBenchmark(`Batch Processing (${batchSize} samples)`, 100, () => {
            solver.predict_batch(batchInput);
        });
    }

    // Benchmark 4: With state reset
    await runBenchmark('Prediction with State Reset', 5000, () => {
        solver.predict(input);
        solver.reset_state();
    });

    // Benchmark 5: Native benchmark function
    console.log('📊 Native Benchmark Function');
    const benchResults = [
        benchmark(100),
        benchmark(1000),
        benchmark(10000),
    ];

    for (const result of benchResults) {
        console.log(`   ${formatNumber(result.iterations)} iterations:`);
        console.log(`     Total: ${result.total_time_ms.toFixed(2)} ms`);
        console.log(`     Avg: ${result.avg_latency_us.toFixed(2)} µs`);
        console.log(`     Throughput: ${formatNumber(result.throughput_ops_sec)} ops/sec`);
    }

    // Performance summary
    console.log('\n📈 Performance Summary:');

    const nativeBench = benchmark(10000);
    const avgLatencyUs = nativeBench.avg_latency_us;

    if (avgLatencyUs < 1.0) {
        console.log('   ✅ SUB-MICROSECOND INFERENCE ACHIEVED!');
        console.log(`   🎯 Average latency: ${avgLatencyUs.toFixed(3)} µs`);
    } else if (avgLatencyUs < 10.0) {
        console.log('   ⚡ ULTRA-FAST INFERENCE (<10µs)');
        console.log(`   🎯 Average latency: ${avgLatencyUs.toFixed(2)} µs`);
    } else if (avgLatencyUs < 100.0) {
        console.log('   🚀 FAST INFERENCE (<100µs)');
        console.log(`   🎯 Average latency: ${avgLatencyUs.toFixed(2)} µs`);
    } else {
        console.log(`   📊 Average latency: ${avgLatencyUs.toFixed(2)} µs`);
    }

    console.log(`   💪 Throughput: ${formatNumber(nativeBench.throughput_ops_sec)} operations/second`);

    // System info
    console.log('\n💻 System Information:');
    console.log('   Node.js:', process.version);
    console.log('   Platform:', process.platform);
    console.log('   Architecture:', process.arch);
    console.log('   CPUs:', require('os').cpus().length);
    console.log('   Memory:', Math.round(require('os').totalmem() / 1024 / 1024 / 1024), 'GB');
}

main().catch(console.error);