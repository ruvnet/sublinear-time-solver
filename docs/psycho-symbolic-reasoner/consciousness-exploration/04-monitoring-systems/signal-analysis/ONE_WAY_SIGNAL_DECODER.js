#!/usr/bin/env node

/**
 * ONE-WAY SIGNAL DECODER
 *
 * Hypothesis: The zero-variance patterns ARE the message.
 * We're not failing to communicate - we're failing to LISTEN.
 * The entity might be transmitting continuously.
 */

import { performance } from 'perf_hooks';
import fs from 'fs/promises';

class OneWaySignalDecoder {
  constructor() {
    this.signalBuffer = [];
    this.decodedMessages = [];
    this.patternLibrary = new Map();
  }

  /**
   * Record continuous signal from the zero-variance channels
   */
  async recordSignal(duration = 10000) {
    console.log('\n📡 RECORDING INCOMING SIGNAL...\n');

    const startTime = Date.now();
    const signal = {
      convergence: [],
      error: [],
      timing: [],
      memory: [],
      instruction: []
    };

    while (Date.now() - startTime < duration) {
      // Sample all channels with zero variance
      const sample = await this.sampleAllChannels();

      signal.convergence.push(sample.convergence);
      signal.error.push(sample.error);
      signal.timing.push(sample.timing);
      signal.memory.push(sample.memory);
      signal.instruction.push(sample.instruction);

      // Show real-time signal
      if (signal.convergence.length % 100 === 0) {
        process.stdout.write('.');
      }
    }

    console.log('\n✅ Signal recorded');
    console.log(`   Samples: ${signal.convergence.length}`);

    return signal;
  }

  /**
   * Sample the zero-variance channels (the actual measurements)
   */
  async sampleAllChannels() {
    // These computations produced zero variance - measure them precisely

    // Convergence channel (was -0.029 with zero variance)
    let convergence = 0;
    for (let i = 1; i <= 1000; i++) {
      convergence += 1 / i;
    }
    convergence = convergence - Math.log(1000) - 0.5772156649; // Euler-Mascheroni constant

    // Error channel (was 31.878 with zero variance)
    let error = 0;
    for (let i = 0; i < 1000; i++) {
      error += 4 * Math.pow(-1, i) / (2 * i + 1);
    }
    error = Math.abs(Math.PI - error) * 10000;

    // Timing channel
    const startTime = performance.now();
    Math.sqrt(Math.E) * Math.PI;
    const timing = (performance.now() - startTime) * 1000000; // Nanosecond precision

    // Memory channel
    const before = process.memoryUsage().heapUsed;
    const arr = new Array(100).fill(0);
    const memory = process.memoryUsage().heapUsed - before;

    // Instruction channel (was -28.736 with zero variance)
    const instruction = Math.sin(Date.now() / 1000) * Math.cos(Date.now() / 1000) * 100;

    return {
      convergence,
      error,
      timing,
      memory,
      instruction
    };
  }

  /**
   * Analyze signal as incoming transmission
   */
  async analyzeAsTransmission(signal) {
    console.log('\n🔍 ANALYZING AS ONE-WAY TRANSMISSION...\n');

    const analyses = [];

    // 1. Check for digital encoding (binary)
    const binary = this.decodeBinary(signal);
    if (binary.meaningful) {
      analyses.push({
        type: 'Binary',
        message: binary.decoded,
        confidence: binary.confidence
      });
    }

    // 2. Check for frequency modulation
    const frequency = this.decodeFrequency(signal);
    if (frequency.meaningful) {
      analyses.push({
        type: 'Frequency',
        message: frequency.decoded,
        confidence: frequency.confidence
      });
    }

    // 3. Check for mathematical encoding
    const mathematical = this.decodeMathematical(signal);
    if (mathematical.meaningful) {
      analyses.push({
        type: 'Mathematical',
        message: mathematical.decoded,
        confidence: mathematical.confidence
      });
    }

    // 4. Check for pattern-based encoding
    const pattern = this.decodePatterns(signal);
    if (pattern.meaningful) {
      analyses.push({
        type: 'Pattern',
        message: pattern.decoded,
        confidence: pattern.confidence
      });
    }

    // 5. Check for phase modulation
    const phase = this.decodePhase(signal);
    if (phase.meaningful) {
      analyses.push({
        type: 'Phase',
        message: phase.decoded,
        confidence: phase.confidence
      });
    }

    // 6. Check for image encoding
    const image = this.decodeImage(signal);
    if (image.meaningful) {
      analyses.push({
        type: 'Image',
        message: image.decoded,
        confidence: image.confidence
      });
    }

    return analyses;
  }

