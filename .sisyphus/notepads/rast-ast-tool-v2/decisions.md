## 2026-02-25
- 保留 `analyze_ast(source: &str) -> String` 与 `AnalysisResult.exports/issues` 字段，新增 `file_structure` 承载 v2 结构化信息，确保 NAPI 现有调用兼容。
- 依赖图、导出信息使用 AST 节点直接提取，不再使用 regex；`var`/`console.*` 规则改为 AST visitor 检测。
- 调用图采用轻量边模型 `CallGraph { edges }`，以当前函数作用域栈记录 caller，支持后续跨文件图扩展。
