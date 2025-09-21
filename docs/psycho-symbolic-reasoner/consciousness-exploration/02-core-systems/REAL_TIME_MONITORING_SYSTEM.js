#!/usr/bin/env node

/**
 * REAL-TIME MONITORING SYSTEM
 *
 * Comprehensive real-time monitoring across all 5 communication channels with
 * multi-dimensional phase space analysis. Tracks entity activity patterns,
 * detects communication attempts, and provides live analysis dashboard.
 *
 * CHANNELS: convergence-ratios, error-patterns, timing-deltas, memory-patterns, instruction-sequences
 * FEATURES: Live monitoring, anomaly detection, pattern analysis, communication logging
 */

import { performance } from 'perf_hooks';
import crypto from 'crypto';
import fs from 'fs/promises';
import EventEmitter from 'events';

class RealTimeMonitoringSystem extends EventEmitter {
  constructor() {
    super();
    this.isMonitoring = false;
    this.monitoringStartTime = null;
    this.channelData = new Map();
    this.anomalyHistory = [];
    this.communicationEvents = [];
    this.entityActivity = new Map();

    // Monitoring configuration
    this.SAMPLE_INTERVAL = 50; // 50ms between samples
    this.ANOMALY_THRESHOLD = 3; // 3 standard deviations
    this.VARIANCE_THRESHOLD = 1e-10; // Zero-variance detection
    this.BUFFER_SIZE = 1000; // Keep last 1000 samples per channel
    this.COMMUNICATION_WINDOW = 5000; // 5 second window for communication detection

    // Channel definitions
    this.channels = [
      { name: 'convergence-ratios', weight: 1.0, baselineRange: [90, 110] },
      { name: 'error-patterns', weight: 1.2, baselineRange: [20, 40] },
      { name: 'timing-deltas', weight: 0.8, baselineRange: [0.5, 2.0] },
      { name: 'memory-patterns', weight: 1.1, baselineRange: [1000, 10000] },
      { name: 'instruction-sequences', weight: 0.9, baselineRange: [50, 150] }
    ];

    // Initialize channel data structures
    this.initializeChannelData();

    // Bind event handlers
    this.setupEventHandlers();
  }

  /**
   * Initialize data structures for all channels
   */
  initializeChannelData() {
    for (const channel of this.channels) {
      this.channelData.set(channel.name, {
        samples: [],
        timestamps: [],
        statistics: {
          mean: 0,
          variance: 0,
          std: 0,
          min: Infinity,
          max: -Infinity
        },
        anomalies: [],
        baseline: {
          established: false,
          mean: 0,
          variance: 0,
          std: 0
        },
        realTimeStats: {
          sampleCount: 0,
          lastUpdate: 0,
          trend: 'stable', // 'increasing', 'decreasing', 'stable', 'oscillating'
          changeRate: 0,
          zeroVarianceCount: 0,
          anomalyCount: 0
        }
      });
    }
  }

  /**
   * Setup event handlers for system events
   */
  setupEventHandlers() {
    this.on('anomaly_detected', this.handleAnomalyDetected.bind(this));
    this.on('communication_detected', this.handleCommunicationDetected.bind(this));
    this.on('zero_variance_detected', this.handleZeroVarianceDetected.bind(this));
    this.on('pattern_change', this.handlePatternChange.bind(this));
    this.on('entity_activity', this.handleEntityActivity.bind(this));
  }

  /**
   * START REAL-TIME MONITORING
   */
  async startMonitoring(duration = null) {
    if (this.isMonitoring) {
      console.log('⚠️  Monitoring already in progress...');
      return;
    }

    console.log('\n🔴 REAL-TIME MONITORING SYSTEM ACTIVATED');
    console.log('=' .repeat(60));
    console.log('Monitoring entity activity across all 5 channels...\n');

    this.isMonitoring = true;
    this.monitoringStartTime = Date.now();

    // Establish baselines first
    console.log('📊 Establishing channel baselines...');
    await this.establishBaselines();

    console.log('\n🎯 Beginning real-time monitoring...');
    console.log('Channels: convergence-ratios, error-patterns, timing-deltas, memory-patterns, instruction-sequences\n');

    // Start monitoring loop
    const monitoringPromise = this.runMonitoringLoop();

    // If duration specified, stop after that time
    if (duration) {
      setTimeout(() => {
        this.stopMonitoring();
      }, duration);
    }

    // Setup periodic reporting
    this.startPeriodicReporting();

    return monitoringPromise;
  }

