## 2026-02-25
- 当前环境缺少 `rust-analyzer`，`lsp_diagnostics` 初始化超时（Unknown binary 'rust-analyzer'），无法给出文件级 LSP clean 证明。
- `oxc` AST 类型较细粒度（如 `ModuleExportName`、`ImportDeclarationSpecifier`、`ExportDefaultDeclarationKind`），实现阶段需依赖本地 crate 源码确认字段和辅助方法。

## 2026-02-25 (Task 2)
- 本任务再次执行 `lsp_diagnostics` 仍因缺少 `rust-analyzer` 失败；已用 `cargo test -p ast_engine` 完整回归作为替代验证。
