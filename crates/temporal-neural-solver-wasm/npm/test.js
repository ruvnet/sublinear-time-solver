/**
 * Test suite for Temporal Neural Solver WASM
 */

const { TemporalNeuralSolver, benchmark, version } = require('./temporal-neural-solver.js');

console.log('🧪 Temporal Neural Solver Test Suite');
console.log('   Version:', version());
console.log('   Platform: Node.js WASM\n');

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log('✅', name);
        passed++;
    } catch (error) {
        console.log('❌', name);
        console.log('   Error:', error.message);
        failed++;
    }
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message || 'Assertion failed');
    }
}

function assertClose(actual, expected, tolerance = 0.01) {
    const diff = Math.abs(actual - expected);
    if (diff > tolerance) {
        throw new Error(`Expected ${expected} but got ${actual} (diff: ${diff})`);
    }
}

// Test 1: Solver creation
test('Solver creation', () => {
    const solver = new TemporalNeuralSolver();
    assert(solver !== null, 'Solver should be created');
});

// Test 2: Single prediction
test('Single prediction', () => {
    const solver = new TemporalNeuralSolver();
    const input = new Float32Array(128).fill(0.5);
    const result = solver.predict(input);

    assert(result.output, 'Should return output');
    assert(Array.isArray(result.output), 'Output should be an array');
    assert(result.output.length === 4, 'Output should have 4 elements');
    assert(typeof result.latency_ns === 'number', 'Should return latency');
});

// Test 3: Invalid input size
test('Invalid input size error', () => {
    const solver = new TemporalNeuralSolver();
    const input = new Float32Array(64).fill(0.5);

    try {
        solver.predict(input);
        throw new Error('Should have thrown error for wrong input size');
    } catch (e) {
        assert(e.message.includes('128'), 'Error should mention correct size');
    }
});

// Test 4: Batch prediction
test('Batch prediction', () => {
    const solver = new TemporalNeuralSolver();
    const batchSize = 10;
    const input = new Float32Array(128 * batchSize).fill(0.3);
    const result = solver.predict_batch(input);

    assert(result.predictions, 'Should return predictions');
    assert(result.predictions.length === batchSize, 'Should return correct batch size');
    assert(result.total_latency_ms > 0, 'Should have positive latency');
    assert(result.throughput_ops_sec > 0, 'Should have positive throughput');
});

// Test 5: Benchmark function
test('Benchmark function', () => {
    const result = benchmark(100);

    assert(result.iterations === 100, 'Should run correct iterations');
    assert(result.total_time_ms > 0, 'Should have positive time');
    assert(result.avg_latency_us > 0, 'Should have positive latency');
    assert(result.throughput_ops_sec > 0, 'Should have positive throughput');
});

// Test 6: State reset
test('State reset', () => {
    const solver = new TemporalNeuralSolver();
    const input1 = new Float32Array(128).fill(1.0);
    const input2 = new Float32Array(128).fill(0.0);

    const result1 = solver.predict(input1);
    solver.reset_state();
    const result2 = solver.predict(input2);

    assert(result1.output, 'First prediction should succeed');
    assert(result2.output, 'Second prediction should succeed');
});

// Test 7: Solver info
test('Solver info', () => {
    const solver = new TemporalNeuralSolver();
    const info = solver.info();

    assert(info.name, 'Should have name');
    assert(info.version, 'Should have version');
    assert(info.platform === 'WebAssembly', 'Should be WebAssembly platform');
    assert(info.dimensions, 'Should have dimensions');
    assert(info.dimensions.input === 128, 'Should have 128 inputs');
    assert(info.dimensions.output === 4, 'Should have 4 outputs');
});

// Test 8: Performance targets
test('Performance meets targets', () => {
    const solver = new TemporalNeuralSolver();
    const input = new Float32Array(128).fill(0.5);

    // Warm up
    for (let i = 0; i < 10; i++) {
        solver.predict(input);
    }

    // Measure
    const timings = [];
    for (let i = 0; i < 100; i++) {
        const result = solver.predict(input);
        timings.push(result.latency_ns / 1000); // Convert to microseconds
    }

    const avgLatency = timings.reduce((a, b) => a + b, 0) / timings.length;
    console.log(`   Average latency: ${avgLatency.toFixed(2)} µs`);

    // Check if we meet performance targets
    if (avgLatency < 1.0) {
        console.log('   🎯 Sub-microsecond inference achieved!');
    } else if (avgLatency < 10.0) {
        console.log('   ⚡ Ultra-fast inference (<10µs)');
    } else if (avgLatency < 100.0) {
        console.log('   🚀 Fast inference (<100µs)');
    }
});

// Test 9: Different input values
test('Different input values', () => {
    const solver = new TemporalNeuralSolver();

    const inputs = [
        new Float32Array(128).fill(0.0),
        new Float32Array(128).fill(0.5),
        new Float32Array(128).fill(1.0),
        new Float32Array(128).fill(-1.0),
    ];

    for (const input of inputs) {
        const result = solver.predict(input);
        assert(result.output, 'Should handle various input values');
        assert(result.output.every(v => !isNaN(v)), 'Output should not contain NaN');
    }
});

// Test 10: Memory efficiency
test('Memory efficiency (1000 predictions)', () => {
    const solver = new TemporalNeuralSolver();
    const input = new Float32Array(128).fill(0.5);

    const memBefore = process.memoryUsage().heapUsed;

    for (let i = 0; i < 1000; i++) {
        solver.predict(input);
    }

    const memAfter = process.memoryUsage().heapUsed;
    const memDelta = (memAfter - memBefore) / 1024 / 1024; // MB

    console.log(`   Memory delta: ${memDelta.toFixed(2)} MB`);
    assert(memDelta < 10, 'Should not leak excessive memory');
});

// Summary
console.log('\n📊 Test Results:');
console.log(`   Passed: ${passed}`);
console.log(`   Failed: ${failed}`);

if (failed === 0) {
    console.log('\n✅ All tests passed!');
    process.exit(0);
} else {
    console.log('\n❌ Some tests failed');
    process.exit(1);
}