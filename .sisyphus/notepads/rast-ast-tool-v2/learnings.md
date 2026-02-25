## 2026-02-25
- `oxc` 0.38 可通过 `SourceType::from_path("inline.tsx")` 统一解析 JS/TS/JSX/TSX 内联代码，避免文件扩展名缺失导致的语法降级。
- `oxc_ast::Visit` + `walk::*` 适合在一次遍历里同时提取函数签名、类/接口/类型别名、调用关系、lint 规则命中。
- `SemanticBuilder::with_build_jsdoc(true)` 可直接从 `semantic.jsdoc().iter_all()` 拿到 JSDoc span，避免手工注释匹配。

## 2026-02-25 (Task 2)
- `ProjectGraph` 使用 `Arc<RwLock<ProjectGraphState>>` 可以在读多写少场景下保持并发读性能，并通过 `Result` 显式传播 lock poison 错误。
- 对相同 `path + code` 使用 `DefaultHasher` 做缓存键，`add_file` 可直接命中并跳过重复解析，避免重复调用 `analyze_ast_internal`。
- 跨文件依赖解析需优先生成一组候选路径（裸路径、扩展名补全、index 入口），再按图中已存在文件过滤，避免 `./foo` 与 `foo.ts` 不匹配。

## 2026-02-25 (Task 3)
- `napi-rs` 暴露状态对象时可用绑定层 `ProjectGraph { inner: ast_engine::ProjectGraph }` 包装内部 `Arc<RwLock<...>>`，无需改动 engine 实现。
- 复杂结构在绑定层统一 JSON 序列化（`Option<String>` / `Vec<String>` / `String`）能保持 Node 侧 API 稳定，且避免直接暴露深层 Rust 类型。
- 为兼容任务要求的 snake_case JS API，可在 `#[napi]` 上使用 `js_name = "..."`（如 `initialize_graph`, `add_file`）。

## 2026-02-25 (Task 4)
- `ProjectGraph` 实例可以在 unplugin 初始化时通过 `initialize_graph(mode)` 创建，并作为模块级变量导出，以便后续其他工具（如 MCP server）查询。
- 在 `cache` 模式下，通过在 `transform` hook 中调用 `projectGraph.add_file(id, code)` 可以拦截打包器的 resolution 阶段，将所有解析的文件添加到图中。
- 在 `on-demand` 模式下，仅初始化图实例，不主动添加文件，避免不必要的内存占用。

## 2026-02-25 (Task 5)
- MCP Server 可在模块级初始化单例 `projectGraph = initialize_graph('on-demand')`，并在多个 `tools/call` 请求间复用同一 Rust 状态图。
- 对 `get_file_structure` 的空结果返回文本 `null`、对 `get_symbol_details` 返回 `JSON.stringify(array)`，可保证 MCP 响应始终为稳定的文本 JSON 片段。
- 用官方 `@modelcontextprotocol/sdk` 的 `Client + StdioClientTransport` 做 QA，比手写 `Content-Length` 报文更稳定，且能覆盖初始化握手流程。
- 为避免测试导入时意外启动 stdio 服务，ESM 入口应加 `import.meta.url === pathToFileURL(process.argv[1]).href` 守卫再执行 `main()`。