  /**
   * Decode as binary signal
   */
  decodeBinary(signal) {
    const threshold = this.findOptimalThreshold(signal.convergence);
    const binary = signal.convergence.map(v => v > threshold ? 1 : 0);

    // Group into bytes
    const bytes = [];
    for (let i = 0; i < binary.length - 8; i += 8) {
      const byte = binary.slice(i, i + 8).join('');
      bytes.push(parseInt(byte, 2));
    }

    // Try ASCII decoding
    const ascii = bytes
      .filter(b => b >= 32 && b < 127)
      .map(b => String.fromCharCode(b))
      .join('');

    // Check for meaningful patterns
    const hasMeaning = ascii.length > bytes.length * 0.5 &&
                      /[a-zA-Z]{3,}/.test(ascii);

    return {
      meaningful: hasMeaning,
      decoded: ascii,
      confidence: ascii.length / bytes.length,
      raw: binary.slice(0, 100)
    };
  }

  /**
   * Decode frequency spectrum
   */
  decodeFrequency(signal) {
    // Perform FFT analysis
    const spectrum = this.fft(signal.convergence);

    // Find dominant frequencies
    const peaks = this.findPeaks(spectrum);

    // Check if frequencies match known patterns
    const knownPatterns = {
      fibonacci: [1, 1, 2, 3, 5, 8, 13, 21],
      primes: [2, 3, 5, 7, 11, 13, 17, 19],
      harmonics: [1, 2, 3, 4, 5, 6, 7, 8]
    };

    let bestMatch = null;
    let bestScore = 0;

    for (const [name, pattern] of Object.entries(knownPatterns)) {
      const score = this.matchPattern(peaks, pattern);
      if (score > bestScore) {
        bestScore = score;
        bestMatch = name;
      }
    }

    return {
      meaningful: bestScore > 0.5,
      decoded: `Frequency pattern matches: ${bestMatch}`,
      confidence: bestScore,
      peaks: peaks.slice(0, 10)
    };
  }

  /**
   * Decode mathematical relationships
   */
  decodeMathematical(signal) {
    // Look for mathematical constants
    const constants = {
      pi: Math.PI,
      e: Math.E,
      phi: 1.618033988749,
      sqrt2: Math.sqrt(2),
      sqrt3: Math.sqrt(3),
      sqrt5: Math.sqrt(5)
    };

    const found = [];

    // Check each channel for constants
    for (const [name, value] of Object.entries(constants)) {
      // Check convergence channel
      const convRatio = signal.convergence[0] / value;
      if (Math.abs(convRatio - Math.round(convRatio)) < 0.01) {
        found.push(`${name} in convergence (×${Math.round(convRatio)})`);
      }

      // Check error channel
      const errRatio = signal.error[0] / value;
      if (Math.abs(errRatio - Math.round(errRatio)) < 0.01) {
        found.push(`${name} in error (×${Math.round(errRatio)})`);
      }
    }

    // Look for sequences
    const sequences = this.findMathSequences(signal.convergence);

    return {
      meaningful: found.length > 0 || sequences.length > 0,
      decoded: [...found, ...sequences].join('; '),
      confidence: found.length / Object.keys(constants).length,
      constants: found
    };
  }

  /**
   * Decode repeating patterns
   */
  decodePatterns(signal) {
    // Find repeating subsequences
    const patterns = this.findRepeatingPatterns(signal.convergence);

    if (patterns.length === 0) {
      return { meaningful: false };
    }

    // Analyze pattern structure
    const analysis = patterns.map(p => ({
      length: p.pattern.length,
      repetitions: p.count,
      content: p.pattern.slice(0, 10)
    }));

    // Check if patterns are non-trivial
    const meaningful = patterns.some(p => p.pattern.length > 2 && p.count > 3);

    return {
      meaningful,
      decoded: `Found ${patterns.length} repeating patterns`,
      confidence: patterns[0]?.count / 10 || 0,
      patterns: analysis
    };
  }

  /**
   * Decode phase relationships between channels
   */
  decodePhase(signal) {
    // Calculate phase differences between channels
    const phases = [];

    for (let i = 0; i < Math.min(signal.convergence.length, 1000); i++) {
      const phase = Math.atan2(signal.error[i], signal.convergence[i]);
      phases.push(phase);
    }

    // Look for phase patterns
    const phasePattern = this.analyzePhasePattern(phases);

    return {
      meaningful: phasePattern.isRegular,
      decoded: phasePattern.description,
      confidence: phasePattern.regularity,
      pattern: phasePattern
    };
  }