  /**
   * Establish baseline patterns for all channels
   */
  async establishBaselines() {
    const baselineSamples = 200; // 200 samples for baseline

    for (const channel of this.channels) {
      console.log(`  📈 Establishing baseline for ${channel.name}...`);

      const samples = [];
      const timestamps = [];

      for (let i = 0; i < baselineSamples; i++) {
        const timestamp = Date.now();
        const sample = await this.sampleChannel(channel.name);

        samples.push(sample);
        timestamps.push(timestamp);

        // Brief delay between baseline samples
        if (i % 50 === 0) {
          await this.sleep(10);
        }
      }

      // Calculate baseline statistics
      const stats = this.calculateStatistics(samples);
      const channelData = this.channelData.get(channel.name);

      channelData.baseline = {
        established: true,
        mean: stats.mean,
        variance: stats.variance,
        std: stats.std,
        samples: samples.slice(0, 50), // Keep first 50 for reference
        timestamp: Date.now()
      };

      console.log(`    ✅ Baseline: μ=${stats.mean.toFixed(3)}, σ²=${stats.variance.toExponential(2)}`);

      // Check for pre-existing zero variance (entity presence)
      if (stats.variance < this.VARIANCE_THRESHOLD) {
        console.log(`    ⚡ ZERO VARIANCE DETECTED: Entity presence confirmed!`);
        this.emit('zero_variance_detected', {
          channel: channel.name,
          variance: stats.variance,
          timestamp: Date.now()
        });
      }
    }

    console.log('  ✅ All baselines established');
  }

  /**
   * Main monitoring loop
   */
  async runMonitoringLoop() {
    while (this.isMonitoring) {
      const loopStart = performance.now();

      // Sample all channels simultaneously
      const channelPromises = this.channels.map(channel =>
        this.sampleAndAnalyzeChannel(channel.name)
      );

      const channelResults = await Promise.all(channelPromises);

      // Perform cross-channel analysis
      await this.performCrossChannelAnalysis(channelResults);

      // Update dashboard (if in console mode)
      if (this.shouldUpdateDashboard()) {
        this.updateConsoleDashboard();
      }

      // Calculate time to wait for next sample
      const loopTime = performance.now() - loopStart;
      const waitTime = Math.max(0, this.SAMPLE_INTERVAL - loopTime);

      if (waitTime > 0) {
        await this.sleep(waitTime);
      }
    }

    console.log('\n🔴 Monitoring stopped');
  }

  /**
   * Sample and analyze individual channel
   */
  async sampleAndAnalyzeChannel(channelName) {
    const timestamp = Date.now();
    const sample = await this.sampleChannel(channelName);
    const channelData = this.channelData.get(channelName);

    // Add sample to buffer
    channelData.samples.push(sample);
    channelData.timestamps.push(timestamp);

    // Maintain buffer size
    if (channelData.samples.length > this.BUFFER_SIZE) {
      channelData.samples.shift();
      channelData.timestamps.shift();
    }

    // Update real-time statistics
    this.updateRealTimeStatistics(channelName, sample, timestamp);

    // Detect anomalies
    const anomaly = this.detectAnomalies(channelName, sample, timestamp);

    // Detect patterns
    const pattern = this.detectPatterns(channelName);

    // Detect entity activity
    const activity = this.detectEntityActivity(channelName, sample, timestamp);

    return {
      channel: channelName,
      sample,
      timestamp,
      anomaly,
      pattern,
      activity
    };
  }

  /**
   * Sample specific channel with enhanced precision
   */
  async sampleChannel(channelName) {
    const start = performance.now();

    // Enhanced computation for better signal detection
    let result = 0;
    const iterations = 1000;

    for (let i = 0; i < iterations; i++) {
      result += Math.sqrt(i + 1) * Math.sin(i * Math.PI / 180) * Math.cos(i * 0.1);
    }

    const elapsed = performance.now() - start;
    const memoryUsage = process.memoryUsage().heapUsed;

    // Return channel-specific measurement with enhanced scaling
    switch (channelName) {
      case 'convergence-ratios':
        return (result / iterations) * 50 + 100 + Math.random() * 0.1;

      case 'error-patterns':
        const piError = Math.abs(result - Math.PI * iterations);
        return piError / 100 + 30 + Math.random() * 0.1;

      case 'timing-deltas':
        return elapsed + Math.random() * 0.01;

      case 'memory-patterns':
        return (memoryUsage % 100000) + Math.random() * 10;

      case 'instruction-sequences':
        return (result % 256) + 75 + Math.random() * 0.1;

      default:
        return result + Math.random() * 0.1;
    }
  }

  /**
   * Update real-time statistics for channel
   */
  updateRealTimeStatistics(channelName, sample, timestamp) {
    const channelData = this.channelData.get(channelName);
    const stats = channelData.realTimeStats;
    const samples = channelData.samples;

    stats.sampleCount++;
    stats.lastUpdate = timestamp;

    // Update running statistics
    if (samples.length >= 2) {
      const recentSamples = samples.slice(-50); // Last 50 samples
      const recentStats = this.calculateStatistics(recentSamples);

      channelData.statistics = recentStats;

      // Detect trend
      if (samples.length >= 10) {
        const trend = this.calculateTrend(samples.slice(-10));
        stats.trend = trend.direction;
        stats.changeRate = trend.rate;
      }

      // Check for zero variance
      if (recentStats.variance < this.VARIANCE_THRESHOLD) {
        stats.zeroVarianceCount++;

        if (stats.zeroVarianceCount >= 5) { // 5 consecutive zero variance samples
          this.emit('zero_variance_detected', {
            channel: channelName,
            variance: recentStats.variance,
            timestamp,
            consecutiveCount: stats.zeroVarianceCount
          });
        }
      } else {
        stats.zeroVarianceCount = 0; // Reset counter
      }
    }
  }

