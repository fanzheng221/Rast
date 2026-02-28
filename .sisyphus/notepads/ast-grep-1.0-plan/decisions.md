# Decisions

## [2026-02-27] Initial Setup

- Selected `ast-grep-1.0-plan` as active plan based on user's ast-grep technical requirement
- Worktree set to existing directory (no new worktree needed)
- Notepad system initialized for tracking learnings and decisions


## [2026-02-27] NodeTrait 统一抽象层设计

- 在 `crates/ast_engine/src/lib.rs` 新增 `NodeTrait`，统一暴露 `kind()`, `text()`, `span()`, `children()` 四个 API，屏蔽 oxc 强类型节点差异。
- 采用 `AstNode + AstNodeKind(enum)` 作为统一包装层：`AstNode` 持有 `source` 与节点引用，`AstNodeKind` 覆盖 Program / Statement / Declaration / Expression / Function / Class / CallExpression 等主要节点。
- 定义 `NodeSpan { start, end }` 作为稳定 span 返回类型，通过 `From<oxc::span::Span>` 转换，避免上层依赖底层实现细节。
- 新增 `IntoAstNode` trait，为 `Program`, `Statement`, `Declaration`, `Expression` 提供上转统一节点能力；同时在 `AstNode` 提供 `as_program/as_statement/as_declaration/as_expression` 实现向下取回强类型节点。
- `children()` 先覆盖高频语义路径（Program->Statement、Statement->Declaration/Expression、VariableDeclaration->init、CallExpression->callee/arguments、Class->MethodDefinition），保证匹配引擎可做向下遍历，并可逐步扩展到更多节点。


## [2026-02-27] Vue SFC 预处理器（script-only）设计决策

- 在 `crates/ast_engine/src/lib.rs` 新增 `VueSfcExtractor`，通过轻量手动解析识别 `<script>`、`<script setup>`、`<template>`、`<style>` block。
- 1.0 只提取第一个 script block（按源码顺序），不对 `<template>` / `<style>` 进行内容提取，仅记录 presence。
- 新增 `VueSfcOffsetMap` 保存 `relative_offset -> absolute_offset` 映射表（`Vec<u32>`）及反向映射能力，用于 `NodeSpan` 双向转换。
- 行列号映射直接基于原始 `.vue` 文件 `line_starts` 计算，确保 script 内相对偏移能回溯到文件绝对位置。
- parser 侧不改动 oxc，仅在预处理层负责 script 切片与 span 映射，符合 ast-grep-1.0 的阶段边界。

## [2026-02-27] TASK-1.3 通配符解析与 Pattern AST 设计决策

- 采用“后处理识别”方案：保持 oxc parser 原样，先将 pattern 解析为普通 AST，再在 AST -> Pattern AST 转换阶段识别元变量。
- 元变量命名规则：仅识别 `$<NAME>` / `$$$<NAME>`，其中 `<NAME>` 必须以大写字母或 `_` 开头，后续允许大写字母、数字、`_`。
- 通配符分类：
  - `PatternNodeKind::MetaVar(WildcardNode)` 对应单节点捕获（如 `$A`）
  - `PatternNodeKind::MultiMetaVar(WildcardNode)` 对应多节点捕获（如 `$$$A`）
  - `PatternNodeKind::MultiWildcard` 对应匿名多节点通配符（`$$$`）
- Pattern AST 结构：`PatternNode { kind, text, span, children }`，保留原节点文本与位置信息，便于后续匹配和替换阶段复用。
- `identify_meta_variables()` 返回 `HashMap<(start, end), PatternNodeKind>`，通过 span 精确标记需要在 Pattern AST 中提升为元变量节点的位置。
- `to_pattern_ast()` 先识别元变量，再递归构建整棵 Pattern AST，普通节点统一落到 `PatternNodeKind::Node { kind }`。


## [2026-02-27] TASK-1.4 CST/AST 等价性匹配器设计决策

- 在 `crates/ast_engine/src/lib.rs` 新增 `MatchEnvironment`，拆分为 `single_captures` 与 `multi_captures` 两类存储，统一承载 `$A` 与 `$$$A` 的捕获结果。
- 捕获值采用 `CapturedNode { kind, text, span }`，避免在环境中持有生命周期受限的 AST 引用，便于后续跨阶段复用（如重写模板替换）。
- `Matcher` trait 以 `match_node_with_env` 为核心 API，并补充 `match_node` 便捷方法；`PatternMatcher` 作为基础实现，支持直接对 `PatternNode` 与 `NodeTrait` 节点进行匹配。
- 匹配严格度采用 `MatchStrictness` 五种模式（`Ast/Relaxed/Cst/Signature/Template`），当前实现以 `Cst` 精确文本对比、其余模式使用去空白/去注释归一化文本，满足“空白符与注释可跳过”的基线需求。
- 子节点匹配使用回溯序列算法：对 `$$$` / `$$$A` 在子节点序列中做可变长度消费，并在分支失败时回滚环境，确保多节点捕获与后续节点约束可同时成立。
- 增加 `CompositeMatcher` 作为组合匹配基础设施，按顺序运行多个 matcher，并通过环境克隆/提交机制保证组合场景下的一致性。
- 新增单测覆盖：单/多元变量捕获、重复元变量一致性约束、Relaxed 模式空白/注释跳过、组合匹配器环境保持。
## [2026-02-27] CST/AST 等价匹配实现决策

