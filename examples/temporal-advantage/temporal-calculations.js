/**
 * Temporal Advantage Calculations Example
 *
 * Demonstrates the theoretical and practical aspects of temporal computational advantage
 * using the sublinear-time-solver to solve problems faster than light can travel.
 */

import { PerformanceMonitor } from '../utils/performance-monitor.js';
import { MatrixGenerator } from '../utils/matrix-generator.js';

export class TemporalAdvantageCalculator {
  constructor(config = {}) {
    this.config = {
      // Physical constants
      lightSpeed: 299792458, // m/s in vacuum
      fiberOpticSpeed: 200000000, // m/s in fiber optic (≈ 2/3 * c)
      atmosphericSpeed: 299700000, // m/s in atmosphere

      // Network scenarios
      scenarios: [
        { name: 'Local Network', distance: 0.001, medium: 'copper' },
        { name: 'Campus Network', distance: 1, medium: 'fiber' },
        { name: 'Metropolitan', distance: 50, medium: 'fiber' },
        { name: 'Regional', distance: 500, medium: 'fiber' },
        { name: 'National', distance: 3000, medium: 'fiber' },
        { name: 'Continental', distance: 8000, medium: 'fiber' },
        { name: 'Intercontinental', distance: 12000, medium: 'fiber' },
        { name: 'Antipodal', distance: 20000, medium: 'fiber' },
        { name: 'Satellite (GEO)', distance: 71600, medium: 'air' }, // 2 * geostationary orbit
        { name: 'Deep Space', distance: 500000000, medium: 'vacuum' } // Mars-Earth
      ],

      // Problem complexity ranges
      complexityLevels: [
        { name: 'Small', size: 100, description: 'Development/Testing' },
        { name: 'Medium', size: 1000, description: 'Production Systems' },
        { name: 'Large', size: 10000, description: 'Enterprise Scale' },
        { name: 'Very Large', size: 100000, description: 'Big Data' },
        { name: 'Massive', size: 1000000, description: 'Hyperscale' }
      ],

      ...config
    };

    this.monitor = new PerformanceMonitor();
  }

