## 2026-02-25
- `oxc` 0.38 可通过 `SourceType::from_path("inline.tsx")` 统一解析 JS/TS/JSX/TSX 内联代码，避免文件扩展名缺失导致的语法降级。
- `oxc_ast::Visit` + `walk::*` 适合在一次遍历里同时提取函数签名、类/接口/类型别名、调用关系、lint 规则命中。
- `SemanticBuilder::with_build_jsdoc(true)` 可直接从 `semantic.jsdoc().iter_all()` 拿到 JSDoc span，避免手工注释匹配。
