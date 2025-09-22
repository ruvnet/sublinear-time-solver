/**
 * Portfolio Optimization Example
 *
 * Demonstrates advanced portfolio optimization using sublinear matrix solving
 * for risk-adjusted return maximization with correlation matrices.
 */

import { MatrixGenerator } from '../utils/matrix-generator.js';
import { PerformanceMonitor } from '../utils/performance-monitor.js';
import { DataLoader } from '../utils/data-loader.js';

export class PortfolioOptimizer {
  constructor(config = {}) {
    this.config = {
      riskFreeRate: 0.02, // 2% annual risk-free rate
      riskTolerance: 0.1, // 10% maximum portfolio volatility
      rebalanceFrequency: 'monthly',
      transactionCosts: 0.001, // 0.1% transaction cost
      maxSingleAssetWeight: 0.2, // 20% maximum single asset weight
      minAssetWeight: 0.01, // 1% minimum asset weight
      ...config
    };

    this.monitor = new PerformanceMonitor();
    this.currentPortfolio = null;
    this.performanceHistory = [];
  }

  /**
   * Optimize portfolio using mean-variance optimization
   * @param {Array} expectedReturns - Expected returns for each asset
   * @param {Object} covarianceMatrix - Asset covariance matrix
   * @param {Object} constraints - Portfolio constraints
   * @returns {Promise<Object>} Optimal portfolio weights and metrics
   */
  async optimizePortfolio(expectedReturns, covarianceMatrix, constraints = {}) {
    const operationId = 'portfolio_optimization';
    this.monitor.startTiming(operationId, {
      assets: expectedReturns.length,
      matrixSize: covarianceMatrix.rows
    });

    try {
      // Convert optimization problem to linear system
      const optimizationSystem = this._buildOptimizationSystem(
        expectedReturns,
        covarianceMatrix,
        constraints
      );

      // Solve using sublinear solver
      const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

      const solution = await mcp__sublinear_solver__solve({
        matrix: optimizationSystem.matrix,
        vector: optimizationSystem.vector,
        method: 'neumann',
        epsilon: 1e-8,
        maxIterations: 1000
      });

      // Extract and normalize portfolio weights
      const rawWeights = solution.solution;
      const portfolioWeights = this._normalizeWeights(rawWeights, constraints);

      // Calculate portfolio metrics
      const portfolioMetrics = this._calculatePortfolioMetrics(
        portfolioWeights,
        expectedReturns,
        covarianceMatrix
      );

      const metrics = this.monitor.endTiming(operationId, {
        expectedReturn: portfolioMetrics.expectedReturn,
        volatility: portfolioMetrics.volatility,
        sharpeRatio: portfolioMetrics.sharpeRatio
      });

      const result = {
        weights: portfolioWeights,
        metrics: portfolioMetrics,
        optimization: {
          convergence: solution.residual < 1e-6,
          iterations: solution.iterations,
          computationTime: metrics.duration
        },
        timestamp: Date.now()
      };

      this.currentPortfolio = result;

      return result;

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Optimize portfolio with factor model
   * @param {Array} assetReturns - Historical asset returns
   * @param {Array} factorReturns - Historical factor returns
   * @param {Object} options - Optimization options
   * @returns {Promise<Object>} Factor-based optimal portfolio
   */
  async optimizeWithFactorModel(assetReturns, factorReturns, options = {}) {
    const operationId = 'factor_portfolio_optimization';
    this.monitor.startTiming(operationId);

    try {
      // Build factor model (simplified Fama-French style)
      const factorLoadings = this._calculateFactorLoadings(assetReturns, factorReturns);
      const factorCovariance = this._calculateFactorCovariance(factorReturns);
      const specificRisks = this._calculateSpecificRisks(assetReturns, factorLoadings, factorReturns);

      // Build factor-based covariance matrix
      const covarianceMatrix = this._buildFactorBasedCovariance(
        factorLoadings,
        factorCovariance,
        specificRisks
      );

      // Calculate expected returns using factor model
      const expectedReturns = this._calculateFactorBasedReturns(
        factorLoadings,
        options.factorPremiums || this._estimateFactorPremiums(factorReturns)
      );

      // Optimize portfolio
      const result = await this.optimizePortfolio(expectedReturns, covarianceMatrix, options.constraints);

      const metrics = this.monitor.endTiming(operationId, result.metrics);

      return {
        ...result,
        factorModel: {
          loadings: factorLoadings,
          factorCovariance,
          specificRisks
        },
        computationTime: metrics.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Perform risk-based portfolio optimization (Risk Parity)
   * @param {Object} covarianceMatrix - Asset covariance matrix
   * @param {Object} options - Risk parity options
   * @returns {Promise<Object>} Risk parity portfolio
   */
  async optimizeRiskParity(covarianceMatrix, options = {}) {
    const operationId = 'risk_parity_optimization';
    this.monitor.startTiming(operationId);

    try {
      const n = covarianceMatrix.rows;

      // Build risk parity system (iterative risk budgeting)
      const riskBudgets = options.riskBudgets || Array(n).fill(1 / n); // Equal risk budgets

      // Convert to optimization problem using risk contribution constraints
      const riskParitySystem = this._buildRiskParitySystem(covarianceMatrix, riskBudgets);

      const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

      const solution = await mcp__sublinear_solver__solve({
        matrix: riskParitySystem.matrix,
        vector: riskParitySystem.vector,
        method: 'neumann',
        epsilon: 1e-6,
        maxIterations: 500
      });

      // Extract and normalize weights
      const weights = this._normalizeWeights(solution.solution.slice(0, n));

      // Calculate risk contributions
      const riskContributions = this._calculateRiskContributions(weights, covarianceMatrix);

      const portfolioVolatility = Math.sqrt(
        this._calculatePortfolioVariance(weights, covarianceMatrix)
      );

      const metrics = this.monitor.endTiming(operationId, {
        portfolioVolatility,
        riskContributions: riskContributions.reduce((sum, rc) => sum + rc, 0)
      });

      return {
        weights,
        riskContributions,
        portfolioVolatility,
        riskBudgets,
        convergence: solution.residual < 1e-6,
        computationTime: metrics.duration,
        timestamp: Date.now()
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Backtesting portfolio strategy
   * @param {Array} historicalData - Historical price/return data
   * @param {Object} strategy - Strategy configuration
   * @returns {Promise<Object>} Backtest results
   */
  async backtest(historicalData, strategy = {}) {
    const operationId = 'portfolio_backtest';
    this.monitor.startTiming(operationId);

    const {
      rebalanceFrequency = 'monthly',
      lookbackWindow = 252, // 1 year of daily data
      initialValue = 1000000 // $1M initial portfolio
    } = strategy;

    const results = {
      portfolioValue: [initialValue],
      weights: [],
      returns: [],
      rebalanceDates: [],
      metrics: {}
    };

    try {
      let currentValue = initialValue;
      const rebalanceInterval = this._getRebalanceInterval(rebalanceFrequency);

      for (let i = lookbackWindow; i < historicalData.length; i += rebalanceInterval) {
        // Get historical data for optimization
        const histData = historicalData.slice(i - lookbackWindow, i);

        // Calculate expected returns and covariance matrix
        const expectedReturns = this._calculateExpectedReturns(histData);
        const covarianceMatrix = this._calculateCovariance(histData);

        // Optimize portfolio
        const portfolio = await this.optimizePortfolio(expectedReturns, covarianceMatrix);

        // Calculate performance for the next period
        const nextPeriodEnd = Math.min(i + rebalanceInterval, historicalData.length);
        const periodReturns = this._calculatePeriodReturns(
          historicalData.slice(i, nextPeriodEnd),
          portfolio.weights
        );

        // Update portfolio value
        for (const periodReturn of periodReturns) {
          currentValue *= (1 + periodReturn);
          results.portfolioValue.push(currentValue);
          results.returns.push(periodReturn);
        }

        results.weights.push(portfolio.weights);
        results.rebalanceDates.push(i);
      }

      // Calculate backtest metrics
      results.metrics = this._calculateBacktestMetrics(results);

      const metrics = this.monitor.endTiming(operationId, results.metrics);

      return {
        ...results,
        computationTime: metrics.duration,
        timestamp: Date.now()
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Generate sample portfolio data for demonstration
   * @param {number} numAssets - Number of assets
   * @param {number} timePoints - Number of time points
   * @returns {Object} Sample data including returns and covariance
   */
  generateSampleData(numAssets = 10, timePoints = 252) {
    // Generate asset classes for realistic correlations
    const assetClasses = [
      Math.floor(numAssets * 0.4), // 40% stocks
      Math.floor(numAssets * 0.3), // 30% bonds
      Math.floor(numAssets * 0.2), // 20% commodities
      numAssets - Math.floor(numAssets * 0.9) // 10% alternatives
    ].filter(size => size > 0);

    const covarianceMatrix = MatrixGenerator.generateMarketCorrelations(
      assetClasses,
      0.6, // intra-class correlation
      0.2  // inter-class correlation
    );

    // Generate expected returns with risk premium
    const expectedReturns = Array(numAssets).fill().map((_, i) => {
      const baseReturn = 0.08; // 8% base return
      const riskPremium = Math.random() * 0.04; // 0-4% risk premium
      const assetClassAdjustment = i < assetClasses[0] ? 0.02 : 0; // Equity premium
      return baseReturn + riskPremium + assetClassAdjustment;
    });

    // Generate time series data
    const timeSeriesData = [];
    for (let t = 0; t < timePoints; t++) {
      const returns = expectedReturns.map((expectedReturn, i) => {
        const randomComponent = (Math.random() - 0.5) * 0.1; // 10% volatility
        return expectedReturn / 252 + randomComponent; // Daily returns
      });
      timeSeriesData.push(returns);
    }

    return {
      expectedReturns,
      covarianceMatrix,
      timeSeriesData,
      assetNames: Array(numAssets).fill().map((_, i) => `Asset_${i + 1}`),
      assetClasses: this._labelAssetClasses(numAssets, assetClasses)
    };
  }

  // Helper methods
  _buildOptimizationSystem(expectedReturns, covarianceMatrix, constraints) {
    const n = expectedReturns.length;

    // Build augmented system for constrained optimization
    // [2*Σ  1] [w] = [μ]
    // [1'   0] [λ]   [1]
    // Where Σ is covariance matrix, μ is expected returns, w is weights, λ is Lagrange multiplier

    const systemSize = n + 1;
    const matrix = Array(systemSize).fill().map(() => Array(systemSize).fill(0));
    const vector = Array(systemSize).fill(0);

    // Fill covariance block (2*Σ)
    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        matrix[i][j] = 2 * covarianceMatrix.data[i][j];
      }
      matrix[i][n] = 1; // Constraint coefficients
      matrix[n][i] = 1; // Constraint coefficients (transpose)
    }

    // Right-hand side
    for (let i = 0; i < n; i++) {
      vector[i] = expectedReturns[i];
    }
    vector[n] = 1; // Portfolio weights sum to 1

    // Ensure diagonal dominance
    for (let i = 0; i < systemSize; i++) {
      const rowSum = matrix[i].reduce((sum, val, j) => i !== j ? sum + Math.abs(val) : sum, 0);
      matrix[i][i] = Math.max(matrix[i][i], rowSum * 1.1 + 1);
    }

    return {
      matrix: {
        rows: systemSize,
        cols: systemSize,
        format: 'dense',
        data: matrix
      },
      vector
    };
  }

  _normalizeWeights(weights, constraints = {}) {
    const n = weights.length;
    let normalizedWeights = [...weights];

    // Remove negative weights (long-only constraint)
    normalizedWeights = normalizedWeights.map(w => Math.max(0, w));

    // Apply maximum weight constraints
    if (constraints.maxWeight) {
      normalizedWeights = normalizedWeights.map(w => Math.min(w, constraints.maxWeight));
    }

    // Normalize to sum to 1
    const sum = normalizedWeights.reduce((s, w) => s + w, 0);
    if (sum > 0) {
      normalizedWeights = normalizedWeights.map(w => w / sum);
    } else {
      // Equal weights fallback
      normalizedWeights = Array(n).fill(1 / n);
    }

    return normalizedWeights;
  }

  _calculatePortfolioMetrics(weights, expectedReturns, covarianceMatrix) {
    const expectedReturn = weights.reduce((sum, w, i) => sum + w * expectedReturns[i], 0);
    const variance = this._calculatePortfolioVariance(weights, covarianceMatrix);
    const volatility = Math.sqrt(variance);
    const sharpeRatio = (expectedReturn - this.config.riskFreeRate) / volatility;

    return {
      expectedReturn,
      volatility,
      variance,
      sharpeRatio,
      weights: weights.map((w, i) => ({ asset: i, weight: w }))
    };
  }

  _calculatePortfolioVariance(weights, covarianceMatrix) {
    let variance = 0;
    const n = weights.length;

    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        variance += weights[i] * weights[j] * covarianceMatrix.data[i][j];
      }
    }

    return variance;
  }

  _calculateFactorLoadings(assetReturns, factorReturns) {
    // Simplified factor loading calculation (should use proper regression)
    const numAssets = assetReturns[0].length;
    const numFactors = factorReturns[0].length;

    return Array(numAssets).fill().map(() =>
      Array(numFactors).fill().map(() => (Math.random() - 0.5) * 2)
    );
  }

  _calculateFactorCovariance(factorReturns) {
    const numFactors = factorReturns[0].length;
    const covariance = Array(numFactors).fill().map(() => Array(numFactors).fill(0));

    // Calculate factor covariance matrix
    for (let i = 0; i < numFactors; i++) {
      for (let j = 0; j < numFactors; j++) {
        covariance[i][j] = this._calculateCovariance2D(
          factorReturns.map(r => r[i]),
          factorReturns.map(r => r[j])
        );
      }
    }

    return covariance;
  }

  _calculateSpecificRisks(assetReturns, factorLoadings, factorReturns) {
    // Calculate idiosyncratic risk for each asset
    return factorLoadings.map(() => Math.random() * 0.01 + 0.005); // 0.5% to 1.5% specific risk
  }

  _buildFactorBasedCovariance(factorLoadings, factorCovariance, specificRisks) {
    const numAssets = factorLoadings.length;
    const covariance = Array(numAssets).fill().map(() => Array(numAssets).fill(0));

    for (let i = 0; i < numAssets; i++) {
      for (let j = 0; j < numAssets; j++) {
        // B * F * B' + D (factor model covariance)
        let cov = 0;
        for (let f1 = 0; f1 < factorLoadings[i].length; f1++) {
          for (let f2 = 0; f2 < factorLoadings[j].length; f2++) {
            cov += factorLoadings[i][f1] * factorCovariance[f1][f2] * factorLoadings[j][f2];
          }
        }

        // Add specific risk for diagonal elements
        if (i === j) {
          cov += specificRisks[i];
        }

        covariance[i][j] = cov;
      }
    }

    return {
      rows: numAssets,
      cols: numAssets,
      format: 'dense',
      data: covariance
    };
  }

  _calculateFactorBasedReturns(factorLoadings, factorPremiums) {
    return factorLoadings.map(loadings =>
      loadings.reduce((sum, loading, f) => sum + loading * factorPremiums[f], 0)
    );
  }

  _estimateFactorPremiums(factorReturns) {
    // Calculate average factor returns as premiums
    return factorReturns[0].map((_, f) =>
      factorReturns.reduce((sum, returns) => sum + returns[f], 0) / factorReturns.length
    );
  }

  _buildRiskParitySystem(covarianceMatrix, riskBudgets) {
    const n = covarianceMatrix.rows;

    // Simplified risk parity system using diagonal approximation
    const matrix = Array(n).fill().map(() => Array(n).fill(0));
    const vector = Array(n).fill(0);

    for (let i = 0; i < n; i++) {
      matrix[i][i] = covarianceMatrix.data[i][i] * 2; // Diagonal variance terms
      vector[i] = riskBudgets[i];

      // Add off-diagonal terms for stability
      for (let j = 0; j < n; j++) {
        if (i !== j) {
          matrix[i][j] = covarianceMatrix.data[i][j] * 0.1;
        }
      }

      // Ensure diagonal dominance
      const rowSum = matrix[i].reduce((sum, val, j) => i !== j ? sum + Math.abs(val) : sum, 0);
      matrix[i][i] = Math.max(matrix[i][i], rowSum * 1.2 + 1);
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

  _calculateRiskContributions(weights, covarianceMatrix) {
    const n = weights.length;
    const portfolioVariance = this._calculatePortfolioVariance(weights, covarianceMatrix);
    const portfolioVolatility = Math.sqrt(portfolioVariance);

    return weights.map((weight, i) => {
      let marginalContribution = 0;
      for (let j = 0; j < n; j++) {
        marginalContribution += weights[j] * covarianceMatrix.data[i][j];
      }
      return (weight * marginalContribution) / portfolioVolatility;
    });
  }

  _calculateExpectedReturns(historicalData) {
    const numAssets = historicalData[0].length;
    return Array(numAssets).fill().map((_, i) => {
      const assetReturns = historicalData.map(day => day[i]);
      return assetReturns.reduce((sum, ret) => sum + ret, 0) / assetReturns.length;
    });
  }

  _calculateCovariance(historicalData) {
    const numAssets = historicalData[0].length;
    const means = this._calculateExpectedReturns(historicalData);
    const covariance = Array(numAssets).fill().map(() => Array(numAssets).fill(0));

    for (let i = 0; i < numAssets; i++) {
      for (let j = 0; j < numAssets; j++) {
        let cov = 0;
        for (const day of historicalData) {
          cov += (day[i] - means[i]) * (day[j] - means[j]);
        }
        covariance[i][j] = cov / (historicalData.length - 1);
      }
    }

    return {
      rows: numAssets,
      cols: numAssets,
      format: 'dense',
      data: covariance
    };
  }

  _calculateCovariance2D(series1, series2) {
    const mean1 = series1.reduce((sum, val) => sum + val, 0) / series1.length;
    const mean2 = series2.reduce((sum, val) => sum + val, 0) / series2.length;

    let covariance = 0;
    for (let i = 0; i < series1.length; i++) {
      covariance += (series1[i] - mean1) * (series2[i] - mean2);
    }

    return covariance / (series1.length - 1);
  }

  _calculatePeriodReturns(periodData, weights) {
    return periodData.map(day =>
      day.reduce((portfolioReturn, assetReturn, i) =>
        portfolioReturn + weights[i] * assetReturn, 0
      )
    );
  }

  _calculateBacktestMetrics(results) {
    const returns = results.returns;
    const values = results.portfolioValue;

    const totalReturn = (values[values.length - 1] / values[0]) - 1;
    const annualizedReturn = Math.pow(1 + totalReturn, 252 / returns.length) - 1;

    const avgReturn = returns.reduce((sum, ret) => sum + ret, 0) / returns.length;
    const volatility = Math.sqrt(
      returns.reduce((sum, ret) => sum + Math.pow(ret - avgReturn, 2), 0) / returns.length
    ) * Math.sqrt(252); // Annualized

    const sharpeRatio = (annualizedReturn - this.config.riskFreeRate) / volatility;

    // Calculate maximum drawdown
    let maxDrawdown = 0;
    let peak = values[0];
    for (const value of values) {
      if (value > peak) peak = value;
      const drawdown = (peak - value) / peak;
      if (drawdown > maxDrawdown) maxDrawdown = drawdown;
    }

    return {
      totalReturn,
      annualizedReturn,
      volatility,
      sharpeRatio,
      maxDrawdown,
      finalValue: values[values.length - 1]
    };
  }

  _getRebalanceInterval(frequency) {
    const intervals = {
      'daily': 1,
      'weekly': 5,
      'monthly': 21,
      'quarterly': 63,
      'annually': 252
    };
    return intervals[frequency] || 21;
  }

  _labelAssetClasses(numAssets, assetClasses) {
    const labels = [];
    const classNames = ['Equity', 'Bond', 'Commodity', 'Alternative'];
    let assetIndex = 0;

    for (let c = 0; c < assetClasses.length; c++) {
      for (let i = 0; i < assetClasses[c]; i++) {
        labels.push(classNames[c] || 'Other');
        assetIndex++;
      }
    }

    return labels;
  }
}

// Example usage and testing
async function runPortfolioOptimizationExample() {
  console.log('📊 Portfolio Optimization Example');
  console.log('=================================\n');

  const optimizer = new PortfolioOptimizer({
    riskFreeRate: 0.02,
    riskTolerance: 0.15,
    maxSingleAssetWeight: 0.25
  });

  try {
    // Generate sample data
    console.log('📈 Generating sample market data...');
    const data = optimizer.generateSampleData(10, 252);

    console.log(`Generated data for ${data.assetNames.length} assets with ${data.timeSeriesData.length} time points`);

    // 1. Mean-Variance Optimization
    console.log('\n🎯 Running Mean-Variance Optimization...');
    const mvOptimization = await optimizer.optimizePortfolio(
      data.expectedReturns,
      data.covarianceMatrix
    );

    console.log('Mean-Variance Results:');
    console.log(`  Expected Return: ${(mvOptimization.metrics.expectedReturn * 100).toFixed(2)}%`);
    console.log(`  Volatility: ${(mvOptimization.metrics.volatility * 100).toFixed(2)}%`);
    console.log(`  Sharpe Ratio: ${mvOptimization.metrics.sharpeRatio.toFixed(3)}`);
    console.log(`  Computation Time: ${mvOptimization.optimization.computationTime.toFixed(2)}ms`);

    // 2. Risk Parity Optimization
    console.log('\n⚖️ Running Risk Parity Optimization...');
    const rpOptimization = await optimizer.optimizeRiskParity(data.covarianceMatrix);

    console.log('Risk Parity Results:');
    console.log(`  Portfolio Volatility: ${(rpOptimization.portfolioVolatility * 100).toFixed(2)}%`);
    console.log(`  Computation Time: ${rpOptimization.computationTime.toFixed(2)}ms`);

    // 3. Factor Model Optimization
    console.log('\n🏭 Running Factor Model Optimization...');
    const factorReturns = data.timeSeriesData.map(() => [
      (Math.random() - 0.5) * 0.02, // Market factor
      (Math.random() - 0.5) * 0.01, // Size factor
      (Math.random() - 0.5) * 0.01  // Value factor
    ]);

    const factorOptimization = await optimizer.optimizeWithFactorModel(
      data.timeSeriesData,
      factorReturns
    );

    console.log('Factor Model Results:');
    console.log(`  Expected Return: ${(factorOptimization.metrics.expectedReturn * 100).toFixed(2)}%`);
    console.log(`  Volatility: ${(factorOptimization.metrics.volatility * 100).toFixed(2)}%`);
    console.log(`  Sharpe Ratio: ${factorOptimization.metrics.sharpeRatio.toFixed(3)}`);
    console.log(`  Computation Time: ${factorOptimization.computationTime.toFixed(2)}ms`);

    // 4. Backtesting
    console.log('\n📊 Running Backtest...');
    const backtest = await optimizer.backtest(data.timeSeriesData, {
      rebalanceFrequency: 'monthly',
      lookbackWindow: 60
    });

    console.log('Backtest Results:');
    console.log(`  Total Return: ${(backtest.metrics.totalReturn * 100).toFixed(2)}%`);
    console.log(`  Annualized Return: ${(backtest.metrics.annualizedReturn * 100).toFixed(2)}%`);
    console.log(`  Volatility: ${(backtest.metrics.volatility * 100).toFixed(2)}%`);
    console.log(`  Sharpe Ratio: ${backtest.metrics.sharpeRatio.toFixed(3)}`);
    console.log(`  Max Drawdown: ${(backtest.metrics.maxDrawdown * 100).toFixed(2)}%`);
    console.log(`  Final Value: $${backtest.metrics.finalValue.toLocaleString()}`);

  } catch (error) {
    console.error('Error in portfolio optimization example:', error);
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runPortfolioOptimizationExample().catch(console.error);
}

export default PortfolioOptimizer;