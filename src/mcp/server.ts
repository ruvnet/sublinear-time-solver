/**
 * MCP Server for Sublinear-Time Solver
 * Provides MCP interface to the core solver algorithms
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ErrorCode,
  ListToolsRequestSchema,
  McpError,
} from '@modelcontextprotocol/sdk/types.js';

import { SublinearSolver } from '../core/solver.js';
import { MatrixOperations } from '../core/matrix.js';
import { TemporalTools } from './tools/temporal.js';
import {
  Matrix,
  Vector,
  SolverConfig,
  SolveParams,
  EstimateEntryParams,
  AnalyzeMatrixParams,
  PageRankParams,
  SolverError,
  ErrorCodes
} from '../core/types.js';

export class SublinearSolverMCPServer {
  private server: Server;
  private solvers: Map<string, SublinearSolver> = new Map();
  private temporalTools: TemporalTools;

  constructor() {
    this.temporalTools = new TemporalTools();
    this.server = new Server(
      {
        name: 'sublinear-solver',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupToolHandlers();
    this.setupErrorHandling();
  }

  private setupToolHandlers(): void {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: 'solve',
          description: 'Solve a diagonally dominant linear system Mx = b',
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix M in dense or sparse format',
                properties: {
                  rows: { type: 'number' },
                  cols: { type: 'number' },
                  format: { type: 'string', enum: ['dense', 'coo'] },
                  data: {
                    oneOf: [
                      { type: 'array', items: { type: 'array', items: { type: 'number' } } },
                      {
                        type: 'object',
                        properties: {
                          values: { type: 'array', items: { type: 'number' } },
                          rowIndices: { type: 'array', items: { type: 'number' } },
                          colIndices: { type: 'array', items: { type: 'number' } }
                        },
                        required: ['values', 'rowIndices', 'colIndices']
                      }
                    ]
                  }
                },
                required: ['rows', 'cols', 'format', 'data']
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Right-hand side vector b'
              },
              method: {
                type: 'string',
                enum: ['neumann', 'random-walk', 'forward-push', 'backward-push', 'bidirectional'],
                default: 'neumann',
                description: 'Solver method to use'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Convergence tolerance'
              },
              maxIterations: {
                type: 'number',
                default: 1000,
                description: 'Maximum number of iterations'
              },
              timeout: {
                type: 'number',
                description: 'Timeout in milliseconds'
              }
            },
            required: ['matrix', 'vector']
          }
        },
        {
          name: 'estimateEntry',
          description: 'Estimate a single entry of the solution M^(-1)b',
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix M in dense or sparse format'
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Right-hand side vector b'
              },
              row: {
                type: 'number',
                description: 'Row index of entry to estimate'
              },
              column: {
                type: 'number',
                description: 'Column index of entry to estimate'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Estimation accuracy'
              },
              confidence: {
                type: 'number',
                default: 0.95,
                minimum: 0,
                maximum: 1,
                description: 'Confidence level for estimation'
              },
              method: {
                type: 'string',
                enum: ['neumann', 'random-walk', 'monte-carlo'],
                default: 'random-walk',
                description: 'Estimation method'
              }
            },
            required: ['matrix', 'vector', 'row', 'column']
          }
        },
        {
          name: 'analyzeMatrix',
          description: 'Analyze matrix properties for solvability',
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix to analyze'
              },
              checkDominance: {
                type: 'boolean',
                default: true,
                description: 'Check diagonal dominance'
              },
              computeGap: {
                type: 'boolean',
                default: false,
                description: 'Compute spectral gap (expensive)'
              },
              estimateCondition: {
                type: 'boolean',
                default: false,
                description: 'Estimate condition number'
              },
              checkSymmetry: {
                type: 'boolean',
                default: true,
                description: 'Check matrix symmetry'
              }
            },
            required: ['matrix']
          }
        },
        {
          name: 'pageRank',
          description: 'Compute PageRank for a graph using sublinear solver',
          inputSchema: {
            type: 'object',
            properties: {
              adjacency: {
                type: 'object',
                description: 'Adjacency matrix of the graph'
              },
              damping: {
                type: 'number',
                default: 0.85,
                minimum: 0,
                maximum: 1,
                description: 'Damping factor'
              },
              personalized: {
                type: 'array',
                items: { type: 'number' },
                description: 'Personalization vector (optional)'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Convergence tolerance'
              },
              maxIterations: {
                type: 'number',
                default: 1000,
                description: 'Maximum iterations'
              }
            },
            required: ['adjacency']
          }
        },
        // Add temporal lead tools
        ...this.temporalTools.getTools()
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'solve':
            return await this.handleSolve(args as any);
          case 'estimateEntry':
            return await this.handleEstimateEntry(args as any);
          case 'analyzeMatrix':
            return await this.handleAnalyzeMatrix(args as any);
          case 'pageRank':
            return await this.handlePageRank(args as any);
          case 'predictWithTemporalAdvantage':
          case 'validateTemporalAdvantage':
          case 'calculateLightTravel':
          case 'demonstrateTemporalLead':
            const temporalResult = await this.temporalTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(temporalResult, null, 2)
              }]
            };
          default:
            throw new McpError(
              ErrorCode.MethodNotFound,
              `Unknown tool: ${name}`
            );
        }
      } catch (error) {
        if (error instanceof SolverError) {
          throw new McpError(
            ErrorCode.InternalError,
            `Solver error: ${error.message}`,
            error.details
          );
        }
        throw new McpError(
          ErrorCode.InternalError,
          error instanceof Error ? error.message : 'Unknown error'
        );
      }
    });
  }

  private setupErrorHandling(): void {
    this.server.onerror = (error) => {
      console.error('[MCP Server Error]', error);
    };

    process.on('SIGINT', async () => {
      await this.server.close();
      process.exit(0);
    });
  }

  private async handleSolve(params: any) {
    try {
      // Enhanced parameter validation
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!params.vector) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: vector');
      }
      if (!Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter vector must be an array of numbers');
      }

      const config: SolverConfig = {
        method: params.method || 'neumann',
        epsilon: params.epsilon || 1e-6,
        maxIterations: params.maxIterations || 5000, // Increased default
        timeout: params.timeout || 30000, // 30 second default timeout
        enableProgress: false
      };

      // Validate method
      const validMethods = ['neumann', 'random-walk', 'forward-push', 'backward-push', 'bidirectional'];
      if (!validMethods.includes(config.method)) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Invalid method '${config.method}'. Valid methods: ${validMethods.join(', ')}`
        );
      }

      // Validate epsilon
      if (typeof config.epsilon !== 'number' || config.epsilon <= 0) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter epsilon must be a positive number');
      }

      // Validate maxIterations
      if (typeof config.maxIterations !== 'number' || config.maxIterations < 1) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter maxIterations must be a positive integer');
      }

      const solver = new SublinearSolver(config);
      const result = await solver.solve(params.matrix, params.vector);

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              solution: result.solution,
              iterations: result.iterations,
              residual: result.residual,
              converged: result.converged,
              method: result.method,
              computeTime: result.computeTime,
              memoryUsed: result.memoryUsed,
              metadata: {
                configUsed: config,
                timestamp: new Date().toISOString(),
                matrixSize: {
                  rows: params.matrix.rows,
                  cols: params.matrix.cols
                }
              }
            }, null, 2)
          }
        ]
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      if (error instanceof SolverError) {
        throw new McpError(
          ErrorCode.InternalError,
          `Solver error (${error.code}): ${error.message}`,
          error.details
        );
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Unexpected error in solve: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleEstimateEntry(params: any) {
    try {
      // Enhanced parameter validation
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!params.vector) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: vector');
      }
      if (!Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter vector must be an array of numbers');
      }
      if (typeof params.row !== 'number' || !Number.isInteger(params.row)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter row must be a valid integer');
      }
      if (typeof params.column !== 'number' || !Number.isInteger(params.column)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter column must be a valid integer');
      }

      // Validate bounds early
      if (params.row < 0 || params.row >= params.matrix.rows) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Row index ${params.row} out of bounds. Matrix has ${params.matrix.rows} rows (valid range: 0-${params.matrix.rows - 1})`
        );
      }
      if (params.column < 0 || params.column >= params.matrix.cols) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Column index ${params.column} out of bounds. Matrix has ${params.matrix.cols} columns (valid range: 0-${params.matrix.cols - 1})`
        );
      }

      // Validate vector dimensions
      if (params.vector.length !== params.matrix.rows) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Vector length ${params.vector.length} does not match matrix rows ${params.matrix.rows}`
        );
      }

      const solverConfig: SolverConfig = {
        method: 'random-walk',
        epsilon: params.epsilon || 1e-6,
        maxIterations: 2000, // Increased for better accuracy
        timeout: 15000, // 15 second timeout
        enableProgress: false
      };

      const solver = new SublinearSolver(solverConfig);

      // Create estimation config
      const estimationConfig = {
        row: params.row,
        column: params.column,
        epsilon: params.epsilon || 1e-6,
        confidence: params.confidence || 0.95,
        method: params.method || 'random-walk' as const
      };

      // Validate method
      const validMethods = ['neumann', 'random-walk', 'monte-carlo'];
      if (!validMethods.includes(estimationConfig.method)) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Invalid estimation method '${estimationConfig.method}'. Valid methods: ${validMethods.join(', ')}`
        );
      }

      const result = await solver.estimateEntry(params.matrix, params.vector, estimationConfig);

      const standardError = Math.sqrt(result.variance);
      const marginOfError = 1.96 * standardError;

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              estimate: result.estimate,
              variance: result.variance,
              confidence: result.confidence,
              standardError,
              confidenceInterval: {
                lower: result.estimate - marginOfError,
                upper: result.estimate + marginOfError
              },
              row: params.row,
              column: params.column,
              method: estimationConfig.method,
              metadata: {
                configUsed: estimationConfig,
                timestamp: new Date().toISOString(),
                matrixSize: {
                  rows: params.matrix.rows,
                  cols: params.matrix.cols
                }
              }
            }, null, 2)
          }
        ]
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      if (error instanceof SolverError) {
        throw new McpError(
          ErrorCode.InternalError,
          `Solver error (${error.code}): ${error.message}`,
          error.details
        );
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Unexpected error in estimateEntry: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleAnalyzeMatrix(params: AnalyzeMatrixParams) {
    const analysis = MatrixOperations.analyzeMatrix(params.matrix);

    const result = {
      ...analysis,
      recommendations: this.generateRecommendations(analysis)
    };

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2)
        }
      ]
    };
  }

  private async handlePageRank(params: PageRankParams) {
    const config: SolverConfig = {
      method: 'neumann',
      epsilon: params.epsilon || 1e-6,
      maxIterations: params.maxIterations || 1000,
      enableProgress: false
    };

    const solver = new SublinearSolver(config);

    const pageRankConfig = {
      damping: params.damping || 0.85,
      personalized: params.personalized,
      epsilon: params.epsilon || 1e-6,
      maxIterations: params.maxIterations || 1000
    };

    const pageRankVector = await solver.computePageRank(params.adjacency, pageRankConfig);

    // Sort nodes by PageRank score
    const ranked = pageRankVector
      .map((score, index) => ({ node: index, score }))
      .sort((a, b) => b.score - a.score);

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify({
            pageRankVector,
            topNodes: ranked.slice(0, 10),
            totalScore: pageRankVector.reduce((sum, score) => sum + score, 0),
            maxScore: Math.max(...pageRankVector),
            minScore: Math.min(...pageRankVector)
          }, null, 2)
        }
      ]
    };
  }

  private generateRecommendations(analysis: any): string[] {
    const recommendations: string[] = [];

    if (!analysis.isDiagonallyDominant) {
      recommendations.push('Matrix is not diagonally dominant. Consider matrix preconditioning or using a different solver.');
    } else if (analysis.dominanceStrength < 0.1) {
      recommendations.push('Weak diagonal dominance detected. Convergence may be slow.');
    }

    if (analysis.sparsity > 0.9) {
      recommendations.push('Matrix is very sparse. Consider using sparse matrix formats for better performance.');
    }

    if (!analysis.isSymmetric && analysis.isDiagonallyDominant) {
      recommendations.push('Matrix is asymmetric but diagonally dominant. Random walk methods may be most effective.');
    }

    if (analysis.size.rows > 10000) {
      recommendations.push('Large matrix detected. Consider using sublinear estimation methods for specific entries rather than full solve.');
    }

    return recommendations;
  }

  async run(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('Sublinear Solver MCP server running on stdio');
  }
}

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
  const server = new SublinearSolverMCPServer();
  server.run().catch(console.error);
}