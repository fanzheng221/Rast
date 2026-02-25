## 2026-02-25
- 当前环境缺少 `rust-analyzer`，`lsp_diagnostics` 初始化超时（Unknown binary 'rust-analyzer'），无法给出文件级 LSP clean 证明。
- `oxc` AST 类型较细粒度（如 `ModuleExportName`、`ImportDeclarationSpecifier`、`ExportDefaultDeclarationKind`），实现阶段需依赖本地 crate 源码确认字段和辅助方法。