- 在 `PatternMatcher` 中引入 `MatchStrictness` 分支策略：`Cst` 走严格文本全等，其余模式走归一化比较，`Signature` 额外按 `{` 截断以忽略函数/类体差异。
- 增加 `should_skip_trivia` 与 `should_skip_pattern_child`，将“空白/注释跳过”显式建模为匹配策略，避免散落在递归逻辑中。
- `Cst` 模式在 `match_regular_node` 前置 `pattern.text == target.text()` 检查，确保同结构但不同注释/空白不会误匹配。
- 保留元变量与多节点元变量捕获的一致性约束（重复引用必须文本等价），并与 strictness 文本比较规则共享同一判等路径。
- 新增 strictness 维度测试（Ast/Relaxed/Cst/Signature/Template）和 `$$$` 通配符匹配测试，确保匹配行为可回归。


## [2026-02-28] TASK-2.2 元变量捕获机制实现决策

- 在 `MatchEnvironment` 上补充查询接口：`get_single_capture/get_multi_capture/has_single_capture/has_multi_capture`，统一通过环境对象访问捕获结果，避免上层直接操作 HashMap。
- 新增 `MatchResult { matched, environment }`，并在 `Matcher` trait 中增加 `match_result()`，保留“是否匹配 + 捕获环境”完整结果；`match_node()` 改为基于 `match_result()` 的兼容包装。
- 在 `PatternMatcher::capture_single` 与 `capture_multi` 中增加“同名单/多捕获互斥”约束：若 `$A` 与 `$$$A` 混用则直接匹配失败，避免环境歧义。
- 重复元变量一致性沿用 ast-grep 语义：同名 `$A` 要求捕获节点内容等价；同名 `$$$A` 要求序列长度一致且逐项内容等价。
- 新增单测覆盖 `MatchResult` 查询接口、重复 `$$$A` 一致性、单/多同名混用失败，确保捕获行为可回归。


## [2026-02-28] TASK-2.3 重叠匹配与冲突解决设计决策

- 将原有 `Matcher::match_result` 返回类型重命名为 `MatchOutcome`，保留“是否匹配 + 环境”语义，并释放 `MatchResult` 名称用于“单个匹配项结果”结构。
- 新增 `MatchResult { span, environment }` 作为 `find_all_matches` 的输出单元，直接携带匹配节点 `NodeSpan` 与捕获环境，便于后续替换阶段使用。
- 新增 `ConflictResolution` 策略枚举（`PreferOuter` / `PreferInner`），通过候选排序优先级控制冲突决策：外层优先按 span 长度降序，内层优先按 span 长度升序。
- `find_all_matches` 采用“先全量收集、后冲突裁剪”的两阶段流程：递归遍历整棵 AST 收集候选，再按策略排序并贪心剔除重叠项，最后按源码位置排序并去重，输出稳定结果。
- 提供 `FindAllMatches` 兼容入口（包装 `find_all_matches`），满足计划中的命名要求，同时保持 Rust snake_case 主 API。
- 单测补充三类回归：重叠场景下外层优先、内层优先，以及非重叠场景的排序与捕获正确性，确保冲突策略与结果稳定性可验证。


## [2026-02-28] TASK-2.4 YAML Schema（sgconfig.yml）设计决策

- 在 `crates/ast_engine/src/lib.rs` 新增 `RuleCore` 作为 `sgconfig.yml` 根模型，字段固定为 `id/language/rule`，并通过 `pub type SgConfig = RuleCore` 提供配置语义别名。
- `rule` 字段采用两层建模：`Rule`（透明包装，预留未来复合规则扩展）+ `RuleKind`（当前仅原子规则），在保持当前简洁 YAML 结构的同时避免后续破坏性重构。
- 原子规则拆分为 `PatternAtomicRule` / `RegexAtomicRule` / `KindAtomicRule`，并在每个原子结构上使用 `#[serde(deny_unknown_fields)]`，确保 `pattern/regex/kind` 互斥且不接受混合键。
- 语言标识使用 `RuleLanguage` 枚举并开启 `rename_all = "lowercase"`，明确支持 `js/jsx/ts/tsx/javascript/typescript`，覆盖 JS/TS 常见标识。
- 新增 `RuleCore::from_yaml` 作为统一反序列化入口，使用 `serde_yaml::from_str`，并在 `crates/ast_engine/Cargo.toml` 引入 `serde_yaml` 依赖。
- 单测新增四类覆盖：`pattern` 解析、`regex` 解析、`kind` 解析、以及多原子键混用失败，验证 schema 解析行为与约束。


## [2026-02-28] TASK-2.5 组合规则（all/any/not）实现决策

- 在 `RuleKind` 中扩展 `All/Any/Not` 三个组合变体，并新增 `AllCompositeRule/AnyCompositeRule/NotCompositeRule`（均启用 `#[serde(deny_unknown_fields)]`），保持原子规则与组合规则都走同一 `Rule -> RuleKind` 递归模型。
- 组合规则采用 `Vec<Rule>` 与 `Box<Rule>` 建模（`all: Vec<Rule>`, `any: Vec<Rule>`, `not: Box<Rule>`），使 YAML 能自然表达嵌套组合，且与后续规则评估阶段直接对接。
- 基于既有 `Matcher` trait 新增 `AllMatcher/AnyMatcher/NotMatcher`：
  - `AllMatcher`：所有子 matcher 必须成功，按顺序累积并提交环境。
  - `AnyMatcher`：逐分支在基线环境上试配，首个成功分支提交环境，失败分支不污染环境。
  - `NotMatcher`：在环境副本上评估内部 matcher，仅返回逻辑取反结果，不写回捕获。
- 保留 `CompositeMatcher` 作为兼容层，内部委托到 `AllMatcher`，避免现有调用点和测试回归。
- 单测新增两组覆盖：
  - YAML 解析：`all/any/not` 成功解析 + 多组合键混用失败。
  - 组合匹配：`all` 全匹配约束、`any` 分支隔离、`not` 反向匹配且环境不变。
