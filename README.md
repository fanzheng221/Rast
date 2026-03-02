# Rast 🚀

[English](./README.en.md) | 中文

[![npm version](https://img.shields.io/npm/v/@rast/cli.svg)](https://www.npmjs.com/package/@rast/cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

**Rast** 是一个基于 Rust 和 [oxc](https://github.com/oxc-project/oxc) 开发的高性能 AST 模式匹配与代码改写工具链。我们提供从底层的快速 Rust 引擎到 Node.js API 绑定、开箱即用的命令行工具（CLI）、构建工具插件，以及标准的 MCP Server 的一整套代码分析与重构解决方案。

---

## 🌟 核心特性

| 特性 | 描述 |
| :--- | :--- |
| **🚀 极致性能** | 核心分析层由 Rust 构建，AST 匹配与重写速度相较于同类工具（如 ast-grep）有 **3~4 倍** 的性能优势。 |
| **🧩 灵活的规则引擎** | 支持通过 YAML 定义匹配模式和替换规则，支持多种组合逻辑（`pattern`, `regex`, `kind`, `all`, `any`, `not`, `inside`, `has`）。 |
| **📦 无缝生态集成** | 支持单文件替换、批量目录扫描，同时可通过 Unplugin 完美融入 Vite、Webpack、Rollup 等前端工程流水线。 |
| **🤖 AI/MCP 原生支持** | 内置标准的 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 服务器，允许大语言模型和 AI 助手直接调用 AST 级重构能力完成海量代码改写。 |
| **💚 Vue SFC 友好** | 原生支持解析并重写 Vue 3 单文件组件（SFC）的 Script 块，可精确返回匹配区域的绝对/相对偏移量与行列信息。 |

---

## 🚀 快速开始

### 1️⃣ 全局安装 CLI (推荐)

最快速体验 Rast 的方式是将其安装为全局命令行工具。你可以使用 npm、yarn 或 pnpm 安装 `@rast/cli` 包：

```bash
# 全局安装 CLI
npm install -g @rast/cli

# 或通过 pnpm
pnpm add -g @rast/cli
```

*(如果你不想全局安装，也可以在下文命令中直接使用 `npx @rast/cli` 替代 `rast`)*

### 2️⃣ 编写规则与目录扫描

创建一个简单的 YAML 规则文件（例如 `rules/no-console.yml`）：

```yaml
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
```

**单文件转换预览（结果输出到终端，不修改原文件）:**
```bash
rast run src/app.ts rules/no-console.yml --output text
```

**扫描整个目录（仅预览结果 Dry-run 模式）:**
```bash
rast scan src/ rules/no-console.yml --dry-run
```

**扫描目录并直接改写所有匹配文件:**
```bash
rast scan src/ rules/no-console.yml
```

> **CLI 常用参数说明：**
> - `--dry-run`：仅统计并打印匹配信息，不执行磁盘覆写。
> - `--extensions js,ts,vue,tsx`：自定义需要扫描的文件扩展名（默认 `js,ts,jsx,tsx`）。
> - `--output json|text`：自定义输出格式，用于集成到其他管道。

---

## 📖 高级用法与生态 API

除了 CLI 之外，Rast 还可以作为模块或插件集成到你的各类工程流中。

### Node.js 绑定 API (`@rast/bindings`)

对于需要在自己的 Node.js 脚本中调用 AST 分析引擎的场景，你可以直接使用 Rast 提供的高性能 NAPI 绑定层。

```bash
npm install @rast/bindings
```

```javascript
const rast = require('@rast/bindings');

const rule = `
id: replace-foo
language: ts
rule:
  pattern: foo($A)
fix: bar($A)
`;

// 同步调用底层 Rust 引擎进行改写
const result = rast.apply_rule('foo("hello");', rule);
console.log(result); // 输出: bar("hello");
```

### 构建工具插件 (`@rast/unplugin`)

Rast 可以拦截并转化构建阶段的代码（支持 Vite / Webpack / Rollup）。

```bash
npm install -D @rast/unplugin
```

**Vite 配置示例 (`vite.config.ts`):**
```typescript
import { defineConfig } from 'vite';
import rast from '@rast/unplugin';

export default defineConfig({
  plugins: [
    rast.vite({
      rules: ['./rules'], // 指定你的 YAML 规则目录
      injectIssues: true, // 在构建流中主动暴露规则匹配警告
      logIssues: true,
      mode: 'on-demand',  // 'on-demand' 或 'cache'
    }),
  ],
});
```

### MCP 服务器 (`@rast/mcp-server`)

Rast 暴露了 7 个强大的底层工具给 AI 助手，包含 `scan_directory`、`apply_rule`、`get_file_structure`、`analyze_ast` 等。

你可以在支持 MCP 协议的 AI 客户端（如 Claude Desktop, Cursor 等）配置文件中加入 Rast 服务器：

```json
{
  "mcpServers": {
    "rast-mcp": {
      "command": "npx",
      "args": ["-y", "@rast/mcp-server"]
    }
  }
}
```
配置完成后，你的 AI 助手便拥有了 AST 级别的批量代码重写与工程分析能力。

---

## ⚡ 性能基准 (Benchmark)

Rast 在针对大型仓库（10,000 个文件）进行单次重构扫描时的性能基准结果：

| Tool | Files | Rules | Total Time | 结论 |
| --- | --- | --- | ---: | --- |
| **rast scan** | 10000 | 10 | **7.23 s** | **3.33x Faster** 🚀 |
| ast-grep | 10000 | 10 | 24.05 s | 1x |

*(测试机器环境: macOS 14 / Apple M2 / 16GB / Node v22.20 / Rust 1.93。说明：实际速度受硬件及规则复杂度影响)*

---

## 👨‍💻 参与贡献 (Contributing)

对于想要参与 Rast 核心引擎开发的贡献者，你可以克隆本项目并进行本地构建。我们使用 pnpm workspace 管理代码。

**本地开发环境要求：** 
- Node.js (>= v22.0)
- pnpm (>= 10.0)
- Rust (>= 1.93)

```bash
# 1. 安装所有依赖包
pnpm install

# 2. 构建整个工作区 (编译 Rust 引擎及 NAPI)
pnpm build

# 3. 运行完整测试与端到端验证
pnpm verify:all
```

<details>
<summary><b>点击查看开发者验证脚本（Verification Scripts）说明</b></summary>

- `pnpm verify:benchmark`: 测试扫描性能并运行 ast-grep 对比测试（需本地预装 ast-grep）。
- `pnpm verify:vue`: 运行针对 Vue 3 SFC 源码行列偏移等特性的兼容解析验证。
- `pnpm verify:napi-mcp`: NAPI 接口与 MCP 服务的端到端联调自动化测试。

</details>

### 项目架构目录

```text
Rast/
├── crates/ast_engine        # [Rust] 底层高性能 AST 分析引擎与规则系统
├── packages/bindings        # [Node] 基于 NAPI-RS 桥接的 Rust 绑定包
├── packages/rast-cli        # [Node] 面向用户的命令行工具 (CLI)
├── packages/unplugin        # [Node] 构建工具插件 (Vite/Webpack 适配器)
├── packages/mcp-server      # [Node] 标准 MCP 服务器实现
└── scripts/                 # [Task] 内部基准测试、构建脚本及 CI 验证脚本
```

---

## 📄 开源协议 (License)

本项目采用 [MIT License](https://opensource.org/licenses/MIT) 进行开源。  
© 2026-PRESENT Rast Contributors
