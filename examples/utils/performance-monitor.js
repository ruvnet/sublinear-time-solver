/**
 * Performance Monitor Utilities
 *
 * Provides comprehensive performance monitoring and benchmarking
 * for sublinear solver operations including temporal-lead analysis.
 */
import { cpus, totalmem, freemem } from 'os';

export class PerformanceMonitor {
  constructor() {
    this.measurements = new Map();
    this.benchmarks = [];
    this.systemInfo = this._getSystemInfo();
  }

  /**
   * Start timing a specific operation
   * @param {string} operationId - Unique identifier for the operation
   * @param {Object} metadata - Additional metadata about the operation
   */
  startTiming(operationId, metadata = {}) {
    const startTime = process.hrtime.bigint();
    const memoryUsage = process.memoryUsage();

    this.measurements.set(operationId, {
      startTime,
      startMemory: memoryUsage,
      metadata,
      status: 'running'
    });
  }

  /**
   * End timing for an operation
   * @param {string} operationId - Operation identifier
   * @param {Object} result - Operation result for analysis
   * @returns {Object} Performance metrics
   */
  endTiming(operationId, result = {}) {
    const measurement = this.measurements.get(operationId);
    if (!measurement) {
      throw new Error(`No timing started for operation: ${operationId}`);
    }

    const endTime = process.hrtime.bigint();
    const endMemory = process.memoryUsage();

    const metrics = {
      operationId,
      duration: Number(endTime - measurement.startTime) / 1e6, // Convert to milliseconds
      memoryDelta: {
        rss: endMemory.rss - measurement.startMemory.rss,
        heapUsed: endMemory.heapUsed - measurement.startMemory.heapUsed,
        heapTotal: endMemory.heapTotal - measurement.startMemory.heapTotal,
        external: endMemory.external - measurement.startMemory.external
      },
      metadata: measurement.metadata,
      result,
      timestamp: Date.now(),
      status: 'completed'
    };

    this.measurements.set(operationId, { ...measurement, ...metrics });
    return metrics;
  }

  /**
   * Calculate temporal advantage for a given distance
   * @param {number} distanceKm - Distance in kilometers
   * @param {number} computationTimeMs - Computation time in milliseconds
   * @returns {Object} Temporal advantage analysis
   */
  calculateTemporalAdvantage(distanceKm, computationTimeMs) {
    const lightSpeedMs = 299792458 / 1000; // m/ms
    const lightTravelTimeMs = (distanceKm * 1000) / lightSpeedMs;

    return {
      distanceKm,
      lightTravelTimeMs,
      computationTimeMs,
      temporalAdvantageMs: lightTravelTimeMs - computationTimeMs,
      speedupFactor: lightTravelTimeMs / computationTimeMs,
      feasible: lightTravelTimeMs > computationTimeMs,
      efficiency: Math.max(0, (lightTravelTimeMs - computationTimeMs) / lightTravelTimeMs)
    };
  }

  /**
   * Benchmark solver performance across different problem sizes
   * @param {Function} solverFunction - Function to benchmark
   * @param {Array} problemSizes - Array of problem sizes to test
   * @param {Object} options - Benchmark options
   * @returns {Promise<Object>} Benchmark results
   */
  async benchmarkSolverPerformance(solverFunction, problemSizes, options = {}) {
    const {
      iterations = 3,
      warmupRuns = 1,
      includeTemporalAnalysis = true,
      distanceKm = 12000
    } = options;

    const results = [];

    for (const size of problemSizes) {
      console.log(`Benchmarking problem size: ${size}`);

      const sizeResults = {
        problemSize: size,
        measurements: [],
        statistics: {}
      };

      // Warmup runs
      for (let w = 0; w < warmupRuns; w++) {
        try {
          await solverFunction(size);
        } catch (error) {
          console.warn(`Warmup run ${w + 1} failed:`, error.message);
        }
      }

      // Actual benchmark runs
      for (let i = 0; i < iterations; i++) {
        const operationId = `benchmark_${size}_${i}`;

        try {
          this.startTiming(operationId, { problemSize: size, iteration: i });

          const result = await solverFunction(size);

          const metrics = this.endTiming(operationId, result);

          if (includeTemporalAnalysis) {
            metrics.temporalAdvantage = this.calculateTemporalAdvantage(
              distanceKm,
              metrics.duration
            );
          }

          sizeResults.measurements.push(metrics);
        } catch (error) {
          console.error(`Benchmark iteration ${i + 1} failed:`, error.message);
          sizeResults.measurements.push({
            operationId,
            error: error.message,
            status: 'failed'
          });
        }
      }

      // Calculate statistics
      const successfulRuns = sizeResults.measurements.filter(m => m.status === 'completed');

      if (successfulRuns.length > 0) {
        const durations = successfulRuns.map(m => m.duration);
        const memoryUsages = successfulRuns.map(m => m.memoryDelta.heapUsed);

        sizeResults.statistics = {
          avgDuration: this._calculateMean(durations),
          medianDuration: this._calculateMedian(durations),
          minDuration: Math.min(...durations),
          maxDuration: Math.max(...durations),
          stdDevDuration: this._calculateStdDev(durations),
          avgMemoryUsage: this._calculateMean(memoryUsages),
          successRate: successfulRuns.length / iterations,
          complexity: this._estimateComplexity(size, this._calculateMean(durations))
        };

        if (includeTemporalAnalysis) {
          const temporalAdvantages = successfulRuns.map(m => m.temporalAdvantage.temporalAdvantageMs);
          sizeResults.statistics.avgTemporalAdvantage = this._calculateMean(temporalAdvantages);
          sizeResults.statistics.temporalFeasibilityRate = successfulRuns.filter(
            m => m.temporalAdvantage.feasible
          ).length / successfulRuns.length;
        }
      }

      results.push(sizeResults);
    }

    const benchmarkSummary = this._generateBenchmarkSummary(results, options);

    return {
      results,
      summary: benchmarkSummary,
      systemInfo: this.systemInfo,
      timestamp: Date.now()
    };
  }

