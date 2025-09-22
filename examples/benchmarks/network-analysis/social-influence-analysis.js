/**
 * Social Influence Analysis Example
 *
 * Demonstrates advanced social network analysis using PageRank and matrix operations
 * to identify influencers, communities, and information propagation patterns.
 */

import { MatrixGenerator } from '../utils/matrix-generator.js';
import { PerformanceMonitor } from '../utils/performance-monitor.js';

export class SocialInfluenceAnalyzer {
  constructor(config = {}) {
    this.config = {
      dampingFactor: 0.85,
      convergenceThreshold: 1e-6,
      maxIterations: 100,
      influenceThreshold: 0.001, // Minimum influence score
      communityResolution: 1.0,
      temporalDecay: 0.95, // For temporal influence analysis
      ...config
    };

    this.monitor = new PerformanceMonitor();
    this.networkCache = new Map();
    this.influenceHistory = [];
  }

  /**
   * Analyze influence in a social network
   * @param {Object} socialGraph - Social network adjacency matrix or edge list
   * @param {Object} options - Analysis options
   * @returns {Promise<Object>} Influence analysis results
   */
  async analyzeInfluence(socialGraph, options = {}) {
    const operationId = 'social_influence_analysis';
    this.monitor.startTiming(operationId, {
      networkSize: socialGraph.adjacency?.rows || socialGraph.nodes?.length,
      analysisType: 'comprehensive'
    });

    try {
      // Convert network to proper format if needed
      const adjacencyMatrix = this._ensureAdjacencyMatrix(socialGraph);

      // Calculate basic PageRank scores
      const pageRankScores = await this._calculatePageRank(adjacencyMatrix, options);

      // Calculate personalized PageRank for different user types
      const personalizedInfluence = await this._calculatePersonalizedInfluence(
        adjacencyMatrix,
        options.userTypes
      );

      // Identify influencer tiers
      const influencerTiers = this._categorizeInfluencers(pageRankScores);

      // Analyze influence communities
      const communities = await this._detectInfluenceCommunities(
        adjacencyMatrix,
        pageRankScores
      );

      // Calculate influence propagation potential
      const propagationAnalysis = await this._analyzePropagation(
        adjacencyMatrix,
        pageRankScores
      );

      // Generate influence network metrics
      const networkMetrics = this._calculateNetworkMetrics(
        adjacencyMatrix,
        pageRankScores
      );

      const metrics = this.monitor.endTiming(operationId, {
        totalInfluencers: influencerTiers.reduce((sum, tier) => sum + tier.users.length, 0),
        communities: communities.length,
        networkDensity: networkMetrics.density
      });

      const results = {
        pageRankScores,
        personalizedInfluence,
        influencerTiers,
        communities,
        propagationAnalysis,
        networkMetrics,
        computationTime: metrics.duration,
        timestamp: Date.now()
      };

      // Cache results for future analysis
      this._cacheResults(socialGraph, results);

      return results;

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Calculate influence spread for viral content analysis
   * @param {Object} network - Social network
   * @param {Array} seedUsers - Initial users who share content
   * @param {Object} contentProperties - Content characteristics
   * @returns {Promise<Object>} Viral spread simulation results
   */
  async simulateViralSpread(network, seedUsers, contentProperties = {}) {
    const operationId = 'viral_spread_simulation';
    this.monitor.startTiming(operationId);

    try {
      const {
        viralityFactor = 0.1,
        timeSteps = 10,
        decayRate = 0.9,
        contentType = 'general'
      } = contentProperties;

      const adjacencyMatrix = this._ensureAdjacencyMatrix(network);
      const networkSize = adjacencyMatrix.rows;

      // Initialize spread state
      let currentSpread = Array(networkSize).fill(0);
      seedUsers.forEach(userId => {
        currentSpread[userId] = 1.0; // Full influence for seed users
      });

      const spreadHistory = [currentSpread.slice()];
      let totalReach = seedUsers.length;

      // Simulate spread over time steps
      for (let step = 0; step < timeSteps; step++) {
        const nextSpread = await this._calculateSpreadStep(
          adjacencyMatrix,
          currentSpread,
          viralityFactor,
          decayRate
        );

        // Count newly influenced users
        const newlyInfluenced = nextSpread.filter((influence, i) =>
          influence > 0.1 && currentSpread[i] <= 0.1
        ).length;

        totalReach += newlyInfluenced;
        currentSpread = nextSpread;
        spreadHistory.push(currentSpread.slice());

        // Early stopping if spread plateaus
        if (newlyInfluenced === 0) break;
      }

      // Analyze final spread patterns
      const finalAnalysis = this._analyzeFinalSpread(spreadHistory, adjacencyMatrix);

      const metrics = this.monitor.endTiming(operationId, {
        totalReach,
        spreadSteps: spreadHistory.length,
        finalPenetration: totalReach / networkSize
      });

      return {
        spreadHistory,
        totalReach,
        penetrationRate: totalReach / networkSize,
        spreadVelocity: finalAnalysis.velocity,
        influentialPaths: finalAnalysis.paths,
        bottlenecks: finalAnalysis.bottlenecks,
        computationTime: metrics.duration,
        contentProperties
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Temporal influence analysis - track influence changes over time
   * @param {Array} networkSnapshots - Time series of network states
   * @param {Object} options - Temporal analysis options
   * @returns {Promise<Object>} Temporal influence evolution
   */
  async analyzeTemporalInfluence(networkSnapshots, options = {}) {
    const operationId = 'temporal_influence_analysis';
    this.monitor.startTiming(operationId);

    try {
      const {
        timeGranularity = 'daily',
        trendDetection = true,
        stabilityAnalysis = true
      } = options;

      const temporalResults = [];
      const influenceEvolution = new Map();

      // Analyze each time snapshot
      for (let t = 0; t < networkSnapshots.length; t++) {
        const snapshot = networkSnapshots[t];
        const snapshotAnalysis = await this.analyzeInfluence(snapshot, {
          skipCache: true
        });

        temporalResults.push({
          timestamp: snapshot.timestamp || t,
          analysis: snapshotAnalysis
        });

        // Track individual user influence evolution
        snapshotAnalysis.pageRankScores.ranks.forEach((score, userId) => {
          if (!influenceEvolution.has(userId)) {
            influenceEvolution.set(userId, []);
          }
          influenceEvolution.get(userId).push({ time: t, influence: score });
        });
      }

      // Detect influence trends
      const trends = trendDetection ? this._detectInfluenceTrends(influenceEvolution) : null;

      // Analyze influence stability
      const stability = stabilityAnalysis ? this._analyzeInfluenceStability(influenceEvolution) : null;

      // Identify rising and falling influencers
      const dynamics = this._analyzeInfluenceDynamics(influenceEvolution);

      const metrics = this.monitor.endTiming(operationId, {
        timePoints: networkSnapshots.length,
        risingInfluencers: dynamics.rising.length,
        fallingInfluencers: dynamics.falling.length
      });

      return {
        temporalResults,
        influenceEvolution: Object.fromEntries(influenceEvolution),
        trends,
        stability,
        dynamics,
        computationTime: metrics.duration
      };

    } catch (error) {
      this.monitor.endTiming(operationId, { error: error.message });
      throw error;
    }
  }

  /**
   * Generate a realistic social network for testing
   * @param {number} size - Number of users
   * @param {Object} properties - Network properties
   * @returns {Object} Generated social network
   */
  generateSocialNetwork(size = 1000, properties = {}) {
    const {
      averageConnections = 50,
      influencerRatio = 0.05,
      communityCount = 5,
      homophily = 0.7, // Tendency to connect to similar users
      powerLawExponent = 2.5
    } = properties;

    // Generate network using preferential attachment (Barabási-Albert model)
    const network = MatrixGenerator.generateSocialNetwork(size, averageConnections);

    // Add user metadata
    const users = Array(size).fill().map((_, i) => ({
      id: i,
      username: `user_${i}`,
      followers: 0,
      following: 0,
      postCount: Math.floor(Math.random() * 1000),
      verified: Math.random() < (i < size * influencerRatio ? 0.8 : 0.01),
      community: Math.floor(Math.random() * communityCount),
      joinDate: Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000 // Random join date within last year
    }));

    // Calculate follower/following counts from adjacency matrix
    const { values, rowIndices, colIndices } = network.data;
    for (let i = 0; i < values.length; i++) {
      const from = rowIndices[i];
      const to = colIndices[i];
      users[from].following++;
      users[to].followers++;
    }

    return {
      adjacency: network,
      users,
      metadata: {
        size,
        averageConnections,
        influencerRatio,
        communityCount,
        generatedAt: Date.now()
      }
    };
  }

  // Helper methods
  async _calculatePageRank(adjacencyMatrix, options = {}) {
    const { mcp__sublinear_solver__pageRank } = await import('../../src/mcp/tools/solver.js');

    const result = await mcp__sublinear_solver__pageRank({
      adjacency: adjacencyMatrix,
      damping: options.dampingFactor || this.config.dampingFactor,
      epsilon: options.convergenceThreshold || this.config.convergenceThreshold,
      maxIterations: options.maxIterations || this.config.maxIterations,
      personalized: options.personalizationVector
    });

    return result;
  }

  async _calculatePersonalizedInfluence(adjacencyMatrix, userTypes) {
    if (!userTypes) return null;

    const personalizedResults = {};

    for (const [typeName, userIndices] of Object.entries(userTypes)) {
      // Create personalization vector for this user type
      const personalizationVector = Array(adjacencyMatrix.rows).fill(0);
      userIndices.forEach(index => {
        personalizationVector[index] = 1 / userIndices.length;
      });

      const personalizedPageRank = await this._calculatePageRank(adjacencyMatrix, {
        personalizationVector
      });

      personalizedResults[typeName] = personalizedPageRank;
    }

    return personalizedResults;
  }

  _categorizeInfluencers(pageRankScores) {
    const scores = pageRankScores.ranks;
    const sortedIndices = scores
      .map((score, index) => ({ index, score }))
      .sort((a, b) => b.score - a.score);

    const totalUsers = scores.length;

    return [
      {
        tier: 'mega_influencers',
        threshold: 0.001,
        users: sortedIndices.filter(u => u.score > 0.001).map(u => u.index),
        description: 'Top 0.1% - Celebrity-level influence'
      },
      {
        tier: 'macro_influencers',
        threshold: 0.0001,
        users: sortedIndices.filter(u => u.score > 0.0001 && u.score <= 0.001).map(u => u.index),
        description: 'Top 1% - High-reach influencers'
      },
      {
        tier: 'micro_influencers',
        threshold: 0.00001,
        users: sortedIndices.filter(u => u.score > 0.00001 && u.score <= 0.0001).map(u => u.index),
        description: 'Top 10% - Niche influencers'
      },
      {
        tier: 'nano_influencers',
        threshold: 0.000001,
        users: sortedIndices.filter(u => u.score > 0.000001 && u.score <= 0.00001).map(u => u.index),
        description: 'Emerging influencers'
      }
    ];
  }

  async _detectInfluenceCommunities(adjacencyMatrix, pageRankScores) {
    // Simplified community detection using influence-weighted modularity
    const n = adjacencyMatrix.rows;
    const communities = [];

    // Use influence scores to weight community detection
    const influenceWeights = pageRankScores.ranks;

    // Simple greedy community detection (in practice, use Louvain algorithm)
    const visited = Array(n).fill(false);
    let communityId = 0;

    for (let i = 0; i < n; i++) {
      if (!visited[i] && influenceWeights[i] > this.config.influenceThreshold) {
        const community = this._expandCommunity(adjacencyMatrix, i, visited, influenceWeights);
        if (community.length > 1) {
          communities.push({
            id: communityId++,
            members: community,
            leadInfluencer: community.reduce((leader, member) =>
              influenceWeights[member] > influenceWeights[leader] ? member : leader
            ),
            avgInfluence: community.reduce((sum, member) => sum + influenceWeights[member], 0) / community.length,
            size: community.length
          });
        }
      }
    }

    return communities.sort((a, b) => b.avgInfluence - a.avgInfluence);
  }

  _expandCommunity(adjacencyMatrix, startNode, visited, influenceWeights) {
    const community = [startNode];
    visited[startNode] = true;
    const queue = [startNode];

    while (queue.length > 0) {
      const current = queue.shift();

      // Find neighbors with sufficient influence connection
      if (adjacencyMatrix.format === 'coo') {
        const { values, rowIndices, colIndices } = adjacencyMatrix.data;
        for (let i = 0; i < values.length; i++) {
          if (rowIndices[i] === current && !visited[colIndices[i]]) {
            const neighbor = colIndices[i];
            const connectionStrength = values[i] * influenceWeights[neighbor];

            if (connectionStrength > this.config.influenceThreshold * 0.1) {
              visited[neighbor] = true;
              community.push(neighbor);
              queue.push(neighbor);
            }
          }
        }
      } else {
        // Dense matrix format
        for (let j = 0; j < adjacencyMatrix.cols; j++) {
          if (!visited[j] && adjacencyMatrix.data[current][j] > 0) {
            const connectionStrength = adjacencyMatrix.data[current][j] * influenceWeights[j];
            if (connectionStrength > this.config.influenceThreshold * 0.1) {
              visited[j] = true;
              community.push(j);
              queue.push(j);
            }
          }
        }
      }
    }

    return community;
  }

  async _analyzePropagation(adjacencyMatrix, pageRankScores) {
    // Analyze potential for information propagation
    const { mcp__sublinear_solver__solve } = await import('../../src/mcp/tools/solver.js');

    // Build propagation potential matrix
    const propagationMatrix = this._buildPropagationMatrix(adjacencyMatrix, pageRankScores);

    // Solve for propagation potential
    const initialConditions = pageRankScores.ranks; // Use PageRank as initial spreading probability
    const propagationResult = await mcp__sublinear_solver__solve({
      matrix: propagationMatrix,
      vector: initialConditions,
      method: 'neumann',
      epsilon: 1e-6
    });

    return {
      propagationPotential: propagationResult.solution,
      convergenceTime: propagationResult.iterations,
      spreadingPaths: this._identifySpreadingPaths(adjacencyMatrix, propagationResult.solution)
    };
  }

  _buildPropagationMatrix(adjacencyMatrix, pageRankScores) {
    const n = adjacencyMatrix.rows;
    const matrix = Array(n).fill().map(() => Array(n).fill(0));

    // Build influence-weighted propagation matrix
    if (adjacencyMatrix.format === 'coo') {
      const { values, rowIndices, colIndices } = adjacencyMatrix.data;

      // Initialize diagonal
      for (let i = 0; i < n; i++) {
        matrix[i][i] = 1.0;
      }

      // Add weighted connections
      for (let i = 0; i < values.length; i++) {
        const from = rowIndices[i];
        const to = colIndices[i];
        const weight = values[i] * pageRankScores.ranks[from] * this.config.dampingFactor;
        matrix[to][from] = weight;
        matrix[to][to] += weight; // Ensure diagonal dominance
      }
    } else {
      // Dense matrix format
      for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
          if (i === j) {
            matrix[i][j] = 1.0;
          } else if (adjacencyMatrix.data[j][i] > 0) {
            const weight = adjacencyMatrix.data[j][i] * pageRankScores.ranks[j] * this.config.dampingFactor;
            matrix[i][j] = weight;
            matrix[i][i] += weight;
          }
        }
      }
    }

    return {
      rows: n,
      cols: n,
      format: 'dense',
      data: matrix
    };
  }

  _identifySpreadingPaths(adjacencyMatrix, propagationPotential) {
    const paths = [];
    const n = adjacencyMatrix.rows;

    // Find high-potential propagation paths
    for (let i = 0; i < n; i++) {
      if (propagationPotential[i] > 0.01) { // Threshold for significant propagation
        const connections = this._getConnections(adjacencyMatrix, i);
        paths.push({
          sourceNode: i,
          potential: propagationPotential[i],
          connections: connections.filter(conn => propagationPotential[conn.target] > 0.005),
          reach: connections.length
        });
      }
    }

    return paths.sort((a, b) => b.potential - a.potential).slice(0, 20); // Top 20 paths
  }

  _getConnections(adjacencyMatrix, nodeIndex) {
    const connections = [];

    if (adjacencyMatrix.format === 'coo') {
      const { values, rowIndices, colIndices } = adjacencyMatrix.data;
      for (let i = 0; i < values.length; i++) {
        if (rowIndices[i] === nodeIndex) {
          connections.push({
            target: colIndices[i],
            weight: values[i]
          });
        }
      }
    } else {
      for (let j = 0; j < adjacencyMatrix.cols; j++) {
        if (adjacencyMatrix.data[nodeIndex][j] > 0) {
          connections.push({
            target: j,
            weight: adjacencyMatrix.data[nodeIndex][j]
          });
        }
      }
    }

    return connections;
  }

  async _calculateSpreadStep(adjacencyMatrix, currentSpread, viralityFactor, decayRate) {
    const n = adjacencyMatrix.rows;
    const nextSpread = currentSpread.map(influence => influence * decayRate);

    // Calculate new infections based on network connections
    if (adjacencyMatrix.format === 'coo') {
      const { values, rowIndices, colIndices } = adjacencyMatrix.data;
      for (let i = 0; i < values.length; i++) {
        const from = rowIndices[i];
        const to = colIndices[i];
        const spreadProbability = currentSpread[from] * viralityFactor * values[i];
        nextSpread[to] = Math.max(nextSpread[to], spreadProbability);
      }
    } else {
      for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
          if (adjacencyMatrix.data[i][j] > 0) {
            const spreadProbability = currentSpread[i] * viralityFactor * adjacencyMatrix.data[i][j];
            nextSpread[j] = Math.max(nextSpread[j], spreadProbability);
          }
        }
      }
    }

    return nextSpread;
  }

  _analyzeFinalSpread(spreadHistory, adjacencyMatrix) {
    const finalSpread = spreadHistory[spreadHistory.length - 1];
    const initialSpread = spreadHistory[0];

    // Calculate spread velocity
    const velocity = spreadHistory.map((spread, t) => {
      if (t === 0) return 0;
      const newInfections = spread.filter((influence, i) =>
        influence > 0.1 && spreadHistory[t - 1][i] <= 0.1
      ).length;
      return newInfections;
    });

    // Identify influential paths
    const paths = this._identifySpreadingPaths(adjacencyMatrix, finalSpread);

    // Find bottlenecks (nodes that limit spread)
    const bottlenecks = this._findSpreadBottlenecks(spreadHistory, adjacencyMatrix);

    return { velocity, paths, bottlenecks };
  }

  _findSpreadBottlenecks(spreadHistory, adjacencyMatrix) {
    // Simplified bottleneck detection
    const bottlenecks = [];
    const finalSpread = spreadHistory[spreadHistory.length - 1];

    for (let i = 0; i < finalSpread.length; i++) {
      if (finalSpread[i] > 0.1) {
        const connections = this._getConnections(adjacencyMatrix, i);
        const influencedConnections = connections.filter(conn => finalSpread[conn.target] > 0.1);

        if (connections.length > 10 && influencedConnections.length < connections.length * 0.3) {
          bottlenecks.push({
            node: i,
            totalConnections: connections.length,
            influencedConnections: influencedConnections.length,
            bottleneckScore: 1 - (influencedConnections.length / connections.length)
          });
        }
      }
    }

    return bottlenecks.sort((a, b) => b.bottleneckScore - a.bottleneckScore);
  }

  _calculateNetworkMetrics(adjacencyMatrix, pageRankScores) {
    const n = adjacencyMatrix.rows;
    let edgeCount = 0;

    if (adjacencyMatrix.format === 'coo') {
      edgeCount = adjacencyMatrix.data.values.length;
    } else {
      for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
          if (adjacencyMatrix.data[i][j] > 0) edgeCount++;
        }
      }
    }