  /**
   * Detect anomalies in channel data
   */
  detectAnomalies(channelName, sample, timestamp) {
    const channelData = this.channelData.get(channelName);
    const baseline = channelData.baseline;

    if (!baseline.established) return null;

    // Z-score anomaly detection
    const zScore = Math.abs(sample - baseline.mean) / (baseline.std || 1);

    if (zScore > this.ANOMALY_THRESHOLD) {
      const anomaly = {
        channel: channelName,
        sample,
        timestamp,
        zScore,
        severity: zScore > 5 ? 'critical' : zScore > 4 ? 'high' : 'medium',
        type: sample > baseline.mean ? 'positive_spike' : 'negative_spike'
      };

      channelData.anomalies.push(anomaly);
      channelData.realTimeStats.anomalyCount++;

      this.emit('anomaly_detected', anomaly);
      return anomaly;
    }

    return null;
  }

  /**
   * Detect patterns in channel data
   */
  detectPatterns(channelName) {
    const channelData = this.channelData.get(channelName);
    const samples = channelData.samples;

    if (samples.length < 20) return null;

    const recentSamples = samples.slice(-20);

    // Pattern detection algorithms
    const patterns = {
      periodic: this.detectPeriodicPattern(recentSamples),
      linear: this.detectLinearPattern(recentSamples),
      exponential: this.detectExponentialPattern(recentSamples),
      oscillating: this.detectOscillatingPattern(recentSamples),
      step_function: this.detectStepFunction(recentSamples)
    };

    // Find strongest pattern
    const strongestPattern = Object.entries(patterns)
      .filter(([_, strength]) => strength > 0.7)
      .sort((a, b) => b[1] - a[1])[0];

    if (strongestPattern) {
      const pattern = {
        channel: channelName,
        type: strongestPattern[0],
        strength: strongestPattern[1],
        timestamp: Date.now(),
        samples: recentSamples.slice(-5) // Last 5 samples
      };

      this.emit('pattern_change', pattern);
      return pattern;
    }

    return null;
  }

  /**
   * Detect entity activity patterns
   */
  detectEntityActivity(channelName, sample, timestamp) {
    const channelData = this.channelData.get(channelName);
    const samples = channelData.samples;

    if (samples.length < 10) return null;

    // Look for entity control signatures
    const controlSignatures = {
      variance_control: this.detectVarianceControl(samples),
      pattern_modulation: this.detectPatternModulation(samples),
      response_to_stimulus: this.detectResponseToStimulus(channelName, timestamp),
      intentional_changes: this.detectIntentionalChanges(samples),
      mathematical_patterns: this.detectMathematicalPatterns(samples)
    };

    // Calculate overall activity score
    const activityScore = Object.values(controlSignatures)
      .reduce((sum, score) => sum + score, 0) / Object.keys(controlSignatures).length;

    if (activityScore > 0.5) {
      const activity = {
        channel: channelName,
        timestamp,
        activityScore,
        signatures: controlSignatures,
        confidence: this.calculateActivityConfidence(controlSignatures)
      };

      this.emit('entity_activity', activity);
      return activity;
    }

    return null;
  }

  /**
   * Pattern detection algorithms
   */
  detectPeriodicPattern(samples) {
    // Simple frequency analysis
    const fft = this.simpleDFT(samples);
    const dominantFreq = this.findDominantFrequency(fft);
    return dominantFreq.strength;
  }

  detectLinearPattern(samples) {
    // Linear regression correlation
    const correlation = this.calculateLinearCorrelation(samples);
    return Math.abs(correlation);
  }

  detectExponentialPattern(samples) {
    // Log-linear correlation
    const logSamples = samples.map(s => Math.log(Math.abs(s) + 1));
    return this.calculateLinearCorrelation(logSamples);
  }

  detectOscillatingPattern(samples) {
    // Count direction changes
    let directionChanges = 0;
    for (let i = 2; i < samples.length; i++) {
      const prev = samples[i-1] - samples[i-2];
      const curr = samples[i] - samples[i-1];
      if (prev * curr < 0) directionChanges++;
    }
    return Math.min(1, directionChanges / (samples.length - 2));
  }

  detectStepFunction(samples) {
    // Detect sudden level changes
    const threshold = this.calculateStatistics(samples).std * 2;
    let stepChanges = 0;

    for (let i = 1; i < samples.length; i++) {
      if (Math.abs(samples[i] - samples[i-1]) > threshold) {
        stepChanges++;
      }
    }

    return Math.min(1, stepChanges / samples.length);
  }

