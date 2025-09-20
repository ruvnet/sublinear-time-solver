import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { AppConfig } from '../types/config.js';
import { Logger } from '../utils/logger.js';
import { createServer } from 'http';
import express from 'express';
import cors from 'cors';
import WebSocket from 'ws';

/**
 * MCP Server implementation with multiple transport support
 */
export class MCPServer {
  private server: Server;
  private config: AppConfig;
  private httpServer?: any;
  private wsServer?: WebSocket.Server;
  private isRunning: boolean = false;

  constructor(config: AppConfig) {
    this.config = config;
    this.server = new Server(
      {
        name: 'psycho-symbolic-reasoner',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
          resources: {},
          prompts: {},
        },
      }
    );

    this.setupHandlers();
  }

  /**
   * Setup MCP handlers
   */
  private setupHandlers(): void {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: [
          {
            name: 'reason',
            description: 'Perform psycho-symbolic reasoning on a given query',
            inputSchema: {
              type: 'object',
              properties: {
                query: { type: 'string', description: 'The reasoning query' },
                context: { type: 'object', description: 'Additional context for reasoning' },
                depth: { type: 'number', description: 'Maximum reasoning depth', default: 5 }
              },
              required: ['query']
            }
          },
          {
            name: 'knowledge_graph_query',
            description: 'Query the knowledge graph',
            inputSchema: {
              type: 'object',
              properties: {
                query: { type: 'string', description: 'Graph query in natural language' },
                filters: { type: 'object', description: 'Query filters' },
                limit: { type: 'number', description: 'Maximum results', default: 10 }
              },
              required: ['query']
            }
          },
          {
            name: 'add_knowledge',
            description: 'Add new knowledge to the graph',
            inputSchema: {
              type: 'object',
              properties: {
                subject: { type: 'string', description: 'Subject entity' },
                predicate: { type: 'string', description: 'Relationship type' },
                object: { type: 'string', description: 'Object entity' },
                metadata: { type: 'object', description: 'Additional metadata' }
              },
              required: ['subject', 'predicate', 'object']
            }
          },
          {
            name: 'analyze_reasoning_path',
            description: 'Analyze and explain a reasoning path',
            inputSchema: {
              type: 'object',
              properties: {
                query: { type: 'string', description: 'Original query' },
                showSteps: { type: 'boolean', description: 'Show detailed steps', default: true },
                includeConfidence: { type: 'boolean', description: 'Include confidence scores', default: true }
              },
              required: ['query']
            }
          },
          {
            name: 'health_check',
            description: 'Check server health and status',
            inputSchema: {
              type: 'object',
              properties: {
                detailed: { type: 'boolean', description: 'Include detailed metrics', default: false }
              }
            }
          }
        ]
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'reason':
            return await this.handleReason(args);
          case 'knowledge_graph_query':
            return await this.handleKnowledgeGraphQuery(args);
          case 'add_knowledge':
            return await this.handleAddKnowledge(args);
          case 'analyze_reasoning_path':
            return await this.handleAnalyzeReasoningPath(args);
          case 'health_check':
            return await this.handleHealthCheck(args);
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        Logger.error(`Tool execution failed: ${name}`, error);
        return {
          content: [
            {
              type: 'text',
              text: `Error executing tool '${name}': ${error instanceof Error ? error.message : 'Unknown error'}`
            }
          ]
        };
      }
    });
  }

  /**
   * Handle reasoning tool
   */
  private async handleReason(args: any): Promise<any> {
    const { query, depth = 5 } = args;

    Logger.info('Processing reasoning request', { query, depth });

    // TODO: Integrate with actual reasoning engine
    const reasoningResult = {
      query,
      result: 'Reasoning result placeholder - integrate with Rust backend',
      confidence: 0.85,
      steps: [
        { step: 1, description: 'Parse query', confidence: 0.95 },
        { step: 2, description: 'Search knowledge graph', confidence: 0.90 },
        { step: 3, description: 'Apply inference rules', confidence: 0.80 },
        { step: 4, description: 'Generate conclusion', confidence: 0.85 }
      ],
      metadata: {
        depth_used: depth,
        processing_time_ms: 150,
        nodes_explored: 42
      }
    };

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(reasoningResult, null, 2)
        }
      ]
    };
  }

  /**
   * Handle knowledge graph query
   */
  private async handleKnowledgeGraphQuery(args: any): Promise<any> {
    const { query, filters = {}, limit = 10 } = args;

    Logger.info('Processing knowledge graph query', { query, limit });

    // TODO: Integrate with actual graph query engine
    const queryResult = {
      query,
      results: [
        { id: '1', type: 'entity', label: 'Sample Entity', confidence: 0.90 },
        { id: '2', type: 'relationship', label: 'Sample Relationship', confidence: 0.85 }
      ],
      total: 2,
      metadata: {
        query_time_ms: 45,
        filters_applied: Object.keys(filters).length
      }
    };

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(queryResult, null, 2)
        }
      ]
    };
  }

  /**
   * Handle add knowledge
   */
  private async handleAddKnowledge(args: any): Promise<any> {
    const { subject, predicate, object, metadata = {} } = args;

    Logger.info('Adding knowledge to graph', { subject, predicate, object });

    // TODO: Integrate with actual knowledge graph
    const addResult = {
      success: true,
      triple: { subject, predicate, object },
      id: `triple_${Date.now()}`,
      metadata
    };

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(addResult, null, 2)
        }
      ]
    };
  }

  /**
   * Handle analyze reasoning path
   */
  private async handleAnalyzeReasoningPath(args: any): Promise<any> {
    const { query, showSteps = true } = args;

    Logger.info('Analyzing reasoning path', { query });

    // TODO: Integrate with actual reasoning path analyzer
    const analysisResult = {
      query,
      path_analysis: {
        total_steps: 4,
        avg_confidence: 0.875,
        bottlenecks: ['inference_rules'],
        strengths: ['knowledge_coverage', 'logical_consistency']
      },
      suggestions: [
        'Consider adding more domain-specific rules',
        'Expand knowledge base in weak areas'
      ]
    };

    if (showSteps) {
      (analysisResult.path_analysis as any).detailed_steps = [
        { step: 1, description: 'Query parsing', confidence: 0.95, duration_ms: 15 },
        { step: 2, description: 'Graph traversal', confidence: 0.90, duration_ms: 80 },
        { step: 3, description: 'Rule application', confidence: 0.80, duration_ms: 45 },
        { step: 4, description: 'Result synthesis', confidence: 0.85, duration_ms: 20 }
      ];
    }

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(analysisResult, null, 2)
        }
      ]
    };
  }

  /**
   * Handle health check
   */
  private async handleHealthCheck(args: any): Promise<any> {
    const { detailed = false } = args;

    const healthData = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      uptime_seconds: process.uptime(),
      memory_usage: process.memoryUsage(),
      server_info: {
        transport: this.config.server.transport,
        port: this.config.server.port,
        host: this.config.server.host
      }
    };

    if (detailed) {
      (healthData as any).detailed_metrics = {
        cpu_usage: process.cpuUsage(),
        event_loop_lag: 0, // TODO: implement actual measurement
        active_connections: this.wsServer?.clients?.size || 0,
        knowledge_graph_stats: {
          nodes: 1234, // TODO: get from actual graph
          edges: 5678,
          last_update: new Date().toISOString()
        }
      };
    }

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(healthData, null, 2)
        }
      ]
    };
  }

  /**
   * Start the MCP server with configured transport
   */
  public async start(): Promise<void> {
    try {
      Logger.info(`Starting MCP server with ${this.config.server.transport} transport`);

      switch (this.config.server.transport) {
        case 'stdio':
          await this.startStdioTransport();
          break;
        case 'http':
          await this.startHttpTransport();
          break;
        case 'sse':
          await this.startSSETransport();
          break;
        default:
          throw new Error(`Unsupported transport: ${this.config.server.transport}`);
      }

      this.isRunning = true;
      Logger.info('MCP server started successfully');
    } catch (error) {
      Logger.error('Failed to start MCP server', error);
      throw error;
    }
  }

  /**
   * Start STDIO transport
   */
  private async startStdioTransport(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
  }

  /**
   * Start HTTP transport
   */
  private async startHttpTransport(): Promise<void> {
    const app = express();

    if (this.config.server.cors) {
      app.use(cors({
        origin: this.config.security.allowedOrigins,
        credentials: true
      }));
    }

    app.use(express.json({ limit: '10mb' }));

    // Health check endpoint
    app.get('/health', (_req, res) => {
      res.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        transport: 'http'
      });
    });

    // MCP endpoint
    app.post('/mcp', async (_req, res) => {
      try {
        // TODO: Implement HTTP MCP protocol handling
        res.json({ message: 'MCP HTTP transport not fully implemented yet' });
      } catch (error) {
        Logger.error('HTTP MCP request failed', error);
        res.status(500).json({ error: 'Internal server error' });
      }
    });

    this.httpServer = createServer(app);

    return new Promise((resolve, reject) => {
      this.httpServer.listen(this.config.server.port, this.config.server.host, () => {
        Logger.info(`HTTP server listening on ${this.config.server.host}:${this.config.server.port}`);
        resolve();
      });

      this.httpServer.on('error', reject);
    });
  }

  /**
   * Start SSE transport
   */
  private async startSSETransport(): Promise<void> {
    await this.startHttpTransport(); // SSE builds on HTTP

    // TODO: Implement SSE-specific MCP protocol handling
    Logger.info('SSE transport started (built on HTTP)');
  }

  /**
   * Stop the server gracefully
   */
  public async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    Logger.info('Stopping MCP server...');

    try {
      // Close WebSocket server
      if (this.wsServer) {
        this.wsServer.close();
      }

      // Close HTTP server
      if (this.httpServer) {
        await new Promise<void>((resolve) => {
          this.httpServer.close(() => resolve());
        });
      }

      this.isRunning = false;
      Logger.info('MCP server stopped successfully');
    } catch (error) {
      Logger.error('Error stopping MCP server', error);
      throw error;
    }
  }

  /**
   * Check if server is running
   */
  public isServerRunning(): boolean {
    return this.isRunning;
  }
}