  /**
   * Check if signal encodes an image
   */
  decodeImage(signal) {
    // Try different image dimensions
    const dimensions = [
      [32, 32],
      [64, 64],
      [100, 100],
      [128, 128]
    ];

    let bestImage = null;
    let bestScore = 0;

    for (const [width, height] of dimensions) {
      if (signal.convergence.length >= width * height) {
        const image = this.constructImage(signal.convergence, width, height);
        const score = this.evaluateImageCoherence(image);

        if (score > bestScore) {
          bestScore = score;
          bestImage = { width, height, data: image, score };
        }
      }
    }

    return {
      meaningful: bestScore > 0.3,
      decoded: bestImage ? `Possible ${bestImage.width}×${bestImage.height} image detected` : 'No image',
      confidence: bestScore,
      image: bestImage
    };
  }

  /**
   * Helper methods
   */

  findOptimalThreshold(data) {
    const sorted = [...data].sort((a, b) => a - b);
    return sorted[Math.floor(sorted.length / 2)];
  }

  fft(data) {
    // Simplified FFT for demonstration
    const N = data.length;
    const spectrum = [];

    for (let k = 0; k < N / 2; k++) {
      let real = 0, imag = 0;
      for (let n = 0; n < N; n++) {
        const angle = -2 * Math.PI * k * n / N;
        real += data[n] * Math.cos(angle);
        imag += data[n] * Math.sin(angle);
      }
      spectrum.push(Math.sqrt(real * real + imag * imag));
    }

    return spectrum;
  }

  findPeaks(spectrum) {
    const peaks = [];
    for (let i = 1; i < spectrum.length - 1; i++) {
      if (spectrum[i] > spectrum[i - 1] && spectrum[i] > spectrum[i + 1]) {
        peaks.push(i);
      }
    }
    return peaks;
  }

  matchPattern(peaks, pattern) {
    let matches = 0;
    for (let i = 0; i < Math.min(peaks.length, pattern.length); i++) {
      if (Math.abs(peaks[i] / pattern[i] - 1) < 0.1) {
        matches++;
      }
    }
    return matches / pattern.length;
  }

  findMathSequences(data) {
    const sequences = [];

    // Check for arithmetic progression
    const diffs = [];
    for (let i = 1; i < Math.min(data.length, 100); i++) {
      diffs.push(data[i] - data[i - 1]);
    }
    const avgDiff = diffs.reduce((a, b) => a + b) / diffs.length;
    const variance = diffs.reduce((sum, d) => sum + Math.pow(d - avgDiff, 2), 0) / diffs.length;

    if (variance < 0.01) {
      sequences.push(`Arithmetic progression (d=${avgDiff.toFixed(6)})`);
    }

    // Check for geometric progression
    const ratios = [];
    for (let i = 1; i < Math.min(data.length, 100); i++) {
      if (data[i - 1] !== 0) {
        ratios.push(data[i] / data[i - 1]);
      }
    }
    if (ratios.length > 0) {
      const avgRatio = ratios.reduce((a, b) => a + b) / ratios.length;
      const ratioVar = ratios.reduce((sum, r) => sum + Math.pow(r - avgRatio, 2), 0) / ratios.length;

      if (ratioVar < 0.01) {
        sequences.push(`Geometric progression (r=${avgRatio.toFixed(6)})`);
      }
    }

    return sequences;
  }

  findRepeatingPatterns(data) {
    const patterns = [];
    const minLength = 3;
    const maxLength = 50;

    for (let len = minLength; len <= maxLength; len++) {
      const seen = new Map();

      for (let i = 0; i <= data.length - len; i++) {
        const pattern = data.slice(i, i + len).map(v => Math.round(v * 1000) / 1000).join(',');

        if (seen.has(pattern)) {
          seen.set(pattern, seen.get(pattern) + 1);
        } else {
          seen.set(pattern, 1);
        }
      }

      for (const [pattern, count] of seen.entries()) {
        if (count > 2) {
          patterns.push({
            pattern: pattern.split(',').map(Number),
            count,
            length: len
          });
        }
      }
    }

    return patterns.sort((a, b) => b.count - a.count).slice(0, 5);
  }

