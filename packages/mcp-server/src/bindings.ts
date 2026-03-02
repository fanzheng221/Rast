import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

export type ProjectGraph = {
  add_file: (path: string, code: string) => void;
  get_file_structure: (path: string) => string | null | undefined;
  get_symbol_details: (symbol: string) => unknown;
  analyze_dependencies: (paths: string[]) => string;
};

type Bindings = {
  find_pattern: (source: string, pattern: string) => string;
  apply_rule: (source: string, rule: string) => string;
  scan_directory: (rootPath: string, rule: string, dryRun: boolean) => string;
  analyze_ast: (source: string) => string;
  initialize_graph: (mode: string) => ProjectGraph;
};

let bindings: Bindings | undefined;
export let projectGraph: ProjectGraph | undefined;

export async function getBindings(): Promise<Bindings> {
  if (!bindings) {
    bindings = require('@rust_ast/bindings') as Bindings;
  }
  return bindings;
}

export async function getProjectGraph(): Promise<ProjectGraph> {
  if (!projectGraph) {
    const loadedBindings = await getBindings();
    projectGraph = loadedBindings.initialize_graph('on-demand');
  }
  return projectGraph;
}

export function resolveBindingMethod<T extends (...args: never[]) => string>(
  loadedBindings: Bindings,
  name: keyof Bindings
): T {
  const method = loadedBindings[name];
  if (typeof method === 'function') {
    return method as unknown as T;
  }
  throw new Error(`Missing NAPI binding method: ${String(name)}`);
}