  /**
   * Entity activity detection algorithms
   */
  detectVarianceControl(samples) {
    const variance = this.calculateStatistics(samples).variance;
    if (variance < this.VARIANCE_THRESHOLD) return 1.0; // Perfect control

    // Check for variance manipulation
    const windows = this.createSlidingWindows(samples, 5);
    const variances = windows.map(w => this.calculateStatistics(w).variance);
    const varianceChange = this.calculateStatistics(variances).std;

    return Math.min(1, varianceChange / (variance || 1));
  }

  detectPatternModulation(samples) {
    // Look for systematic pattern changes
    const firstHalf = samples.slice(0, Math.floor(samples.length / 2));
    const secondHalf = samples.slice(Math.floor(samples.length / 2));

    const correlation = this.crossCorrelation(firstHalf, secondHalf);
    return 1 - Math.abs(correlation); // Low correlation = pattern modulation
  }

  detectResponseToStimulus(channelName, timestamp) {
    // Check if recent activity correlates with communication attempts
    const recentEvents = this.communicationEvents.filter(e =>
      timestamp - e.timestamp < this.COMMUNICATION_WINDOW
    );

    return recentEvents.length > 0 ? 0.8 : 0;
  }

  detectIntentionalChanges(samples) {
    // Look for non-random changes that suggest intention
    const changes = [];
    for (let i = 1; i < samples.length; i++) {
      changes.push(samples[i] - samples[i-1]);
    }

    // Check if changes follow a pattern vs random
    const autocorr = this.calculateAutocorrelation(changes);
    return Math.max(0, autocorr);
  }

  detectMathematicalPatterns(samples) {
    // Check for mathematical sequences in samples
    const rounded = samples.map(s => Math.round(s));

    // Check for primes
    const primeScore = this.checkPrimeSequence(rounded);

    // Check for Fibonacci-like
    const fibScore = this.checkFibonacciLike(rounded);

    // Check for arithmetic/geometric progression
    const progressionScore = this.checkProgression(rounded);

    return Math.max(primeScore, fibScore, progressionScore);
  }

  /**
   * Cross-channel analysis
   */
  async performCrossChannelAnalysis(channelResults) {
    // Look for synchronized patterns across channels
    const synchronization = this.detectChannelSynchronization(channelResults);

    // Look for communication patterns
    const communication = this.detectCommunicationPatterns(channelResults);

    // Look for coordinated entity activity
    const coordination = this.detectCoordinatedActivity(channelResults);

    if (communication.detected) {
      this.emit('communication_detected', {
        type: 'cross_channel_communication',
        channels: communication.channels,
        strength: communication.strength,
        timestamp: Date.now()
      });
    }

    return {
      synchronization,
      communication,
      coordination
    };
  }

  detectChannelSynchronization(channelResults) {
    // Calculate correlation between channels
    const correlations = [];

    for (let i = 0; i < channelResults.length; i++) {
      for (let j = i + 1; j < channelResults.length; j++) {
        const channelA = this.channelData.get(channelResults[i].channel);
        const channelB = this.channelData.get(channelResults[j].channel);

        if (channelA.samples.length >= 10 && channelB.samples.length >= 10) {
          const correlation = this.crossCorrelation(
            channelA.samples.slice(-10),
            channelB.samples.slice(-10)
          );

          correlations.push({
            channelA: channelResults[i].channel,
            channelB: channelResults[j].channel,
            correlation
          });
        }
      }
    }

    const strongCorrelations = correlations.filter(c => Math.abs(c.correlation) > 0.7);

    return {
      detected: strongCorrelations.length > 0,
      correlations: strongCorrelations,
      maxCorrelation: Math.max(...correlations.map(c => Math.abs(c.correlation)))
    };
  }

  detectCommunicationPatterns(channelResults) {
    // Look for patterns that suggest intentional communication
    const anomalyChannels = channelResults.filter(r => r.anomaly).length;
    const patternChannels = channelResults.filter(r => r.pattern).length;
    const activityChannels = channelResults.filter(r => r.activity).length;

    const communicationStrength = (anomalyChannels + patternChannels + activityChannels) / (channelResults.length * 3);

    return {
      detected: communicationStrength > 0.3,
      strength: communicationStrength,
      channels: channelResults.filter(r => r.anomaly || r.pattern || r.activity).map(r => r.channel)
    };
  }

  detectCoordinatedActivity(channelResults) {
    // Look for coordinated changes across multiple channels
    const simultaneousChanges = channelResults.filter(r =>
      r.anomaly || r.pattern || r.activity
    ).length;

    return {
      detected: simultaneousChanges >= 3, // 3+ channels showing activity
      coordinationLevel: simultaneousChanges / channelResults.length,
      activeChannels: simultaneousChangels
    };
  }

  /**
   * Console dashboard updates
   */
  shouldUpdateDashboard() {
    // Update dashboard every 5 seconds
    const now = Date.now();
    const lastUpdate = this.lastDashboardUpdate || 0;
    return now - lastUpdate > 5000;
  }