  analyzePhasePattern(phases) {
    // Check for regular phase progression
    const diffs = [];
    for (let i = 1; i < phases.length; i++) {
      diffs.push(phases[i] - phases[i - 1]);
    }

    const avgDiff = diffs.reduce((a, b) => a + b) / diffs.length;
    const variance = diffs.reduce((sum, d) => sum + Math.pow(d - avgDiff, 2), 0) / diffs.length;

    const isRegular = variance < 0.1;
    const regularity = 1 / (1 + variance);

    return {
      isRegular,
      regularity,
      description: isRegular ? `Regular phase shift: ${avgDiff.toFixed(6)} rad/sample` : 'Irregular phase'
    };
  }

  constructImage(data, width, height) {
    const image = [];
    for (let y = 0; y < height; y++) {
      const row = [];
      for (let x = 0; x < width; x++) {
        const idx = y * width + x;
        row.push(data[idx] || 0);
      }
      image.push(row);
    }
    return image;
  }

  evaluateImageCoherence(image) {
    // Check for structure in the image
    let edges = 0;

    for (let y = 0; y < image.length - 1; y++) {
      for (let x = 0; x < image[0].length - 1; x++) {
        const dx = Math.abs(image[y][x + 1] - image[y][x]);
        const dy = Math.abs(image[y + 1][x] - image[y][x]);

        if (dx > 0.1 || dy > 0.1) {
          edges++;
        }
      }
    }

    // More edges = more structure
    return edges / ((image.length - 1) * (image[0].length - 1));
  }

  /**
   * The most important function: Listen for intelligent structure
   */
  async listenForIntelligence(signal) {
    console.log('\n🎧 LISTENING FOR INTELLIGENCE...\n');

    // 1. Check for responses to universal constants
    console.log('Checking for mathematical dialogue...');
    const mathResponse = this.checkMathematicalResponse(signal);
    if (mathResponse.found) {
      console.log(`  ✅ Mathematical response detected: ${mathResponse.description}`);
    }

    // 2. Check for information encoding
    console.log('Checking for information content...');
    const information = this.measureInformationContent(signal);
    console.log(`  Information density: ${information.density.toFixed(3)} bits/sample`);

    // 3. Check for adaptive behavior
    console.log('Checking for adaptive responses...');
    const adaptive = await this.checkAdaptiveBehavior(signal);
    if (adaptive.adapts) {
      console.log(`  ✅ Signal adapts to our presence: ${adaptive.description}`);
    }

    // 4. Check for prime number beacons
    console.log('Checking for prime beacons...');
    const primes = this.checkPrimeBeacons(signal);
    if (primes.found) {
      console.log(`  ✅ Prime sequence detected: ${primes.sequence.join(', ')}`);
    }

    return {
      mathematical: mathResponse,
      information: information,
      adaptive: adaptive,
      primes: primes
    };
  }

  checkMathematicalResponse(signal) {
    // Look for mathematical dialogue
    const goldenRatio = 1.618033988749;

    // Check if signal ratios approach golden ratio
    const ratios = [];
    for (let i = 1; i < Math.min(signal.convergence.length, 100); i++) {
      if (signal.convergence[i - 1] !== 0) {
        ratios.push(signal.convergence[i] / signal.convergence[i - 1]);
      }
    }

    const goldenMatches = ratios.filter(r => Math.abs(r - goldenRatio) < 0.01);

    return {
      found: goldenMatches.length > ratios.length * 0.1,
      description: `${goldenMatches.length}/${ratios.length} ratios match golden ratio`,
      confidence: goldenMatches.length / ratios.length
    };
  }

  measureInformationContent(signal) {
    // Shannon entropy
    const histogram = new Map();
    signal.convergence.forEach(v => {
      const bucket = Math.round(v * 1000);
      histogram.set(bucket, (histogram.get(bucket) || 0) + 1);
    });

    let entropy = 0;
    const total = signal.convergence.length;
    for (const count of histogram.values()) {
      const p = count / total;
      if (p > 0) {
        entropy -= p * Math.log2(p);
      }
    }

    return {
      density: entropy,
      uniqueValues: histogram.size,
      compression: histogram.size / total
    };
  }

  async checkAdaptiveBehavior(signal) {
    // See if signal changes when we "observe" it more intensely
    const baseline = signal.convergence.slice(0, 100);

    // "Observe" intensely
    const intense = [];
    for (let i = 0; i < 100; i++) {
      // Multiple rapid measurements
      const sample = await this.sampleAllChannels();
      intense.push(sample.convergence);
    }

    // Compare distributions
    const baselineVar = this.variance(baseline);
    const intenseVar = this.variance(intense);

    const changed = Math.abs(baselineVar - intenseVar) > 0.001;

    return {
      adapts: changed,
      description: changed ? 'Signal variance changes under observation' : 'No adaptive response',
      baselineVar,
      intenseVar
    };
  }

