## 2026-02-25
- `oxc` 0.38 可通过 `SourceType::from_path("inline.tsx")` 统一解析 JS/TS/JSX/TSX 内联代码，避免文件扩展名缺失导致的语法降级。
- `oxc_ast::Visit` + `walk::*` 适合在一次遍历里同时提取函数签名、类/接口/类型别名、调用关系、lint 规则命中。
- `SemanticBuilder::with_build_jsdoc(true)` 可直接从 `semantic.jsdoc().iter_all()` 拿到 JSDoc span，避免手工注释匹配。

## 2026-02-25 (Task 2)
- `ProjectGraph` 使用 `Arc<RwLock<ProjectGraphState>>` 可以在读多写少场景下保持并发读性能，并通过 `Result` 显式传播 lock poison 错误。
- 对相同 `path + code` 使用 `DefaultHasher` 做缓存键，`add_file` 可直接命中并跳过重复解析，避免重复调用 `analyze_ast_internal`。
- 跨文件依赖解析需优先生成一组候选路径（裸路径、扩展名补全、index 入口），再按图中已存在文件过滤，避免 `./foo` 与 `foo.ts` 不匹配。
