# Rast 任务完成检查清单

## 任务完成后必须执行的检查

### 1. 自动化检查（必须全部通过）

#### TypeScript/Node.js
- [ ] `lsp_diagnostics` 返回零错误（项目级别）
- [ ] `pnpm build` 成功（exit code 0）
- [ ] `pnpm test` 全部通过

#### Rust
- [ ] `cargo check` 成功（exit code 0）
- [ ] `cargo test` 全部通过
- [ ] `cargo clippy` 无警告（或解释为何忽略）

### 2. 代码审查（手动检查）

#### 检查所有修改的文件
- [ ] 读取每个修改的文件
- [ ] 验证逻辑是否符合任务要求
- [ ] 检查是否有 stub、TODO、硬编码值
- [ ] 检查逻辑错误和边界情况
- [ ] 验证遵循现有代码模式
- [ ] 检查导入是否正确和完整

#### 交叉验证
- [ ] subagent 的声明与实际代码一致
- [ ] 所有承诺的功能都已实现

### 3. 验证要求

#### Task 1: Serena Init & Monorepo Setup
- [ ] `pnpm install` 成功
- [ ] `cargo check` 成功
- [ ] 目录结构完整：
  - `crates/ast_engine/` 存在
  - `packages/bindings/` 存在
  - `packages/unplugin/` 存在
  - `packages/mcp-server/` 存在
- [ ] 所有 `package.json` 文件存在且配置正确
- [ ] `Cargo.toml` workspace 配置正确
- [ ] `pnpm-workspace.yaml` 配置正确

#### Task 2: Rust AST Engine
- [ ] `cargo test -p ast_engine` 全部通过
- [ ] 使用 `oxc` 实现 AST 遍历
- [ ] 实现导出提取功能
- [ ] 实现基本 linting 模式
- [ ] 所有公共函数有文档注释

#### Task 3: NAPI Bindings
- [ ] `pnpm --filter bindings run build` 成功
- [ ] `node -e "require('./packages/bindings/index.js')"` 无错误
- [ ] Rust 函数正确暴露给 Node.js
- [ ] 序列化效率优化

#### Task 4: Unplugin Wrapper
- [ ] `pnpm --filter unplugin run test` 通过
- [ ] vitest 配置正确
- [ ] 支持 Vite 集成
- [ ] 支持 Rollup 集成
- [ ] 调用 NAPI 绑定正确

#### Task 5: MCP Server
- [ ] `pnpm --filter mcp-server run build` 成功
- [ ] MCP stdio 传输正确
- [ ] `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | node packages/mcp-server/dist/index.js` 返回有效 JSON
- [ ] AST 分析暴露为 MCP Tools

#### Task 6: E2E Integration & CI
- [ ] `pnpm run test:e2e` 通过
- [ ] E2E 测试验证 Node.js -> Rust 流程
- [ ] GitHub Actions CI 配置正确
- [ ] 跨平台二进制构建配置

### 4. 文件检查

#### 检查清单
- [ ] 没有遗留的调试代码（console.log、println! 等）
- [ ] 没有硬编码的测试值
- [ ] 没有 TODO 注释（除非合理）
- [ ] 所有导入都被使用
- [ ] 代码风格一致
- [ ] 文档注释完整

### 5. 更新 Memory（如果架构/规范变更）

- [ ] 更新 `project_overview`（如果架构变化）
- [ ] 更新 `suggested_commands`（如果命令变化）
- [ ] 更新 `style_conventions`（如果风格变化）

### 6. Git 提交前检查

- [ ] `git status` 显示预期更改
- [ ] `git diff` 检查所有修改
- [ ] 没有敏感信息（API 密钥、密码等）
- [ ] 提交信息格式正确

### 7. 特定包验证

#### bindings 包
- [ ] 生成的 `.node` 文件存在
- [ ] TypeScript 类型定义存在（`index.d.ts`）

#### unplugin 包
- [ ] `dist/` 目录存在且包含构建产物
- [ ] 支持 ESM 和 CommonJS

#### mcp-server 包
- [ ] `dist/` 目录存在
- [ ] 可执行权限设置正确

### 8. 跨平台检查（如果适用）

- [ ] 代码在 Darwin/macOS 上运行
- [ ] 没有平台特定的硬编码路径
- [ ] 使用 `std::path` 或 `path` 模块处理路径

### 9. 性能检查（可选但推荐）

- [ ] Rust 代码使用 `cargo bench` 进行基准测试
- [ ] 没有明显的性能瓶颈
- [ ] 内存使用合理

### 10. 安全检查

- [ ] 没有已知的安全漏洞（使用 `cargo audit` 和 `pnpm audit`）
- [ ] 输入验证充分
- [ ] 错误处理适当

## 验证命令速查表

| 检查类型 | 命令 |
|---------|------|
| LSP 诊断 | `lsp_diagnostics .` |
| 构建 | `pnpm build && cargo build` |
| 测试 | `pnpm test && cargo test` |
| Lint | `pnpm lint && cargo clippy` |
| Format | `pnpm format && cargo fmt` |
| 审查代码 | 手动读取所有修改文件 |
| 计划状态 | 读取 `.sisyphus/plans/rast-ast-tool.md` |

## 常见问题排查

### 构建失败
1. 检查依赖版本冲突
2. 确认所有依赖已安装
3. 清理并重新构建

### 测试失败
1. 查看测试输出
2. 检查测试用例是否正确
3. 验证预期行为

### LSP 错误
1. 重启语言服务器
2. 确认项目已激活
3. 检查文件编码

## 任务标记规则

任务完成后，必须：
1. 验证所有检查项
2. 手动审查代码
3. 更新 `.sisyphus/plans/rast-ast-tool.md` 中的任务复选框
4. 在 `.sisyphus/notepads/rast-ast-tool/` 中记录完成情况
