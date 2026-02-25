import { pathToFileURL } from 'node:url';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { analyzeAst, initialize_graph } from '@rast/bindings';

export const projectGraph = initialize_graph('on-demand');

export const tools = [
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
  {
    name: 'get_file_structure',
    description: 'Get signatures, exports, imports and JSDoc for a file from project graph',
    inputSchema: {
      type: 'object',
      properties: {
        path: { type: 'string', description: 'Absolute or project-relative file path' },
      },
      required: ['path'],
    },
  },
  {
    name: 'get_symbol_details',
    description: 'Get full symbol implementation details and call context by symbol name',
    inputSchema: {
      type: 'object',
      properties: {
        symbol: { type: 'string', description: 'Symbol name to query' },
      },
      required: ['symbol'],
    },
  },
  {
    name: 'analyze_dependencies',
    description: 'Analyze call graph and module dependencies for a list of files',
    inputSchema: {
      type: 'object',
      properties: {
        paths: {
          type: 'array',
          items: { type: 'string' },
          description: 'File paths to analyze',
        },
      },
      required: ['paths'],
    },
  },
] as const;

type ToolName = (typeof tools)[number]['name'];

export function callTool(name: ToolName, args: Record<string, unknown> | undefined): string {
  switch (name) {
    case 'analyze_ast': {
      const source = args?.source as string;
      if (typeof source !== 'string') {
        throw new Error('source argument is required and must be a string');
      }
      return analyzeAst(source);
    }
    case 'get_file_structure': {
      const path = args?.path as string;
      if (typeof path !== 'string') {
        throw new Error('path argument is required and must be a string');
      }
      return projectGraph.get_file_structure(path) ?? 'null';
    }
    case 'get_symbol_details': {
      const symbol = args?.symbol as string;
      if (typeof symbol !== 'string') {
        throw new Error('symbol argument is required and must be a string');
      }
      return JSON.stringify(projectGraph.get_symbol_details(symbol));
    }
    case 'analyze_dependencies': {
      const paths = args?.paths;
      if (!Array.isArray(paths) || !paths.every((path) => typeof path === 'string')) {
        throw new Error('paths argument is required and must be an array of strings');
      }
      return projectGraph.analyze_dependencies(paths);
    }
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
}

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
    tools,
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const name = request.params.name as ToolName;
  const text = callTool(name, request.params.arguments as Record<string, unknown> | undefined);
  return {
    content: [{ type: 'text', text }],
  };
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(console.error);
}
