import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

import { callTool } from './tools/call-tool.js';
import { tools } from './tools/definitions.js';

export const server = new Server(
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
  const name = request.params.name as (typeof tools)[number]['name'];
  const text = await callTool(
    name,
    request.params.arguments as Record<string, unknown> | undefined
  );
  return {
    content: [{ type: 'text', text }],
  };
});
