# Rast 1.0 产品与架构演进规划 (对标 ast-grep)

## 1. 产品能力对标分析 (Product Gap Analysis)
- **当前状态 (MVP)**: 基于 oxc 提供单次/基础的 JS/TS AST 提取、导入/导出分析、简单函数调用图。不支持模式匹配与自动代码重写。
- **目标状态 (1.0)**: 构建前端专用的高性能代码搜索与重写引擎，对标 ast-grep。支持结构化搜索 (`console.log($A)` -> `logger.info($A)`)、YAML 声明式 Lint 规则系统、Codemod 重写。
- **语言支持扩展**: 深度聚焦前端生态，除了基础的 JS/TS 外，重点突破 JSX 和 Vue SFC (单文件组件) 的结构化解析与匹配。

## 2. 架构决策 (Architecture Decisions)
- **核心匹配引擎**: 摒弃 tree-sitter，全面基于 `oxc` AST 原生实现一套轻量级的 CST/AST 模式匹配算法 (支持 `$` 通配符)。
- **统一节点抽象层 (Node Trait)**: 在 oxc 强类型 AST 之上构建统一抽象，使其支持类似 tree-sitter 的 `kind()`, `text()`, `children()` 遍历，抹平 oxc 强类型带来的多态遍历难题。
- **规则配置系统**: 设计 Rast 专属的 YAML Schema (不直接复用 ast-grep 规则，因为底层节点命名不同)，但结构保持一致 (`id`, `language`, `rule`, `fix`)。
- **代码重写机制 (Find & Patch)**: 基于精确的 Span 进行文本替换。修改后的格式化交给外部工具 (Prettier/Biome/oxc_fmt)，Rast 只做安全的字符串级别 Patch。
- **前端扩展 (Vue)**: 实现预处理器 (Preprocessor) 架构，提取 `.vue` 文件中的 `<script>` 和 `<script setup>` 内容及偏移量，交由 oxc 处理后，再映射回原文件绝对位置进行 Patch (1.0 明确不支持 `<template>` 的结构化匹配)。

## 3. 护栏与范围控制 (Guardrails & Scope)
- **IN (包含)**: 
  - JS/TS/JSX 原生解析与 Codemod。
  - Vue SFC 的 script/script setup 解析与绝对位置映射重写。
  - 核心 CLI 工具与 Rust Library。
  - 基础规则支持 (atomic rules: `pattern`, `regex`, `kind`; composite rules: `all`, `any`, `not`; 常用 relational: `inside`, `has`)。
- **OUT (排除)**:
  - Vue `<template>` 或 `<style>` 的 AST 匹配。
  - 其他语言 (Rust, Python, Go) 的支持。
  - 编辑器 LSP 集成 (推迟到 2.0)。
  - 高成本关联规则 (`follows`, `precedes`)。
  - 复杂的自动代码格式化缩进调整 (Rast 仅做基于 Span 的精确替换)。

## 4. 边缘情况处理策略 (Edge Cases)
- **重叠匹配 (Overlapping Matches)**: 实现类似 ESLint 的冲突解决机制，跳过重叠的子匹配。
- **JSX Fragment**: 明确 JSX 匹配的等价性规则，处理 `{foo}` 与文本节点的差异。
- **多行通配符与空白符**: 对于 `$$$` 变长匹配，定义严格的 Trivia (空白/注释) 吸附策略，避免删除节点后遗留空行。


## 5. 里程碑与任务拆解 (Milestones & Tasks)

### Phase 1: 核心节点抽象层与解析引擎 (Node Abstraction & Engine)
1. **[TASK-1.1]** 设计统一的 `NodeTrait`：包装 `oxc` 强类型节点，暴露统一的 `kind()`, `text()`, `span()`, `children()` API。
   - _QA Scenario_: 解析一段 JS 代码，断言任意节点都能向上/向下转换为统一形式，获取正确的 span 和文本。
2. **[TASK-1.2]** Vue SFC 预处理器管道实现：编写正则/提取器从 `.vue` 中分离 `<script>`，并建立相对偏移量 (`offset`) 到文件绝对 `Span` 的双向映射映射表。
   - _QA Scenario_: 输入包含 `<template>` 的 `.vue` 文件，成功提取脚本内容给 `oxc` 解析，返回结果包含正确的原始文件行列。
3. **[TASK-1.3]** 构建通配符 `$A` 解析与 AST 模式树的生成逻辑：使 `oxc` 可以解析包含 `$A` 的代码模式并生成一棵"模式 AST" (Pattern AST)。
   - _QA Scenario_: 传入 `console.log($MSG)`，成功构造一棵包含通配符节点 (`WildcardNode("MSG")`) 的模式树。

### Phase 2: 结构化模式匹配引擎 (Structural Matching)
1. **[TASK-2.1]** 实现 CST/AST 等价性匹配算法：基于 `NodeTrait` 对比目标 AST 和模式 AST，处理空白符跳过。
   - _QA Scenario_: `console.log($A)` 应匹配到 `console.log ( "hello" )`，无论其中有多少换行与空格。
