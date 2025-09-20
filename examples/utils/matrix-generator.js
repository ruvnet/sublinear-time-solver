/**
 * Matrix Generator Utilities
 *
 * Provides utilities for generating test matrices with specific properties
 * including diagonally dominant matrices for sublinear solver testing.
 */

import crypto from 'crypto';

export class MatrixGenerator {
  /**
   * Generate a diagonally dominant matrix suitable for sublinear solving
   * @param {number} size - Matrix dimension (size x size)
   * @param {number} dominanceFactor - How dominant the diagonal should be (default: 2.0)
   * @param {number} sparsity - Fraction of non-zero off-diagonal elements (default: 0.1)
   * @param {string} format - Output format: 'dense' or 'coo' (default: 'coo')
   * @returns {Object} Matrix in requested format
   */
  static generateDiagonallyDominant(size, dominanceFactor = 2.0, sparsity = 0.1, format = 'coo') {
    const matrix = Array(size).fill().map(() => Array(size).fill(0));

    // Generate random off-diagonal elements
    for (let i = 0; i < size; i++) {
      let rowSum = 0;
      for (let j = 0; j < size; j++) {
        if (i !== j && Math.random() < sparsity) {
          const value = (Math.random() - 0.5) * 2; // Random value between -1 and 1
          matrix[i][j] = value;
          rowSum += Math.abs(value);
        }
      }

      // Set diagonal to ensure diagonal dominance
      matrix[i][i] = rowSum * dominanceFactor + Math.random() + 1;
    }

    if (format === 'dense') {
      return {
        rows: size,
        cols: size,
        format: 'dense',
        data: matrix
      };
    } else {
      // Convert to COO (Coordinate) format
      const values = [];
      const rowIndices = [];
      const colIndices = [];

      for (let i = 0; i < size; i++) {
        for (let j = 0; j < size; j++) {
          if (matrix[i][j] !== 0) {
            values.push(matrix[i][j]);
            rowIndices.push(i);
            colIndices.push(j);
          }
        }
      }

      return {
        rows: size,
        cols: size,
        format: 'coo',
        data: {
          values,
          rowIndices,
          colIndices
        }
      };
    }
  }

  /**
   * Generate a random vector
   * @param {number} size - Vector dimension
   * @param {number} min - Minimum value (default: -10)
   * @param {number} max - Maximum value (default: 10)
   * @returns {Array} Random vector
   */
  static generateRandomVector(size, min = -10, max = 10) {
    return Array(size).fill().map(() =>
      Math.random() * (max - min) + min
    );
  }

  /**
   * Generate a correlation matrix (symmetric, positive semi-definite)
   * Useful for portfolio optimization and risk modeling
   * @param {number} size - Matrix dimension
   * @param {number} sparsity - Fraction of non-zero correlations
   * @returns {Object} Correlation matrix in dense format
   */
  static generateCorrelationMatrix(size, sparsity = 0.3) {
    const matrix = Array(size).fill().map(() => Array(size).fill(0));

    // Set diagonal to 1 (perfect self-correlation)
    for (let i = 0; i < size; i++) {
      matrix[i][i] = 1.0;
    }

    // Generate symmetric correlations
    for (let i = 0; i < size; i++) {
      for (let j = i + 1; j < size; j++) {
        if (Math.random() < sparsity) {
          const correlation = (Math.random() - 0.5) * 1.8; // Correlation between -0.9 and 0.9
          matrix[i][j] = correlation;
          matrix[j][i] = correlation; // Ensure symmetry
        }
      }
    }

    return {
      rows: size,
      cols: size,
      format: 'dense',
      data: matrix
    };
  }

  /**
   * Generate a social network adjacency matrix
   * Follows power-law distribution typical of real social networks
   * @param {number} size - Number of nodes
   * @param {number} avgDegree - Average node degree
   * @returns {Object} Adjacency matrix in COO format
   */
  static generateSocialNetwork(size, avgDegree = 10) {
    const values = [];
    const rowIndices = [];
    const colIndices = [];

    // Generate power-law degree distribution
    const degrees = this._generatePowerLawDegrees(size, avgDegree);

    // Create edges based on preferential attachment
    for (let i = 0; i < size; i++) {
      const targetDegree = degrees[i];
      const connectedNodes = new Set();

      for (let edge = 0; edge < targetDegree && connectedNodes.size < size - 1; edge++) {
        let target;
        do {
          // Preferential attachment: higher degree nodes more likely to connect
          target = this._selectPreferentialTarget(size, degrees, connectedNodes);
        } while (target === i || connectedNodes.has(target));

        connectedNodes.add(target);

        // Add edge (undirected graph)
        values.push(1);
        rowIndices.push(i);
        colIndices.push(target);

        values.push(1);
        rowIndices.push(target);
        colIndices.push(i);
      }
    }

    return {
      rows: size,
      cols: size,
      format: 'coo',
      data: {
        values,
        rowIndices,
        colIndices
      }
    };
  }

