import assert from 'node:assert/strict';
import test from 'node:test';
import { callTool, projectGraph, tools } from '../src/index';

test('registers all expected MCP tools', () => {
  const names = tools.map((tool) => tool.name);
  assert.deepEqual(names, [
    'analyze_ast',
    'get_file_structure',
    'get_symbol_details',
    'analyze_dependencies',
  ]);
});

test('get_file_structure returns parsed file structure JSON text', () => {
  const path = 'mcp-server-test-file.ts';
  const code = 'export function demo() { return 1; }';
  projectGraph.add_file(path, code);

  const result = callTool('get_file_structure', { path });
  assert.ok(typeof result === 'string');
  assert.notEqual(result, 'null');

  const parsed = JSON.parse(result) as { exports?: string[]; signatures?: unknown[] };
  assert.ok(Array.isArray(parsed.exports));
  assert.ok(Array.isArray(parsed.signatures));
});

test('get_symbol_details returns JSON array text', () => {
  const path = 'mcp-server-symbol-test.ts';
  const code = 'export function toolSymbol() { return true; }';
  projectGraph.add_file(path, code);

  const result = callTool('get_symbol_details', { symbol: 'toolSymbol' });
  const parsed = JSON.parse(result);
  assert.ok(Array.isArray(parsed));
});

test('analyze_dependencies returns JSON text for target paths', () => {
  const targetPath = 'mcp-server-deps-test.ts';
  const code = "import { helper } from './helper'; export const value = helper();";
  projectGraph.add_file(targetPath, code);

  const result = callTool('analyze_dependencies', { paths: [targetPath] });
  assert.ok(typeof result === 'string');
  assert.doesNotThrow(() => JSON.parse(result));
});
