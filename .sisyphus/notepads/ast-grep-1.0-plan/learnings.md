# Learnings

## [2026-02-27] Session Start

### Initial Context
- Plan: ast-grep-1.0-plan (对标 ast-grep 的 Rast 1.0)
- Worktree: /Users/smart/Desktop/Rast (main branch)
- Session: ses_361405cc7ffeSLD6WgH1XgVumM

### Key Requirements
- Use deepwiki mcp service for ast-grep technical details
- Do not rely on intuition - verify with deepwiki results
- Implement CST/AST pattern matching using oxc
- Support JS/TS/JSX and Vue SFC (script only)

### Serena Constraints
- Use only `serena_*` tools for file operations
- No native glob/grep/read tools


## [2026-02-27] 架构探索发现

### 当前项目状态总结

#### 1. crates/ast_engine (Rust 核心引擎)
- 使用 oxc 进行 AST 解析和语义分析
- 实现了基本的 AST Visitor 模式
- 已实现功能：
  - analyze_ast() - 单文件 AST 分析
  - ProjectGraph - 有状态项目图，支持跨文件依赖
  - 提取 exports, symbols, classes, interfaces, type aliases
  - 提取 imports 和 dependencies
  - 构建 call graph
  - 基本 lint 规则（检测 var 和 console.log）
  - JSDoc 和 comments 提取
- **缺失功能**（ast-grep 对标需要）：
  - ❌ 没有统一节点抽象层
  - ❌ 没有通配符（$A, $$$）支持
  - ❌ 没有结构化模式匹配引擎
  - ❌ 没有 Vue SFC 预处理器
  - ❌ 没有代码重写/patch 功能
  - ❌ 没有声明式规则系统（YAML）

#### 2. packages/bindings (NAPI 绑定)
- 使用 napi-rs 暴露 Rust 功能到 Node.js
- 已暴露 API：
  - initialize_graph(mode: String)
  - add_file(path, code)
  - get_file_structure(path)
  - get_symbol_details(symbol)
  - analyze_dependencies(paths)
- **缺失功能**：
  - ❌ 没有代码重写相关的 API

#### 3. packages/mcp-server (MCP 服务器)
- 基于 @modelcontextprotocol/sdk 实现 stdio 服务器
- 已暴露工具：
  - analyze_ast
  - get_file_structure
  - get_symbol_details
  - analyze_dependencies
- **缺失工具**：
  - ❌ find_pattern - 结构化模式搜索
  - ❌ apply_rule - 应用规则

#### 4. packages/unplugin (构建器插件)
- 支持 cache 和 on-demand 模式
- 当前仅支持 AST 分析和 lint 注入
- **缺失功能**：
  - ❌ 不支持代码重写
  - ❌ 没有模式搜索

### 技术栈分析
- Rust: oxc (已使用) ✅
- Node.js: napi-rs (已使用) ✅
- 需要实现的核心技术：
  - 统一节点抽象 (NodeTrait)
  - 通配符解析和模式 AST 生成
  - CST/AST 等价性匹配算法
  - 元变量捕获机制
  - 代码重写和 patch 应用

### 结论
当前项目是 MVP 版本（基于 rast-ast-tool.md 计划），已完成基本的 AST 分析能力。
ast-grep-1.0-plan 需要在此基础上扩展，新增：
1. 模式匹配引擎
2. 规则系统
3. 代码重写能力
4. Vue SFC 支持


## [2026-02-27] ast-grep 技术细节研究

### 来自 deepWiki 的核心发现

#### 1. Pattern AST 构建
- Pattern::new() 接受 pattern 字符串和 Language
- Language::pre_process_pattern() 对 pattern 进行预处理（例如 Rust 将 $ 替换为 µ）
- 解析 pattern 字符串为 Tree-sitter AST
- extract_var_from_node() 识别元变量
- PatternNode::MetaVar 枚举区分：
  - 单节点捕获（$A）
  - 多节点捕获（$$$A）
  - 多节点通配符（$$$）

#### 2. 匹配算法
- Matcher trait 定义 match_node_with_env() 方法
- 模式 AST 和目标 AST 的比较
- 匹配时将元变量捕获到 MetaVarEnv
- MatchStrictness 设置影响匹配严格程度

#### 3. 元变量捕获
- MetaVarEnv 存储单节点匹配和多节点匹配
- 如果同一元变量在 pattern 中出现多次，必须在目标 AST 中匹配相同的内容

#### 4. 代码重写
- 使用 fix 模板字符串，可以包含元变量
- gen_replacement() 函数负责生成替换字符串
- create_template() 解析 fix 模板
- generate_replacement() 将 MetaVarEnv 中捕获的元变量替换到模板
- 处理缩进以确保重写后的代码保持正确格式化
- Root::replace() 方法编排整个查找-替换操作

### 对 Rast 实现的启示

1. **需要实现的组件**：
   - 统一节点抽象层 - 对应 ast-grep 的 Node trait
   - 通配符解析和模式 AST 生成
   - 模式匹配引擎（将 tree-sitter 匹配算法适配到 oxc）
   - 元变量捕获机制
   - 代码重写引擎

2. **关键区别**：
   - ast-grep 使用 tree-sitter，Rast 将使用 oxc
   - 需要在 oxc 强类型 AST 之上构建统一抽象
   - $ 和 $$$ 是合法的 JS Identifier，不需要修改 oxc parser

