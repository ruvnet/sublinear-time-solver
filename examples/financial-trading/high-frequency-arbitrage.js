/**
 * High-Frequency Arbitrage Example
 *
 * Demonstrates temporal advantage in high-frequency trading by detecting
 * and exploiting arbitrage opportunities before market data propagates
 * to all market participants.
 */

import { performance } from 'perf_hooks';
import { MatrixGenerator } from '../utils/matrix-generator.js';
import { PerformanceMonitor } from '../utils/performance-monitor.js';

export class HighFrequencyArbitrage {
  constructor(config = {}) {
    this.config = {
      exchanges: ['NYSE', 'NASDAQ', 'LSE'],
      symbols: ['AAPL', 'MSFT', 'GOOGL', 'AMZN'],
      temporalAdvantageThreshold: 5, // 5ms minimum
      maxPositionSize: 100000,
      riskTolerance: 0.01,
      latencyTargets: {
        'NYSE-NASDAQ': 0.5,    // 500μs (same city)
        'NYSE-LSE': 18.6,      // 18.6ms (trans-Atlantic)
        'NYSE-TSE': 36.3,      // 36.3ms (trans-Pacific)
      },
      ...config
    };

    this.monitor = new PerformanceMonitor();
    this.positions = new Map();
    this.isRunning = false;
  }

  /**
   * Start the arbitrage detection system
   */
  async start() {
    console.log('🚀 Starting High-Frequency Arbitrage System');
    console.log(`📊 Monitoring ${this.config.symbols.length} symbols across ${this.config.exchanges.length} exchanges`);

    this.isRunning = true;

    // Simulate market data feeds
    await this.simulateMarketDataFeed();
  }

  /**
   * Stop the arbitrage system
   */
  stop() {
    this.isRunning = false;
    console.log('🛑 Arbitrage system stopped');
  }

  /**
   * Detect arbitrage opportunities using temporal advantage
   * @param {Object} marketData - Real-time market data
   * @returns {Promise<Array>} Arbitrage opportunities
   */
  async detectArbitrageOpportunities(marketData) {
    const operationId = 'arbitrage_detection';
    this.monitor.startTiming(operationId, {
      symbols: marketData.length,
      exchanges: this.config.exchanges.length
    });

    try {
      // Build price correlation matrix
      const priceMatrix = this._buildPriceMatrix(marketData);

      // Use sublinear solver to detect price discrepancies
      const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

      const arbitrageSystem = {
        matrix: priceMatrix,
        vector: this._generateExpectedPrices(marketData),
        method: 'neumann',
        epsilon: 1e-6,
        maxIterations: 100
      };

      const solution = await mcp__sublinear_solver__solve(arbitrageSystem);

      // Calculate temporal advantage for each opportunity
      const opportunities = this._identifyOpportunities(solution, marketData);

      const metrics = this.monitor.endTiming(operationId, {
        opportunitiesFound: opportunities.length,
        avgPriceDivergence: this._calculateAvgDivergence(opportunities)
      });

      // Filter opportunities by temporal advantage
      const feasibleOpportunities = opportunities.filter(opp => {
        const temporalAdvantage = this.monitor.calculateTemporalAdvantage(
          this._getExchangeDistance(opp.buyExchange, opp.sellExchange),
          metrics.duration
        );
        return temporalAdvantage.feasible && temporalAdvantage.temporalAdvantageMs > this.config.temporalAdvantageThreshold;
      });

      return feasibleOpportunities;

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Execute arbitrage trades with temporal advantage
   * @param {Array} opportunities - Detected arbitrage opportunities
   */
  async executeArbitrageTrades(opportunities) {
    const results = [];

    for (const opportunity of opportunities) {
      try {
        const executionId = `execute_${opportunity.symbol}_${Date.now()}`;
        this.monitor.startTiming(executionId, { opportunity });

        // Calculate optimal position size
        const positionSize = this._calculateOptimalPositionSize(opportunity);

        // Execute trades simultaneously (buy low, sell high)
        const [buyResult, sellResult] = await Promise.all([
          this._executeBuyOrder(opportunity, positionSize),
          this._executeSellOrder(opportunity, positionSize)
        ]);

        const profit = this._calculateProfit(buyResult, sellResult, positionSize);

        const metrics = this.monitor.endTiming(executionId, {
          profit,
          positionSize,
          buyPrice: buyResult.price,
          sellPrice: sellResult.price
        });

        results.push({
          opportunity,
          execution: { buyResult, sellResult },
          profit,
          metrics
        });

        console.log(`✅ Arbitrage executed: ${opportunity.symbol} | Profit: $${profit.toFixed(2)} | Duration: ${metrics.duration.toFixed(2)}ms`);

      } catch (error) {
        console.error(`❌ Failed to execute arbitrage for ${opportunity.symbol}:`, error.message);
        results.push({
          opportunity,
          error: error.message
        });
      }
    }

    return results;
  }

  /**
   * Simulate real-time market data feed
   */
  async simulateMarketDataFeed() {
    let iteration = 0;

    while (this.isRunning && iteration < 100) { // Limit for demo
      try {
        // Generate simulated market data
        const marketData = this._generateSimulatedMarketData();

        // Detect arbitrage opportunities
        const opportunities = await this.detectArbitrageOpportunities(marketData);

        if (opportunities.length > 0) {
          console.log(`🎯 Found ${opportunities.length} arbitrage opportunities`);

          // Execute profitable trades
          const results = await this.executeArbitrageTrades(opportunities);

          // Update positions and P&L
          this._updatePositions(results);
        }

        // Wait before next iteration (simulating real-time feed)
        await new Promise(resolve => setTimeout(resolve, 100)); // 100ms intervals

        iteration++;

      } catch (error) {
        console.error('Error in market data processing:', error.message);
        await new Promise(resolve => setTimeout(resolve, 1000)); // Wait longer on error
      }
    }

    // Generate final report
    this._generateReport();
  }

  /**
   * Generate performance report
   */
  _generateReport() {
    const report = this.monitor.generateReport({ includeSystemInfo: true });

    console.log('\n📊 ARBITRAGE PERFORMANCE REPORT');
    console.log('==================================');
    console.log(`Total Operations: ${report.summary.totalOperations}`);
    console.log(`Completed Operations: ${report.summary.completedOperations}`);
    console.log(`Average Duration: ${report.summary.avgDuration.toFixed(2)}ms`);

    // Calculate total P&L
    const totalPnL = Array.from(this.positions.values())
      .reduce((sum, position) => sum + position.unrealizedPnL, 0);

    console.log(`Total P&L: $${totalPnL.toFixed(2)}`);

    // Performance by symbol
    console.log('\nPerformance by Symbol:');
    for (const [symbol, position] of this.positions) {
      console.log(`  ${symbol}: $${position.unrealizedPnL.toFixed(2)}`);
    }

    return report;
  }

  // Helper methods
  _buildPriceMatrix(marketData) {
    const n = marketData.length;
    const matrix = Array(n).fill().map(() => Array(n).fill(0));

    // Build correlation-based price matrix
    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        if (i === j) {
          matrix[i][j] = 1.0; // Perfect self-correlation
        } else {
          // Calculate price correlation (simplified)
          const correlation = this._calculatePriceCorrelation(marketData[i], marketData[j]);
          matrix[i][j] = correlation;
        }
      }

      // Ensure diagonal dominance for solver stability
      const rowSum = matrix[i].reduce((sum, val, idx) => idx !== i ? sum + Math.abs(val) : sum, 0);
      matrix[i][i] = rowSum * 1.1 + 1;
    }

    return {
      rows: n,
      cols: n,
      format: 'dense',
      data: matrix
    };
  }

