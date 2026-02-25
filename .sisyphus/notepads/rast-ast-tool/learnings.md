
## [2026-02-25] Task 1: Serena Init & Monorepo Setup

### 完成项
- ✅ Serena 项目已激活（支持 Rust 和 TypeScript）
- ✅ 执行了 `serena_onboarding`，创建了项目 memory
- ✅ Cargo workspace 配置完成（包含 ast_engine 和 bindings）
- ✅ pnpm workspace 配置完成
- ✅ 目录结构创建完成：
  - `crates/ast_engine/`
  - `packages/bindings/`
  - `packages/unplugin/`
  - `packages/mcp-server/`
- ✅ 所有 `package.json` 文件已初始化
- ✅ TypeScript 构建：
  - unplugin ✅ (tsup 构建 ESM + CJS)
  - mcp-server ✅ (tsup 构建 ESM)
- ✅ Rust 依赖检查通过：`cargo check`
- ✅ Rust 测试通过：`cargo test -p ast_engine`

### 待解决项
- ⚠️ bindings 包的 NAPI 构建需要正确的 Node.js 链接配置（Task 3 中解决）

### 关键配置
- Serena 项目支持 Rust 和 TypeScript 语言
- 使用 oxc 0.38 作为 AST 解析引擎
- 使用 napi-rs 进行 Rust 到 Node.js 的 FFI 绑定
- 使用 unplugin 作为通用 bundler 插件框架
- 使用 @modelcontextprotocol/sdk 作为 MCP 服务器

### 遇到的问题
1. Rust 工具链需要手动安装（用户已安装）
2. bindings 包的构建遇到 Node.js 链接问题（符号未找到）- 这是预期的，将在 Task 3 中实现完整的 NAPI 绑定时解决

## [2026-02-25] Task 2: 实现 Rust AST Engine

### 完成项
- ✅ 实现了基于 regex 的 AST 分析引擎（MVP 版本）
- ✅ 支持提取 exports：
  - `export function`
  - `export const`
  - `export class`
  - `export interface`
  - `export type`
- ✅ 实现了基本 linting 规则：
  - 检测 `var` 声明（建议使用 const/let）
  - 检测 console 语句
- ✅ 所有测试通过：`cargo test -p ast_engine`

### 技术挑战
- ⚠️ oxc 0.38 API 与预期差异较大
- ⚠️ 由于 API 复杂性，MVP 使用 regex-based 解析
- 💡 未来可以升级到使用 oxc Visitor 模式的完整 AST 遍历

### 数据结构
```rust
pub struct ExportInfo {
    pub name: String,           // 导出项名称
    pub kind: String,           // 类型: function, variable, class, type, interface
    pub location: Option<(usize, usize)>,  // 行列位置
}

pub struct LintIssue {
    pub category: String,        // 类别: parse, best-practices, dev-code
    pub severity: String,        // 严重性: error, warning
    pub message: String,         // 问题描述
    pub location: Option<(usize, usize)>,  // 位置
}

pub struct AnalysisResult {
    pub exports: Vec<ExportInfo>,  // 导出列表
    pub issues: Vec<LintIssue>,    // 问题列表
}
```

### 依赖更新
- 添加了 `serde` 和 `serde_json` 到依赖

### 下一步
- Task 3: 实现 NAPI Bindings，将 Rust 函数暴露给 Node.js

## [2026-02-25] Task 3: 修复 NAPI Bindings

### 完成项
- ✅ 修复了 `packages/bindings/src/lib.rs` 中的名称冲突问题（将 `ast_engine::analyze_ast` 重命名为 `internal_analyze_ast`）
- ✅ 修复了 `packages/bindings/src/lib.rs` 测试代码中 `String` 和 `&str` 的类型不匹配问题
- ✅ 修复了 `packages/bindings` 缺少 `build.rs` 导致的 macOS 链接错误（`Undefined symbols for architecture arm64: _napi_add_env_cleanup_hook`）
- ✅ 成功编译并生成了 `.node` 文件
- ✅ 验证了生成的 Node.js 模块可以正常调用并返回有效的 JSON 结果

