# Rast 🚀

English | [中文](./README.md)

[![npm version](https://img.shields.io/npm/v/@rast/cli.svg)](https://www.npmjs.com/package/@rast/cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

**Rast** is a high-performance AST pattern-matching and code-rewrite toolkit powered by Rust and [oxc](https://github.com/oxc-project/oxc). We provide a complete code analysis and refactoring solution, ranging from a fast Rust engine to Node.js API bindings, an out-of-the-box CLI, bundler plugins, and a standard MCP Server.

---

## 🌟 Core Features

| Feature | Description |
| :--- | :--- |
| **🚀 Extreme Performance** | Core analysis layer built in Rust, delivering **3~4x faster** AST matching and rewriting compared to similar tools like ast-grep. |
| **🧩 Flexible Rule Engine** | Define match patterns and rewrite rules via YAML, supporting composition logics such as `pattern`, `regex`, `kind`, `all`, `any`, `not`, `inside`, and `has`. |
| **📦 Ecosystem Integration** | Perform single-file rewrites, directory-wide scans, or hook into Vite, Webpack, and Rollup build pipelines via Unplugin. |
| **🤖 Native AI/MCP Support** | Built-in [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server allows LLMs and AI assistants to directly invoke AST-level refactoring for massive codebase changes. |
| **💚 Vue SFC Friendly** | Natively parses and rewrites `<script>` blocks within Vue 3 Single-File Components (SFC), providing accurate absolute/relative offsets and line/column information. |

---

## 🚀 Quick Start

### 1️⃣ Global CLI Installation (Recommended)

The fastest way to experience Rast is to install the CLI globally. You can use npm, yarn, or pnpm to install the `@rast/cli` package:

```bash
# Install CLI globally
npm install -g @rast/cli

# Or using pnpm
pnpm add -g @rast/cli
```

*(If you prefer not to install globally, you can substitute `rast` with `npx @rast/cli` in the commands below)*

### 2️⃣ Write Rules and Scan Directories

Create a simple YAML rule file (e.g., `rules/no-console.yml`):

```yaml
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
```

**Single-file preview (Outputs transformation to terminal without modifying the file):**
```bash
rast run src/app.ts rules/no-console.yml --output text
```

**Directory preview (Dry-run mode to see matches):**
```bash
rast scan src/ rules/no-console.yml --dry-run
```

**Scan directory and automatically rewrite matched files:**
```bash
rast scan src/ rules/no-console.yml
```

> **Common CLI Flags:**
> - `--dry-run`: Collect and print matches without writing to files.
> - `--extensions js,ts,vue,tsx`: Restrict scanning file extensions (default: `js,ts,jsx,tsx`).
> - `--output json|text`: Output format, useful for piping into other tools.

---

## 📖 Advanced Usage & Ecosystem APIs

Rast can also be used as a module or plugin deeply integrated into your workflows.

### Node.js Bindings API (`@rast/bindings`)

For usage directly inside Node.js scripts, we provide high-performance NAPI bindings bridging the Rust engine.

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

// Synchronously call the Rust engine to apply rewrite rules
const result = rast.apply_rule('foo("hello");', rule);
console.log(result); // Output: bar("hello");
```

### Bundler Plugin (`@rast/unplugin`)

Rast can intercept and transform code dynamically during the build phase (supports Vite, Webpack, and Rollup).

```bash
npm install -D @rast/unplugin
```

**Vite Configuration Example (`vite.config.ts`):**
```typescript
import { defineConfig } from 'vite';
import rast from '@rast/unplugin';

export default defineConfig({
  plugins: [
    rast.vite({
      rules: ['./rules'], // Your YAML rules directory
      injectIssues: true, // Inject match warnings into the build stream
      logIssues: true,
      mode: 'on-demand',  // 'on-demand' or 'cache'
    }),
  ],
});
```

### MCP Server (`@rast/mcp-server`)

Rast exposes 7 powerful underlying tools to AI assistants, including `scan_directory`, `apply_rule`, `get_file_structure`, `analyze_ast`, etc.

Simply configure your MCP-compatible AI client (like Claude Desktop or Cursor) to use the Rast server:

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
Once configured, your AI assistant will gain the ability to perform AST-level batch code rewrites and structural analysis.

---

## ⚡ Performance Benchmark

Rast benchmark results against ast-grep on a single directory scan over a large repository (10,000 files):

| Tool | Files | Rules | Total Time | Result |
| --- | --- | --- | ---: | --- |
| **rast scan** | 10000 | 10 | **7.23 s** | **3.33x Faster** 🚀 |
| ast-grep | 10000 | 10 | 24.05 s | 1x |

*(Test Environment: macOS 14 / Apple M2 / 16GB / Node v22.20 / Rust 1.93. Note: Actual speeds vary based on hardware and rule complexity)*

---

## 👨‍💻 Contributing

If you wish to contribute to the core development of Rast, you can clone the repository and build it locally. We use pnpm workspace for monorepo management.

**Local Development Requirements:** 
- Node.js (>= v22.0)
- pnpm (>= 10.0)
- Rust (>= 1.93)

```bash
# 1. Install dependencies
pnpm install

# 2. Build the workspace (compiles Rust engine and NAPI bindings)
pnpm build

# 3. Run tests and verifications
pnpm verify:all
```

<details>
<summary><b>Click to View Verification Scripts Details</b></summary>

- `pnpm verify:benchmark`: Tests scan performance and compares against ast-grep (requires ast-grep to be installed locally).
- `pnpm verify:vue`: Tests row/col offset parsing features against Vue 3 SFC source codes.
- `pnpm verify:napi-mcp`: E2E integration test for NAPI bindings and the MCP service.

</details>

### Repository Architecture

```text
Rast/
├── crates/ast_engine        # [Rust] High-performance AST analysis engine & rule system
├── packages/bindings        # [Node] Rust bindings via NAPI-RS
├── packages/rast-cli        # [Node] User-facing CLI tool
├── packages/unplugin        # [Node] Bundler plugins (Vite/Webpack adapters)
├── packages/mcp-server      # [Node] Standard MCP Server implementation
└── scripts/                 # [Task] Internal benchmark, build, and CI verification scripts
```

---

## 📄 License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).  
© 2026-PRESENT Rast Contributors