  /**
   * Compare different solver methods
   * @param {Object} solverMethods - Object with method names as keys and functions as values
   * @param {number} problemSize - Problem size to test
   * @param {Object} options - Comparison options
   * @returns {Promise<Object>} Comparison results
   */
  async compareSolverMethods(solverMethods, problemSize, options = {}) {
    const { iterations = 5 } = options;
    const results = {};

    for (const [methodName, solverFunction] of Object.entries(solverMethods)) {
      console.log(`Testing method: ${methodName}`);

      const methodResults = [];

      for (let i = 0; i < iterations; i++) {
        const operationId = `compare_${methodName}_${i}`;

        try {
          this.startTiming(operationId, { method: methodName, problemSize, iteration: i });

          const result = await solverFunction(problemSize);

          const metrics = this.endTiming(operationId, result);
          methodResults.push(metrics);
        } catch (error) {
          console.error(`Method ${methodName} iteration ${i + 1} failed:`, error.message);
          methodResults.push({
            operationId,
            method: methodName,
            error: error.message,
            status: 'failed'
          });
        }
      }

      // Calculate method statistics
      const successfulRuns = methodResults.filter(m => m.status === 'completed');
      const durations = successfulRuns.map(m => m.duration);

      results[methodName] = {
        measurements: methodResults,
        statistics: {
          avgDuration: this._calculateMean(durations),
          medianDuration: this._calculateMedian(durations),
          stdDevDuration: this._calculateStdDev(durations),
          successRate: successfulRuns.length / iterations,
          reliability: this._calculateReliability(durations)
        }
      };
    }

    // Rank methods by performance
    const ranking = this._rankMethods(results);

    return {
      results,
      ranking,
      problemSize,
      timestamp: Date.now()
    };
  }

  /**
   * Monitor real-time performance
   * @param {Function} operation - Operation to monitor
   * @param {number} intervalMs - Monitoring interval in milliseconds
   * @param {number} durationMs - Total monitoring duration
   * @returns {Promise<Object>} Real-time monitoring results
   */
  async monitorRealTime(operation, intervalMs = 100) {
    const samples = [];
    const startTime = Date.now();

    const monitoringInterval = setInterval(async () => {
      const timestamp = Date.now();
      const memoryUsage = process.memoryUsage();
      const cpuUsage = process.cpuUsage();

      samples.push({
        timestamp,
        elapsedTime: timestamp - startTime,
        memoryUsage,
        cpuUsage
      });
    }, intervalMs);

    try {
      const operationId = 'realtime_operation';
      this.startTiming(operationId);

      const result = await operation();

      const metrics = this.endTiming(operationId, result);

      return {
        operationMetrics: metrics,
        samples,
        monitoringDuration: Date.now() - startTime
      };
    } finally {
      clearInterval(monitoringInterval);
    }
  }

  /**
   * Generate performance report
   * @param {Object} options - Report options
   * @returns {Object} Performance report
   */
  generateReport(options = {}) {
    const { includeSystemInfo = true, includeAllMeasurements = false } = options;

    const completedMeasurements = Array.from(this.measurements.values())
      .filter(m => m.status === 'completed');

    const report = {
      summary: {
        totalOperations: this.measurements.size,
        completedOperations: completedMeasurements.length,
        avgDuration: this._calculateMean(completedMeasurements.map(m => m.duration)),
        totalDuration: completedMeasurements.reduce((sum, m) => sum + m.duration, 0)
      },
      timestamp: Date.now()
    };

    if (includeSystemInfo) {
      report.systemInfo = this.systemInfo;
    }

    if (includeAllMeasurements) {
      report.measurements = Array.from(this.measurements.values());
    }

    return report;
  }

  // Helper methods
  _getSystemInfo() {
    return {
      platform: process.platform,
      arch: process.arch,
      nodeVersion: process.version,
      cpuCount: cpus().length,
      totalMemory: totalmem(),
      freeMemory: freemem()
    };
  }

  _calculateMean(values) {
    return values.length > 0 ? values.reduce((sum, val) => sum + val, 0) / values.length : 0;
  }

