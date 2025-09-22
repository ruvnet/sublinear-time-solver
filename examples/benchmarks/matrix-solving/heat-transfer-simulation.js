/**
 * Heat Transfer Simulation Example
 *
 * Demonstrates solving 2D steady-state and transient heat transfer problems
 * using the sublinear-time-solver for finite difference discretization.
 */

import { MatrixGenerator } from '../utils/matrix-generator.js';
import { PerformanceMonitor } from '../utils/performance-monitor.js';

export class HeatTransferSolver {
  constructor(config = {}) {
    this.config = {
      // Grid configuration
      gridSize: [50, 50],
      domain: { x: [0, 1], y: [0, 1] },

      // Material properties
      thermalConductivity: 1.0,   // W/(m·K)
      density: 1000,              // kg/m³
      specificHeat: 1000,         // J/(kg·K)
      thermalDiffusivity: null,   // Will be calculated if null

      // Numerical parameters
      boundaryType: 'dirichlet',  // dirichlet, neumann, mixed
      discretization: 'finite_difference',
      timeIntegration: 'implicit', // explicit, implicit, crank_nicolson

      // Solver settings
      solver: {
        method: 'neumann',
        epsilon: 1e-6,
        maxIterations: 1000
      },

      ...config
    };

    // Calculate thermal diffusivity if not provided
    if (!this.config.thermalDiffusivity) {
      this.config.thermalDiffusivity = this.config.thermalConductivity /
        (this.config.density * this.config.specificHeat);
    }

    this.monitor = new PerformanceMonitor();
    this.gridPoints = this.config.gridSize[0] * this.config.gridSize[1];

    // Grid spacing
    this.dx = (this.config.domain.x[1] - this.config.domain.x[0]) / (this.config.gridSize[0] - 1);
    this.dy = (this.config.domain.y[1] - this.config.domain.y[0]) / (this.config.gridSize[1] - 1);
  }