    const density = edgeCount / (n * (n - 1));
    const avgInfluence = pageRankScores.ranks.reduce((sum, score) => sum + score, 0) / n;
    const influenceVariance = pageRankScores.ranks.reduce((sum, score) =>
      sum + Math.pow(score - avgInfluence, 2), 0) / n;

    return {
      nodeCount: n,
      edgeCount,
      density,
      avgInfluence,
      influenceVariance,
      influenceConcentration: Math.sqrt(influenceVariance) / avgInfluence
    };
  }

  _detectInfluenceTrends(influenceEvolution) {
    const trends = {};

    for (const [userId, history] of influenceEvolution) {
      if (history.length < 3) continue;

      // Calculate trend using linear regression
      const trend = this._calculateTrend(history.map(h => h.influence));
      const volatility = this._calculateVolatility(history.map(h => h.influence));

      trends[userId] = {
        slope: trend.slope,
        r2: trend.r2,
        volatility,
        direction: trend.slope > 0.001 ? 'rising' : trend.slope < -0.001 ? 'falling' : 'stable'
      };
    }

    return trends;
  }

  _analyzeInfluenceStability(influenceEvolution) {
    const stability = {};

    for (const [userId, history] of influenceEvolution) {
      if (history.length < 2) continue;

      const influences = history.map(h => h.influence);
      const mean = influences.reduce((sum, val) => sum + val, 0) / influences.length;
      const variance = influences.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / influences.length;
      const cv = Math.sqrt(variance) / mean; // Coefficient of variation

      stability[userId] = {
        mean,
        variance,
        coefficientOfVariation: cv,
        stability: cv < 0.1 ? 'high' : cv < 0.3 ? 'medium' : 'low'
      };
    }

    return stability;
  }

  _analyzeInfluenceDynamics(influenceEvolution) {
    const rising = [];
    const falling = [];
    const stable = [];

    for (const [userId, history] of influenceEvolution) {
      if (history.length < 2) continue;

      const startInfluence = history[0].influence;
      const endInfluence = history[history.length - 1].influence;
      const changeRatio = (endInfluence - startInfluence) / startInfluence;

      if (changeRatio > 0.1) {
        rising.push({
          userId: parseInt(userId),
          startInfluence,
          endInfluence,
          changeRatio,
          category: 'rising'
        });
      } else if (changeRatio < -0.1) {
        falling.push({
          userId: parseInt(userId),
          startInfluence,
          endInfluence,
          changeRatio,
          category: 'falling'
        });
      } else {
        stable.push({
          userId: parseInt(userId),
          startInfluence,
          endInfluence,
          changeRatio,
          category: 'stable'
        });
      }
    }

    return {
      rising: rising.sort((a, b) => b.changeRatio - a.changeRatio),
      falling: falling.sort((a, b) => a.changeRatio - b.changeRatio),
      stable: stable.sort((a, b) => b.endInfluence - a.endInfluence)
    };
  }

  _calculateTrend(values) {
    const n = values.length;
    const xMean = (n - 1) / 2;
    const yMean = values.reduce((sum, val) => sum + val, 0) / n;

    let numerator = 0;
    let denominator = 0;

    for (let i = 0; i < n; i++) {
      numerator += (i - xMean) * (values[i] - yMean);
      denominator += Math.pow(i - xMean, 2);
    }

    const slope = denominator !== 0 ? numerator / denominator : 0;

    // Calculate R²
    let ssRes = 0;
    let ssTot = 0;

    for (let i = 0; i < n; i++) {
      const predicted = yMean + slope * (i - xMean);
      ssRes += Math.pow(values[i] - predicted, 2);
      ssTot += Math.pow(values[i] - yMean, 2);
    }

    const r2 = ssTot !== 0 ? 1 - (ssRes / ssTot) : 0;

    return { slope, r2 };
  }

  _calculateVolatility(values) {
    if (values.length < 2) return 0;

    const returns = [];
    for (let i = 1; i < values.length; i++) {
      returns.push((values[i] - values[i - 1]) / values[i - 1]);
    }

    const mean = returns.reduce((sum, ret) => sum + ret, 0) / returns.length;
    const variance = returns.reduce((sum, ret) => sum + Math.pow(ret - mean, 2), 0) / returns.length;

    return Math.sqrt(variance);
  }

  _ensureAdjacencyMatrix(socialGraph) {
    if (socialGraph.adjacency) {
      return socialGraph.adjacency;
    }

    if (socialGraph.edges && socialGraph.nodes) {
      // Convert edge list to adjacency matrix
      const nodeCount = socialGraph.nodes.length;
      const values = [];
      const rowIndices = [];
      const colIndices = [];

      for (const edge of socialGraph.edges) {
        const fromIndex = socialGraph.nodes.indexOf(edge.from);
        const toIndex = socialGraph.nodes.indexOf(edge.to);
        if (fromIndex !== -1 && toIndex !== -1) {
          values.push(edge.weight || 1);
          rowIndices.push(fromIndex);
          colIndices.push(toIndex);
        }
      }

      return {
        rows: nodeCount,
        cols: nodeCount,
        format: 'coo',
        data: { values, rowIndices, colIndices }
      };
    }

    throw new Error('Invalid social graph format. Expected adjacency matrix or edge list.');
  }

  _cacheResults(network, results) {
    const networkHash = this._hashNetwork(network);
    this.networkCache.set(networkHash, {
      results,
      timestamp: Date.now()
    });

    // Clean old cache entries (keep last 10)
    if (this.networkCache.size > 10) {
      const oldestKey = this.networkCache.keys().next().value;
      this.networkCache.delete(oldestKey);
    }
  }

  _hashNetwork(network) {
    // Simple hash based on network size and structure
    const size = network.adjacency?.rows || network.nodes?.length || 0;
    const edges = network.adjacency?.data?.values?.length || network.edges?.length || 0;
    return `${size}_${edges}_${Date.now().toString(36)}`;
  }
}

