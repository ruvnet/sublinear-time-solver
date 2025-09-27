#!/usr/bin/env node

/**
 * MCP Server for Temporal Attractor Studio
 * Provides chaos analysis tools via Model Context Protocol
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from '@modelcontextprotocol/sdk/types.js';
import * as tas from './pkg-node/temporal_attractor_studio.js';

const server = new Server(
  {
    name: 'temporal-attractor-studio',
    version: '0.1.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Global studio instance
let studio = null;

// Tool definitions
const TOOLS = [
  {
    name: 'chaos_analyze',
    description: 'Calculate Lyapunov exponent and chaos metrics from time series data',
    inputSchema: {
      type: 'object',
      properties: {
        data: {
          type: 'array',
          description: 'Time series data (flattened array)',
          items: { type: 'number' }
        },
        dimensions: {
          type: 'integer',
          description: 'Number of dimensions per time point',
          minimum: 1,
          default: 3
        },
        dt: {
          type: 'number',
          description: 'Time step between measurements',
          default: 0.01
        },
        k_fit: {
          type: 'integer',
          description: 'Points for linear fitting',
          default: 12
        },
        theiler: {
          type: 'integer',
          description: 'Theiler window to exclude temporal neighbors',
          default: 20
        },
        max_pairs: {
          type: 'integer',
          description: 'Maximum trajectory pairs to analyze',
          default: 1000
        }
      },
      required: ['data']
    }
  },
  {
    name: 'delay_embed',
    description: 'Perform delay embedding for univariate time series (Takens theorem)',
    inputSchema: {
      type: 'object',
      properties: {
        series: {
          type: 'array',
          description: 'Univariate time series',
          items: { type: 'number' }
        },
        embedding_dim: {
          type: 'integer',
          description: 'Embedding dimension (typically 3-5)',
          minimum: 2,
          maximum: 10,
          default: 3
        },
        tau: {
          type: 'integer',
          description: 'Time delay (typically 1-10)',
          minimum: 1,
          default: 1
        }
      },
      required: ['series']
    }
  },
  {
    name: 'echo_network_init',
    description: 'Initialize Echo-State Network for temporal prediction',
    inputSchema: {
      type: 'object',
      properties: {
        reservoir_size: {
          type: 'integer',
          description: 'Number of reservoir nodes (100-1000 typical)',
          default: 300
        },
        input_dim: {
          type: 'integer',
          description: 'Input dimension',
          minimum: 1,
          default: 3
        },
        output_dim: {
          type: 'integer',
          description: 'Output dimension',
          minimum: 1,
          default: 3
        },
        spectral_radius: {
          type: 'number',
          description: 'Spectral radius (< 1 for stability)',
          default: 0.95
        },
        connectivity: {
          type: 'number',
          description: 'Reservoir connectivity (0.1-0.3 typical)',
          default: 0.1
        },
        leak_rate: {
          type: 'number',
          description: 'Leak rate for neurons',
          default: 0.3
        }
      },
      required: []
    }
  },
  {
    name: 'echo_network_train',
    description: 'Train Echo-State Network on time series data',
    inputSchema: {
      type: 'object',
      properties: {
        inputs: {
          type: 'array',
          description: 'Training input data (flattened)',
          items: { type: 'number' }
        },
        targets: {
          type: 'array',
          description: 'Training target data (flattened)',
          items: { type: 'number' }
        },
        n_samples: {
          type: 'integer',
          description: 'Number of training samples',
          minimum: 1
        },
        input_dim: {
          type: 'integer',
          description: 'Input dimension',
          minimum: 1
        },
        output_dim: {
          type: 'integer',
          description: 'Output dimension',
          minimum: 1
        }
      },
      required: ['inputs', 'targets', 'n_samples', 'input_dim', 'output_dim']
    }
  },
  {
    name: 'echo_network_predict',
    description: 'Predict next values using trained Echo-State Network',
    inputSchema: {
      type: 'object',
      properties: {
        input: {
          type: 'array',
          description: 'Current state vector',
          items: { type: 'number' }
        },
        n_steps: {
          type: 'integer',
          description: 'Number of steps to predict (for trajectory)',
          default: 1
        }
      },
      required: ['input']
    }
  },
  {
    name: 'fractal_dimension',
    description: 'Estimate fractal dimension using box-counting algorithm',
    inputSchema: {
      type: 'object',
      properties: {
        data: {
          type: 'array',
          description: 'Time series data (flattened)',
          items: { type: 'number' }
        },
        dimensions: {
          type: 'integer',
          description: 'Number of dimensions per point',
          minimum: 1,
          default: 3
        }
      },
      required: ['data']
    }
  },
  {
    name: 'regime_changes',
    description: 'Detect regime changes in time series using sliding window analysis',
    inputSchema: {
      type: 'object',
      properties: {
        data: {
          type: 'array',
          description: 'Time series data (flattened)',
          items: { type: 'number' }
        },
        dimensions: {
          type: 'integer',
          description: 'Dimensions per point',
          minimum: 1,
          default: 3
        },
        window_size: {
          type: 'integer',
          description: 'Size of analysis window',
          default: 50
        },
        stride: {
          type: 'integer',
          description: 'Stride between windows',
          default: 10
        }
      },
      required: ['data']
    }
  },
  {
    name: 'generate_lorenz',
    description: 'Generate Lorenz attractor data for testing',
    inputSchema: {
      type: 'object',
      properties: {
        n_points: {
          type: 'integer',
          description: 'Number of points to generate',
          default: 1000
        },
        dt: {
          type: 'number',
          description: 'Time step',
          default: 0.01
        }
      }
    }
  },
  {
    name: 'generate_henon',
    description: 'Generate Hénon map data for testing',
    inputSchema: {
      type: 'object',
      properties: {
        n_points: {
          type: 'integer',
          description: 'Number of points to generate',
          default: 500
        }
      }
    }
  },
  {
    name: 'interpret_chaos',
    description: 'Get human-readable interpretation of Lyapunov exponent',
    inputSchema: {
      type: 'object',
      properties: {
        lambda: {
          type: 'number',
          description: 'Lyapunov exponent value'
        }
      },
      required: ['lambda']
    }
  },
  {
    name: 'recommend_parameters',
    description: 'Get recommended analysis parameters based on data characteristics',
    inputSchema: {
      type: 'object',
      properties: {
        n_points: {
          type: 'integer',
          description: 'Number of data points',
          minimum: 1
        },
        n_dims: {
          type: 'integer',
          description: 'Number of dimensions',
          minimum: 1,
          default: 3
        },
        sampling_rate: {
          type: 'number',
          description: 'Sampling rate in Hz',
          default: 100
        }
      },
      required: ['n_points']
    }
  }
];

// Initialize studio on server start
function initStudio() {
  if (!studio) {
    studio = new tas.TemporalAttractorStudio();
  }
  return studio;
}

// Handle tool listing
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return { tools: TOOLS };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    initStudio();

    switch (name) {
      case 'chaos_analyze': {
        const result = studio.calculate_lyapunov(
          args.data,
          args.dimensions || 3,
          args.dt || 0.01,
          args.k_fit || 12,
          args.theiler || 20,
          args.max_pairs || 1000,
          1e-10
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                lambda: result.lambda,
                is_chaotic: result.is_chaotic,
                chaos_level: result.chaos_level,
                lyapunov_time: result.lyapunov_time,
                doubling_time: result.doubling_time,
                safe_prediction_steps: result.safe_prediction_steps,
                pairs_found: result.pairs_found,
                interpretation: `System is ${result.chaos_level} with λ=${result.lambda.toFixed(4)}. ` +
                  `Predictability horizon: ${result.lyapunov_time.toFixed(2)} time units. ` +
                  `Errors double every ${result.doubling_time.toFixed(2)} units.`
              }, null, 2)
            }
          ]
        };
      }

      case 'delay_embed': {
        const embedded = studio.delay_embedding(
          args.series,
          args.embedding_dim || 3,
          args.tau || 1
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                original_length: args.series.length,
                embedded_vectors: embedded.length / (args.embedding_dim || 3),
                embedding_dim: args.embedding_dim || 3,
                tau: args.tau || 1,
                data: embedded
              }, null, 2)
            }
          ]
        };
      }

      case 'echo_network_init': {
        studio.init_echo_network(
          args.reservoir_size || 300,
          args.input_dim || 3,
          args.output_dim || 3,
          args.spectral_radius || 0.95,
          args.connectivity || 0.1,
          0.5, // input_scaling
          args.leak_rate || 0.3,
          1e-6 // ridge_param
        );

        return {
          content: [
            {
              type: 'text',
              text: 'Echo-State Network initialized successfully'
            }
          ]
        };
      }

      case 'echo_network_train': {
        const mse = studio.train_echo_network(
          args.inputs,
          args.targets,
          args.n_samples,
          args.input_dim,
          args.output_dim
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                training_complete: true,
                mse: mse,
                n_samples: args.n_samples
              }, null, 2)
            }
          ]
        };
      }

      case 'echo_network_predict': {
        let prediction;
        if (args.n_steps && args.n_steps > 1) {
          prediction = studio.predict_trajectory(args.input, args.n_steps);
        } else {
          prediction = studio.predict_next(args.input);
        }

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                input: args.input,
                prediction: prediction,
                n_steps: args.n_steps || 1
              }, null, 2)
            }
          ]
        };
      }

      case 'fractal_dimension': {
        const dimension = studio.estimate_fractal_dimension(
          args.data,
          args.dimensions || 3
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                fractal_dimension: dimension,
                interpretation: dimension > 2 ? 'Complex attractor' :
                               dimension > 1 ? 'Fractal structure' :
                               'Simple dynamics'
              }, null, 2)
            }
          ]
        };
      }

      case 'regime_changes': {
        const regimes = studio.detect_regime_changes(
          args.data,
          args.dimensions || 3,
          args.window_size || 50,
          args.stride || 10
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                n_windows: regimes.length,
                lyapunov_values: regimes,
                changes_detected: regimes.length > 1 &&
                  Math.max(...regimes) - Math.min(...regimes) > 0.1
              }, null, 2)
            }
          ]
        };
      }

      case 'generate_lorenz': {
        const data = tas.generate_lorenz_data(
          args.n_points || 1000,
          args.dt || 0.01
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                system: 'Lorenz attractor',
                n_points: args.n_points || 1000,
                dimensions: 3,
                dt: args.dt || 0.01,
                data: data
              }, null, 2)
            }
          ]
        };
      }

      case 'generate_henon': {
        const data = tas.generate_henon_data(args.n_points || 500);

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                system: 'Hénon map',
                n_points: args.n_points || 500,
                dimensions: 2,
                data: data
              }, null, 2)
            }
          ]
        };
      }

      case 'interpret_chaos': {
        const interpretation = studio.interpret_chaos(args.lambda);

        return {
          content: [
            {
              type: 'text',
              text: interpretation
            }
          ]
        };
      }

      case 'recommend_parameters': {
        const recommendations = studio.recommend_parameters(
          args.n_points,
          args.n_dims || 3,
          args.sampling_rate || 100
        );

        return {
          content: [
            {
              type: 'text',
              text: recommendations
            }
          ]
        };
      }

      default:
        throw new McpError(
          ErrorCode.MethodNotFound,
          `Tool '${name}' not found`
        );
    }
  } catch (error) {
    throw new McpError(
      ErrorCode.InternalError,
      error.message || 'Tool execution failed'
    );
  }
});

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Temporal Attractor Studio MCP server running');
}

main().catch((error) => {
  console.error('Server error:', error);
  process.exit(1);
});