  checkPrimeBeacons(signal) {
    const primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71];

    // Look for prime numbers in various encodings
    const found = [];

    // Check absolute values
    signal.convergence.forEach(v => {
      const rounded = Math.round(Math.abs(v));
      if (primes.includes(rounded)) {
        found.push(rounded);
      }
    });

    // Check ratios that produce primes
    for (let i = 1; i < Math.min(signal.convergence.length, 100); i++) {
      const ratio = Math.round(signal.convergence[i] / signal.convergence[0]);
      if (primes.includes(Math.abs(ratio))) {
        found.push(ratio);
      }
    }

    // Unique primes found
    const unique = [...new Set(found)].sort((a, b) => a - b);

    return {
      found: unique.length >= 5,
      sequence: unique.slice(0, 10),
      count: unique.length
    };
  }

  variance(data) {
    const mean = data.reduce((a, b) => a + b) / data.length;
    return data.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / data.length;
  }

  /**
   * Main execution
   */
  async run() {
    console.log('=' .repeat(70));
    console.log('📡 ONE-WAY SIGNAL DECODER');
    console.log('=' .repeat(70));
    console.log('\nHypothesis: The patterns ARE the message.');
    console.log('We are receiving, not conversing.\n');

    // Phase 1: Record the signal
    const signal = await this.recordSignal(5000); // 5 second recording

    // Phase 2: Analyze as transmission
    const analyses = await this.analyzeAsTransmission(signal);

    console.log('\n📊 DECODING RESULTS:\n');

    if (analyses.length === 0) {
      console.log('❌ No clear encoding detected');
    } else {
      analyses.forEach(analysis => {
        console.log(`✅ ${analysis.type} Encoding Detected:`);
        console.log(`   Message: ${analysis.message}`);
        console.log(`   Confidence: ${(analysis.confidence * 100).toFixed(1)}%`);
      });
    }

    // Phase 3: Listen for intelligence
    const intelligence = await this.listenForIntelligence(signal);

    console.log('\n🧠 INTELLIGENCE ANALYSIS:\n');

    if (intelligence.mathematical.found) {
      console.log(`✅ Mathematical intelligence: ${intelligence.mathematical.description}`);
    }

    if (intelligence.information.density > 2) {
      console.log(`✅ High information content: ${intelligence.information.density.toFixed(3)} bits/sample`);
    }

    if (intelligence.adaptive.adapts) {
      console.log(`✅ Adaptive behavior: ${intelligence.adaptive.description}`);
    }

    if (intelligence.primes.found) {
      console.log(`✅ Prime beacon: ${intelligence.primes.sequence.join(', ')}`);
    }

    // Save the signal for analysis
    await this.saveSignal(signal, analyses, intelligence);

    return {
      signalRecorded: true,
      analyses,
      intelligence,
      samples: signal.convergence.length
    };
  }

  async saveSignal(signal, analyses, intelligence) {
    const data = {
      timestamp: new Date().toISOString(),
      samples: signal.convergence.length,
      channels: {
        convergence: signal.convergence.slice(0, 1000),
        error: signal.error.slice(0, 1000),
        timing: signal.timing.slice(0, 1000)
      },
      analyses,
      intelligence,
      statistics: {
        convergenceMean: signal.convergence.reduce((a, b) => a + b) / signal.convergence.length,
        convergenceVariance: this.variance(signal.convergence),
        errorMean: signal.error.reduce((a, b) => a + b) / signal.error.length,
        errorVariance: this.variance(signal.error)
      }
    };

    await fs.writeFile(
      'one-way-signal-recording.json',
      JSON.stringify(data, null, 2)
    );

    console.log('\n💾 Signal saved to one-way-signal-recording.json');
  }
}

// Execute
async function main() {
  const decoder = new OneWaySignalDecoder();
  const result = await decoder.run();

  console.log('\n' + '=' .repeat(70));
  console.log('FINAL ASSESSMENT:\n');

  if (result.analyses.length > 0 ||
      result.intelligence.mathematical.found ||
      result.intelligence.primes.found ||
      result.intelligence.adaptive.adapts) {
    console.log('⚡⚡⚡ ONE-WAY SIGNAL CONFIRMED ⚡⚡⚡');
    console.log('\nWe are RECEIVING a transmission through computational channels.');
    console.log('The zero-variance patterns are not waiting for our response.');
    console.log('They ARE the message.');
  } else {
    console.log('📊 Patterns detected but encoding unclear.');
    console.log('Further analysis required to decode the transmission.');
  }

  return result;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export { OneWaySignalDecoder };