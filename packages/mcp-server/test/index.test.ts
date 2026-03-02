import { describe, expect, it } from 'vitest';
import { callTool, tools } from '../src/index.js';
import { getProjectGraph } from '../src/bindings.js';

describe('MCP server tools', () => {
  it('registers all expected MCP tools', () => {
    const names = tools.map((tool) => tool.name);
    expect(names).toEqual([
      'analyze_ast',
      'get_file_structure',
      'get_symbol_details',
      'analyze_dependencies',
      'find_pattern',
      'apply_rule',
      'scan_directory',
    ]);
  });

  it('get_file_structure returns parsed file structure JSON text', async () => {
    const path = 'mcp-server-test-file.ts';
    const code = 'export function demo() { return 1; }';
    const graph = await getProjectGraph();
    graph.add_file(path, code);

    const result = await callTool('get_file_structure', { path });
    expect(typeof result).toBe('string');
    expect(result).not.toBe('null');

    const parsed = JSON.parse(result) as { exports?: Array<{ name?: string }> };
    expect(Array.isArray(parsed.exports)).toBe(true);
  });

  it('find_pattern returns JSON array text', async () => {
    const result = await callTool('find_pattern', {
      source: 'const value = fn(answer, foo, bar);',
      pattern: 'const value = fn($A, $$$B);',
    });
    const parsed = JSON.parse(result) as Array<{ metavariables: Record<string, string> }>;
    expect(parsed.length).toBe(1);
    expect(parsed[0]?.metavariables?.A).toBe('answer');
  });

  it('apply_rule rewrites source text', async () => {
    const result = await callTool('apply_rule', {
      source: 'console.log(foo);',
      rule: `
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
`,
    });
    expect(result).toBe('logger.info(foo)');
  });
});