### 遇到的问题
1. **名称冲突**：`ast_engine::analyze_ast` 和本地导出的 `#[napi] pub fn analyze_ast` 冲突。通过 `use ast_engine::analyze_ast as internal_analyze_ast;` 解决。
2. **链接错误**：在 macOS 上使用 `napi-rs` 时，如果没有 `build.rs` 调用 `napi_build::setup()`，会导致链接器找不到 NAPI 符号。通过添加 `build.rs` 解决。

### 关键配置
- `napi-rs` 项目必须包含 `build.rs` 文件，并在其中调用 `napi_build::setup()`，否则在 macOS 等平台上会遇到链接错误。

## MCP Server Implementation
- Successfully implemented the MCP server using `@modelcontextprotocol/sdk`.
- Registered the `analyze_ast` tool using `server.setRequestHandler(ListToolsRequestSchema, ...)` and `server.setRequestHandler(CallToolRequestSchema, ...)`.
- Integrated `@rast/bindings` by adding it as a workspace dependency and calling `analyzeAst(source)` within the tool handler.
- Verified that the stdio transport works correctly by sending JSON-RPC requests via stdin and receiving responses via stdout.

## [2026-02-25] Task 4: 实现 Unplugin Wrapper

### 完成项
- ✅ 在 `packages/unplugin/package.json` 中添加了 `@rast/bindings` 和 `unplugin` 依赖
- ✅ 实现了 `packages/unplugin/src/index.ts` 中的 `transform` 钩子，集成了 `@rast/bindings` 的 `analyzeAst` 函数
- ✅ 支持 `injectIssues` 和 `logIssues` 配置选项，可以控制是否在代码中注入注释或在控制台打印警告
- ✅ 创建了 `packages/unplugin/vitest.config.ts` 配置
- ✅ 编写了 `packages/unplugin/test/basic.test.ts` 测试用例，验证了 `transform` 钩子的行为
- ✅ 成功运行了 `pnpm --filter unplugin run build` 和 `pnpm --filter unplugin run test`

### 遇到的问题
1. **Vitest 默认行为**：`vitest` 默认以 watch 模式运行，导致 CI/自动化脚本挂起。通过将 `package.json` 中的 `test` 脚本修改为 `vitest run` 解决。
2. **TypeScript 类型**：在测试文件中使用 `Function` 类型和 `any` 类型会导致 Biome linting 警告。通过使用更精确的类型 `(code: string, id: string) => { code: string; map: unknown } | undefined` 解决。

### 关键配置
- `unplugin` 的 `transform` 钩子可以返回 `{ code, map }` 对象，如果不需要修改代码，可以返回原始 `code` 和 `map: null`。
- 通过 `JSON.parse(analyzeAst(code))` 解析 Rust 引擎返回的 JSON 字符串，获取 `issues` 和 `exports` 信息。

## E2E Integration & CI Setup (Task 6)
- **E2E Testing**: Created a comprehensive E2E test script (`scripts/e2e-test.js`) that verifies the entire toolchain:
  - Direct bindings call (`@rast/bindings`)
  - Unplugin transform (`@rast/unplugin`)
  - MCP server communication (`@rast/mcp-server`)
- **CI Workflows**: Added GitHub Actions workflows for:
  - `ci.yml`: Standard CI for formatting, linting, building, and running E2E tests.
  - `napi-rs.yml`: Cross-platform build matrix for NAPI binaries (macOS, Linux, Windows).
  - `napi-release.yml`: Automated release workflow for publishing to npm.
- **Testing Strategy**: The E2E test script uses `child_process.spawn` to test the MCP server via stdio, simulating a real MCP client. It also directly tests the Unplugin's `transform` hook to ensure issues are correctly injected as comments.
