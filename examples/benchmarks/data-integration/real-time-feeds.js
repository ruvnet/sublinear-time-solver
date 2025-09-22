/**
 * Real-Time Data Integration Example
 *
 * Demonstrates integration with various real-time data sources
 * and processing them with the sublinear-time-solver for immediate insights.
 */

import { EventEmitter } from 'events';
import { PerformanceMonitor } from '../utils/performance-monitor.js';
import { DataLoader } from '../utils/data-loader.js';

export class RealTimeDataProcessor extends EventEmitter {
  constructor(config = {}) {
    super();

    this.config = {
      // Data source configurations
      marketDataFeed: {
        url: 'wss://stream.example.com/market',
        symbols: ['AAPL', 'MSFT', 'GOOGL', 'AMZN'],
        updateInterval: 1000 // 1 second
      },

      socialMediaFeed: {
        url: 'wss://stream.example.com/social',
        keywords: ['finance', 'trading', 'stocks'],
        influenceThreshold: 0.001
      },

      networkMetrics: {
        updateInterval: 5000, // 5 seconds
        metricsToTrack: ['centrality', 'clustering', 'influence']
      },

      // Processing parameters
      batchSize: 100,
      processingTimeout: 30000,
      temporalAdvantageThreshold: 5, // 5ms minimum

      // Solver configuration
      solver: {
        method: 'neumann',
        epsilon: 1e-6,
        maxIterations: 500
      },

      ...config
    };

    this.monitor = new PerformanceMonitor();
    this.dataBuffer = new Map();
    this.isProcessing = false;
    this.connections = new Map();
    this.metrics = {
      totalProcessed: 0,
      avgProcessingTime: 0,
      temporalAdvantages: []
    };
  }

  /**
   * Start real-time data processing
   */
  async start() {
    console.log('🚀 Starting Real-Time Data Processing');
    console.log('=====================================\n');

    try {
      // Initialize data connections
      await this._initializeConnections();

      // Start processing loop
      this._startProcessingLoop();

      console.log('✅ Real-time processing started successfully');
      this.emit('started');

    } catch (error) {
      console.error('❌ Failed to start real-time processing:', error);
      throw error;
    }
  }

  /**
   * Stop real-time data processing
   */
  async stop() {
    console.log('🛑 Stopping real-time data processing...');

    this.isProcessing = false;

    // Close all connections
    for (const [name, connection] of this.connections) {
      try {
        await this._closeConnection(connection);
        console.log(`  Closed ${name} connection`);
      } catch (error) {
        console.error(`  Failed to close ${name} connection:`, error.message);
      }
    }

    this.connections.clear();
    this.emit('stopped');

    console.log('✅ Real-time processing stopped');
  }

