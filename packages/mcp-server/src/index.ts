import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { analyzeAst } from '@rast/bindings';

const server = new Server(
  {
    name: 'rast-mcp-server',
    version: '0.1.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'analyze_ast',
        description: 'Analyze JavaScript/TypeScript source code and extract AST information',
        inputSchema: {
          type: 'object',
          properties: {
            source: { type: 'string', description: 'Source code to analyze' },
          },
          required: ['source'],
        },
      },
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === 'analyze_ast') {
    const source = request.params.arguments?.source as string;
    if (typeof source !== 'string') {
      throw new Error('source argument is required and must be a string');
    }
    const result = analyzeAst(source);
    return {
      content: [{ type: 'text', text: result }],
    };
  }
  throw new Error(`Unknown tool: ${request.params.name}`);
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
