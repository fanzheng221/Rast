# Rast 项目概览

## 项目目的
构建一个基于 Rust 的 AST 分析工具，作为通用 bundler 插件（通过 unplugin）原生集成，并暴露 MCP 服务器供 AI 查询。

## 技术栈

### 后端（Rust）
- **Cargo Workspace**: Rust monorepo 管理
- **oxc**: 高性能 JavaScript/TypeScript AST 解析和分析库
- **napi-rs**: Rust 到 Node.js 的 FFI 绑定

### 前端（Node.js/TypeScript）
- **pnpm Workspaces**: JavaScript/TypeScript monorepo 管理
- **TypeScript**: 类型安全的 JavaScript 超集
- **unplugin**: 通用 bundler 插件框架（支持 Vite、Webpack、Rollup 等）
- **@modelcontextprotocol/sdk**: MCP 协议实现

### 开发工具
- **tsup**: TypeScript 打包工具
- **vitest**: 单元测试框架
- **ESLint**: 代码检查
- **Prettier**: 代码格式化

## 项目架构

### Monorepo 结构
```
Rast/
├── crates/                    # Rust workspace
│   └── ast_engine/           # 核心 AST 分析引擎
│       ├── src/
│       │   └── lib.rs        # 主要库入口
│       └── Cargo.toml
├── packages/                 # pnpm workspace
│   ├── bindings/            # NAPI 绑定（Node.js <-> Rust）
│   │   ├── index.js         # 生成的绑定入口
│   │   └── package.json
│   ├── unplugin/            # 通用 bundler 插件
│   │   ├── src/
│   │   │   └── index.ts     # 插件主入口
│   │   └── package.json
│   └── mcp-server/          # MCP 服务器
│       ├── src/
│       │   └── index.ts     # 服务器入口
│       └── package.json
├── Cargo.toml               # Rust workspace 配置
├── pnpm-workspace.yaml      # pnpm workspace 配置
├── package.json             # 根 package.json
├── tsconfig.json            # TypeScript 配置
└── opencode.md              # 项目工作指南
```

### 工作流程
1. **AST 分析**（Rust）: `oxc` 解析 JavaScript/TypeScript 代码，提取 AST 信息
2. **FFI 绑定**（Rust + Node.js）: `napi-rs` 暴露 Rust 函数给 Node.js
3. **Bundler 集成**: `unplugin` 在构建时调用绑定进行 AST 分析
4. **AI 查询**: MCP 服务器提供标准接口供 AI 工具查询 AST 信息

## 核心约束

1. **Serena 工具链**: 所有文件修改必须使用 `serena_*` 工具
2. **只读分析**: MVP 阶段仅支持 AST 提取和 linting，不支持复杂 AST 变换
3. **Stdio 传输**: MCP 服务器使用 stdio 传输（SSE/HTTP 延迟到 v2）
4. **测试重点**: Vite 和 Rollup 集成测试（Webpack/Rspack 测试延后）

## 开发阶段

| 阶段 | 任务 | 描述 |
|------|------|------|
| Wave 1 | Task 1 | Serena 初始化 & Monorepo 设置 |
| Wave 2 | Task 2 | Rust AST 引擎实现 |
| Wave 3 | Task 3 | NAPI 绑定实现 |
| Wave 4 | Task 4 | Unplugin 包装器实现 |
| Wave 4 | Task 5 | MCP 服务器实现 |
| Wave 5 | Task 6 | E2E 集成 & CI 设置 |

## 系统信息
- 操作系统: Darwin (macOS)
- 编程语言: TypeScript, Rust
- 文件编码: UTF-8
