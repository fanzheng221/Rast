import { initialize_graph, ProjectGraph } from '@rast/bindings';

export let projectGraph: ProjectGraph | null = null;

export function ensureProjectGraph(mode: 'cache' | 'on-demand'): ProjectGraph {
  if (!projectGraph) {
    projectGraph = initialize_graph(mode);
  }
  return projectGraph;
}

export function cacheSourceIfNeeded(
  mode: 'cache' | 'on-demand',
  id: string,
  code: string
): void {
  if (mode !== 'cache') {
    return;
  }
  const graph = ensureProjectGraph(mode);
  graph.add_file(id, code);
}
