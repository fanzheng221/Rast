# Rast 常用命令

## 项目初始化

### 安装依赖
```bash
# 安装所有 Node.js 依赖
pnpm install

# 检查 Rust 依赖
cargo check
```

### 激活 Serena 项目
```bash
# 激活当前项目
serena activate-project /Users/smart/Desktop/Rast

# 检查 onboarding 状态
serena check-onboarding-performed

# 执行 onboarding
serena onboarding
```

## 构建

### 构建所有包
```bash
# 构建所有 Node.js 包
pnpm build

# 构建所有 Rust crate
cargo build --release
```

### 按包构建
```bash
# 构建 NAPI 绑定
pnpm --filter bindings run build

# 构建 unplugin
pnpm --filter unplugin run build

# 构建 MCP 服务器
pnpm --filter mcp-server run build
```

## 测试

### 运行所有测试
```bash
# 运行所有测试
pnpm test

# 运行 E2E 测试
pnpm run test:e2e
```

### 按包测试
```bash
# 测试 Rust AST 引擎
cargo test -p ast_engine

# 测试 unplugin
pnpm --filter unplugin run test

# 测试 MCP 服务器
pnpm --filter mcp-server run test
```

### 开发模式
```bash
# Watch 模式测试
pnpm --filter unplugin run dev

# Rust Watch 模式
cargo watch -x check -x test
```

## 代码质量

### Lint 和 Format
```bash
# 运行 ESLint
pnpm lint

# 格式化代码
pnpm format

# Rust 格式化
cargo fmt

# Rust Lint
cargo clippy
```

## MCP 服务器

### 运行 MCP 服务器
```bash
# 构建并运行
pnpm --filter mcp-server run build
node packages/mcp-server/dist/index.js

# 测试 MCP 工具列表
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | node packages/mcp-server/dist/index.js
```

## 系统命令（Darwin/macOS）

### 文件操作
```bash
# 列出文件（长格式）
ls -la

# 查找文件
find . -name "*.rs"

# 查找文件（原生 grep 作为后备，优先推荐 Serena 的 search-for-pattern）
grep -r "pattern" .

# 更高效的 ripgrep（作为后备）
rg "pattern"

### Git 操作
```bash
# 查看状态
git status

# 查看修改
git diff

# 提交更改
git add .
git commit -m "type: description"

# 推送到远程
git push
```

## Serena 工具

### 项目信息
```bash
# 获取当前配置
serena get-current-config

# 列出目录
serena list-dir .

# 查找文件 (推荐，替代 native glob)
serena find-file "*.rs" .

# 读取文件 (推荐，替代 native read)
serena read-file crates/ast_engine/src/lib.rs

# 全局内容搜索 (推荐，替代 native grep/rg)
serena search-for-pattern "pattern"

### 符号操作
```bash
# 查找符号
serena find-symbol "analyze_ast"

# 查找引用
serena find-referencing-symbols "analyze_ast" crates/ast_engine/src/lib.rs

# 获取符号概览
serena get-symbols-overview crates/ast_engine/src/lib.rs
```

### 内存管理
```bash
# 列出所有内存
serena list-memories

# 读取内存
serena read-memory project_overview

# 写入内存
serena write-memory project_overview "content"

# 更新内存
serena edit-memory project_overview "needle" "replacement"
```

## Docker（如果使用）

### 构建 Docker 镜像
```bash
docker build -t rast:latest .
```

### 运行容器
```bash
docker run -it rast:latest
```

## CI/CD

### 本地运行 CI 检查
```bash
# 模拟 GitHub Actions
pnpm install
pnpm build
pnpm test
cargo clippy --all-targets --all-features
```

## 故障排除

### 清理构建产物
```bash
# 清理 Node.js
rm -rf node_modules dist

# 清理 Rust
cargo clean

# 重新安装依赖
pnpm install
```

### 检查依赖冲突
```bash
# 检查 pnpm workspace 依赖
pnpm list --depth=0

# 检查 Cargo 依赖
cargo tree
```

## VS Code 相关

### 打开 Serena 仪表板
```bash
serena open-dashboard
```

### 重启语言服务器
```bash
serena restart-language-server
```

## 调试

### Rust 调试
```bash
# 构建调试版本
cargo build

# 使用 lldb/gdb
lldb target/debug/rast
```

### Node.js 调试
```bash
# 使用 --inspect 标志
node --inspect packages/mcp-server/dist/index.js
```

## 快速参考

| 任务 | 命令 |
|------|------|
| 初始设置 | `pnpm install && cargo check` |
| 构建全部 | `pnpm build` |
| 测试全部 | `pnpm test` |
| 构建绑定 | `pnpm --filter bindings run build` |
| 测试 Rust | `cargo test -p ast_engine` |
| 格式化 | `pnpm format && cargo fmt` |
| Lint | `pnpm lint && cargo clippy` |
| MCP 测试 | `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \| node packages/mcp-server/dist/index.js` |