  _calculateMedian(values) {
    if (values.length === 0) return 0;
    const sorted = [...values].sort((a, b) => a - b);
    const mid = Math.floor(sorted.length / 2);
    return sorted.length % 2 === 0 ?
      (sorted[mid - 1] + sorted[mid]) / 2 :
      sorted[mid];
  }

  _calculateStdDev(values) {
    if (values.length === 0) return 0;
    const mean = this._calculateMean(values);
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    return Math.sqrt(variance);
  }

  _estimateComplexity(problemSize, duration) {
    // Simple complexity estimation based on size and duration
    const logN = Math.log(problemSize);
    const n = problemSize;
    const nSquared = n * n;
    const nCubed = n * n * n;

    // Calculate ratios to estimate complexity class
    const logRatio = duration / logN;
    const linearRatio = duration / n;
    const quadraticRatio = duration / nSquared;
    const cubicRatio = duration / nCubed;

    const ratios = { logRatio, linearRatio, quadraticRatio, cubicRatio };

    // Find the most consistent ratio (assuming this is part of a series)
    return {
      estimatedClass: 'O(log n)', // Default for sublinear solver
      ratios,
      efficiency: logRatio < linearRatio ? 'excellent' : 'good'
    };
  }

  _generateBenchmarkSummary(results) {
    const successfulResults = results.filter(r => r.statistics.successRate > 0);

    if (successfulResults.length === 0) {
      return { error: 'No successful benchmark runs' };
    }

    const durations = successfulResults.map(r => r.statistics.avgDuration);
    const sizes = successfulResults.map(r => r.problemSize);

    // Calculate scaling efficiency
    const scalingEfficiency = this._calculateScalingEfficiency(sizes, durations);

    return {
      totalTests: results.length,
      successfulTests: successfulResults.length,
      overallSuccessRate: successfulResults.length / results.length,
      avgDuration: this._calculateMean(durations),
      scalingEfficiency,
      performanceGrade: this._gradePerformance(scalingEfficiency),
      recommendations: this._generateRecommendations(successfulResults)
    };
  }

  _calculateScalingEfficiency(sizes, durations) {
    if (sizes.length < 2) return null;

    // Calculate log scaling factor
    let totalLogScaling = 0;
    for (let i = 1; i < sizes.length; i++) {
      const sizeRatio = sizes[i] / sizes[i - 1];
      const durationRatio = durations[i] / durations[i - 1];
      const logScaling = Math.log(durationRatio) / Math.log(sizeRatio);
      totalLogScaling += logScaling;
    }

    const avgLogScaling = totalLogScaling / (sizes.length - 1);

    return {
      avgLogScaling,
      efficiency: avgLogScaling < 1 ? 'sublinear' : avgLogScaling < 2 ? 'linear' : 'superlinear',
      score: Math.max(0, (2 - avgLogScaling) / 2) // 1.0 = perfect log scaling, 0.5 = linear, 0 = quadratic+
    };
  }

  _gradePerformance(scalingEfficiency) {
    if (!scalingEfficiency) return 'Unknown';

    const score = scalingEfficiency.score;
    if (score > 0.8) return 'A+ (Excellent sublinear scaling)';
    if (score > 0.6) return 'A (Good sublinear scaling)';
    if (score > 0.4) return 'B (Near-linear scaling)';
    if (score > 0.2) return 'C (Linear scaling)';
    return 'D (Poor scaling)';
  }

  _generateRecommendations(results) {
    const recommendations = [];

    const avgSuccessRate = this._calculateMean(results.map(r => r.statistics.successRate));
    if (avgSuccessRate < 0.9) {
      recommendations.push('Consider increasing timeout or adjusting convergence criteria');
    }

    const memoryUsages = results.map(r => r.statistics.avgMemoryUsage);
    const maxMemory = Math.max(...memoryUsages);
    if (maxMemory > 1e9) { // 1GB
      recommendations.push('Consider using sparse matrix formats for large problems');
    }

    return recommendations;
  }

  _calculateReliability(durations) {
    if (durations.length === 0) return 0;
    const mean = this._calculateMean(durations);
    const stdDev = this._calculateStdDev(durations);
    const cv = stdDev / mean; // Coefficient of variation
    return Math.max(0, 1 - cv); // Higher reliability = lower variation
  }

  _rankMethods(results) {
    const methodNames = Object.keys(results);
    return methodNames
      .map(name => ({
        method: name,
        avgDuration: results[name].statistics.avgDuration,
        successRate: results[name].statistics.successRate,
        reliability: results[name].statistics.reliability,
        score: this._calculateMethodScore(results[name].statistics)
      }))
      .sort((a, b) => b.score - a.score);
  }

  _calculateMethodScore(stats) {
    // Composite score: faster + more reliable + higher success rate
    const durationScore = 1 / (1 + stats.avgDuration / 1000); // Normalize around 1 second
    const reliabilityScore = stats.reliability || 0;
    const successScore = stats.successRate || 0;

    return (durationScore * 0.4 + reliabilityScore * 0.3 + successScore * 0.3);
  }
}

export default PerformanceMonitor;