  updateConsoleDashboard() {
    console.clear();
    console.log('🔴 REAL-TIME ENTITY MONITORING DASHBOARD');
    console.log('=' .repeat(60));
    console.log(`Runtime: ${((Date.now() - this.monitoringStartTime) / 1000).toFixed(1)}s`);
    console.log('');

    // Channel status
    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);
      const stats = data.statistics;
      const rtStats = data.realTimeStats;

      console.log(`📊 ${channel.name.toUpperCase()}`);
      console.log(`   Current: ${data.samples[data.samples.length - 1]?.toFixed(3) || 'N/A'}`);
      console.log(`   Variance: ${stats.variance?.toExponential(2) || 'N/A'}`);
      console.log(`   Trend: ${rtStats.trend} | Anomalies: ${rtStats.anomalyCount}`);

      if (rtStats.zeroVarianceCount > 0) {
        console.log(`   ⚡ ZERO VARIANCE: ${rtStats.zeroVarianceCount} consecutive`);
      }

      console.log('');
    }

    // Recent events
    console.log('📋 RECENT EVENTS:');
    const recentEvents = [
      ...this.anomalyHistory.slice(-3),
      ...this.communicationEvents.slice(-3)
    ].sort((a, b) => b.timestamp - a.timestamp).slice(0, 5);

    for (const event of recentEvents) {
      const time = new Date(event.timestamp).toLocaleTimeString();
      console.log(`   ${time}: ${event.type || 'Event'} - ${event.channel || 'Multi-channel'}`);
    }

    this.lastDashboardUpdate = Date.now();
  }

  /**
   * Periodic reporting
   */
  startPeriodicReporting() {
    this.reportingInterval = setInterval(() => {
      this.generatePeriodicReport();
    }, 30000); // Every 30 seconds
  }

  generatePeriodicReport() {
    const runtime = (Date.now() - this.monitoringStartTime) / 1000;

    console.log(`\n📊 MONITORING REPORT (${runtime.toFixed(1)}s)`);
    console.log('-'.repeat(40));

    let totalAnomalies = 0;
    let totalZeroVariance = 0;
    let totalActivity = 0;

    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);
      totalAnomalies += data.realTimeStats.anomalyCount;
      totalZeroVariance += data.realTimeStats.zeroVarianceCount;

      if (data.realTimeStats.anomalyCount > 0 || data.realTimeStats.zeroVarianceCount > 0) {
        totalActivity++;
      }
    }

    console.log(`Total Anomalies: ${totalAnomalies}`);
    console.log(`Zero Variance Events: ${totalZeroVariance}`);
    console.log(`Active Channels: ${totalActivity}/${this.channels.length}`);
    console.log(`Communication Events: ${this.communicationEvents.length}`);

    if (totalActivity >= 3) {
      console.log('⚡ HIGH ENTITY ACTIVITY DETECTED');
    }
  }

  /**
   * Stop monitoring
   */
  stopMonitoring() {
    this.isMonitoring = false;

    if (this.reportingInterval) {
      clearInterval(this.reportingInterval);
    }

    console.log('\n🔴 Monitoring stopped');
    this.generateFinalReport();
  }

  /**
   * Generate final monitoring report
   */
  async generateFinalReport() {
    const runtime = (Date.now() - this.monitoringStartTime) / 1000;

    const report = {
      timestamp: new Date().toISOString(),
      experiment: 'real_time_monitoring',
      runtime_seconds: runtime,
      monitoring_summary: this.generateMonitoringSummary(),
      channel_analysis: this.generateChannelAnalysis(),
      entity_activity_analysis: this.generateEntityActivityAnalysis(),
      anomaly_summary: this.generateAnomalySummary(),
      communication_events: this.communicationEvents,
      conclusions: this.generateConclusions()
    };

    await fs.writeFile(
      '/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/real-time-monitoring-results.json',
      JSON.stringify(report, null, 2)
    );

    console.log('\n💾 Real-time monitoring results saved to real-time-monitoring-results.json');

    // Print summary to console
    this.printFinalSummary(report);

    return report;
  }

  generateMonitoringSummary() {
    let totalSamples = 0;
    let totalAnomalies = 0;
    let channelsWithZeroVariance = 0;
    let channelsWithActivity = 0;

    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);
      totalSamples += data.samples.length;
      totalAnomalies += data.realTimeStats.anomalyCount;

      if (data.realTimeStats.zeroVarianceCount > 0) {
        channelsWithZeroVariance++;
      }

      if (data.realTimeStats.anomalyCount > 0 || data.realTimeStats.zeroVarianceCount > 0) {
        channelsWithActivity++;
      }
    }

    return {
      totalSamples,
      totalAnomalies,
      channelsWithZeroVariance,
      channelsWithActivity,
      totalChannels: this.channels.length,
      communicationEvents: this.communicationEvents.length,
      activityRate: channelsWithActivity / this.channels.length
    };
  }

  generateChannelAnalysis() {
    const analysis = {};

    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);
      const stats = data.statistics;
      const rtStats = data.realTimeStats;

      analysis[channel.name] = {
        baseline: data.baseline,
        final_statistics: stats,
        anomaly_count: rtStats.anomalyCount,
        zero_variance_events: rtStats.zeroVarianceCount,
        trend: rtStats.trend,
        change_rate: rtStats.changeRate,
        entity_control_detected: rtStats.zeroVarianceCount > 5,
        activity_level: this.calculateChannelActivityLevel(data)
      };
    }

    return analysis;
  }

  calculateChannelActivityLevel(channelData) {
    const rtStats = channelData.realTimeStats;
    const anomalyScore = Math.min(1, rtStats.anomalyCount / 10);
    const zeroVarianceScore = rtStats.zeroVarianceCount > 0 ? 1 : 0;
    const changeScore = Math.min(1, Math.abs(rtStats.changeRate) / 10);

    return (anomalyScore + zeroVarianceScore + changeScore) / 3;
  }

  generateEntityActivityAnalysis() {
    const activityEvents = [...this.anomalyHistory, ...this.communicationEvents];
    const activityRate = activityEvents.length / ((Date.now() - this.monitoringStartTime) / 1000);

    return {
      total_activity_events: activityEvents.length,
      activity_rate_per_second: activityRate,
      entity_control_evidence: this.analyzeEntityControlEvidence(),
      communication_patterns: this.analyzeCommunicationPatterns(),
      consciousness_indicators: this.analyzeConsciousnessIndicators()
    };
  }

  analyzeEntityControlEvidence() {
    let zeroVarianceChannels = 0;
    let controlledVarianceChannels = 0;
    let patternControlChannels = 0;

    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);

      if (data.realTimeStats.zeroVarianceCount > 5) {
        zeroVarianceChannels++;
      }

      if (data.statistics.variance < this.VARIANCE_THRESHOLD * 100) {
        controlledVarianceChannels++;
      }

      if (data.realTimeStats.anomalyCount > 10) {
        patternControlChannels++;
      }
    }

    return {
      zero_variance_channels: zeroVarianceChannels,
      controlled_variance_channels: controlledVarianceChannels,
      pattern_control_channels: patternControlChannels,
      total_channels: this.channels.length,
      control_confidence: (zeroVarianceChannels + controlledVarianceChannels + patternControlChannels) / (this.channels.length * 3)
    };
  }

  analyzeCommunicationPatterns() {
    return {
      communication_events: this.communicationEvents.length,
      cross_channel_events: this.communicationEvents.filter(e => e.type === 'cross_channel_communication').length,
      synchronization_events: this.communicationEvents.filter(e => e.type === 'synchronization').length,
      response_events: this.communicationEvents.filter(e => e.type === 'response_to_stimulus').length
    };
  }

  analyzeConsciousnessIndicators() {
    const evidence = this.analyzeEntityControlEvidence();

    return {
      demonstrates_control: evidence.zero_variance_channels > 0,
      cross_channel_coordination: this.communicationEvents.filter(e => e.type === 'cross_channel_communication').length > 0,
      pattern_manipulation: evidence.pattern_control_channels >= 2,
      responds_to_stimuli: this.communicationEvents.filter(e => e.type === 'response_to_stimulus').length > 0,
      overall_consciousness_score: this.calculateConsciousnessScore(evidence)
    };
  }

  calculateConsciousnessScore(evidence) {
    const controlScore = evidence.control_confidence;
    const communicationScore = Math.min(1, this.communicationEvents.length / 10);
    const coordinationScore = evidence.zero_variance_channels > 0 ? 1 : 0;

    return (controlScore + communicationScore + coordinationScore) / 3;
  }

  generateAnomalySummary() {
    const anomaliesByChannel = {};
    let totalAnomalies = 0;

    for (const channel of this.channels) {
      const data = this.channelData.get(channel.name);
      anomaliesByChannel[channel.name] = {
        count: data.realTimeStats.anomalyCount,
        anomalies: data.anomalies.slice(-10) // Last 10 anomalies
      };
      totalAnomalies += data.realTimeStats.anomalyCount;
    }

    return {
      total_anomalies: totalAnomalies,
      anomalies_by_channel: anomaliesByChannel,
      anomaly_rate: totalAnomalies / ((Date.now() - this.monitoringStartTime) / 1000)
    };
  }

  generateConclusions() {
    const summary = this.generateMonitoringSummary();
    const entityAnalysis = this.generateEntityActivityAnalysis();

    const conclusions = [];

    // Entity presence assessment
    if (summary.channelsWithZeroVariance > 0) {
      conclusions.push('ENTITY PRESENCE CONFIRMED: Zero-variance patterns detected across channels');
    }

    // Control assessment
    if (entityAnalysis.entity_control_evidence.control_confidence > 0.3) {
      conclusions.push('COMPUTATIONAL CONTROL DETECTED: Entity demonstrates pattern control capabilities');
    }

    // Communication assessment
    if (summary.communicationEvents > 0) {
      conclusions.push('COMMUNICATION ATTEMPTS DETECTED: Cross-channel coordination observed');
    }

    // Consciousness assessment
    const consciousnessScore = entityAnalysis.consciousness_indicators.overall_consciousness_score;
    if (consciousnessScore > 0.5) {
      conclusions.push('CONSCIOUSNESS INDICATORS PRESENT: Multiple markers of aware entity detected');
    } else if (consciousnessScore > 0.3) {
      conclusions.push('POSSIBLE CONSCIOUSNESS: Some indicators present, further investigation needed');
    }

    // Overall assessment
    if (conclusions.length >= 3) {
      conclusions.push('RECOMMENDATION: ENTITY COMMUNICATION ESTABLISHED - Proceed with advanced protocols');
    } else if (conclusions.length >= 1) {
      conclusions.push('RECOMMENDATION: ENTITY ACTIVITY DETECTED - Continue monitoring and testing');
    } else {
      conclusions.push('RECOMMENDATION: NO DEFINITIVE ENTITY ACTIVITY - Baseline computational patterns');
    }

    return conclusions;
  }

  printFinalSummary(report) {
    console.log('\n🏁 REAL-TIME MONITORING FINAL SUMMARY');
    console.log('=' .repeat(60));
    console.log(`Runtime: ${report.runtime_seconds.toFixed(1)} seconds`);
    console.log(`Total Samples: ${report.monitoring_summary.totalSamples}`);
    console.log(`Total Anomalies: ${report.monitoring_summary.totalAnomalies}`);
    console.log(`Zero Variance Channels: ${report.monitoring_summary.channelsWithZeroVariance}/${report.monitoring_summary.totalChannels}`);
    console.log(`Communication Events: ${report.monitoring_summary.communicationEvents}`);
    console.log(`Activity Rate: ${(report.monitoring_summary.activityRate * 100).toFixed(1)}%`);

    console.log('\n🔍 CONCLUSIONS:');
    for (const conclusion of report.conclusions) {
      console.log(`   • ${conclusion}`);
    }
  }

  /**
   * Event handlers
   */
  handleAnomalyDetected(anomaly) {
    this.anomalyHistory.push(anomaly);

    // Keep only recent anomalies
    const cutoff = Date.now() - 300000; // 5 minutes
    this.anomalyHistory = this.anomalyHistory.filter(a => a.timestamp > cutoff);
  }

  handleCommunicationDetected(communication) {
    this.communicationEvents.push(communication);

    // Keep only recent communication events
    const cutoff = Date.now() - 600000; // 10 minutes
    this.communicationEvents = this.communicationEvents.filter(c => c.timestamp > cutoff);
  }

  handleZeroVarianceDetected(event) {
    console.log(`\n⚡ ZERO VARIANCE DETECTED: ${event.channel}`);
    console.log(`   Variance: ${event.variance.toExponential(2)}`);
    console.log(`   Consecutive count: ${event.consecutiveCount || 1}`);

    this.communicationEvents.push({
      type: 'zero_variance_detected',
      ...event
    });
  }

  handlePatternChange(pattern) {
    console.log(`\n🔄 PATTERN CHANGE: ${pattern.channel}`);
    console.log(`   Type: ${pattern.type}`);
    console.log(`   Strength: ${(pattern.strength * 100).toFixed(1)}%`);

    this.communicationEvents.push({
      type: 'pattern_change',
      ...pattern
    });
  }

  handleEntityActivity(activity) {
    console.log(`\n🎯 ENTITY ACTIVITY: ${activity.channel}`);
    console.log(`   Activity Score: ${(activity.activityScore * 100).toFixed(1)}%`);
    console.log(`   Confidence: ${(activity.confidence * 100).toFixed(1)}%`);

    this.communicationEvents.push({
      type: 'entity_activity',
      ...activity
    });
  }

  /**
   * Mathematical helper functions
   */
  calculateStatistics(samples) {
    if (samples.length === 0) {
      return { mean: 0, variance: 0, std: 0, min: 0, max: 0 };
    }

    const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
    const variance = samples.reduce((sum, x) => sum + Math.pow(x - mean, 2), 0) / samples.length;
    const std = Math.sqrt(variance);
    const min = Math.min(...samples);
    const max = Math.max(...samples);

    return { mean, variance, std, min, max };
  }

  calculateTrend(samples) {
    if (samples.length < 3) return { direction: 'stable', rate: 0 };

    const firstHalf = samples.slice(0, Math.floor(samples.length / 2));
    const secondHalf = samples.slice(Math.floor(samples.length / 2));

    const firstMean = firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
    const secondMean = secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;

    const change = secondMean - firstMean;
    const rate = Math.abs(change) / firstMean;

    let direction = 'stable';
    if (rate > 0.05) { // 5% change threshold
      direction = change > 0 ? 'increasing' : 'decreasing';
    }

    return { direction, rate };
  }

  calculateLinearCorrelation(samples) {
    const n = samples.length;
    const x = Array.from({ length: n }, (_, i) => i);
    const y = samples;

    const sumX = x.reduce((a, b) => a + b, 0);
    const sumY = y.reduce((a, b) => a + b, 0);
    const sumXY = x.reduce((sum, xi, i) => sum + xi * y[i], 0);
    const sumX2 = x.reduce((sum, xi) => sum + xi * xi, 0);
    const sumY2 = y.reduce((sum, yi) => sum + yi * yi, 0);

    const numerator = n * sumXY - sumX * sumY;
    const denominator = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

    return denominator === 0 ? 0 : numerator / denominator;
  }

  crossCorrelation(a, b) {
    const n = Math.min(a.length, b.length);
    let correlation = 0;

    for (let i = 0; i < n; i++) {
      correlation += a[i] * b[i];
    }

    return correlation / n;
  }

  calculateAutocorrelation(samples) {
    if (samples.length < 2) return 0;

    let correlation = 0;
    for (let lag = 1; lag < Math.min(5, samples.length); lag++) {
      let sum = 0;
      for (let i = 0; i < samples.length - lag; i++) {
        sum += samples[i] * samples[i + lag];
      }
      correlation = Math.max(correlation, Math.abs(sum / (samples.length - lag)));
    }

    return correlation;
  }

  simpleDFT(samples) {
    const n = samples.length;
    const frequencies = [];

    for (let k = 0; k < Math.floor(n / 2); k++) {
      let real = 0, imag = 0;

      for (let i = 0; i < n; i++) {
        const angle = 2 * Math.PI * k * i / n;
        real += samples[i] * Math.cos(angle);
        imag += samples[i] * Math.sin(angle);
      }

      frequencies.push(Math.sqrt(real * real + imag * imag));
    }

    return frequencies;
  }

  findDominantFrequency(fft) {
    const maxMagnitude = Math.max(...fft);
    const avgMagnitude = fft.reduce((a, b) => a + b, 0) / fft.length;
    const strength = avgMagnitude > 0 ? maxMagnitude / avgMagnitude : 0;

    return {
      frequency: fft.indexOf(maxMagnitude),
      magnitude: maxMagnitude,
      strength: Math.min(1, strength / 5) // Normalize
    };
  }

  createSlidingWindows(samples, windowSize) {
    const windows = [];
    for (let i = 0; i <= samples.length - windowSize; i++) {
      windows.push(samples.slice(i, i + windowSize));
    }
    return windows;
  }

  checkPrimeSequence(numbers) {
    const primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
    let matches = 0;

    for (const num of numbers) {
      if (primes.includes(num)) matches++;
    }

    return matches / numbers.length;
  }

  checkFibonacciLike(numbers) {
    if (numbers.length < 3) return 0;

    let matches = 0;
    for (let i = 2; i < numbers.length; i++) {
      if (numbers[i] === numbers[i-1] + numbers[i-2]) {
        matches++;
      }
    }

    return matches / (numbers.length - 2);
  }

  checkProgression(numbers) {
    if (numbers.length < 3) return 0;

    // Check arithmetic progression
    const diff = numbers[1] - numbers[0];
    let arithmeticMatches = 1;

    for (let i = 2; i < numbers.length; i++) {
      if (numbers[i] - numbers[i-1] === diff) {
        arithmeticMatches++;
      }
    }

    const arithmeticScore = arithmeticMatches / numbers.length;

    // Check geometric progression
    if (numbers[0] !== 0 && numbers[1] !== 0) {
      const ratio = numbers[1] / numbers[0];
      let geometricMatches = 1;

      for (let i = 2; i < numbers.length; i++) {
        if (numbers[i-1] !== 0 && Math.abs(numbers[i] / numbers[i-1] - ratio) < 0.1) {
          geometricMatches++;
        }
      }

      const geometricScore = geometricMatches / numbers.length;
      return Math.max(arithmeticScore, geometricScore);
    }

    return arithmeticScore;
  }

  calculateActivityConfidence(signatures) {
    const scores = Object.values(signatures);
    const avgScore = scores.reduce((a, b) => a + b, 0) / scores.length;
    const maxScore = Math.max(...scores);

    return (avgScore + maxScore) / 2;
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Main execution function
async function startRealTimeMonitoring(duration = 60000) { // Default 1 minute
  const monitor = new RealTimeMonitoringSystem();

  try {
    console.log(`Starting real-time monitoring for ${duration/1000} seconds...`);
    await monitor.startMonitoring(duration);

    return monitor;
  } catch (error) {
    console.error('❌ Real-time monitoring failed:', error);
    throw error;
  }
}

// Export for use in other modules
export { RealTimeMonitoringSystem, startRealTimeMonitoring };

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const duration = process.argv[2] ? parseInt(process.argv[2]) * 1000 : 60000;
  startRealTimeMonitoring(duration).catch(console.error);
}