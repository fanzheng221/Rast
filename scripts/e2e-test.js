const { analyzeAst } = require('../packages/bindings');
const path = require('node:path');
const { rastUnplugin } = require('../packages/unplugin/dist/index.cjs');

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function getTextContent(result, context) {
  assert(result && Array.isArray(result.content), `${context}: missing content array`);
  assert(result.content[0] && typeof result.content[0].text === 'string', `${context}: missing text payload`);
  return result.content[0].text;
}

async function runCodebaseOracleWorkflowChecks() {
  const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');
  const mcpModule = await import(mcpPath);
  const { callTool, projectGraph, tools } = mcpModule;

  assert(Array.isArray(tools), 'Test 4 failed: tools should be an array');
  const toolNames = new Set(tools.map((tool) => tool.name));
  assert(toolNames.has('get_file_structure'), 'Test 4 failed: get_file_structure tool missing');
  assert(toolNames.has('get_symbol_details'), 'Test 4 failed: get_symbol_details tool missing');
  assert(toolNames.has('analyze_dependencies'), 'Test 4 failed: analyze_dependencies tool missing');

  const modelPath = 'src/models/user.ts';
  const servicePath = 'src/services/user-service.ts';
  const modelCode = [
    'export interface User {',
    '  id: string;',
    '  name: string;',
    '}',
    '',
    'export const userSeed: User = { id: "1", name: "Ada" };',
  ].join('\n');
  const serviceCode = [
    'import { userSeed, type User } from "../models/user";',
    '',
    '/** Build display name for UI */',
    'export function buildDisplayName(user: User): string {',
    '  return user.name + "#" + user.id;',
    '}',
    '',
    'export const defaultDisplayName = buildDisplayName(userSeed);',
  ].join('\n');

  projectGraph.add_file(modelPath, modelCode);
  projectGraph.add_file(servicePath, serviceCode);

  const structureText = callTool('get_file_structure', { path: servicePath });
  assert(structureText && structureText !== 'null', 'Test 4 failed: get_file_structure returned null for known file');
  const structure = JSON.parse(structureText);
  assert(Array.isArray(structure.imports), 'Test 4 failed: file structure imports should be an array');
  assert(Array.isArray(structure.exports), 'Test 4 failed: file structure exports should be an array');
  assert(
    structure.exports.some((symbol) => symbol.name === 'buildDisplayName'),
    'Test 4 failed: expected buildDisplayName export in file structure'
  );

  const symbolDetailsText = callTool('get_symbol_details', { symbol: 'buildDisplayName' });
  const symbolDetails = JSON.parse(symbolDetailsText);
  assert(Array.isArray(symbolDetails), 'Test 4 failed: symbol details should be an array');
  assert(symbolDetails.length > 0, 'Test 4 failed: expected at least one symbol detail');
  const firstSymbol = JSON.parse(symbolDetails[0]);
  assert(firstSymbol.name === 'buildDisplayName', 'Test 4 failed: incorrect symbol returned');

  const dependenciesText = callTool('analyze_dependencies', { paths: [servicePath] });
  const dependencies = JSON.parse(dependenciesText);
  assert(Array.isArray(dependencies), 'Test 4 failed: dependencies should be an array');
  assert(Array.isArray(dependencies[0]), 'Test 4 failed: dependencies entry should be tuple-like array');
  assert(dependencies[0][0] === servicePath, 'Test 4 failed: dependencies should preserve requested file path');
  assert(Array.isArray(dependencies[0][1]), 'Test 4 failed: dependencies list should be array');
  assert(
    dependencies[0][1].some((dep) => dep.source === modelPath),
    'Test 4 failed: expected dependency on src/models/user.ts'
  );

  let invalidArgThrows = false;
  try {
    callTool('get_symbol_details', {});
  } catch (error) {
    invalidArgThrows = /symbol argument/.test(String(error));
  }
  assert(invalidArgThrows, 'Test 4 failed: get_symbol_details should validate missing symbol argument');
}

async function runMcpStdioCommunicationChecks() {
  const { Client } = await import('@modelcontextprotocol/sdk/client/index.js');
  const { StdioClientTransport } = await import('@modelcontextprotocol/sdk/client/stdio.js');
  const mcpPath = path.resolve(__dirname, '../packages/mcp-server/dist/index.js');

  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [mcpPath],
  });
  const client = new Client({ name: 'rast-e2e-client', version: '0.1.0' }, { capabilities: {} });

  await client.connect(transport);
  try {
    const { tools } = await client.listTools();
    const toolNames = new Set(tools.map((tool) => tool.name));
    assert(toolNames.has('get_file_structure'), 'Test 4 failed: stdio listTools missing get_file_structure');
    assert(toolNames.has('get_symbol_details'), 'Test 4 failed: stdio listTools missing get_symbol_details');
    assert(toolNames.has('analyze_dependencies'), 'Test 4 failed: stdio listTools missing analyze_dependencies');

    const astResult = await client.callTool({
      name: 'analyze_ast',
      arguments: { source: 'export const viaMcp = 1;' },
    });
    const astText = getTextContent(astResult, 'Test 4 analyze_ast');
    const parsedAst = JSON.parse(astText);
    assert(Array.isArray(parsedAst.exports), 'Test 4 failed: stdio analyze_ast should return exports array');

    const emptyStructureResult = await client.callTool({
      name: 'get_file_structure',
      arguments: { path: 'src/unknown.ts' },
    });
    const emptyStructureText = getTextContent(emptyStructureResult, 'Test 4 get_file_structure');
    assert(emptyStructureText === 'null', 'Test 4 failed: unknown path should return null from MCP server');
  } finally {
    await client.close();
  }
}

async function runTests() {
  try {
    // Test 1: Direct bindings call
    console.log('Test 1: Direct bindings call...');
    const result1 = analyzeAst('export const x = 1;');
    const parsed1 = JSON.parse(result1);
    if (!parsed1.exports || !Array.isArray(parsed1.exports)) {
      throw new Error('Test 1 failed: exports should be an array');
    }
    console.log('✓ Test 1 passed');

    // Test 2: With issues
    console.log('Test 2: Code with linting issues...');
    const result2 = analyzeAst('var x = 1;');
    const parsed2 = JSON.parse(result2);
    if (!parsed2.issues || !Array.isArray(parsed2.issues)) {
      throw new Error('Test 2 failed: issues should be an array');
    }
    if (parsed2.issues.length === 0) {
      throw new Error('Test 2 failed: expected at least one issue');
    }
    console.log('✓ Test 2 passed');

    // Test 3: Unplugin
    console.log('Test 3: Unplugin transform...');
    const plugin = rastUnplugin.vite({ injectIssues: true, logIssues: false });
    const transformResult = plugin.transform('var x = 1;', 'test.js');
    if (!transformResult || !transformResult.code) {
      throw new Error('Test 3 failed: transform should return code');
    }
    if (!transformResult.code.includes('Rast Issues:')) {
      throw new Error('Test 3 failed: transform should inject issues comment');
    }
    console.log('✓ Test 3 passed');

    console.log('Test 4: MCP server communication and Codebase Oracle workflow...');
    await runCodebaseOracleWorkflowChecks();
    console.log('  - Codebase Oracle tool checks passed');
    await runMcpStdioCommunicationChecks();
    console.log('  - MCP stdio communication checks passed');
    console.log('✓ Test 4 passed');

    console.log('All E2E tests passed!');
    process.exit(0);
  } catch (error) {
    console.error('E2E test failed:', error);
    process.exit(1);
  }
}

runTests();