  _generateExpectedPrices(marketData) {
    // Generate expected price vector based on theoretical fair value
    return marketData.map(data => {
      const midPrice = (data.bid + data.ask) / 2;
      const theoreticalValue = midPrice * (1 + (Math.random() - 0.5) * 0.001); // Small random adjustment
      return theoreticalValue;
    });
  }

  _identifyOpportunities(solution, marketData) {
    const opportunities = [];
    const priceDivergences = solution.solution;

    for (let i = 0; i < marketData.length; i++) {
      const data = marketData[i];
      const divergence = Math.abs(priceDivergences[i]);

      if (divergence > 0.001) { // 0.1% minimum divergence threshold
        // Determine if we should buy or sell
        const isBuyOpportunity = priceDivergences[i] > 0;

        opportunities.push({
          symbol: data.symbol,
          divergence,
          buyExchange: isBuyOpportunity ? data.exchange : this._findAlternativeExchange(data.exchange),
          sellExchange: isBuyOpportunity ? this._findAlternativeExchange(data.exchange) : data.exchange,
          buyPrice: isBuyOpportunity ? data.ask : data.bid,
          sellPrice: isBuyOpportunity ? data.bid : data.ask,
          expectedProfit: divergence * data.last,
          confidence: Math.min(1, divergence * 100) // Confidence based on divergence magnitude
        });
      }
    }

    return opportunities.sort((a, b) => b.expectedProfit - a.expectedProfit);
  }

  _calculateOptimalPositionSize(opportunity) {
    const maxRiskAmount = this.config.maxPositionSize * this.config.riskTolerance;
    const riskPerShare = Math.abs(opportunity.buyPrice - opportunity.sellPrice) * 0.1; // 10% of spread as risk
    const riskBasedSize = Math.floor(maxRiskAmount / riskPerShare);

    return Math.min(riskBasedSize, this.config.maxPositionSize);
  }

