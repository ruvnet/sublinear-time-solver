/**
 * Data Loader Utilities
 *
 * Provides utilities for loading and formatting data from various sources
 * including files, APIs, and databases for sublinear solver examples.
 */

import fs from 'fs/promises';
import path from 'path';

export class DataLoader {
  /**
   * Load matrix data from JSON file
   * @param {string} filePath - Path to JSON file
   * @returns {Promise<Object>} Matrix data
   */
  static async loadMatrixFromJSON(filePath) {
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const matrix = JSON.parse(data);

      // Validate matrix structure
      if (!matrix.rows || !matrix.cols || !matrix.format || !matrix.data) {
        throw new Error('Invalid matrix format: missing required fields');
      }

      return matrix;
    } catch (error) {
      throw new Error(`Failed to load matrix from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Load vector data from JSON file
   * @param {string} filePath - Path to JSON file
   * @returns {Promise<Array>} Vector data
   */
  static async loadVectorFromJSON(filePath) {
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const vector = JSON.parse(data);

      if (!Array.isArray(vector)) {
        throw new Error('Vector data must be an array');
      }

      return vector;
    } catch (error) {
      throw new Error(`Failed to load vector from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Load CSV data and convert to matrix/vector format
   * @param {string} filePath - Path to CSV file
   * @param {Object} options - Parsing options
   * @returns {Promise<Object>} Parsed data
   */
  static async loadCSV(filePath, options = {}) {
    const {
      hasHeader = true,
      delimiter = ',',
      matrixFormat = 'dense'
    } = options;

    try {
      const data = await fs.readFile(filePath, 'utf8');
      const lines = data.trim().split('\n');

      let startIndex = hasHeader ? 1 : 0;
      const rows = [];

      for (let i = startIndex; i < lines.length; i++) {
        const row = lines[i].split(delimiter).map(cell => parseFloat(cell.trim()));
        if (row.some(isNaN)) {
          throw new Error(`Invalid numeric data on line ${i + 1}`);
        }
        rows.push(row);
      }

      const numRows = rows.length;
      const numCols = rows[0]?.length || 0;

      if (matrixFormat === 'dense') {
        return {
          rows: numRows,
          cols: numCols,
          format: 'dense',
          data: rows
        };
      } else {
        // Convert to COO format
        const values = [];
        const rowIndices = [];
        const colIndices = [];

        for (let i = 0; i < numRows; i++) {
          for (let j = 0; j < numCols; j++) {
            if (rows[i][j] !== 0) {
              values.push(rows[i][j]);
              rowIndices.push(i);
              colIndices.push(j);
            }
          }
        }

        return {
          rows: numRows,
          cols: numCols,
          format: 'coo',
          data: {
            values,
            rowIndices,
            colIndices
          }
        };
      }
    } catch (error) {
      throw new Error(`Failed to load CSV from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Load financial market data
   * @param {string} filePath - Path to market data file
   * @returns {Promise<Object>} Market data with timestamps and prices
   */
  static async loadMarketData(filePath) {
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const marketData = JSON.parse(data);

      // Validate market data structure
      if (!marketData.symbols || !marketData.timestamps || !marketData.prices) {
        throw new Error('Invalid market data format');
      }

      // Convert to correlation matrix if multiple symbols
      if (marketData.symbols.length > 1) {
        const correlationMatrix = this._calculateCorrelationMatrix(marketData.prices);
        return {
          ...marketData,
          correlationMatrix
        };
      }

      return marketData;
    } catch (error) {
      throw new Error(`Failed to load market data from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Load social network data
   * @param {string} filePath - Path to network data file
   * @returns {Promise<Object>} Network data with adjacency matrix
   */
  static async loadNetworkData(filePath) {
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const networkData = JSON.parse(data);

      if (networkData.format === 'edgelist') {
        // Convert edge list to adjacency matrix
        const adjacencyMatrix = this._edgeListToAdjacency(networkData.edges, networkData.nodeCount);
        return {
          ...networkData,
          adjacencyMatrix
        };
      }

      return networkData;
    } catch (error) {
      throw new Error(`Failed to load network data from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Save data to JSON file
   * @param {string} filePath - Output file path
   * @param {Object} data - Data to save
   * @param {Object} options - Save options
   */
  static async saveToJSON(filePath, data, options = {}) {
    const { indent = 2, createDirectories = true } = options;

    try {
      if (createDirectories) {
        const dir = path.dirname(filePath);
        await fs.mkdir(dir, { recursive: true });
      }

      const jsonData = JSON.stringify(data, null, indent);
      await fs.writeFile(filePath, jsonData, 'utf8');
    } catch (error) {
      throw new Error(`Failed to save data to ${filePath}: ${error.message}`);
    }
  }

  /**
   * Load configuration from file
   * @param {string} configPath - Path to configuration file
   * @returns {Promise<Object>} Configuration object
   */
  static async loadConfig(configPath) {
    try {
      const data = await fs.readFile(configPath, 'utf8');
      return JSON.parse(data);
    } catch (error) {
      // Return default config if file doesn't exist
      if (error.code === 'ENOENT') {
        return this._getDefaultConfig();
      }
      throw new Error(`Failed to load config from ${configPath}: ${error.message}`);
    }
  }

  /**
   * Load time series data with temporal metadata
   * @param {string} filePath - Path to time series file
   * @returns {Promise<Object>} Time series data with temporal information
   */
  static async loadTimeSeriesData(filePath) {
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const timeSeriesData = JSON.parse(data);

      // Calculate temporal properties
      const temporalProps = this._analyzeTemporalProperties(timeSeriesData);

      return {
        ...timeSeriesData,
        temporal: temporalProps
      };
    } catch (error) {
      throw new Error(`Failed to load time series data from ${filePath}: ${error.message}`);
    }
  }

  /**
   * Create sample data directory structure
   * @param {string} baseDir - Base directory for sample data
   */
  static async createSampleDataStructure(baseDir) {
    const directories = [
      'financial',
      'networks',
      'matrices',
      'time-series',
      'benchmarks'
    ];

    for (const dir of directories) {
      await fs.mkdir(path.join(baseDir, dir), { recursive: true });
    }
  }

  // Helper methods
  static _calculateCorrelationMatrix(prices) {
    const n = prices.length;
    const m = prices[0].length;
    const correlations = Array(n).fill().map(() => Array(n).fill(0));

    // Calculate means
    const means = prices.map(series =>
      series.reduce((sum, val) => sum + val, 0) / m
    );

    // Calculate correlation coefficients
    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        if (i === j) {
          correlations[i][j] = 1.0;
        } else {
          let numerator = 0;
          let denomI = 0;
          let denomJ = 0;

          for (let k = 0; k < m; k++) {
            const diffI = prices[i][k] - means[i];
            const diffJ = prices[j][k] - means[j];
            numerator += diffI * diffJ;
            denomI += diffI * diffI;
            denomJ += diffJ * diffJ;
          }

          const correlation = numerator / Math.sqrt(denomI * denomJ);
          correlations[i][j] = isNaN(correlation) ? 0 : correlation;
        }
      }
    }

    return {
      rows: n,
      cols: n,
      format: 'dense',
      data: correlations
    };
  }

  static _edgeListToAdjacency(edges, nodeCount) {
    const matrix = Array(nodeCount).fill().map(() => Array(nodeCount).fill(0));

    for (const edge of edges) {
      const [from, to, weight = 1] = edge;
      matrix[from][to] = weight;

      // Assume undirected unless specified
      if (!edge.directed) {
        matrix[to][from] = weight;
      }
    }

    return {
      rows: nodeCount,
      cols: nodeCount,
      format: 'dense',
      data: matrix
    };
  }

  static _analyzeTemporalProperties(timeSeriesData) {
    const { timestamps, values } = timeSeriesData;

    if (!timestamps || !values || timestamps.length !== values.length) {
      throw new Error('Invalid time series data structure');
    }

    // Calculate time intervals
    const intervals = [];
    for (let i = 1; i < timestamps.length; i++) {
      intervals.push(timestamps[i] - timestamps[i - 1]);
    }

    const avgInterval = intervals.reduce((sum, val) => sum + val, 0) / intervals.length;
    const minInterval = Math.min(...intervals);
    const maxInterval = Math.max(...intervals);

    // Calculate volatility
    const returns = [];
    for (let i = 1; i < values.length; i++) {
      returns.push((values[i] - values[i - 1]) / values[i - 1]);
    }

    const volatility = Math.sqrt(
      returns.reduce((sum, ret) => sum + ret * ret, 0) / returns.length
    );

    return {
      length: timestamps.length,
      duration: timestamps[timestamps.length - 1] - timestamps[0],
      avgInterval,
      minInterval,
      maxInterval,
      volatility,
      frequency: 1000 / avgInterval, // Hz assuming millisecond timestamps
      isRegular: Math.abs(maxInterval - minInterval) < avgInterval * 0.1
    };
  }

  static _getDefaultConfig() {
    return {
      solver: {
        method: 'neumann',
        epsilon: 1e-6,
        maxIterations: 1000,
        timeout: 30000
      },
      temporal: {
        defaultDistance: 12000, // km
        lightSpeed: 299792458 // m/s
      },
      matrix: {
        defaultFormat: 'coo',
        sparsityThreshold: 0.9
      },
      pagerank: {
        damping: 0.85,
        epsilon: 1e-6,
        maxIterations: 100
      }
    };
  }
}

export default DataLoader;