import { pathToFileURL } from 'node:url';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

import { projectGraph } from './bindings.js';
import { server } from './server.js';
import { callTool } from './tools/call-tool.js';
import { tools } from './tools/definitions.js';

export { callTool, projectGraph, tools };

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(console.error);
}
