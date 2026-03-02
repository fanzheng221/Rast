import { createUnplugin } from 'unplugin';

import { analyzeSource, formatIssuesComment } from './analysis';
import { cacheSourceIfNeeded, ensureProjectGraph } from './graph';
import type { RastPluginOptions } from './types';

function shouldAnalyze(id: string): boolean {
  return id.endsWith('.ts') || id.endsWith('.js') || id.endsWith('.tsx') || id.endsWith('.jsx');
}

export const rastUnplugin = createUnplugin((options: RastPluginOptions = {}) => {
  const { injectIssues = false, logIssues = true, mode = 'on-demand' } = options;
  ensureProjectGraph(mode);

  return {
    name: 'rast-unplugin',
    enforce: 'post',
    transform(code, id) {
      if (!shouldAnalyze(id)) {
        return undefined;
      }

      try {
        cacheSourceIfNeeded(mode, id, code);
        const result = analyzeSource(code);

        if (logIssues && result.issues && result.issues.length > 0) {
          console.warn(`[rast] Found ${result.issues.length} issues in ${id}`);
          result.issues.forEach((issue) => {
            console.warn(`  - ${issue.message}`);
          });
        }

        if (injectIssues) {
          const issuesComment = formatIssuesComment(result);
          if (issuesComment) {
            return {
              code: code + issuesComment,
              map: null,
            };
          }
        }

        return { code, map: null };
      } catch (e) {
        console.error(`[rast] Failed to analyze ${id}:`, e);
        return { code, map: null };
      }
    },
  };
});
