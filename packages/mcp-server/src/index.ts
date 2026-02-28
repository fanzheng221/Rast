import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

const require = createRequire(import.meta.url);

type ProjectGraph = {
  get_file_structure: (path: string) => string | null | undefined;
  get_symbol_details: (symbol: string) => unknown;
  analyze_dependencies: (paths: string[]) => string;
};

type Bindings = {
  findPattern?: (source: string, pattern: string) => string;
  find_pattern?: (source: string, pattern: string) => string;
  applyRule?: (source: string, rule: string) => string;
  apply_rule?: (source: string, rule: string) => string;
  scanDirectory?: (rootPath: string, rule: string, dryRun: boolean) => string;
  scan_directory?: (rootPath: string, rule: string, dryRun: boolean) => string;
  analyzeAst?: (source: string) => string;
  analyze_ast?: (source: string) => string;
  initialize_graph: (mode: string) => ProjectGraph;
};

let bindings: Bindings | undefined;
export let projectGraph: ProjectGraph | undefined;

async function getBindings() {
  if (!bindings) {
    bindings = require('@rast/bindings') as Bindings;
  }
  return bindings;
}

async function getProjectGraph() {
  if (!projectGraph) {
    const loadedBindings = await getBindings();
    projectGraph = loadedBindings.initialize_graph('on-demand');
  }
  return projectGraph;
}

function resolveBindingMethod<T extends (...args: never[]) => string>(
  loadedBindings: Bindings,
  names: (keyof Bindings)[]
): T {
  for (const name of names) {
    const method = loadedBindings[name];
    if (typeof method === 'function') {
      return method as unknown as T;
    }
  }
  throw new Error(`Missing NAPI binding method: ${names.join(' or ')}`);
}

export const tools = [
  {
    name: 'analyze_ast',
    description:
      'Analyze JavaScript/TypeScript source code and extract AST information',
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
    description:
      'Get signatures, exports, imports and JSDoc for a file from project graph',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Absolute or project-relative file path',
        },
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
  {
    name: 'findPattern',
    description: 'Find all pattern matches in source code with captured metavariables',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Source code to search for patterns' },
        pattern: {
          type: 'string',
          description:
            'Pattern to search for (supports metavariables like $A, $$$A)',
        },
      },
      required: ['source', 'pattern'],
    },
  },
  {
    name: 'applyRule',
    description: 'Apply a YAML rule to source code and return modified code',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Source code to modify' },
        rule: {
          type: 'string',
          description: 'YAML rule (file path or inline YAML string)',
        },
      },
      required: ['source', 'rule'],
    },
  },
  {
    name: 'scanDirectory',
    description: 'Apply a YAML rule to all files in a directory',
    inputSchema: {
      type: 'object',
      properties: {
        rootPath: { type: 'string', description: 'Root directory to scan' },
        rule: {
          type: 'string',
          description: 'YAML rule (file path or inline YAML string)',
        },
        dryRun: {
          type: 'boolean',
          description: 'Show what would change without modifying files',
        },
      },
      required: ['rootPath', 'rule', 'dryRun'],
    },
  },
] as const;

type ToolName =
  | 'analyze_ast'
  | 'get_file_structure'
  | 'get_symbol_details'
  | 'analyze_dependencies'
  | 'findPattern'
  | 'applyRule'
  | 'scanDirectory'
  | 'find_pattern'
  | 'apply_rule'
  | 'scan_directory';

export async function callTool(
  name: ToolName,
  args: Record<string, unknown> | undefined
): Promise<string> {
  switch (name) {
    case 'findPattern':
    case 'find_pattern': {
      const source = args?.source as string;
      const pattern = args?.pattern as string;
      if (typeof source !== 'string') {
        throw new Error('source argument is required and must be a string');
      }
      if (typeof pattern !== 'string') {
        throw new Error('pattern argument is required and must be a string');
      }
      const loadedBindings = await getBindings();
      const findPattern = resolveBindingMethod<
        (source: string, pattern: string) => string
      >(loadedBindings, ['findPattern', 'find_pattern']);
      return findPattern(source, pattern);
    }
    case 'applyRule':
    case 'apply_rule': {
      const source = args?.source as string;
      const rule = args?.rule as string;
      if (typeof source !== 'string') {
        throw new Error('source argument is required and must be a string');
      }
      if (typeof rule !== 'string') {
        throw new Error('rule argument is required and must be a string');
      }
      const loadedBindings = await getBindings();
      const applyRule = resolveBindingMethod<
        (source: string, rule: string) => string
      >(loadedBindings, ['applyRule', 'apply_rule']);
      return applyRule(source, rule);
    }
    case 'scanDirectory':
    case 'scan_directory': {
      const rootPath = args?.rootPath as string;
      const rule = args?.rule as string;
      const dryRun = (args?.dryRun as boolean) ?? false;
      if (typeof rootPath !== 'string') {
        throw new Error('rootPath argument is required and must be a string');
      }
      if (typeof rule !== 'string') {
        throw new Error('rule argument is required and must be a string');
      }
      const loadedBindings = await getBindings();
      const scanDirectory = resolveBindingMethod<
        (rootPath: string, rule: string, dryRun: boolean) => string
      >(loadedBindings, ['scanDirectory', 'scan_directory']);
      return scanDirectory(rootPath, rule, dryRun);
    }
    case 'analyze_ast': {
      const source = args?.source as string;
      if (typeof source !== 'string') {
        throw new Error('source argument is required and must be a string');
      }
      const loadedBindings = await getBindings();
      const analyzeAst = resolveBindingMethod<(source: string) => string>(
        loadedBindings,
        ['analyzeAst', 'analyze_ast']
      );
      return analyzeAst(source);
    }
    case 'get_file_structure': {
      const path = args?.path as string;
      if (typeof path !== 'string') {
        throw new Error('path argument is required and must be a string');
      }
      const graph = await getProjectGraph();
      return graph.get_file_structure(path) ?? 'null';
    }
    case 'get_symbol_details': {
      const symbol = args?.symbol as string;
      if (typeof symbol !== 'string') {
        throw new Error('symbol argument is required and must be a string');
      }
      const graph = await getProjectGraph();
      return JSON.stringify(graph.get_symbol_details(symbol));
    }
    case 'analyze_dependencies': {
      const paths = args?.paths;
      if (!Array.isArray(paths) || !paths.every((item) => typeof item === 'string')) {
        throw new Error('paths argument is required and must be an array of strings');
      }
      const graph = await getProjectGraph();
      return graph.analyze_dependencies(paths);
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
  const text = await callTool(
    name,
    request.params.arguments as Record<string, unknown> | undefined
  );
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
