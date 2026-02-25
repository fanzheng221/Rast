## 2026-02-25
- 保留 `analyze_ast(source: &str) -> String` 与 `AnalysisResult.exports/issues` 字段，新增 `file_structure` 承载 v2 结构化信息，确保 NAPI 现有调用兼容。
- 依赖图、导出信息使用 AST 节点直接提取，不再使用 regex；`var`/`console.*` 规则改为 AST visitor 检测。
- 调用图采用轻量边模型 `CallGraph { edges }`，以当前函数作用域栈记录 caller，支持后续跨文件图扩展。

## 2026-02-25 (Task 2)
- `ProjectGraph` 放在 `ast_engine::lib` 内与 `analyze_ast_internal` 共域，避免暴露额外解析 API，同时保持 `analyze_ast` 对外接口不变。
- `find_symbol` 以 `symbols` 为主、`exports` 补充，避免 interface/type/class 在同文件被重复返回，同时保留导出变量类符号的可检索性。

## 2026-02-25 (Task 3)
- bindings 层新增 `#[napi] pub struct ProjectGraph` 包装器，不直接对外暴露 `ast_engine::ProjectGraph`，将并发与缓存策略封装在 Rust 内部。
- `initialize_graph(mode)` 保留 `mode` 参数用于未来扩展，当前版本不参与分支逻辑，仅用于 API 前向兼容。
- `analyze_dependencies(paths)` 返回 `Vec<(String, Vec<DependencyInfo>)>` 的 JSON 字符串，确保可一次查询多文件且保留每个输入路径的上下文。

## 2026-02-25 (Task 4)
- 在 `packages/unplugin/src/index.ts` 中导出 `projectGraph` 实例，使其可被外部（如 MCP server）访问。
- `RastPluginOptions` 增加 `mode?: 'cache' | 'on-demand'` 选项，默认值为 `'on-demand'`，以保持向后兼容并避免默认占用过多内存。
- 保持现有的 `analyzeAst(code)` 逻辑用于 linting，与 `ProjectGraph` 的构建解耦，确保 `injectIssues` 和 `logIssues` 功能在两种模式下均正常工作。

## 2026-02-25 (Task 5)
- MCP 层保留并复用 `analyze_ast`，同时新增 `get_file_structure/get_symbol_details/analyze_dependencies`，统一通过 `tools` 常量注册，确保向后兼容与工具清单可维护性。
- `CallToolRequestSchema` 处理统一委托到 `callTool(name, args)`，把参数校验与错误消息集中在一个入口，避免 handler 分支重复。
- 不引入 `@rast/unplugin` 依赖，直接在 MCP server 使用 `@rast/bindings` 初始化图实例，降低包间耦合并满足有状态绑定接入要求。
- `src/index.ts` 增加 CLI 入口守卫，仅在直接执行脚本时启动 `main()`，允许测试安全导入 `callTool/tools`。
