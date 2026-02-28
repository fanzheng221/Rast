# Rast 1.0 产品与架构演进规划 (对标 ast-grep)

## Context
Upgrading the Rast MVP to a 1.0 architecture with ast-grep parity. This includes a structural matching engine, YAML declarative rules, Codemod rewriting capabilities, and Vue SFC support. The plan has been reformatted to meet the Sisyphus complete executable plan checklist format, ensuring all tasks have strict dependencies, assignees, effort estimates, and executable QA criteria.

## Task Dependency Graph

| Task | Depends On | Reason |
|------|------------|--------|
| TASK-1.1 | None | Foundational NodeTrait abstraction required for all AST operations. |
| TASK-1.2 | None | Vue SFC preprocessor is independent of the core matching engine. |
| TASK-1.3 | TASK-1.1 | Wildcard parsing needs the NodeTrait to represent pattern nodes. |
| TASK-2.1 | TASK-1.1, TASK-1.3 | Equivalence matching compares NodeTraits and Pattern ASTs. |
| TASK-2.2 | TASK-2.1 | Metavariable capture happens during the matching process. |
| TASK-2.3 | TASK-2.1 | Overlap resolution requires the base matching engine to find all candidates. |
| TASK-3.1 | None | YAML schema definition is independent of the engine implementation. |
| TASK-3.2 | TASK-3.1, TASK-2.1 | Composite rules combine base matching logic with YAML definitions. |
| TASK-3.3 | TASK-3.1, TASK-1.1 | Relational rules require NodeTrait traversal and YAML definitions. |
| TASK-4.1 | TASK-2.2 | Text interpolation requires captured metavariables. |
| TASK-4.2 | TASK-4.1 | Span mutator applies the interpolated text to the source code. |
| TASK-4.3 | TASK-4.2 | Trivia absorption is an enhancement on top of the base mutator. |
| TASK-5.1 | TASK-4.2 | NAPI bindings expose the mutator capabilities to Node.js. |
| TASK-5.2 | TASK-4.2, TASK-3.1 | CLI requires the mutator and YAML rule parser. |
| TASK-5.3 | TASK-5.1 | MCP server uses the NAPI bindings to expose tools. |

## Tasks

### Wave 1 (Start Immediately - No Dependencies)
|- [x] **[TASK-1.1] NodeTrait Abstraction**
  - _Description_: Design unified `NodeTrait` wrapping `oxc` nodes, exposing `kind()`, `text()`, `span()`, `children()`.
  - _Status_: TODO
  - _Dependencies_: []
  - _Assignee_: Auto (ultrabrain)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test node_trait_tests`
|- [x] **[TASK-1.2] Vue SFC Preprocessor**
  - _Description_: Extract `<script>` from `.vue` files and build a bidirectional offset mapping table.
  - _Status_: TODO
  - _Dependencies_: []
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1d
  - _QA Scenario_: `cargo test -p ast_engine --test vue_sfc_tests`
|- [x] **[TASK-3.1] YAML Schema Definition**
  - _Description_: Define `sgconfig.yml` schema and deserialize into Rust `RuleCore` objects.
  - _Status_: TODO
  - _Dependencies_: []
  - _Assignee_: Auto (quick)
  - _Effort_: 1d
  - _QA Scenario_: `cargo test -p ast_engine --test yaml_schema_tests`

### Wave 2 (After Wave 1 Completes)
|- [x] **[TASK-1.3] Wildcard Parsing**
  - _Description_: Enable `oxc` to parse `$A` and `$$$` into a Pattern AST. Handle invalid identifier contexts.
  - _Status_: TODO
  - _Dependencies_: [TASK-1.1]
  - _Assignee_: Auto (deep)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test wildcard_parsing_tests`