  /**
   * Process market data for arbitrage opportunities
   * @param {Object} marketUpdate - Real-time market data
   */
  async processMarketData(marketUpdate) {
    const operationId = `market_${Date.now()}`;
    this.monitor.startTiming(operationId, { symbols: marketUpdate.symbols?.length });

    try {
      // Build price correlation matrix from recent data
      const priceMatrix = await this._buildPriceCorrelationMatrix(marketUpdate);

      // Detect arbitrage opportunities using sublinear solver
      const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

      const arbitrageSystem = {
        matrix: priceMatrix.matrix,
        vector: priceMatrix.expectedPrices,
        method: this.config.solver.method,
        epsilon: this.config.solver.epsilon,
        maxIterations: this.config.solver.maxIterations
      };

      const solution = await mcp__sublinear_solver__solve(arbitrageSystem);

      // Calculate temporal advantage
      const temporalAdvantage = this.monitor.calculateTemporalAdvantage(
        12000, // Global trading distance
        this.monitor.measurements.get(operationId)?.duration || 0
      );

      // Identify profitable opportunities
      const opportunities = this._identifyTradingOpportunities(
        solution,
        marketUpdate,
        temporalAdvantage
      );

      const metrics = this.monitor.endTiming(operationId, {
        opportunities: opportunities.length,
        temporalAdvantage: temporalAdvantage.temporalAdvantageMs
      });

      // Emit opportunities for trading systems
      if (opportunities.length > 0) {
        this.emit('tradingOpportunities', {
          opportunities,
          temporalAdvantage,
          processingTime: metrics.duration,
          timestamp: Date.now()
        });
      }

      return {
        opportunities,
        temporalAdvantage,
        processingTime: metrics.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      this.emit('error', { type: 'market_processing', error });
      throw error;
    }
  }

  /**
   * Process social media data for influence analysis
   * @param {Object} socialUpdate - Real-time social media data
   */
  async processSocialData(socialUpdate) {
    const operationId = `social_${Date.now()}`;
    this.monitor.startTiming(operationId);

    try {
      // Build influence network from social interactions
      const influenceNetwork = await this._buildInfluenceNetwork(socialUpdate);

      // Calculate PageRank scores for influence measurement
      const { mcp__sublinear_solver__pageRank } = await import('../../src/mcp/tools/solver.js');

      const influenceScores = await mcp__sublinear_solver__pageRank({
        adjacency: influenceNetwork.adjacency,
        damping: 0.85,
        epsilon: 1e-6
      });

      // Detect trending topics and influential users
      const trends = this._detectTrends(influenceScores, socialUpdate);

      // Calculate sentiment impact on market
      const sentimentImpact = await this._calculateSentimentImpact(trends, socialUpdate);

      const metrics = this.monitor.endTiming(operationId, {
        influentialUsers: trends.influentialUsers.length,
        trendingTopics: trends.topics.length
      });

      this.emit('socialInsights', {
        trends,
        sentimentImpact,
        influenceScores: influenceScores.ranks,
        processingTime: metrics.duration,
        timestamp: Date.now()
      });

      return {
        trends,
        sentimentImpact,
        processingTime: metrics.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      this.emit('error', { type: 'social_processing', error });
      throw error;
    }
  }

  /**
   * Process network topology changes
   * @param {Object} networkUpdate - Network topology update
   */
  async processNetworkUpdate(networkUpdate) {
    const operationId = `network_${Date.now()}`;
    this.monitor.startTiming(operationId);

    try {
      // Update network adjacency matrix
      const updatedNetwork = await this._updateNetworkTopology(networkUpdate);

      // Recalculate centrality measures
      const { mcp__sublinear_solver__pageRank } = await import('../../src/mcp/tools/solver.js');

      const centralityScores = await mcp__sublinear_solver__pageRank({
        adjacency: updatedNetwork.adjacency,
        damping: 0.85,
        epsilon: 1e-6
      });

      // Detect topology changes and critical nodes
      const topologyChanges = this._analyzeTopologyChanges(
        centralityScores,
        networkUpdate
      );

      // Calculate network resilience metrics
      const resilienceMetrics = await this._calculateNetworkResilience(
        updatedNetwork,
        centralityScores
      );

      const metrics = this.monitor.endTiming(operationId, {
        nodesChanged: topologyChanges.nodesChanged,
        criticalNodes: topologyChanges.criticalNodes.length
      });

      this.emit('networkAnalysis', {
        topologyChanges,
        resilienceMetrics,
        centralityScores: centralityScores.ranks,
        processingTime: metrics.duration,
        timestamp: Date.now()
      });

      return {
        topologyChanges,
        resilienceMetrics,
        processingTime: metrics.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      this.emit('error', { type: 'network_processing', error });
      throw error;
    }
  }

  /**
   * Simulate real-time data feeds for demonstration
   */
  async simulateRealTimeFeeds() {
    console.log('📡 Simulating real-time data feeds...\n');

    const feedTypes = ['market', 'social', 'network'];
    let iteration = 0;

    const simulationInterval = setInterval(async () => {
      const feedType = feedTypes[iteration % feedTypes.length];

      try {
        switch (feedType) {
          case 'market':
            const marketData = this._generateSampleMarketData();
            await this.processMarketData(marketData);
            break;

          case 'social':
            const socialData = this._generateSampleSocialData();
            await this.processSocialData(socialData);
            break;

          case 'network':
            const networkData = this._generateSampleNetworkData();
            await this.processNetworkUpdate(networkData);
            break;
        }

        iteration++;

        // Stop simulation after 20 iterations
        if (iteration >= 20) {
          clearInterval(simulationInterval);
          console.log('\n✅ Simulation completed');
          this._printSimulationSummary();
        }

      } catch (error) {
        console.error(`Error in ${feedType} simulation:`, error.message);
      }
    }, 2000); // 2-second intervals
  }

  // Helper methods
  async _initializeConnections() {
    // Market data connection (simulated)
    const marketConnection = {
      type: 'websocket',
      url: this.config.marketDataFeed.url,
      status: 'connected',
      lastUpdate: Date.now()
    };

    // Social media connection (simulated)
    const socialConnection = {
      type: 'websocket',
      url: this.config.socialMediaFeed.url,
      status: 'connected',
      lastUpdate: Date.now()
    };

    // Network metrics connection (simulated)
    const networkConnection = {
      type: 'polling',
      interval: this.config.networkMetrics.updateInterval,
      status: 'active',
      lastUpdate: Date.now()
    };

    this.connections.set('market', marketConnection);
    this.connections.set('social', socialConnection);
    this.connections.set('network', networkConnection);

    console.log('🔗 Initialized data connections:');
    for (const [name, conn] of this.connections) {
      console.log(`  ${name}: ${conn.status}`);
    }
  }

  _startProcessingLoop() {
    this.isProcessing = true;

    // Update metrics periodically
    const metricsInterval = setInterval(() => {
      if (!this.isProcessing) {
        clearInterval(metricsInterval);
        return;
      }

      this._updateMetrics();
    }, 5000); // Update every 5 seconds
  }

  async _buildPriceCorrelationMatrix(marketUpdate) {
    const symbols = marketUpdate.symbols || this.config.marketDataFeed.symbols;
    const n = symbols.length;

    // Simulate price correlation calculation
    const matrix = Array(n).fill().map(() => Array(n).fill(0));
    const expectedPrices = Array(n).fill(0);

    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        if (i === j) {
          matrix[i][j] = 1.0 + Math.random() * 0.5; // Diagonal dominance
        } else {
          const correlation = 0.3 + Math.random() * 0.4; // 0.3-0.7 correlation
          matrix[i][j] = correlation;
        }
      }

      // Ensure diagonal dominance
      const rowSum = matrix[i].reduce((sum, val, idx) => idx !== i ? sum + Math.abs(val) : sum, 0);
      matrix[i][i] = rowSum * 1.2 + 1;

      expectedPrices[i] = 100 + Math.random() * 200; // $100-300 price range
    }

    return {
      matrix: {
        rows: n,
        cols: n,
        format: 'dense',
        data: matrix
      },
      expectedPrices,
      symbols
    };
  }

  _identifyTradingOpportunities(solution, marketUpdate, temporalAdvantage) {
    if (!temporalAdvantage.feasible) return [];

    const opportunities = [];
    const priceDivergences = solution.solution;

    for (let i = 0; i < priceDivergences.length; i++) {
      const divergence = Math.abs(priceDivergences[i]);

      if (divergence > 0.001 && temporalAdvantage.temporalAdvantageMs > this.config.temporalAdvantageThreshold) {
        opportunities.push({
          symbol: this.config.marketDataFeed.symbols[i],
          divergence,
          expectedProfit: divergence * 100, // Simplified profit calculation
          confidence: Math.min(1, divergence * 100),
          temporalWindow: temporalAdvantage.temporalAdvantageMs,
          timestamp: Date.now()
        });
      }
    }

    return opportunities.sort((a, b) => b.expectedProfit - a.expectedProfit);
  }

  async _buildInfluenceNetwork(socialUpdate) {
    const userCount = socialUpdate.users?.length || 100;

    // Generate adjacency matrix for influence network
    const values = [];
    const rowIndices = [];
    const colIndices = [];

    for (let i = 0; i < userCount; i++) {
      const connections = Math.floor(Math.random() * 10) + 1; // 1-10 connections per user

      for (let c = 0; c < connections; c++) {
        const target = Math.floor(Math.random() * userCount);
        if (target !== i) {
          values.push(Math.random()); // Random influence weight
          rowIndices.push(i);
          colIndices.push(target);
        }
      }
    }

    return {
      adjacency: {
        rows: userCount,
        cols: userCount,
        format: 'coo',
        data: { values, rowIndices, colIndices }
      },
      userCount
    };
  }

  _detectTrends(influenceScores, socialUpdate) {
    const topInfluencers = influenceScores.ranks
      .map((score, id) => ({ id, score }))
      .sort((a, b) => b.score - a.score)
      .slice(0, 10);

    return {
      influentialUsers: topInfluencers,
      topics: ['AI', 'blockchain', 'sustainability'], // Simulated trending topics
      sentiment: {
        positive: 0.6,
        neutral: 0.3,
        negative: 0.1
      }
    };
  }

  async _calculateSentimentImpact(trends, socialUpdate) {
    // Simplified sentiment impact calculation
    const marketImpact = trends.sentiment.positive - trends.sentiment.negative;
    const influenceMultiplier = trends.influentialUsers.reduce((sum, user) => sum + user.score, 0);

    return {
      marketImpact: marketImpact * influenceMultiplier,
      confidence: 0.7,
      timeHorizon: '1-4 hours',
      affectedSymbols: ['AAPL', 'TSLA'] // Simulated
    };
  }

  async _updateNetworkTopology(networkUpdate) {
    // Simulate network topology update
    const nodeCount = networkUpdate.nodeCount || 50;

    const values = [];
    const rowIndices = [];
    const colIndices = [];

    for (let i = 0; i < nodeCount; i++) {
      const degree = Math.floor(Math.random() * 5) + 1;

      for (let d = 0; d < degree; d++) {
        const target = Math.floor(Math.random() * nodeCount);
        if (target !== i) {
          values.push(1);
          rowIndices.push(i);
          colIndices.push(target);
        }
      }
    }

    return {
      adjacency: {
        rows: nodeCount,
        cols: nodeCount,
        format: 'coo',
        data: { values, rowIndices, colIndices }
      },
      nodeCount
    };
  }

  _analyzeTopologyChanges(centralityScores, networkUpdate) {
    const topNodes = centralityScores.ranks
      .map((score, id) => ({ id, score }))
      .sort((a, b) => b.score - a.score)
      .slice(0, 5);

    return {
      nodesChanged: networkUpdate.changedNodes || Math.floor(Math.random() * 10),
      criticalNodes: topNodes,
      stabilityScore: 0.85 + Math.random() * 0.1
    };
  }

  async _calculateNetworkResilience(network, centralityScores) {
    // Simplified resilience calculation
    const avgCentrality = centralityScores.ranks.reduce((sum, score) => sum + score, 0) / centralityScores.ranks.length;
    const centralityVariance = centralityScores.ranks.reduce((sum, score) => sum + Math.pow(score - avgCentrality, 2), 0) / centralityScores.ranks.length;

    return {
      robustnessScore: 1 - (centralityVariance / avgCentrality),
      redundancyLevel: 0.7,
      failureResistance: 0.8,
      recoveryTime: '< 5 minutes'
    };
  }

  _generateSampleMarketData() {
    return {
      symbols: this.config.marketDataFeed.symbols,
      prices: this.config.marketDataFeed.symbols.map(() => 100 + Math.random() * 200),
      volumes: this.config.marketDataFeed.symbols.map(() => Math.floor(Math.random() * 1000000)),
      timestamp: Date.now()
    };
  }

  _generateSampleSocialData() {
    return {
      users: Array(50).fill().map((_, i) => ({
        id: i,
        influence: Math.random(),
        mentions: Math.floor(Math.random() * 100)
      })),
      posts: Array(20).fill().map(() => ({
        sentiment: Math.random() > 0.5 ? 'positive' : 'negative',
        reach: Math.floor(Math.random() * 10000)
      })),
      timestamp: Date.now()
    };
  }

  _generateSampleNetworkData() {
    return {
      nodeCount: 50,
      changedNodes: Math.floor(Math.random() * 5),
      changeType: 'topology_update',
      timestamp: Date.now()
    };
  }

  _updateMetrics() {
    const report = this.monitor.generateReport({ includeAllMeasurements: false });

    this.metrics.totalProcessed = report.summary.completedOperations;
    this.metrics.avgProcessingTime = report.summary.avgDuration;

    // Calculate temporal advantage statistics
    const temporalAdvantages = Array.from(this.monitor.measurements.values())
      .filter(m => m.metadata && m.metadata.temporalAdvantage)
      .map(m => m.metadata.temporalAdvantage);

    this.metrics.temporalAdvantages = temporalAdvantages;

    console.log(`📊 Metrics Update: ${this.metrics.totalProcessed} processed, avg: ${this.metrics.avgProcessingTime.toFixed(2)}ms`);
  }

  _printSimulationSummary() {
    console.log('\n📈 SIMULATION SUMMARY');
    console.log('====================');
    console.log(`Total Operations: ${this.metrics.totalProcessed}`);
    console.log(`Average Processing Time: ${this.metrics.avgProcessingTime.toFixed(2)}ms`);

    if (this.metrics.temporalAdvantages.length > 0) {
      const avgAdvantage = this.metrics.temporalAdvantages.reduce((sum, ta) => sum + ta, 0) / this.metrics.temporalAdvantages.length;
      console.log(`Average Temporal Advantage: ${avgAdvantage.toFixed(2)}ms`);
    }

    const report = this.monitor.generateReport({ includeSystemInfo: true });
    console.log(`Memory Usage: ${(report.systemInfo.totalMemory - report.systemInfo.freeMemory) / 1e9}GB`);
  }

  async _closeConnection(connection) {
    // Simulate connection cleanup
    connection.status = 'closed';
    return new Promise(resolve => setTimeout(resolve, 100));
  }
}

// Example usage and testing
async function runRealTimeDataExample() {
  console.log('📡 Real-Time Data Integration Example');
  console.log('====================================\n');

  const processor = new RealTimeDataProcessor({
    marketDataFeed: {
      symbols: ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'TSLA'],
      updateInterval: 1000
    },
    temporalAdvantageThreshold: 3,
    solver: {
      method: 'neumann',
      epsilon: 1e-6
    }
  });