2. **[TASK-2.2]** 实现元变量捕获机制：在匹配过程中，收集 `$A` 对应的真实 AST 节点，记录其环境 (`Environment`)。
   - _QA Scenario_: 匹配 `function $A($$$ARGS)`，环境字典中应包含 `A: Identifier("foo")`, `ARGS: [Param1, Param2]`。
3. **[TASK-2.3]** 实现重叠匹配与冲突解决模块：基于 AST 遍历收集所有匹配项，剔除子节点重叠的匹配。
   - _QA Scenario_: 匹配任意函数调用，当存在 `a(b())` 时，仅返回外层或内层匹配（取决于配置），不能产生冲突重叠。

### Phase 3: 声明式规则系统 (YAML Rule System)
1. **[TASK-3.1]** 定义 Rast专属的 YAML Schema (`sgconfig.yml` 及其规则模型)：支持 `id`, `language`, `rule` (atomic: `pattern`/`regex`/`kind`)。
   - _QA Scenario_: 编写 `rule: { pattern: "eval($A)" }`，成功反序列化为 Rust 的 RuleCore 对象。
2. **[TASK-3.2]** 实现基础组合规则 (Composite Rules)：实现 `all`, `any`, `not` 的匹配逻辑组合。
   - _QA Scenario_: `all: [{pattern: "console.log($A)"}, {not: {regex: "warn"}}]` 正确拒绝 `console.warn`。
3. **[TASK-3.3]** 实现关联过滤规则 (Relational Rules)：实现基于当前节点的向上/向下过滤 `inside`, `has`。
   - _QA Scenario_: `rule: { pattern: "return $A", inside: { kind: "arrow_function" } }` 仅匹配箭头函数内部的 return。

### Phase 4: 代码重写引擎 (Codemod & Patch)
1. **[TASK-4.1]** 基础文本插值与替换 (`fix`)：利用收集到的 Environment 元变量，在 `fix` 模板中进行 `$A` 文本替换。
   - _QA Scenario_: `fix: "logger.info($A)"` 正确提取真实参数替换到字符串中。
2. **[TASK-4.2]** 构建精确的 Span 变异处理器 (Mutator)：通过文件原始内容 + `Span` + `fix` 生成的结果字符串，构造并应用文本 Diff。
   - _QA Scenario_: 对包含大量乱序换行的原始代码应用 Patch 后，未修改部分保持字节级一致。
3. **[TASK-4.3]** `$$$` 多节点替换的 Trivia (空白符) 吸附：删除连续节点时，自动吸收相邻的一个空白缩进/换行，防止残留空行。
   - _QA Scenario_: 删除多行函数调用参数后，格式应当自动紧凑，不留孤立的空行。

### Phase 5: 接口层封装 (CLI & NAPI Bindings)
1. **[TASK-5.1]** 升级 `napi-rs` 绑定，暴露 `@rast/napi` 的可变 API (如 `node.replace()`, `node.commitEdits()`)。
   - _QA Scenario_: 从 Node.js 脚本中调用 Rast 执行一次针对 TS 项目的批量 Find & Patch。
2. **[TASK-5.2]** Rast CLI 构建：实现 `rast run` 单次查找替换命令和 `rast scan` 扫描规则目录命令。
   - _QA Scenario_: 终端运行 `rast run -p 'var $A' -r 'let $A'`，成功重写项目内文件。
3. **[TASK-5.3]** MCP Server 扩展集成：更新 `mcp-server` 的工具，暴露 `findPattern` 和 `applyRule` 接口给 AI 调用。
   - _QA Scenario_: Claude 桌面端通过 MCP 成功结构化搜索 Rast 仓库内的 `Result<T>` 用法。
## 6. 最终验证波次 (Final Verification Wave)
1. **[VERIFY-1]** 性能基准测试: `rast scan` 在 10000 个文件的 JS/TS 项目中扫描 10 条复杂规则，确保耗时优于 ast-grep。
2. **[VERIFY-2]** Vue 集成测试: 对数十个真实的 Vue 3 SFC 项目运行规则，验证偏移量修正完全正确。
3. **[VERIFY-3]** NAPI & MCP 端到端: 使用编写好的 `@rast/napi` 脚本，以及通过 AI 触发 MCP 命令，完整跑通 Codemod。
## 7. 高精度审查补充 (Momus Review Additions)
- **NodeTrait 优化**: 强烈建议直接利用 `oxc_ast::AstKind` 和 `oxc_semantic::AstNode` 来实现统一节点抽象。不要从零手写 Trait Object，而是基于 `AstKind` 暴露 `kind()`, `span()`，并考虑使用 Rust 宏 (Procedural Macro) 自动生成 `children()` 的遍历代码。
- **通配符解析机制**: `$A` 和 `$$$` 是合法的 JS Identifier，这意味着**完全不需要修改 oxc parser**。只需在生成模式 AST 时，将特定命名的 Identifier 标记为元变量节点即可。
- **JSX 等价性 (JSX Equivalence)**: 需要专门处理 JSX 属性的等价匹配（如 `<div id=$A>` 匹配 `<div id="foo">` 和 `<div id={"foo"}>`）。
- **并发安全性**: Codemod 在 CLI 层面是多线程操作的，必须保证多文件并发 Patch 的原子性，防止中途退出导致的文件损坏。
