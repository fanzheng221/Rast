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
    name: 'find_pattern',
    description: 'Find all pattern matches in source code with captured metavariables',
    inputSchema: {
      type: 'object',
      properties: {
        source: { type: 'string', description: 'Source code to search for patterns' },
        pattern: {
          type: 'string',
          description: 'Pattern to search for (supports metavariables like $A, $$$A)',
        },
      },
      required: ['source', 'pattern'],
    },
  },
  {
    name: 'apply_rule',
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
    name: 'scan_directory',
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

export type ToolName = (typeof tools)[number]['name'];