  /**
   * Solve steady-state heat transfer problem
   * @param {Object} problem - Problem specification
   * @returns {Promise<Object>} Temperature field and solution metrics
   */
  async solveSteadyState(problem) {
    const operationId = 'steady_state_heat_transfer';
    this.monitor.startTiming(operationId, {
      gridSize: this.config.gridSize,
      boundaryType: this.config.boundaryType
    });

    try {
      // Build finite difference matrix for Laplace equation ∇²T = -q/k
      const { matrix, vector } = this._buildSteadyStateSystem(problem);

      // Solve using sublinear solver
      const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

      const solution = await mcp__sublinear_solver__solve({
        matrix,
        vector,
        method: this.config.solver.method,
        epsilon: this.config.solver.epsilon,
        maxIterations: this.config.solver.maxIterations
      });

      // Convert solution vector back to 2D temperature field
      const temperatureField = this._vectorToField(solution.solution);

      // Calculate additional metrics
      const metrics = this._calculateHeatTransferMetrics(temperatureField, problem);

      const timing = this.monitor.endTiming(operationId, {
        maxTemperature: metrics.maxTemperature,
        minTemperature: metrics.minTemperature,
        convergence: solution.residual < this.config.solver.epsilon
      });

      return {
        temperatureField,
        solution: solution.solution,
        metrics,
        convergence: {
          residual: solution.residual,
          iterations: solution.iterations,
          converged: solution.residual < this.config.solver.epsilon
        },
        computationTime: timing.duration,
        timestamp: Date.now()
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Solve transient heat transfer problem
   * @param {Object} problem - Problem specification including time parameters
   * @returns {Promise<Object>} Time evolution of temperature field
   */
  async solveTransient(problem) {
    const operationId = 'transient_heat_transfer';
    this.monitor.startTiming(operationId);

    try {
      const {
        timeStep = 0.01,
        finalTime = 1.0,
        saveInterval = 0.1,
        initialTemperature = null
      } = problem;

      // Validate CFL condition for stability
      const cflNumber = this.config.thermalDiffusivity * timeStep / (this.dx * this.dx);
      if (this.config.timeIntegration === 'explicit' && cflNumber > 0.5) {
        console.warn(`CFL number ${cflNumber.toFixed(3)} may cause instability. Consider smaller time step.`);
      }

      // Initialize temperature field
      let currentTemperature = initialTemperature ||
        Array(this.gridPoints).fill(problem.initialCondition || 0);

      // Storage for time evolution
      const timeHistory = [];
      const times = [];
      let nextSaveTime = 0;

      // Build time-stepping matrix
      const { matrix: timeMatrix, baseVector } = this._buildTransientSystem(problem, timeStep);

      // Time stepping loop
      for (let time = 0; time <= finalTime; time += timeStep) {
        // Save data at specified intervals
        if (time >= nextSaveTime) {
          timeHistory.push([...currentTemperature]);
          times.push(time);
          nextSaveTime += saveInterval;
        }

        // Build right-hand side for current time step
        const vector = this._buildTransientRHS(currentTemperature, problem, timeStep, baseVector);

        // Solve for next time step
        const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

        const solution = await mcp__sublinear_solver__solve({
          matrix: timeMatrix,
          vector,
          method: this.config.solver.method,
          epsilon: this.config.solver.epsilon * 10, // Relax tolerance for time stepping
          maxIterations: this.config.solver.maxIterations
        });

        currentTemperature = solution.solution;
      }

      // Final save
      if (times[times.length - 1] < finalTime) {
        timeHistory.push([...currentTemperature]);
        times.push(finalTime);
      }

      // Convert to 2D fields
      const temperatureFields = timeHistory.map(temp => this._vectorToField(temp));

      // Calculate transient metrics
      const transientMetrics = this._calculateTransientMetrics(temperatureFields, times, problem);

      const timing = this.monitor.endTiming(operationId, {
        timeSteps: times.length,
        finalTime,
        avgTemperature: transientMetrics.finalAvgTemperature
      });

      return {
        temperatureFields,
        times,
        transientMetrics,
        cflNumber,
        computationTime: timing.duration,
        timestamp: Date.now()
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Analyze heat transfer performance for different grid sizes
   * @param {Object} problem - Base problem specification
   * @param {Array} gridSizes - Array of grid sizes to test
   * @returns {Promise<Object>} Performance analysis results
   */
  async performanceAnalysis(problem, gridSizes = [20, 40, 60, 80, 100]) {
    const operationId = 'heat_transfer_performance_analysis';
    this.monitor.startTiming(operationId);

    try {
      const results = [];

      for (const gridSize of gridSizes) {
        // Create solver with current grid size
        const solver = new HeatTransferSolver({
          ...this.config,
          gridSize: [gridSize, gridSize]
        });

        console.log(`Testing grid size: ${gridSize}x${gridSize}`);

        // Solve steady-state problem
        const startTime = Date.now();
        const result = await solver.solveSteadyState(problem);
        const endTime = Date.now();

        // Calculate memory usage estimate
        const memoryUsage = gridSize * gridSize * 8 * 5; // Approximate bytes

        results.push({
          gridSize,
          totalPoints: gridSize * gridSize,
          computationTime: endTime - startTime,
          solverTime: result.computationTime,
          memoryUsage,
          accuracy: result.metrics.maxTemperature,
          convergence: result.convergence
        });
      }

      // Analyze scaling behavior
      const scalingAnalysis = this._analyzeScaling(results);

      const timing = this.monitor.endTiming(operationId, {
        gridSizesTested: gridSizes.length,
        scalingEfficiency: scalingAnalysis.efficiency
      });

      return {
        results,
        scalingAnalysis,
        computationTime: timing.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Generate a sample heat transfer problem
   * @param {string} problemType - Type of problem to generate
   * @returns {Object} Sample problem specification
   */
  generateSampleProblem(problemType = 'heated_plate') {
    switch (problemType) {
      case 'heated_plate':
        return {
          heatSources: [
            { x: 0.3, y: 0.3, power: 1000 }, // Heat source at (0.3, 0.3)
            { x: 0.7, y: 0.7, power: 500 }   // Smaller heat source at (0.7, 0.7)
          ],
          boundaryConditions: {
            left: { type: 'dirichlet', value: 20 },   // 20°C
            right: { type: 'dirichlet', value: 20 },  // 20°C
            top: { type: 'neumann', value: 0 },       // Insulated
            bottom: { type: 'dirichlet', value: 0 }   // 0°C (heat sink)
          },
          initialCondition: 20 // 20°C initial temperature
        };

      case 'cooling_fin':
        return {
          heatSources: [
            { x: 0.1, y: 0.5, power: 2000 } // Heat source at base
          ],
          boundaryConditions: {
            left: { type: 'dirichlet', value: 100 },  // 100°C hot base
            right: { type: 'neumann', value: -50 },   // Cooling at tip
            top: { type: 'neumann', value: -10 },     // Convective cooling
            bottom: { type: 'neumann', value: -10 }   // Convective cooling
          },
          initialCondition: 25
        };

      case 'thermal_shock':
        return {
          heatSources: [],
          boundaryConditions: {
            left: { type: 'dirichlet', value: 0 },
            right: { type: 'dirichlet', value: 0 },
            top: { type: 'dirichlet', value: 100 },   // Sudden heating
            bottom: { type: 'dirichlet', value: 0 }
          },
          initialCondition: 0,
          timeStep: 0.001,
          finalTime: 0.1
        };

      default:
        throw new Error(`Unknown problem type: ${problemType}`);
    }
  }

  // Helper methods
  _buildSteadyStateSystem(problem) {
    const n = this.gridPoints;
    const nx = this.config.gridSize[0];
    const ny = this.config.gridSize[1];

    // Build finite difference matrix for ∇²T = -q/k
    const matrix = Array(n).fill().map(() => Array(n).fill(0));
    const vector = Array(n).fill(0);

    for (let i = 0; i < ny; i++) {
      for (let j = 0; j < nx; j++) {
        const idx = i * nx + j;

        // Apply boundary conditions
        if (this._isBoundary(i, j, nx, ny)) {
          matrix[idx][idx] = 1.0;
          vector[idx] = this._getBoundaryValue(i, j, nx, ny, problem.boundaryConditions);
        } else {
          // Interior point: 5-point stencil for Laplacian
          const cx = 1 / (this.dx * this.dx);
          const cy = 1 / (this.dy * this.dy);

          matrix[idx][idx] = -2 * (cx + cy); // Center
          matrix[idx][idx - 1] = cx;         // Left
          matrix[idx][idx + 1] = cx;         // Right
          matrix[idx][idx - nx] = cy;        // Bottom
          matrix[idx][idx + nx] = cy;        // Top

          // Heat source term
          vector[idx] = -this._getHeatSource(i, j, nx, ny, problem.heatSources) /
                       this.config.thermalConductivity;
        }
      }
    }

    // Ensure diagonal dominance
    for (let i = 0; i < n; i++) {
      const rowSum = matrix[i].reduce((sum, val, j) => i !== j ? sum + Math.abs(val) : sum, 0);
      if (Math.abs(matrix[i][i]) <= rowSum) {
        matrix[i][i] = matrix[i][i] < 0 ? -(rowSum + 1) : (rowSum + 1);
      }
    }

    return {
      matrix: {
        rows: n,
        cols: n,
        format: 'dense',
        data: matrix
      },
      vector
    };
  }

  _buildTransientSystem(problem, timeStep) {
    const n = this.gridPoints;
    const nx = this.config.gridSize[0];
    const ny = this.config.gridSize[1];

    const alpha = this.config.thermalDiffusivity;
    const dtAlpha = timeStep * alpha;

    // Build matrix for implicit time stepping: (I - α*dt*∇²)T^{n+1} = T^n + dt*q
    const matrix = Array(n).fill().map(() => Array(n).fill(0));
    const baseVector = Array(n).fill(0);

    for (let i = 0; i < ny; i++) {
      for (let j = 0; j < nx; j++) {
        const idx = i * nx + j;

        if (this._isBoundary(i, j, nx, ny)) {
          matrix[idx][idx] = 1.0;
          baseVector[idx] = this._getBoundaryValue(i, j, nx, ny, problem.boundaryConditions);
        } else {
          // Interior point
          const cx = dtAlpha / (this.dx * this.dx);
          const cy = dtAlpha / (this.dy * this.dy);

          matrix[idx][idx] = 1 + 2 * (cx + cy);  // Center
          matrix[idx][idx - 1] = -cx;            // Left
          matrix[idx][idx + 1] = -cx;            // Right
          matrix[idx][idx - nx] = -cy;           // Bottom
          matrix[idx][idx + nx] = -cy;           // Top

          // Heat source contribution
          baseVector[idx] = timeStep * this._getHeatSource(i, j, nx, ny, problem.heatSources) /
                           (this.config.density * this.config.specificHeat);
        }
      }
    }

    return {
      matrix: {
        rows: n,
        cols: n,
        format: 'dense',
        data: matrix
      },
      baseVector
    };
  }

  _buildTransientRHS(currentTemperature, problem, timeStep, baseVector) {
    const vector = [...baseVector];

    // Add current temperature contribution for non-boundary points
    for (let i = 0; i < this.gridPoints; i++) {
      if (!this._isBoundaryIndex(i)) {
        vector[i] += currentTemperature[i];
      }
    }

    return vector;
  }

  _isBoundary(i, j, nx, ny) {
    return i === 0 || i === ny - 1 || j === 0 || j === nx - 1;
  }

  _isBoundaryIndex(idx) {
    const nx = this.config.gridSize[0];
    const ny = this.config.gridSize[1];
    const i = Math.floor(idx / nx);
    const j = idx % nx;
    return this._isBoundary(i, j, nx, ny);
  }

  _getBoundaryValue(i, j, nx, ny, boundaryConditions) {
    if (!boundaryConditions) return 0;

    if (i === 0) return boundaryConditions.bottom?.value || 0;
    if (i === ny - 1) return boundaryConditions.top?.value || 0;
    if (j === 0) return boundaryConditions.left?.value || 0;
    if (j === nx - 1) return boundaryConditions.right?.value || 0;

    return 0;
  }

  _getHeatSource(i, j, nx, ny, heatSources) {
    if (!heatSources) return 0;

    const x = j * this.dx + this.config.domain.x[0];
    const y = i * this.dy + this.config.domain.y[0];

    let totalHeat = 0;

    for (const source of heatSources) {
      const dx = x - source.x;
      const dy = y - source.y;
      const distance = Math.sqrt(dx * dx + dy * dy);

      // Gaussian heat source
      const sigma = 0.05; // Source width
      if (distance < 3 * sigma) {
        totalHeat += source.power * Math.exp(-(distance * distance) / (2 * sigma * sigma)) /
                    (2 * Math.PI * sigma * sigma);
      }
    }

    return totalHeat;
  }

  _vectorToField(vector) {
    const nx = this.config.gridSize[0];
    const ny = this.config.gridSize[1];
    const field = Array(ny).fill().map(() => Array(nx).fill(0));

    for (let i = 0; i < ny; i++) {
      for (let j = 0; j < nx; j++) {
        field[i][j] = vector[i * nx + j];
      }
    }

    return field;
  }

  _calculateHeatTransferMetrics(temperatureField, problem) {
    const flat = temperatureField.flat();

    const maxTemperature = Math.max(...flat);
    const minTemperature = Math.min(...flat);
    const avgTemperature = flat.reduce((sum, temp) => sum + temp, 0) / flat.length;

    // Calculate heat flux at boundaries (simplified)
    const heatFlux = this._calculateHeatFlux(temperatureField);

    // Calculate total heat generation
    const totalHeatGeneration = problem.heatSources ?
      problem.heatSources.reduce((sum, source) => sum + source.power, 0) : 0;

    return {
      maxTemperature,
      minTemperature,
      avgTemperature,
      temperatureRange: maxTemperature - minTemperature,
      heatFlux,
      totalHeatGeneration,
      thermalGradient: this._calculateMaxGradient(temperatureField)
    };
  }

  _calculateHeatFlux(temperatureField) {
    const ny = temperatureField.length;
    const nx = temperatureField[0].length;

    // Calculate heat flux through boundaries using Fourier's law: q = -k * ∇T
    const k = this.config.thermalConductivity;

    const leftFlux = -k * (temperatureField[Math.floor(ny/2)][1] - temperatureField[Math.floor(ny/2)][0]) / this.dx;
    const rightFlux = -k * (temperatureField[Math.floor(ny/2)][nx-1] - temperatureField[Math.floor(ny/2)][nx-2]) / this.dx;
    const bottomFlux = -k * (temperatureField[1][Math.floor(nx/2)] - temperatureField[0][Math.floor(nx/2)]) / this.dy;
    const topFlux = -k * (temperatureField[ny-1][Math.floor(nx/2)] - temperatureField[ny-2][Math.floor(nx/2)]) / this.dy;

    return { leftFlux, rightFlux, bottomFlux, topFlux };
  }

  _calculateMaxGradient(temperatureField) {
    const ny = temperatureField.length;
    const nx = temperatureField[0].length;
    let maxGradient = 0;

    for (let i = 1; i < ny - 1; i++) {
      for (let j = 1; j < nx - 1; j++) {
        const dTdx = (temperatureField[i][j+1] - temperatureField[i][j-1]) / (2 * this.dx);
        const dTdy = (temperatureField[i+1][j] - temperatureField[i-1][j]) / (2 * this.dy);
        const gradient = Math.sqrt(dTdx * dTdx + dTdy * dTdy);
        maxGradient = Math.max(maxGradient, gradient);
      }
    }

    return maxGradient;
  }

  _calculateTransientMetrics(temperatureFields, times, problem) {
    const finalField = temperatureFields[temperatureFields.length - 1];
    const finalFlat = finalField.flat();

    const finalAvgTemperature = finalFlat.reduce((sum, temp) => sum + temp, 0) / finalFlat.length;
    const finalMaxTemperature = Math.max(...finalFlat);

    // Calculate temperature evolution at center point
    const centerI = Math.floor(this.config.gridSize[1] / 2);
    const centerJ = Math.floor(this.config.gridSize[0] / 2);
    const centerEvolution = temperatureFields.map(field => field[centerI][centerJ]);

    // Calculate thermal penetration depth (time for significant temperature change)
    const initialTemp = centerEvolution[0];
    const finalTemp = centerEvolution[centerEvolution.length - 1];
    const thresholdTemp = initialTemp + 0.63 * (finalTemp - initialTemp); // 63% of final change

    let penetrationTime = times[times.length - 1];
    for (let t = 0; t < centerEvolution.length; t++) {
      if (centerEvolution[t] >= thresholdTemp) {
        penetrationTime = times[t];
        break;
      }
    }

    return {
      finalAvgTemperature,
      finalMaxTemperature,
      centerEvolution,
      penetrationTime,
      thermalDiffusionLength: Math.sqrt(this.config.thermalDiffusivity * penetrationTime)
    };
  }

  _analyzeScaling(results) {
    if (results.length < 2) return { efficiency: 0, trend: 'unknown' };

    // Calculate scaling efficiency (how close to O(log n) we are)
    const logScaling = [];
    for (let i = 1; i < results.length; i++) {
      const sizeRatio = results[i].totalPoints / results[i-1].totalPoints;
      const timeRatio = results[i].solverTime / results[i-1].solverTime;
      const logScaling_i = Math.log(timeRatio) / Math.log(sizeRatio);
      logScaling.push(logScaling_i);
    }

    const avgLogScaling = logScaling.reduce((sum, val) => sum + val, 0) / logScaling.length;
    const efficiency = Math.max(0, (2 - avgLogScaling) / 2); // 1.0 = perfect log scaling

    return {
      efficiency,
      avgLogScaling,
      trend: avgLogScaling < 1 ? 'sublinear' : avgLogScaling < 2 ? 'linear' : 'superlinear',
      scalingData: logScaling
    };
  }
}

// Example usage and testing
async function runHeatTransferExample() {
  console.log('🌡️ Heat Transfer Simulation Example');
  console.log('====================================\n');

  const solver = new HeatTransferSolver({
    gridSize: [80, 80],
    thermalConductivity: 50,  // Aluminum
    density: 2700,
    specificHeat: 900
  });

  try {
    // 1. Steady-state heated plate problem
    console.log('🔥 Solving steady-state heated plate...');
    const heatedPlate = solver.generateSampleProblem('heated_plate');
    const steadyResult = await solver.solveSteadyState(heatedPlate);

    console.log('Steady-State Results:');
    console.log(`  Computation Time: ${steadyResult.computationTime.toFixed(2)}ms`);
    console.log(`  Max Temperature: ${steadyResult.metrics.maxTemperature.toFixed(1)}°C`);
    console.log(`  Min Temperature: ${steadyResult.metrics.minTemperature.toFixed(1)}°C`);
    console.log(`  Avg Temperature: ${steadyResult.metrics.avgTemperature.toFixed(1)}°C`);
    console.log(`  Convergence: ${steadyResult.convergence.converged ? 'Yes' : 'No'} (${steadyResult.convergence.iterations} iterations)`);

    // 2. Transient thermal shock problem
    console.log('\n⚡ Solving transient thermal shock...');
    const thermalShock = solver.generateSampleProblem('thermal_shock');
    const transientResult = await solver.solveTransient(thermalShock);

    console.log('Transient Results:');
    console.log(`  Computation Time: ${transientResult.computationTime.toFixed(2)}ms`);
    console.log(`  Time Steps: ${transientResult.times.length}`);
    console.log(`  Final Avg Temperature: ${transientResult.transientMetrics.finalAvgTemperature.toFixed(1)}°C`);
    console.log(`  Thermal Penetration Time: ${transientResult.transientMetrics.penetrationTime.toFixed(4)}s`);
    console.log(`  CFL Number: ${transientResult.cflNumber.toFixed(3)}`);

    // 3. Performance analysis across grid sizes
    console.log('\n📊 Running performance analysis...');
    const performanceResult = await solver.performanceAnalysis(heatedPlate, [20, 40, 60, 80]);

    console.log('Performance Analysis:');
    console.log(`  Scaling Efficiency: ${(performanceResult.scalingAnalysis.efficiency * 100).toFixed(1)}%`);
    console.log(`  Scaling Trend: ${performanceResult.scalingAnalysis.trend}`);
    console.log(`  Avg Log Scaling: ${performanceResult.scalingAnalysis.avgLogScaling.toFixed(3)}`);

    console.log('\nGrid Size Performance:');
    performanceResult.results.forEach(result => {
      console.log(`  ${result.gridSize}×${result.gridSize}: ${result.solverTime.toFixed(2)}ms (${(result.memoryUsage/1024).toFixed(1)}KB)`);
    });

    // 4. Generate cooling fin analysis
    console.log('\n🔧 Analyzing cooling fin design...');
    const coolingFin = solver.generateSampleProblem('cooling_fin');
    const finResult = await solver.solveSteadyState(coolingFin);

    console.log('Cooling Fin Results:');
    console.log(`  Base Temperature: ${finResult.metrics.maxTemperature.toFixed(1)}°C`);
    console.log(`  Tip Temperature: ${finResult.metrics.minTemperature.toFixed(1)}°C`);
    console.log(`  Temperature Drop: ${(finResult.metrics.maxTemperature - finResult.metrics.minTemperature).toFixed(1)}°C`);
    console.log(`  Max Thermal Gradient: ${finResult.metrics.thermalGradient.toFixed(1)} °C/m`);

  } catch (error) {
    console.error('Error in heat transfer simulation:', error);
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runHeatTransferExample().catch(console.error);
}

export default HeatTransferSolver;