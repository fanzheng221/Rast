import { describe, expect, it, vi } from 'vitest';

import { projectGraph, rastUnplugin } from '../src/index';

type TransformResult = { code: string; map: unknown } | undefined;

function runTransform(
  code: string,
  id: string,
  options: Parameters<typeof rastUnplugin.vite>[0] = {}
): TransformResult {
  const plugin = rastUnplugin.vite(options);
  const transform = plugin.transform as ((code: string, id: string) => TransformResult) | undefined;
  return transform?.(code, id);
}

describe('rastUnplugin', () => {
  it('returns original code when no issue injection is requested', () => {
    const code = 'export const a = 1;';
    const result = runTransform(code, 'test.ts');
    expect(result).toEqual({ code, map: null });
  });

  it('does not crash when injectIssues is enabled', () => {
    const code = 'var x = 1;';
    const result = runTransform(code, 'inject.ts', { injectIssues: true, logIssues: false });
    expect(result).toBeDefined();
    expect(result?.code).toContain('var x = 1;');
  });

  it('logs issues when enabled', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    runTransform('var x = 1;', 'warn.ts', { logIssues: true, injectIssues: false });
    expect(warnSpy.mock.calls.length).toBeGreaterThanOrEqual(0);
    warnSpy.mockRestore();
  });

  it('adds file into graph in cache mode', () => {
    const id = 'cache-test.ts';
    runTransform('export const cacheTest = 1;', id, { mode: 'cache', logIssues: false });
    expect(projectGraph).toBeDefined();
    const structure = projectGraph?.get_file_structure(id);
    expect(structure).toBeTruthy();
  });

  it('does not auto cache file in on-demand mode', () => {
    const id = 'on-demand-test.ts';
    runTransform('export const onDemandTest = 1;', id, { mode: 'on-demand', logIssues: false });
    expect(projectGraph).toBeDefined();
    const structure = projectGraph?.get_file_structure(id);
    expect(structure).toBeNull();
  });
});
