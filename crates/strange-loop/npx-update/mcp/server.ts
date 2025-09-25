#!/usr/bin/env node

/**
 * Strange Loops MCP Server
 * Provides nano-agent, quantum-classical hybrid computing, and temporal prediction tools
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from '@modelcontextprotocol/sdk/types.js';

// Import our Strange Loop library
const StrangeLoop = require('../lib/strange-loop.js');

class StrangeLoopsMCPServer {
  private server: Server;
  private isInitialized: boolean = false;

  constructor() {
    this.server = new Server(
      {
        name: 'strange-loops',
        version: '0.1.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupHandlers();
  }

  private setupHandlers(): void {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: [
          {
            name: 'nano_swarm_create',
            description: 'Create a nano-agent swarm with specified configuration',
            inputSchema: {
              type: 'object',
              properties: {
                agentCount: {
                  type: 'number',
                  description: 'Number of agents in the swarm',
                  default: 1000,
                  minimum: 1,
                  maximum: 100000
                },
                topology: {
                  type: 'string',
                  description: 'Swarm topology',
                  enum: ['mesh', 'hierarchical', 'ring', 'star'],
                  default: 'mesh'
                },
                tickDurationNs: {
                  type: 'number',
                  description: 'Tick duration in nanoseconds',
                  default: 25000
                }
              }
            }
          },
          {
            name: 'nano_swarm_run',
            description: 'Run nano-agent swarm simulation for specified duration',
            inputSchema: {
              type: 'object',
              properties: {
                durationMs: {
                  type: 'number',
                  description: 'Simulation duration in milliseconds',
                  default: 5000,
                  minimum: 100
                }
              },
              required: ['durationMs']
            }
          },
          {
            name: 'quantum_container_create',
            description: 'Create a quantum container for quantum-classical hybrid computing',
            inputSchema: {
              type: 'object',
              properties: {
                qubits: {
                  type: 'number',
                  description: 'Number of qubits',
                  default: 3,
                  minimum: 1,
                  maximum: 16
                }
              }
            }
          },
          {
            name: 'quantum_superposition',
            description: 'Create quantum superposition across all states',
            inputSchema: {
              type: 'object',
              properties: {
                qubits: {
                  type: 'number',
                  description: 'Number of qubits for superposition',
                  default: 3
                }
              }
            }
          },
          {
            name: 'quantum_measure',
            description: 'Measure quantum state (collapses superposition)',
            inputSchema: {
              type: 'object',
              properties: {
                qubits: {
                  type: 'number',
                  description: 'Number of qubits in system',
                  default: 3
                }
              }
            }
          },
          {
            name: 'temporal_predictor_create',
            description: 'Create temporal predictor for future state prediction',
            inputSchema: {
              type: 'object',
              properties: {
                horizonNs: {
                  type: 'number',
                  description: 'Prediction horizon in nanoseconds',
                  default: 10000000
                },
                historySize: {
                  type: 'number',
                  description: 'History buffer size',
                  default: 500
                }
              }
            }
          },
          {
            name: 'temporal_predict',
            description: 'Predict future values based on current input',
            inputSchema: {
              type: 'object',
              properties: {
                currentValues: {
                  type: 'array',
                  items: { type: 'number' },
                  description: 'Current input values for prediction'
                },
                horizonNs: {
                  type: 'number',
                  description: 'Prediction horizon',
                  default: 10000000
                }
              },
              required: ['currentValues']
            }
          },
          {
            name: 'consciousness_evolve',
            description: 'Evolve temporal consciousness one step',
            inputSchema: {
              type: 'object',
              properties: {
                maxIterations: {
                  type: 'number',
                  description: 'Maximum evolution iterations',
                  default: 1000
                },
                enableQuantum: {
                  type: 'boolean',
                  description: 'Enable quantum integration',
                  default: true
                }
              }
            }
          },
          {
            name: 'system_info',
            description: 'Get Strange Loops system information and capabilities',
            inputSchema: {
              type: 'object',
              properties: {}
            }
          },
          {
            name: 'benchmark_run',
            description: 'Run comprehensive performance benchmark',
            inputSchema: {
              type: 'object',
              properties: {
                agentCount: {
                  type: 'number',
                  description: 'Number of agents for benchmark',
                  default: 1000
                },
                durationMs: {
                  type: 'number',
                  description: 'Benchmark duration in milliseconds',
                  default: 5000
                }
              }
            }
          }
        ] as Tool[]
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        // Initialize Strange Loop library if needed
        if (!this.isInitialized) {
          await StrangeLoop.init();
          this.isInitialized = true;
        }

        switch (name) {
          case 'nano_swarm_create': {
            const swarm = await StrangeLoop.createSwarm({
              agentCount: args?.agentCount || 1000,
              topology: args?.topology || 'mesh',
              tickDurationNs: args?.tickDurationNs || 25000
            });

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    swarm: {
                      agentCount: swarm.config.agentCount,
                      topology: swarm.config.topology,
                      tickDurationNs: swarm.config.tickDurationNs,
                      agents: swarm.agents.length
                    },
                    message: `Created nano-agent swarm with ${swarm.config.agentCount} agents`
                  }, null, 2)
                }
              ]
            };
          }

          case 'nano_swarm_run': {
            const swarm = await StrangeLoop.createSwarm({
              agentCount: 1000,
              topology: 'mesh'
            });

            const results = await swarm.run(args?.durationMs || 5000);

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    results: {
                      totalTicks: results.totalTicks,
                      agentCount: results.agentCount,
                      runtimeNs: results.runtimeNs,
                      ticksPerSecond: Math.round(results.ticksPerSecond),
                      budgetViolations: results.budgetViolations,
                      avgCyclesPerTick: Math.round(results.avgCyclesPerTick)
                    },
                    message: `Executed ${results.totalTicks} ticks at ${Math.round(results.ticksPerSecond)} ticks/sec`
                  }, null, 2)
                }
              ]
            };
          }

          case 'quantum_container_create': {
            const quantum = await StrangeLoop.createQuantumContainer(args?.qubits || 3);

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    quantum: {
                      qubits: quantum.qubits,
                      states: quantum.states,
                      isInSuperposition: quantum.isInSuperposition
                    },
                    message: `Created quantum container with ${quantum.qubits} qubits (${quantum.states} states)`
                  }, null, 2)
                }
              ]
            };
          }

          case 'quantum_superposition': {
            const quantum = await StrangeLoop.createQuantumContainer(args?.qubits || 3);
            await quantum.createSuperposition();

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    quantum: {
                      qubits: quantum.qubits,
                      states: quantum.states,
                      isInSuperposition: quantum.isInSuperposition
                    },
                    message: `Created superposition across ${quantum.states} quantum states`
                  }, null, 2)
                }
              ]
            };
          }

          case 'quantum_measure': {
            const quantum = await StrangeLoop.createQuantumContainer(args?.qubits || 3);
            await quantum.createSuperposition();
            const measurement = await quantum.measure();

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    measurement: {
                      result: measurement,
                      qubits: quantum.qubits,
                      collapsedState: measurement,
                      isInSuperposition: quantum.isInSuperposition
                    },
                    message: `Quantum measurement collapsed to state ${measurement}`
                  }, null, 2)
                }
              ]
            };
          }

          case 'temporal_predictor_create': {
            const predictor = await StrangeLoop.createTemporalPredictor({
              horizonNs: args?.horizonNs || 10000000,
              historySize: args?.historySize || 500
            });

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    predictor: {
                      horizonNs: predictor.horizonNs,
                      historySize: predictor.historySize,
                      currentHistory: predictor.history.length
                    },
                    message: `Created temporal predictor with ${predictor.horizonNs}ns horizon`
                  }, null, 2)
                }
              ]
            };
          }

          case 'temporal_predict': {
            const predictor = await StrangeLoop.createTemporalPredictor({
              horizonNs: args?.horizonNs || 10000000,
              historySize: 100
            });

            const currentValues = args?.currentValues || [1.0, 2.0, 3.0];
            const prediction = await predictor.predict(currentValues);

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    prediction: {
                      input: currentValues,
                      predicted: prediction,
                      horizonNs: predictor.horizonNs
                    },
                    message: `Predicted future values with ${predictor.horizonNs/1000000}ms temporal lead`
                  }, null, 2)
                }
              ]
            };
          }

          case 'consciousness_evolve': {
            const consciousness = await StrangeLoop.createTemporalConsciousness({
              maxIterations: args?.maxIterations || 1000,
              enableQuantum: args?.enableQuantum !== false
            });

            const state = await consciousness.evolveStep();

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    consciousness: {
                      iteration: state.iteration,
                      consciousnessIndex: state.consciousnessIndex,
                      temporalPatterns: state.temporalPatterns,
                      quantumInfluence: state.quantumInfluence
                    },
                    message: `Consciousness evolved to iteration ${state.iteration} with index ${state.consciousnessIndex.toFixed(3)}`
                  }, null, 2)
                }
              ]
            };
          }

          case 'system_info': {
            const info = await StrangeLoop.getSystemInfo();

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    system: info,
                    message: 'Strange Loops system information retrieved'
                  }, null, 2)
                }
              ]
            };
          }

          case 'benchmark_run': {
            const results = await StrangeLoop.runBenchmark({
              agentCount: args?.agentCount || 1000,
              duration: args?.durationMs || 5000
            });

            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: true,
                    benchmark: {
                      totalTicks: results.totalTicks,
                      agentCount: results.agentCount,
                      runtimeNs: results.runtimeNs,
                      ticksPerSecond: Math.round(results.ticksPerSecond),
                      budgetViolations: results.budgetViolations,
                      performanceRating: results.ticksPerSecond > 500000 ? 'Excellent' :
                                       results.ticksPerSecond > 250000 ? 'Good' : 'Fair'
                    },
                    message: `Benchmark completed: ${Math.round(results.ticksPerSecond)} ticks/sec`
                  }, null, 2)
                }
              ]
            };
          }

          default:
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({
                    success: false,
                    error: `Unknown tool: ${name}`,
                    availableTools: [
                      'nano_swarm_create', 'nano_swarm_run', 'quantum_container_create',
                      'quantum_superposition', 'quantum_measure', 'temporal_predictor_create',
                      'temporal_predict', 'consciousness_evolve', 'system_info', 'benchmark_run'
                    ]
                  }, null, 2)
                }
              ]
            };
        }
      } catch (error) {
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
                tool: name,
                arguments: args
              }, null, 2)
            }
          ]
        };
      }
    });
  }

  async start(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('Strange Loops MCP Server started');
  }
}

// Start the server
const server = new StrangeLoopsMCPServer();
server.start().catch((error) => {
  console.error('Failed to start Strange Loops MCP Server:', error);
  process.exit(1);
});