  async _executeBuyOrder(opportunity, positionSize) {
    // Simulate order execution with realistic latency
    const latency = Math.random() * 2 + 1; // 1-3ms execution time
    await new Promise(resolve => setTimeout(resolve, latency));

    return {
      symbol: opportunity.symbol,
      side: 'buy',
      quantity: positionSize,
      price: opportunity.buyPrice * (1 + (Math.random() - 0.5) * 0.0001), // Small slippage
      exchange: opportunity.buyExchange,
      timestamp: Date.now()
    };
  }

  async _executeSellOrder(opportunity, positionSize) {
    // Simulate order execution with realistic latency
    const latency = Math.random() * 2 + 1; // 1-3ms execution time
    await new Promise(resolve => setTimeout(resolve, latency));

    return {
      symbol: opportunity.symbol,
      side: 'sell',
      quantity: positionSize,
      price: opportunity.sellPrice * (1 + (Math.random() - 0.5) * 0.0001), // Small slippage
      exchange: opportunity.sellExchange,
      timestamp: Date.now()
    };
  }

  _calculateProfit(buyResult, sellResult, positionSize) {
    const grossProfit = (sellResult.price - buyResult.price) * positionSize;
    const commission = positionSize * 0.001; // $0.001 per share commission
    return grossProfit - commission;
  }

  _updatePositions(results) {
    for (const result of results) {
      if (result.profit) {
        const symbol = result.opportunity.symbol;
        const currentPosition = this.positions.get(symbol) || {
          quantity: 0,
          avgPrice: 0,
          unrealizedPnL: 0
        };

        currentPosition.unrealizedPnL += result.profit;
        this.positions.set(symbol, currentPosition);
      }
    }
  }

  _generateSimulatedMarketData() {
    return this.config.symbols.map(symbol => {
      const basePrice = 100 + Math.random() * 100; // $100-200 range
      const spread = basePrice * 0.001; // 0.1% spread

      return {
        symbol,
        bid: basePrice - spread / 2,
        ask: basePrice + spread / 2,
        last: basePrice,
        volume: Math.floor(Math.random() * 1000000),
        exchange: this.config.exchanges[Math.floor(Math.random() * this.config.exchanges.length)],
        timestamp: Date.now()
      };
    });
  }

  _calculatePriceCorrelation(data1, data2) {
    if (data1.symbol === data2.symbol) {
      return 0.99; // Very high correlation for same symbol on different exchanges
    }

    // Simplified correlation based on sector/market cap (in reality, would use historical data)
    const sectorCorrelations = {
      'AAPL-MSFT': 0.7,
      'GOOGL-AMZN': 0.6,
      'AAPL-GOOGL': 0.5,
    };

    const pair = `${data1.symbol}-${data2.symbol}`;
    const reversePair = `${data2.symbol}-${data1.symbol}`;

    return sectorCorrelations[pair] || sectorCorrelations[reversePair] || 0.3;
  }

  _getExchangeDistance(exchange1, exchange2) {
    const distances = {
      'NYSE-NASDAQ': 1,        // Same city (NYC)
      'NYSE-LSE': 5585,        // NYC to London
      'NYSE-TSE': 10900,       // NYC to Tokyo
      'NASDAQ-LSE': 5585,      // NYC to London
      'NASDAQ-TSE': 10900,     // NYC to Tokyo
      'LSE-TSE': 9600,         // London to Tokyo
    };

    const pair = `${exchange1}-${exchange2}`;
    const reversePair = `${exchange2}-${exchange1}`;

    return distances[pair] || distances[reversePair] || 12000; // Default global distance
  }

  _findAlternativeExchange(currentExchange) {
    const alternatives = this.config.exchanges.filter(ex => ex !== currentExchange);
    return alternatives[Math.floor(Math.random() * alternatives.length)];
  }

  _calculateAvgDivergence(opportunities) {
    if (opportunities.length === 0) return 0;
    return opportunities.reduce((sum, opp) => sum + opp.divergence, 0) / opportunities.length;
  }
}

// Example usage and testing
async function runArbitrageExample() {
  console.log('🎯 High-Frequency Arbitrage Example');
  console.log('===================================\n');

  // Create arbitrage system with configuration
  const arbitrage = new HighFrequencyArbitrage({
    symbols: ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'TSLA'],
    exchanges: ['NYSE', 'NASDAQ', 'LSE'],
    temporalAdvantageThreshold: 3, // 3ms minimum advantage
    maxPositionSize: 50000,
    riskTolerance: 0.005 // 0.5% risk tolerance
  });

  try {
    // Run arbitrage detection
    await arbitrage.start();

  } catch (error) {
    console.error('Error running arbitrage example:', error);
  } finally {
    arbitrage.stop();
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runArbitrageExample().catch(console.error);
}

export default HighFrequencyArbitrage;