- [ ] **[TASK-3.3] Relational Rules**
  |- [x] **[TASK-3.3] Relational Rules**
  - _Status_: TODO
  - _Dependencies_: [TASK-3.1, TASK-1.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 2.5d
  - _QA Scenario_: `cargo test -p ast_engine --test relational_rules_tests`

### Wave 3 (After Wave 2 Completes)
- [ ] **[TASK-2.1] Equivalence Matching**
  - _Description_: Implement CST/AST equivalence matching algorithm comparing target AST and Pattern AST.
  - _Status_: TODO
  - _Dependencies_: [TASK-1.1, TASK-1.3]
  - _Assignee_: Auto (ultrabrain)
  - _Effort_: 3d
  - _QA Scenario_: `cargo test -p ast_engine --test equivalence_matching_tests`

### Wave 4 (After Wave 3 Completes)
- [ ] **[TASK-2.2] Metavariable Capture**
  - _Description_: Collect `$A` nodes into an Environment dictionary during matching.
  - _Status_: TODO
  - _Dependencies_: [TASK-2.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test metavariable_capture_tests`
- [ ] **[TASK-2.3] Overlap Resolution**
  - _Description_: Implement conflict resolution to skip overlapping child matches.
  - _Status_: TODO
  - _Dependencies_: [TASK-2.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1.5d
  - _QA Scenario_: `cargo test -p ast_engine --test overlap_resolution_tests`
- [ ] **[TASK-3.2] Composite Rules**
  - _Description_: Implement `all`, `any`, `not` logic combinations.
  - _Status_: TODO
  - _Dependencies_: [TASK-3.1, TASK-2.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test composite_rules_tests`

### Wave 5 (After Wave 4 Completes)
- [ ] **[TASK-4.1] Text Interpolation**
  - _Description_: Replace `$A` in `fix` templates using the captured Environment.
  - _Status_: TODO
  - _Dependencies_: [TASK-2.2]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1.5d
  - _QA Scenario_: `cargo test -p ast_engine --test text_interpolation_tests`

### Wave 6 (After Wave 5 Completes)
- [ ] **[TASK-4.2] Span Mutator**
  - _Description_: Generate and apply text diffs using Spans. MUST apply patches in reverse order (descending span end index) to prevent offset shifting.
  - _Status_: TODO
  - _Dependencies_: [TASK-4.1]
  - _Assignee_: Auto (ultrabrain)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test span_mutator_tests`

### Wave 7 (After Wave 6 Completes)
- [ ] **[TASK-4.3] Trivia Absorption**
  - _Description_: Automatically absorb adjacent whitespace/newlines when deleting nodes.
  - _Status_: TODO
  - _Dependencies_: [TASK-4.2]
  - _Assignee_: Auto (deep)
  - _Effort_: 2d
  - _QA Scenario_: `cargo test -p ast_engine --test trivia_absorption_tests`
- [ ] **[TASK-5.1] NAPI Bindings Update**
  - _Description_: Expose mutator API to Node.js via `napi-rs`.
  - _Status_: TODO
  - _Dependencies_: [TASK-4.2]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1.5d
  - _QA Scenario_: `pnpm --filter bindings run test`
- [ ] **[TASK-5.2] CLI Build**
  - _Description_: Implement `rast run` and `rast scan` with `--dry-run` mode.
  - _Status_: TODO
  - _Dependencies_: [TASK-4.2, TASK-3.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1.5d
  - _QA Scenario_: `cargo test -p rast_cli`

### Wave 8 (After Wave 7 Completes)
- [ ] **[TASK-5.3] MCP Server Extension**
  - _Description_: Expose `findPattern` and `applyRule` tools in the MCP server.
  - _Status_: TODO
  - _Dependencies_: [TASK-5.1]
  - _Assignee_: Auto (unspecified-high)
  - _Effort_: 1d
  - _QA Scenario_: `pnpm --filter mcp-server run test`

## 6. 最终验证波次 (Final Verification Wave)
- [ ] **[VERIFY-1]** 性能基准测试: `rast scan` 在 10000 个文件的 JS/TS 项目中扫描 10 条复杂规则，确保耗时优于 ast-grep。
- [ ] **[VERIFY-2]** Vue 集成测试: 对数十个真实的 Vue 3 SFC 项目运行规则，验证偏移量修正完全正确。
- [ ] **[VERIFY-3]** NAPI & MCP 端到端: 使用编写好的 `@rast/napi` 脚本，以及通过 AI 触发 MCP 命令，完整跑通 Codemod。

## 7. 高精度审查补充 (Momus Review Additions)
- **NodeTrait 优化**: 强烈建议直接利用 `oxc_ast::AstKind` 和 `oxc_semantic::AstNode` 来实现统一节点抽象。不要从零手写 Trait Object，而是基于 `AstKind` 暴露 `kind()`, `span()`，并考虑使用 Rust 宏 (Procedural Macro) 自动生成 `children()` 的遍历代码。
- **通配符解析机制**: `$A` 和 `$$$` 是合法的 JS Identifier，这意味着**完全不需要修改 oxc parser**。只需在生成模式 AST 时，将特定命名的 Identifier 标记为元变量节点即可。
- **JSX 等价性 (JSX Equivalence)**: 需要专门处理 JSX 属性的等价匹配（如 `<div id=$A>` 匹配 `<div id="foo">` 和 `<div id={"foo"}>`）。
- **并发安全性**: Codemod 在 CLI 层面是多线程操作的，必须保证多文件并发 Patch 的原子性，防止中途退出导致的文件损坏。
