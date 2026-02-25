## 2026-02-25
- 保留 `analyze_ast(source: &str) -> String` 与 `AnalysisResult.exports/issues` 字段，新增 `file_structure` 承载 v2 结构化信息，确保 NAPI 现有调用兼容。
- 依赖图、导出信息使用 AST 节点直接提取，不再使用 regex；`var`/`console.*` 规则改为 AST visitor 检测。
- 调用图采用轻量边模型 `CallGraph { edges }`，以当前函数作用域栈记录 caller，支持后续跨文件图扩展。

## 2026-02-25 (Task 2)
- `ProjectGraph` 放在 `ast_engine::lib` 内与 `analyze_ast_internal` 共域，避免暴露额外解析 API，同时保持 `analyze_ast` 对外接口不变。
- `find_symbol` 以 `symbols` 为主、`exports` 补充，避免 interface/type/class 在同文件被重复返回，同时保留导出变量类符号的可检索性。