  /**
   * Generate market price correlation matrix
   * Models real financial asset correlations with clustering
   * @param {Array} assetClasses - Array of asset class sizes [stocks, bonds, commodities, ...]
   * @param {number} intraClassCorr - Correlation within asset classes
   * @param {number} interClassCorr - Correlation between asset classes
   * @returns {Object} Market correlation matrix
   */
  static generateMarketCorrelations(assetClasses, intraClassCorr = 0.6, interClassCorr = 0.2) {
    const totalAssets = assetClasses.reduce((sum, size) => sum + size, 0);
    const matrix = Array(totalAssets).fill().map(() => Array(totalAssets).fill(0));

    let assetIndex = 0;

    // Set correlations within each asset class
    for (const classSize of assetClasses) {
      for (let i = assetIndex; i < assetIndex + classSize; i++) {
        for (let j = assetIndex; j < assetIndex + classSize; j++) {
          if (i === j) {
            matrix[i][j] = 1.0; // Perfect self-correlation
          } else {
            // Add some noise to intra-class correlation
            const noise = (Math.random() - 0.5) * 0.2;
            matrix[i][j] = Math.max(-0.9, Math.min(0.9, intraClassCorr + noise));
          }
        }
      }
      assetIndex += classSize;
    }

    // Set correlations between asset classes
    let class1Start = 0;
    for (let c1 = 0; c1 < assetClasses.length; c1++) {
      let class2Start = class1Start + assetClasses[c1];

      for (let c2 = c1 + 1; c2 < assetClasses.length; c2++) {
        for (let i = class1Start; i < class1Start + assetClasses[c1]; i++) {
          for (let j = class2Start; j < class2Start + assetClasses[c2]; j++) {
            const noise = (Math.random() - 0.5) * 0.15;
            const correlation = Math.max(-0.9, Math.min(0.9, interClassCorr + noise));
            matrix[i][j] = correlation;
            matrix[j][i] = correlation; // Ensure symmetry
          }
        }
        class2Start += assetClasses[c2];
      }
      class1Start += assetClasses[c1];
    }

    return {
      rows: totalAssets,
      cols: totalAssets,
      format: 'dense',
      data: matrix,
      metadata: {
        assetClasses,
        totalAssets,
        intraClassCorr,
        interClassCorr
      }
    };
  }

  /**
   * Generate time series data for temporal-lead testing
   * @param {number} length - Time series length
   * @param {number} volatility - Price volatility
   * @param {number} trend - Trend component
   * @returns {Object} Time series data with timestamps
   */
  static generateTimeSeriesData(length, volatility = 0.02, trend = 0.001) {
    const data = [];
    const timestamps = [];
    let price = 100; // Starting price

    const startTime = Date.now() - (length * 1000); // 1 second intervals

    for (let i = 0; i < length; i++) {
      const timestamp = startTime + (i * 1000);
      const randomWalk = (Math.random() - 0.5) * volatility;
      const trendComponent = trend * i;

      price *= (1 + randomWalk + trendComponent);

      data.push(price);
      timestamps.push(timestamp);
    }

    return {
      values: data,
      timestamps,
      metadata: {
        length,
        volatility,
        trend,
        startPrice: 100,
        endPrice: data[data.length - 1]
      }
    };
  }

  // Helper methods
  static _generatePowerLawDegrees(size, avgDegree) {
    const degrees = [];
    const alpha = 2.5; // Power-law exponent typical for social networks

    for (let i = 0; i < size; i++) {
      // Generate power-law distributed degree
      const u = Math.random();
      const degree = Math.floor(Math.pow(u, -1 / (alpha - 1)) * avgDegree / 2);
      degrees.push(Math.min(degree, size - 1));
    }

    return degrees;
  }

  static _selectPreferentialTarget(size, degrees, excludeSet) {
    const totalDegree = degrees.reduce((sum, deg) => sum + deg + 1, 0); // +1 to avoid zero degrees
    let target = Math.floor(Math.random() * totalDegree);

    for (let i = 0; i < size; i++) {
      if (!excludeSet.has(i)) {
        target -= (degrees[i] + 1);
        if (target <= 0) {
          return i;
        }
      }
    }

    // Fallback to random selection
    let result;
    do {
      result = Math.floor(Math.random() * size);
    } while (excludeSet.has(result));

    return result;
  }

  /**
   * Validate matrix properties
   * @param {Object} matrix - Matrix to validate
   * @returns {Object} Validation results
   */
  static validateMatrix(matrix) {
    const results = {
      isDiagonallyDominant: false,
      isSymmetric: false,
      isPositiveDefinite: false,
      conditionNumber: null,
      sparsity: 0,
      errors: []
    };

    try {
      if (matrix.format === 'dense') {
        results.isDiagonallyDominant = this._checkDiagonalDominance(matrix.data);
        results.isSymmetric = this._checkSymmetry(matrix.data);
        results.sparsity = this._calculateSparsity(matrix.data);
      } else {
        results.errors.push('Validation only supports dense format currently');
      }
    } catch (error) {
      results.errors.push(`Validation error: ${error.message}`);
    }

    return results;
  }

  static _checkDiagonalDominance(matrix) {
    const n = matrix.length;
    for (let i = 0; i < n; i++) {
      let diagonal = Math.abs(matrix[i][i]);
      let offDiagonalSum = 0;

      for (let j = 0; j < n; j++) {
        if (i !== j) {
          offDiagonalSum += Math.abs(matrix[i][j]);
        }
      }

      if (diagonal <= offDiagonalSum) {
        return false;
      }
    }
    return true;
  }

  static _checkSymmetry(matrix) {
    const n = matrix.length;
    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        if (Math.abs(matrix[i][j] - matrix[j][i]) > 1e-10) {
          return false;
        }
      }
    }
    return true;
  }

  static _calculateSparsity(matrix) {
    const n = matrix.length;
    let nonZeroCount = 0;

    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        if (Math.abs(matrix[i][j]) > 1e-12) {
          nonZeroCount++;
        }
      }
    }

    return 1 - (nonZeroCount / (n * n));
  }
}

export default MatrixGenerator;