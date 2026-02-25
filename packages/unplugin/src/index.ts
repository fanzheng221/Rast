import { createUnplugin } from 'unplugin';
import { analyzeAst } from '@rast/bindings';

export interface RastPluginOptions {
  /**
   * Whether to inject linting issues as comments in the code
   * @default false
   */
  injectIssues?: boolean;
  /**
   * Whether to log issues to the console
   * @default true
   */
  logIssues?: boolean;
}

export const rastUnplugin = createUnplugin((options: RastPluginOptions = {}) => {
  const { injectIssues = false, logIssues = true } = options;

  return {
    name: 'rast-unplugin',
    enforce: 'post',
    transform(code, id) {
      if (id.endsWith('.ts') || id.endsWith('.js') || id.endsWith('.tsx') || id.endsWith('.jsx')) {
        try {
          const resultStr = analyzeAst(code);
          const result = JSON.parse(resultStr) as { issues?: { message: string }[] };

          if (logIssues && result.issues && result.issues.length > 0) {
            console.warn(`[rast] Found ${result.issues.length} issues in ${id}`);
            result.issues.forEach((issue: { message: string }) => {
              console.warn(`  - ${issue.message}`);
            });
          }

          if (injectIssues && result.issues && result.issues.length > 0) {
            const issuesComment = `\n/* Rast Issues:\n${result.issues.map((i: { message: string }) => ` * - ${i.message}`).join('\n')}\n */`;
            return {
              code: code + issuesComment,
              map: null
            };
          }

          return { code, map: null };
        } catch (e) {
          console.error(`[rast] Failed to analyze ${id}:`, e);
          return { code, map: null };
        }
      }
    },
  };
});

export default rastUnplugin;
