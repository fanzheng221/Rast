import * as bindings from '@rust-ast/bindings';

import type { AnalysisResult } from './types';

export function analyzeSource(code: string): AnalysisResult {
  const analyze = (bindings as { analyze_ast?: (input: string) => string; analyzeAst?: (input: string) => string });
  const analyzeAst = analyze.analyze_ast ?? analyze.analyzeAst;
  if (!analyzeAst) {
    throw new Error('Missing bindings analyze function (expected analyze_ast)');
  }
  const resultStr = analyzeAst(code);
  return JSON.parse(resultStr) as AnalysisResult;
}

export function formatIssuesComment(result: AnalysisResult): string | null {
  if (!result.issues || result.issues.length === 0) {
    return null;
  }

  const details = result.issues.map((issue) => ` * - ${issue.message}`).join('\n');
  return `\n/* Rast Issues:\n${details}\n */`;
}
