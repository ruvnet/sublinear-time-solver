/**
 * GOAP MCP Server
 * Main Model Context Protocol server for GOAP planning system
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

import { GoapMCPTools } from './goap-tools.js';
import { PluginRegistry, PluginLoader, costTrackingPlugin, performanceMonitoringPlugin, loggingPlugin, queryDiversificationPlugin } from '../core/plugin-system.js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

export class GoapMCPServer {
  private server: Server;
  private goapTools: GoapMCPTools;
  private pluginRegistry: PluginRegistry;

  constructor() {
    this.server = new Server(
      {
        name: 'goalie-mcp-goap',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.goapTools = new GoapMCPTools();
    this.pluginRegistry = new PluginRegistry();

    this.setupHandlers();
  }

  async initialize(): Promise<void> {
    // Register built-in plugins
    this.pluginRegistry.register(costTrackingPlugin);
    this.pluginRegistry.register(performanceMonitoringPlugin);
    this.pluginRegistry.register(loggingPlugin);
    this.pluginRegistry.register(queryDiversificationPlugin);

    // Load external plugins if specified
    await this.loadExternalPlugins();

    // Initialize GOAP tools
    await this.goapTools.initialize();

    console.log('🚀 GOAP MCP Server initialized successfully');
    console.log(`📦 Registered plugins: ${this.pluginRegistry.getPlugins().length}`);
  }

  private async loadExternalPlugins(): Promise<void> {
    // Load plugins from environment variables
    const pluginPaths = process.env.GOAP_PLUGINS?.split(',').map(p => p.trim()) || [];
    const extensionPaths = process.env.GOAP_EXTENSIONS?.split(',').map(p => p.trim()) || [];

    try {
      if (pluginPaths.length > 0) {
        const plugins = await PluginLoader.loadFromFiles(pluginPaths);
        plugins.forEach(plugin => this.pluginRegistry.register(plugin));
        console.log(`📦 Loaded ${plugins.length} external plugins`);
      }

      if (extensionPaths.length > 0) {
        console.log(`📦 Loading ${extensionPaths.length} extensions (not implemented yet)`);
      }
    } catch (error) {
      console.warn('⚠️ Failed to load some external plugins:', error);
    }
  }

  private setupHandlers(): void {
    // List available tools
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      const tools = this.goapTools.getTools();

      return {
        tools: tools.map(tool => ({
          name: tool.name,
          description: tool.description,
          inputSchema: tool.inputSchema
        }))
      };
    });

    // Handle tool calls
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        let result;

        switch (name) {
          case 'goap.search':
            result = await this.goapTools.executeGoapSearch(args as any);
            break;

          case 'goap.plan.explain':
            result = await this.goapTools.executePlanExplain(args);
            break;

          case 'search.raw':
            result = await this.goapTools.executeRawSearch(args);
            break;

          default:
            throw new Error(`Unknown tool: ${name}`);
        }

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2)
            }
          ]
        };

      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                error: errorMessage,
                tool: name,
                timestamp: new Date().toISOString()
              }, null, 2)
            }
          ],
          isError: true
        };
      }
    });
  }

  async run(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);

    console.error('🎯 GOAP MCP Server running on stdio');
    console.error('⚡ Enhanced with Strange Loop WASM reasoning');
    console.error('🔌 Plugin system active');
    console.error('🎪 Ready for GOAP planning!');
  }
}

// Error handling
process.on('uncaughtException', (error) => {
  console.error('💥 Uncaught exception:', error);
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('💥 Unhandled rejection at:', promise, 'reason:', reason);
  process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.error('👋 Shutting down GOAP MCP Server...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.error('👋 Terminating GOAP MCP Server...');
  process.exit(0);
});