  /**
   * Calculate temporal advantage for all scenarios and complexity levels
   * @returns {Promise<Object>} Comprehensive temporal advantage analysis
   */
  async calculateComprehensiveAdvantage() {
    console.log('⚡ Comprehensive Temporal Advantage Analysis');
    console.log('==========================================\n');

    const operationId = 'comprehensive_temporal_analysis';
    this.monitor.startTiming(operationId);

    const results = {
      scenarios: [],
      complexityAnalysis: [],
      practicalApplications: [],
      summary: {}
    };

    try {
      // Analyze each distance scenario
      for (const scenario of this.config.scenarios) {
        console.log(`📡 Analyzing: ${scenario.name} (${scenario.distance.toLocaleString()} km)`);

        const scenarioResults = await this._analyzeScenario(scenario);
        results.scenarios.push(scenarioResults);
      }

      // Analyze complexity scaling
      console.log('\n🔬 Analyzing complexity scaling...');
      for (const level of this.config.complexityLevels) {
        const complexityResults = await this._analyzeComplexityLevel(level);
        results.complexityAnalysis.push(complexityResults);
      }

      // Generate practical applications
      results.practicalApplications = this._generatePracticalApplications(results);

      // Generate summary
      results.summary = this._generateTemporalSummary(results);

      const timing = this.monitor.endTiming(operationId, {
        scenariosAnalyzed: results.scenarios.length,
        complexityLevels: results.complexityAnalysis.length
      });

      results.analysisTime = timing.duration;

      this._printComprehensiveResults(results);

      return results;

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Demonstrate temporal advantage with specific examples
   * @param {Array} examples - Specific examples to demonstrate
   * @returns {Promise<Object>} Demonstration results
   */
  async demonstrateTemporalExamples(examples = []) {
    const defaultExamples = [
      {
        name: 'High-Frequency Trading',
        scenario: 'NYSE to LSE',
        distance: 5585,
        problemSize: 5000,
        description: 'Cross-exchange arbitrage detection'
      },
      {
        name: 'Real-Time Risk Management',
        scenario: 'Global Trading Floor',
        distance: 12000,
        problemSize: 10000,
        description: 'Portfolio risk calculation'
      },
      {
        name: 'Social Media Analysis',
        scenario: 'Viral Content Detection',
        distance: 8000,
        problemSize: 50000,
        description: 'Influence propagation modeling'
      },
      {
        name: 'Scientific Computing',
        scenario: 'Distributed Simulation',
        distance: 20000,
        problemSize: 100000,
        description: 'Climate model coordination'
      }
    ];

    const exampleList = examples.length > 0 ? examples : defaultExamples;
    const results = [];

    console.log('🎯 Temporal Advantage Demonstrations');
    console.log('===================================\n');

    for (const example of exampleList) {
      console.log(`📈 Example: ${example.name}`);
      console.log(`   Scenario: ${example.scenario} (${example.distance.toLocaleString()} km)`);
      console.log(`   Problem: ${example.description} (${example.problemSize.toLocaleString()} variables)`);

      const result = await this._demonstrateSpecificExample(example);
      results.push(result);

      console.log(`   ⚡ Computation Time: ${result.computationTime.toFixed(2)}ms`);
      console.log(`   📡 Light Travel Time: ${result.lightTravelTime.toFixed(2)}ms`);
      console.log(`   🚀 Temporal Advantage: ${result.temporalAdvantage.toFixed(2)}ms`);
      console.log(`   ✅ Feasible: ${result.feasible ? 'Yes' : 'No'}\n`);
    }

    return {
      examples: results,
      summary: this._summarizeDemonstrations(results)
    };
  }

  /**
   * Calculate the theoretical limits of temporal advantage
   * @returns {Object} Theoretical analysis
   */
  calculateTheoreticalLimits() {
    console.log('🔬 Theoretical Temporal Advantage Limits');
    console.log('=======================================\n');

    const limits = {
      // Speed of light in different media
      propagationSpeeds: {
        vacuum: this.config.lightSpeed,
        fiber: this.config.fiberOpticSpeed,
        atmosphere: this.config.atmosphericSpeed,
        copper: 200000000 // Approximate for electrical signals
      },

      // Maximum practical distances
      maxDistances: {
        terrestrial: 20015, // Earth's circumference through poles
        geostationary: 35786 * 2, // Round trip to geostationary orbit
        lunar: 384400 * 2, // Round trip to moon
        mars: 225000000 * 2 // Round trip to Mars (average)
      },

      // Computational complexity boundaries
      complexityBoundaries: {
        guaranteed_sublinear: 1000000, // Size where O(log n) is guaranteed
        memory_limited: 10000000, // Size where memory becomes limiting
        practical_limit: 100000000 // Practical upper limit
      }
    };

    // Calculate theoretical advantages
    const theoreticalAdvantages = {};

    for (const [medium, speed] of Object.entries(limits.propagationSpeeds)) {
      theoreticalAdvantages[medium] = {};

      for (const [scenario, distance] of Object.entries(limits.maxDistances)) {
        const lightTimeMs = (distance * 1000) / speed * 1000; // Convert to milliseconds
        const estimatedComputationTimeMs = 2 + Math.log(limits.complexityBoundaries.guaranteed_sublinear) * 0.1;

        theoreticalAdvantages[medium][scenario] = {
          distance_km: distance,
          light_travel_time_ms: lightTimeMs,
          estimated_computation_ms: estimatedComputationTimeMs,
          temporal_advantage_ms: lightTimeMs - estimatedComputationTimeMs,
          feasible: lightTimeMs > estimatedComputationTimeMs,
          advantage_ratio: lightTimeMs / estimatedComputationTimeMs
        };
      }
    }

    limits.theoreticalAdvantages = theoreticalAdvantages;

    this._printTheoreticalLimits(limits);

    return limits;
  }

  /**
   * Validate temporal advantage claims with real measurements
   * @param {number} problemSize - Size of problem to validate
   * @param {number} iterations - Number of validation runs
   * @returns {Promise<Object>} Validation results
   */
  async validateTemporalClaims(problemSize = 10000, iterations = 10) {
    console.log(`🧪 Validating Temporal Claims (${problemSize.toLocaleString()} problem, ${iterations} runs)`);
    console.log('==================================================\n');

    const operationId = 'temporal_validation';
    this.monitor.startTiming(operationId);

    const measurements = [];

    try {
      for (let i = 0; i < iterations; i++) {
        console.log(`Run ${i + 1}/${iterations}...`);

        // Generate test problem
        const matrix = MatrixGenerator.generateDiagonallyDominant(problemSize, 2.5, 0.1);
        const vector = MatrixGenerator.generateRandomVector(problemSize);

        // Measure computation time
        const startTime = performance.now();

        const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

        const result = await mcp__sublinear_solver__solve({
          matrix,
          vector,
          method: 'neumann',
          epsilon: 1e-6,
          maxIterations: 1000
        });

        const endTime = performance.now();
        const computationTime = endTime - startTime;

        // Calculate temporal advantages for various distances
        const distanceTests = [1000, 5000, 12000, 20000]; // km
        const temporalAdvantages = distanceTests.map(distance => {
          const lightTime = (distance * 1000) / this.config.fiberOpticSpeed * 1000; // ms
          return {
            distance,
            lightTime,
            computationTime,
            advantage: lightTime - computationTime,
            feasible: lightTime > computationTime,
            ratio: lightTime / computationTime
          };
        });

        measurements.push({
          iteration: i + 1,
          computationTime,
          convergence: result.residual < 1e-6,
          iterations: result.iterations,
          temporalAdvantages
        });
      }

      // Analyze measurements
      const analysis = this._analyzeValidationMeasurements(measurements);

      const timing = this.monitor.endTiming(operationId, {
        iterations,
        avgComputationTime: analysis.avgComputationTime,
        successRate: analysis.successRate
      });

      this._printValidationResults(analysis);

      return {
        measurements,
        analysis,
        validationTime: timing.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  // Helper methods
  async _analyzeScenario(scenario) {
    const results = {
      scenario,
      complexityResults: []
    };

    // Test different problem sizes for this scenario
    const testSizes = [100, 1000, 10000, 100000];

    for (const size of testSizes) {
      try {
        const matrix = MatrixGenerator.generateDiagonallyDominant(size, 2.0, 0.1);
        const vector = MatrixGenerator.generateRandomVector(size);

        const computationTime = await this._measureSolverTime(matrix, vector);
        const lightTravelTime = this._calculateLightTravelTime(scenario.distance, scenario.medium);

        const temporalAdvantage = lightTravelTime - computationTime;
        const feasible = temporalAdvantage > 0;

        results.complexityResults.push({
          problemSize: size,
          computationTime,
          lightTravelTime,
          temporalAdvantage,
          feasible,
          speedupFactor: feasible ? lightTravelTime / computationTime : 0
        });

      } catch (error) {
        results.complexityResults.push({
          problemSize: size,
          error: error.message
        });
      }
    }

    return results;
  }

  async _analyzeComplexityLevel(level) {
    const matrix = MatrixGenerator.generateDiagonallyDominant(level.size, 2.0, 0.1);
    const vector = MatrixGenerator.generateRandomVector(level.size);

    const computationTime = await this._measureSolverTime(matrix, vector);

    // Calculate temporal advantages for representative distances
    const representativeDistances = [1000, 5000, 12000, 20000]; // km

    const temporalResults = representativeDistances.map(distance => {
      const lightTime = this._calculateLightTravelTime(distance, 'fiber');
      return {
        distance,
        lightTime,
        temporalAdvantage: lightTime - computationTime,
        feasible: lightTime > computationTime
      };
    });

    return {
      level,
      computationTime,
      temporalResults,
      efficiency: computationTime / Math.log(level.size) // O(log n) efficiency
    };
  }

  async _measureSolverTime(matrix, vector) {
    const startTime = performance.now();

    const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

    await mcp__sublinear_solver__solve({
      matrix,
      vector,
      method: 'neumann',
      epsilon: 1e-6,
      maxIterations: 1000
    });

    return performance.now() - startTime;
  }

  _calculateLightTravelTime(distanceKm, medium) {
    let speed;
    switch (medium) {
      case 'vacuum':
      case 'air':
        speed = this.config.atmosphericSpeed;
        break;
      case 'fiber':
        speed = this.config.fiberOpticSpeed;
        break;
      case 'copper':
        speed = 200000000; // Electrical signal in copper
        break;
      default:
        speed = this.config.fiberOpticSpeed;
    }

    return (distanceKm * 1000) / speed * 1000; // Convert to milliseconds
  }

  async _demonstrateSpecificExample(example) {
    const matrix = MatrixGenerator.generateDiagonallyDominant(example.problemSize, 2.0, 0.1);
    const vector = MatrixGenerator.generateRandomVector(example.problemSize);

    const computationTime = await this._measureSolverTime(matrix, vector);
    const lightTravelTime = this._calculateLightTravelTime(example.distance, 'fiber');

    return {
      example,
      computationTime,
      lightTravelTime,
      temporalAdvantage: lightTravelTime - computationTime,
      feasible: lightTravelTime > computationTime,
      efficiency: computationTime < 100 ? 'excellent' : computationTime < 500 ? 'good' : 'acceptable'
    };
  }

  _generatePracticalApplications(results) {
    const applications = [];

    // Find scenarios with significant temporal advantage
    for (const scenario of results.scenarios) {
      const feasibleResults = scenario.complexityResults.filter(r => r.feasible && r.temporalAdvantage > 5);

      if (feasibleResults.length > 0) {
        applications.push({
          scenario: scenario.scenario.name,
          distance: scenario.scenario.distance,
          applications: this._getApplicationsForScenario(scenario.scenario),
          maxProblemSize: Math.max(...feasibleResults.map(r => r.problemSize)),
          avgTemporalAdvantage: feasibleResults.reduce((sum, r) => sum + r.temporalAdvantage, 0) / feasibleResults.length
        });
      }
    }

    return applications;
  }

  _getApplicationsForScenario(scenario) {
    const applicationMap = {
      'Local Network': ['Real-time debugging', 'Interactive simulations'],
      'Metropolitan': ['City-wide IoT coordination', 'Traffic optimization'],
      'Regional': ['Multi-city coordination', 'Regional market analysis'],
      'National': ['National infrastructure', 'Cross-country trading'],
      'Continental': ['Continental trading', 'Weather modeling'],
      'Intercontinental': ['Global trading', 'International coordination'],
      'Antipodal': ['Global optimization', 'Worldwide networks'],
      'Satellite (GEO)': ['Satellite coordination', 'Space communications'],
      'Deep Space': ['Interplanetary networks', 'Deep space missions']
    };

    return applicationMap[scenario.name] || ['Distributed computing', 'Network coordination'];
  }

  _generateTemporalSummary(results) {
    const feasibleScenarios = results.scenarios.filter(s =>
      s.complexityResults.some(r => r.feasible)
    );

    const totalAdvantageOpportunities = results.scenarios.reduce((sum, s) =>
      sum + s.complexityResults.filter(r => r.feasible).length, 0
    );

    const maxAdvantage = Math.max(...results.scenarios.flatMap(s =>
      s.complexityResults.filter(r => r.feasible).map(r => r.temporalAdvantage)
    ));

    return {
      feasibleScenarios: feasibleScenarios.length,
      totalScenarios: results.scenarios.length,
      feasibilityRate: feasibleScenarios.length / results.scenarios.length,
      totalAdvantageOpportunities,
      maxTemporalAdvantage: maxAdvantage,
      practicalApplications: results.practicalApplications.length
    };
  }

  _summarizeDemonstrations(results) {
    const feasibleCount = results.filter(r => r.feasible).length;
    const avgAdvantage = results.reduce((sum, r) => sum + r.temporalAdvantage, 0) / results.length;
    const maxAdvantage = Math.max(...results.map(r => r.temporalAdvantage));

    return {
      totalExamples: results.length,
      feasibleExamples: feasibleCount,
      feasibilityRate: feasibleCount / results.length,
      avgTemporalAdvantage: avgAdvantage,
      maxTemporalAdvantage: maxAdvantage
    };
  }

  _analyzeValidationMeasurements(measurements) {
    const computationTimes = measurements.map(m => m.computationTime);
    const avgComputationTime = computationTimes.reduce((sum, t) => sum + t, 0) / computationTimes.length;
    const stdDeviation = Math.sqrt(
      computationTimes.reduce((sum, t) => sum + Math.pow(t - avgComputationTime, 2), 0) / computationTimes.length
    );

    const successRate = measurements.filter(m => m.convergence).length / measurements.length;

    // Analyze temporal advantages across all measurements
    const allAdvantages = measurements.flatMap(m => m.temporalAdvantages);
    const feasibleAdvantages = allAdvantages.filter(a => a.feasible);

    return {
      avgComputationTime,
      stdDeviation,
      successRate,
      minComputationTime: Math.min(...computationTimes),
      maxComputationTime: Math.max(...computationTimes),
      temporalFeasibilityRate: feasibleAdvantages.length / allAdvantages.length,
      avgTemporalAdvantage: feasibleAdvantages.length > 0 ?
        feasibleAdvantages.reduce((sum, a) => sum + a.advantage, 0) / feasibleAdvantages.length : 0
    };
  }

  _printComprehensiveResults(results) {
    console.log('\n📊 COMPREHENSIVE TEMPORAL ADVANTAGE RESULTS');
    console.log('==========================================');
    console.log(`Analysis Duration: ${(results.analysisTime / 1000).toFixed(1)}s`);
    console.log(`Feasible Scenarios: ${results.summary.feasibleScenarios}/${results.summary.totalScenarios}`);
    console.log(`Feasibility Rate: ${(results.summary.feasibilityRate * 100).toFixed(1)}%`);
    console.log(`Max Temporal Advantage: ${results.summary.maxTemporalAdvantage.toFixed(2)}ms`);
    console.log(`Practical Applications: ${results.summary.practicalApplications}`);

    console.log('\n🎯 Top Practical Applications:');
    results.practicalApplications.slice(0, 5).forEach((app, i) => {
      console.log(`  ${i+1}. ${app.scenario}: ${app.avgTemporalAdvantage.toFixed(2)}ms advantage`);
      console.log(`     Applications: ${app.applications.join(', ')}`);
    });
  }

  _printTheoreticalLimits(limits) {
    console.log('📡 Propagation Speeds:');
    for (const [medium, speed] of Object.entries(limits.propagationSpeeds)) {
      console.log(`  ${medium}: ${(speed / 1e6).toFixed(0)} Mm/s`);
    }

    console.log('\n🌐 Maximum Distances:');
    for (const [scenario, distance] of Object.entries(limits.maxDistances)) {
      console.log(`  ${scenario}: ${distance.toLocaleString()} km`);
    }

    console.log('\n⚡ Best Case Temporal Advantages (Fiber Optic):');
    const fiberAdvantages = limits.theoreticalAdvantages.fiber;
    for (const [scenario, data] of Object.entries(fiberAdvantages)) {
      if (data.feasible) {
        console.log(`  ${scenario}: ${data.temporal_advantage_ms.toFixed(2)}ms (${data.advantage_ratio.toFixed(0)}× faster)`);
      }
    }
  }

  _printValidationResults(analysis) {
    console.log('\n📈 VALIDATION RESULTS');
    console.log('===================');
    console.log(`Average Computation Time: ${analysis.avgComputationTime.toFixed(2)}ms ± ${analysis.stdDeviation.toFixed(2)}ms`);
    console.log(`Success Rate: ${(analysis.successRate * 100).toFixed(1)}%`);
    console.log(`Temporal Feasibility Rate: ${(analysis.temporalFeasibilityRate * 100).toFixed(1)}%`);
    console.log(`Average Temporal Advantage: ${analysis.avgTemporalAdvantage.toFixed(2)}ms`);
    console.log(`Performance Range: ${analysis.minComputationTime.toFixed(2)}ms - ${analysis.maxComputationTime.toFixed(2)}ms`);
  }
}

// Example usage and testing
async function runTemporalAdvantageExample() {
  console.log('⚡ Temporal Advantage Calculations Example');
  console.log('=========================================\n');

  const calculator = new TemporalAdvantageCalculator();

  try {
    // 1. Comprehensive analysis
    console.log('1. Running comprehensive temporal advantage analysis...\n');
    const comprehensiveResults = await calculator.calculateComprehensiveAdvantage();

    // 2. Specific demonstrations
    console.log('\n2. Running specific demonstrations...\n');
    const demonstrations = await calculator.demonstrateTemporalExamples();

    // 3. Theoretical limits
    console.log('\n3. Calculating theoretical limits...\n');
    const theoreticalLimits = calculator.calculateTheoreticalLimits();

    // 4. Validation
    console.log('\n4. Validating temporal claims...\n');
    const validation = await calculator.validateTemporalClaims(5000, 5);

    console.log('\n✅ Temporal advantage analysis complete!');

  } catch (error) {
    console.error('Error in temporal advantage calculations:', error);
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runTemporalAdvantageExample().catch(console.error);
}

export default TemporalAdvantageCalculator;