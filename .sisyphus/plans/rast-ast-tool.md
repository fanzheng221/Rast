# Work Plan: Rast AST Analysis Tool

## 1. Goal
Build a Rust-powered AST analysis tool natively integrated as a universal bundler plugin (via unplugin) and exposing an MCP server for AI querying.

## 2. Core Constraints & Directives
- **Serena Toolchain**: ALL file modifications must be done using `serena_*` tools.
- **Initialization**: The first executor MUST run `serena_check_onboarding_performed` and `serena_onboarding`.
- **Architecture**: pnpm workspaces + Cargo workspaces monorepo.
- **Rust Core**: `crates/ast_engine` using `oxc` for pure AST analysis. MVP is read-only.
- **Node Bindings**: `packages/bindings` using `napi-rs`.
- **Bundler Plugin**: `packages/unplugin` exporting Vite/Webpack wrappers.
- **MCP Server**: `packages/mcp-server` using `@modelcontextprotocol/sdk`.

## 3. Scope Boundaries
**IN SCOPE (MVP):**
- Read-only AST extraction and linting capabilities.
- Pre-built binary distribution setup via `@napi-rs/cli` templates.
- Stdio-based MCP server transport.
- Vite and Rollup integration tests.

**OUT OF SCOPE (MVP):**
- Complex AST mutations / code transformation (delayed to v2).
- SSE/HTTP transport for MCP (delayed).
- Webpack/Rspack explicit E2E tests (unplugin supports them, but tests focus on Vite/Rollup for MVP).

## 4. Execution Waves

### Wave 1: Initialization & Scaffold
- **Task 1: Serena Init & Monorepo Setup**
  - Steps: 
    1. Run `serena_check_onboarding_performed`. Run `serena_onboarding` if needed. 
    2. Initialize Cargo workspace (`Cargo.toml`) and pnpm workspace (`pnpm-workspace.yaml`). 
    3. Create directory structures: `crates/ast_engine`, `packages/bindings`, `packages/unplugin`, and `packages/mcp-server`.
    4. Initialize `package.json` files for all packages with base scripts (`build`, `test`).
    5. Add `oxc` and `@napi-rs/cli` to dependencies.
  - QA: `pnpm install && cargo check` passes.

### Wave 2: Rust Core Engine
- **Task 2: Implement Rust AST Engine (`crates/ast_engine`)**
  - Steps: Implement core AST traversal using `oxc` Visitor pattern. Create high-level structs for extracting exports and linting basic patterns.
  - QA: `cargo test -p ast_engine` passes.

### Wave 3: FFI Bindings
- **Task 3: Implement NAPI Bindings (`packages/bindings`)**
  - Steps: Expose Rust engine functions to Node.js via `napi-rs`. Handle Rust <-> JS serialization efficiently.
  - QA: `pnpm --filter bindings run build && node -e "require('./packages/bindings/index.js')"` executes without error.

### Wave 4: Integration Layers (Parallel)
- **Task 4: Implement Unplugin Wrapper (`packages/unplugin`)**
  - Steps: Build the universal bundler plugin invoking the NAPI bindings. Create `vitest` config and `test` script in its `package.json`.
  - QA: `pnpm --filter unplugin run test` passes (vitest).
- **Task 5: Implement MCP Server (`packages/mcp-server`)**
  - Steps: Build stdio server using `@modelcontextprotocol/sdk`. Expose AST analysis as Tools. Add `build` script (e.g. tsc or tsup) to output to `dist/index.js`.
  - QA: `pnpm --filter mcp-server run build && echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | node packages/mcp-server/dist/index.js` returns valid JSON.

### Wave 5: Finalization
- **Task 6: E2E Integration & CI Setup**
  - Steps: Write an E2E test script in the root `package.json` (`"test:e2e"`) verifying the flow from Node.js down to Rust. Setup standard napi-rs GitHub Actions CI for cross-platform binary building.
  - QA: `pnpm run test:e2e` passes.

## Final Verification Wave
- [x] Task 1 Completed (Rust 环境需要用户手动配置)
- [x] Task 2 Completed (使用 regex-based 解析 MVP)
- [ ] Task 3 Completed
- [ ] Task 4 Completed
- [ ] Task 5 Completed
- [ ] Task 6 Completed