  // Set up event listeners
  processor.on('tradingOpportunities', (data) => {
    console.log(`💰 Trading Opportunities: ${data.opportunities.length} found (${data.processingTime.toFixed(2)}ms)`);
    data.opportunities.slice(0, 3).forEach((opp, i) => {
      console.log(`  ${i+1}. ${opp.symbol}: $${opp.expectedProfit.toFixed(2)} profit (${opp.temporalWindow.toFixed(2)}ms window)`);
    });
  });

  processor.on('socialInsights', (data) => {
    console.log(`📱 Social Insights: ${data.trends.influentialUsers.length} influencers, sentiment: ${(data.sentimentImpact.marketImpact * 100).toFixed(1)}%`);
  });

  processor.on('networkAnalysis', (data) => {
    console.log(`🌐 Network Analysis: ${data.topologyChanges.criticalNodes.length} critical nodes, stability: ${(data.resilienceMetrics.robustnessScore * 100).toFixed(1)}%`);
  });

  processor.on('error', (error) => {
    console.error(`❌ Processing Error: ${error.type} - ${error.error.message}`);
  });

  try {
    // Start real-time processing
    await processor.start();

    // Run simulation
    await processor.simulateRealTimeFeeds();

    // Stop processing
    await processor.stop();

  } catch (error) {
    console.error('Error in real-time data example:', error);
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runRealTimeDataExample().catch(console.error);
}

export default RealTimeDataProcessor;