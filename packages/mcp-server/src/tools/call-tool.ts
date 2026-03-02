import { getBindings, getProjectGraph, resolveBindingMethod } from '../bindings.js';
import type { ToolName } from './definitions.js';

export async function callTool(
  name: ToolName,
  args: Record<string, unknown> | undefined
): Promise<string> {
  switch (name) {
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
      >(loadedBindings, 'find_pattern');
      return findPattern(source, pattern);
    }
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
      >(loadedBindings, 'apply_rule');
      return applyRule(source, rule);
    }
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
      >(loadedBindings, 'scan_directory');
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
        'analyze_ast'
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
