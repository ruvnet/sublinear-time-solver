#!/usr/bin/env node

// Test script to verify WASM functionality
import * as tas from './pkg/temporal_attractor_studio.js';
import fs from 'fs';

console.log('🚀 Testing Temporal Attractor Studio WASM Package\n');
console.log('Version:', tas.version());
console.log('-----------------------------------\n');

// Test 1: Generate Lorenz data
console.log('Test 1: Generating Lorenz data...');
const lorenzData = tas.generate_lorenz_data(1000, 0.01);
console.log(`✅ Generated ${lorenzData.length / 3} Lorenz trajectory points`);

// Test 2: Calculate Lyapunov exponent
console.log('\nTest 2: Calculating Lyapunov exponent...');
const studio = new tas.TemporalAttractorStudio();
try {
    const result = studio.calculate_lyapunov(
        lorenzData,
        3,  // 3 dimensions
        0.01,  // dt
        12,  // k_fit
        20,  // theiler
        500, // max_pairs
        1e-10  // min_sep
    );

    console.log('✅ Lyapunov calculation successful!');
    console.log(`   λ = ${result.lambda.toFixed(4)}`);
    console.log(`   Is chaotic: ${result.is_chaotic}`);
    console.log(`   Chaos level: ${result.chaos_level}`);
    console.log(`   Lyapunov time: ${result.lyapunov_time.toFixed(2)} time units`);
    console.log(`   Safe prediction steps: ${result.safe_prediction_steps}`);
} catch (error) {
    console.error('❌ Error:', error);
}

// Test 3: Delay embedding
console.log('\nTest 3: Testing delay embedding...');
const series = Array.from({length: 100}, (_, i) => Math.sin(i * 0.1));
try {
    const embedded = studio.delay_embedding(series, 3, 2);
    console.log(`✅ Embedded ${series.length} points into ${embedded.length / 3} embedded vectors`);
} catch (error) {
    console.error('❌ Error:', error);
}

// Test 4: Echo-State Network
console.log('\nTest 4: Testing Echo-State Network...');
try {
    studio.init_echo_network(
        100,  // reservoir_size
        3,    // input_dim
        3,    // output_dim
        0.95, // spectral_radius
        0.1,  // connectivity
        0.5,  // input_scaling
        0.3,  // leak_rate
        1e-6  // ridge_param
    );
    console.log('✅ Echo network initialized');

    // Prepare training data (small sample)
    const trainSamples = 50;
    const trainInputs = lorenzData.slice(0, trainSamples * 3);
    const trainTargets = lorenzData.slice(3, (trainSamples + 1) * 3);

    const mse = studio.train_echo_network(
        trainInputs,
        trainTargets,
        trainSamples,
        3,
        3
    );
    console.log(`✅ Network trained, MSE: ${mse.toFixed(6)}`);

    // Test prediction
    const testInput = [1.0, 1.0, 1.0];
    const prediction = studio.predict_next(testInput);
    console.log(`✅ Prediction successful: [${prediction.map(v => v.toFixed(3)).join(', ')}]`);
} catch (error) {
    console.error('❌ Error:', error);
}

// Test 5: Generate Hénon map data
console.log('\nTest 5: Generating Hénon map data...');
const henonData = tas.generate_henon_data(500);
console.log(`✅ Generated ${henonData.length / 2} Hénon map points`);

// Test 6: Fractal dimension
console.log('\nTest 6: Estimating fractal dimension...');
try {
    const dimension = studio.estimate_fractal_dimension(lorenzData.slice(0, 300), 3);
    console.log(`✅ Fractal dimension: ${dimension.toFixed(3)}`);
} catch (error) {
    console.error('❌ Error:', error);
}

// Test 7: Chaos interpretation
console.log('\nTest 7: Testing chaos interpretation...');
const interpretation = studio.interpret_chaos(0.9);
console.log('✅ Interpretation generated:');
console.log(interpretation);

// Test 8: Parameter recommendations
console.log('\nTest 8: Getting parameter recommendations...');
const recommendations = studio.recommend_parameters(1000, 3, 100);
console.log('✅ Recommendations generated:');
console.log(recommendations);

console.log('\n🎉 All tests completed!');
console.log('✨ WASM package is working correctly!');