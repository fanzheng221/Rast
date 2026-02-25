# Work Plan: Rast AST Analysis Tool v2 (AI Knowledge Engine)

## 1. Goal
Upgrade Rast to v2, transforming it from a simple linter into a deep codebase analysis engine powered by `oxc`. It will serve as an "AI-Native Bundler Plugin & Codebase Oracle," extracting rich, structured context (AST, module graphs, signatures, comments) to enable precise AI interactions via the MCP server.

## 2. Core Directives & Architecture
- **Serena Toolchain**: ALL file modifications must be done using `serena_*` tools.
- **Rust Engine Upgrade (`crates/ast_engine`)**: Completely replace regex-based parsing with `oxc::allocator::Allocator`, `oxc::parser::Parser`, and `oxc::semantic::SemanticBuilder`.
- **NAPI Bindings (`packages/bindings`)**: Expand to support stateful project graphs and complex queries, enabling both "Cache" and "On-Demand" modes.
- **Unplugin (`packages/unplugin`)**: Implement configuration options to drive the Rust engine's parsing strategy during the build step.
- **MCP Server (`packages/mcp-server`)**: Expose specialized tools for AI context gathering.

## 3. Scope Boundaries
**IN SCOPE (v2):**
- Deep AST parsing for JS/TS using `oxc`.
- Extraction of: Imports/Exports (Dependency Graph), Type Signatures & Interfaces, JSDoc & Comments, Method Call Relationships.
- Framework-specific abstraction (React/Vue component props/state).
- Two Operational Modes (Configurable in Unplugin):
  1. **Cache Mode (Pre-warm)**: Build a project-wide dependency and symbol graph in memory during the build process.
  2. **On-Demand Mode**: Parse files dynamically when requested by the MCP server.
- New MCP Tools: `get_file_structure`, `get_symbol_details`, `analyze_dependencies`.

**OUT OF SCOPE (v2):**
- Complex AST code mutations (source map generation and rewriting are deferred to v3, as the focus is now on read-only deep analysis).
- HTTP/SSE transport for MCP (keeping stdio for broad editor compatibility).

## 4. Execution Waves

### Wave 1: Rust Engine Core Overhaul (`crates/ast_engine`)
- **Task 1: Integrate `oxc` Parser and Semantic Analysis**
  - Steps: Remove regex parsing. Implement `oxc_allocator`, `oxc_parser`, and `oxc_semantic`. Define robust Rust structs to represent `FileStructure`, `SymbolSignature`, `DependencyInfo`, and `CallGraph`.
  - QA: `cargo test -p ast_engine` passes with complex TS examples.

- **Task 2: Build the Global Project Graph State**
  - Steps: Implement a thread-safe, stateful struct in Rust (e.g., `ProjectGraph`) that can store parsed file data, resolve cross-file dependencies, and maintain the cache.
  - QA: Unit tests verify adding files and querying cross-file relationships.

### Wave 2: FFI Bindings Expansion (`packages/bindings`)
- **Task 3: Expose Stateful Engine to Node.js**
  - Steps: Update `napi-rs` bindings to expose the `ProjectGraph` instance. Create methods for `initialize_graph(mode: String)`, `add_file(path, code)`, `get_file_structure(path)`, `get_symbol_details(symbol)`, and `analyze_dependencies(paths)`.
  - QA: Node.js scripts can instantiate the graph and query specific file abstractions.

### Wave 3: Plugin Integration (`packages/unplugin`)
- **Task 4: Implement Cache vs. On-Demand Modes**
  - Steps: Update the unplugin options to accept `{ mode: 'cache' | 'on-demand' }`. 
    - In `cache` mode: Intercept all file reads during the bundler's resolution phase and feed them to the Rust `ProjectGraph`.
    - In `on-demand` mode: Initialize the Rust engine but only pass file paths when queried.
  - QA: Vitest confirms the unplugin correctly configures the Rust engine based on options.

### Wave 4: AI Interface (`packages/mcp-server`)
- **Task 5: Implement Core MCP Tools for AI Context**
  - Steps: Connect the MCP Server to the stateful Rust engine bindings. Register and implement:
    1. `get_file_structure`: Returns signatures, exports, imports, and JSDoc.
    2. `get_symbol_details`: Returns full implementation details and call context for a specific symbol.
    3. `analyze_dependencies`: Returns the call graph and module dependencies.
  - QA: Testing the stdio interface with valid JSON-RPC requests returns expected structural abstractions.

### Wave 5: Finalization & Integration
- **Task 6: E2E Verification & CI Updates**
  - Steps: Update `scripts/e2e-test.js` to test the new "Codebase Oracle" workflow (parsing a complex file and querying its structure via MCP). Ensure CI pipelines cover the stateful NAPI lifecycle.
  - QA: `pnpm run test:e2e` passes.

## Final Verification Wave
- [x] Task 1 Completed
- [x] Task 2 Completed
- [ ] Task 3 Completed
- [ ] Task 4 Completed
- [ ] Task 5 Completed
- [ ] Task 6 Completed