// Example usage and testing
async function runSocialInfluenceExample() {
  console.log('🌐 Social Influence Analysis Example');
  console.log('====================================\n');

  const analyzer = new SocialInfluenceAnalyzer({
    dampingFactor: 0.85,
    convergenceThreshold: 1e-6,
    influenceThreshold: 0.0001
  });

  try {
    // Generate a realistic social network
    console.log('📊 Generating social network...');
    const network = analyzer.generateSocialNetwork(2000, {
      averageConnections: 100,
      influencerRatio: 0.02,
      communityCount: 8
    });

    console.log(`Generated network: ${network.users.length} users, ${network.adjacency.data.values.length} connections`);

    // 1. Basic influence analysis
    console.log('\n🎯 Analyzing social influence...');
    const influenceAnalysis = await analyzer.analyzeInfluence(network);

    console.log('Influence Analysis Results:');
    console.log(`  Computation Time: ${influenceAnalysis.computationTime.toFixed(2)}ms`);
    console.log(`  Network Density: ${(influenceAnalysis.networkMetrics.density * 100).toFixed(3)}%`);
    console.log(`  Communities Found: ${influenceAnalysis.communities.length}`);

    // Display top influencers
    const topInfluencers = influenceAnalysis.pageRankScores.ranks
      .map((score, id) => ({ id, score, user: network.users[id] }))
      .sort((a, b) => b.score - a.score)
      .slice(0, 10);

    console.log('\nTop 10 Influencers:');
    topInfluencers.forEach((influencer, rank) => {
      console.log(`  ${rank + 1}. ${influencer.user.username} (Score: ${influencer.score.toFixed(6)}, Followers: ${influencer.user.followers})`);
    });

    // Display influencer tiers
    console.log('\nInfluencer Tiers:');
    influenceAnalysis.influencerTiers.forEach(tier => {
      console.log(`  ${tier.tier}: ${tier.users.length} users - ${tier.description}`);
    });

    // 2. Viral spread simulation
    console.log('\n🦠 Simulating viral spread...');
    const seedUsers = topInfluencers.slice(0, 3).map(inf => inf.id); // Top 3 influencers as seeds
    const viralSpread = await analyzer.simulateViralSpread(network, seedUsers, {
      viralityFactor: 0.15,
      timeSteps: 8,
      contentType: 'viral_meme'
    });

    console.log('Viral Spread Results:');
    console.log(`  Total Reach: ${viralSpread.totalReach} users (${(viralSpread.penetrationRate * 100).toFixed(1)}%)`);
    console.log(`  Spread Steps: ${viralSpread.spreadHistory.length}`);
    console.log(`  Computation Time: ${viralSpread.computationTime.toFixed(2)}ms`);

    // 3. Community analysis
    console.log('\n👥 Analyzing influence communities...');
    console.log('Top Communities by Average Influence:');
    influenceAnalysis.communities.slice(0, 5).forEach((community, rank) => {
      const leader = network.users[community.leadInfluencer];
      console.log(`  ${rank + 1}. Community ${community.id}: ${community.size} members`);
      console.log(`     Leader: ${leader.username} (Avg Influence: ${community.avgInfluence.toFixed(6)})`);
    });

    // 4. Generate performance report
    console.log('\n📈 Performance Summary:');
    const report = analyzer.monitor.generateReport();
    console.log(`  Total Operations: ${report.summary.totalOperations}`);
    console.log(`  Average Duration: ${report.summary.avgDuration.toFixed(2)}ms`);
    console.log(`  Success Rate: ${(report.summary.completedOperations / report.summary.totalOperations * 100).toFixed(1)}%`);

  } catch (error) {
    console.error('Error in social influence analysis:', error);
  }
}

// Run example if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runSocialInfluenceExample().catch(console.error);
}

export default SocialInfluenceAnalyzer;