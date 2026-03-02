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
  /**
   * Plugin mode.
   * - cache: intercepts all file reads and adds them to project graph
   * - on-demand: initializes graph but only adds files when queried
   * @default 'on-demand'
   */
  mode?: 'cache' | 'on-demand';
}

export type AnalysisIssue = {
  message: string;
};

export type AnalysisResult = {
  issues?: AnalysisIssue[];
};
