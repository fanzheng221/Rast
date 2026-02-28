# 全局工作指南

## 回答风格

- 回答必须使用中文。
- 对总结、计划、任务拆解、以及长内容输出，先做逻辑整理，再优先使用表格呈现。
- 普通简短内容可直接自然语言回答，不强制表格。

# Serena 强制规范（稳定缓存版）

目标：在本项目中统一使用 Serena 工具链，减少规则重复与上下文波动。

## 必须遵守

1. 优先符号级操作：`serena_find_symbol`、`serena_replace_symbol_body`、`serena_insert_*`。
2. 无法符号定位时使用 `serena_replace_content` 做文件级正则修改。

## 推荐读取顺序

1. 结构定位：`serena_list_dir`、`serena_find_file`
2. 语义定位：`serena_get_symbols_overview`、`serena_find_symbol`、`serena_find_referencing_symbols`
3. 跨文件搜索：`serena_search_for_pattern`
4. 仅在需要完整行内容时使用 `serena_read_file`

## Memory 同步（架构/规范变更时）

1. 架构或核心依赖变化 → 更新 `project_overview`
2. 命令/Docker/运维脚本变化 → 更新 `suggested_commands`
3. 代码规范/命名/测试标准变化 → 更新 `style_conventions`

# ⚡ 子 Agent 及并行任务强制约束 (CRITICAL)

由于子 Agent (通过 `task()` 启动的 `explore`, `librarian`, `deep` 等) 默认不具备主会话的上下文，它们倾向于使用原生的 `glob` / `grep` / `read` 等工具。这**严重违反**了 Serena 的全局使用规范。

## 1. 禁用一切原生操作工具
**在任何 Session、任何 Agent (包含 Orchestrator、子 Agent、并行任务) 中，绝对禁止：**
- ❌ 使用 `glob` / `grep` / `rg` / `find`
- ❌ 使用 `ast_grep_search` / `ast_grep_replace`
- ❌ 使用 `bash` 来读写或搜索文件 (`cat`, `sed`, `awk` 等)

## 2. 委托 Task 时的强制 Prompt 注入
当你使用 `task()` 启动任何 Agent 时，**必须**在 `prompt` 中注入以下 Serena 强制指令，并且在可能的情况下将必要的 memory 一并提供或要求子 Agent 去读取：

```markdown
[SERENA MANDATORY PROTOCOL]
1. 禁用原生工具：严禁使用默认的 read, edit, write, glob, grep, ast_grep 工具。
2. 强制 Serena 工具链：
   - 文件/内容搜索：必须且只能使用 `serena_find_file`, `serena_search_for_pattern`
   - 代码/符号定位：必须且只能使用 `serena_get_symbols_overview`, `serena_find_symbol`
   - 代码修改：必须且只能使用 `serena_replace_content`, `serena_replace_symbol_body`, `serena_insert_after_symbol` 等
3. 读取记忆：任务开始前，必须先使用 `serena_read_memory` 读取当前环境的规范记忆 (如 style_conventions, project_overview)。
```

**示例 (`task` 调用方式)：**
```typescript
task(
  subagent_type="explore",
  run_in_background=true,
  load_skills=[],
  description="Search for existing AST traversal patterns",
  prompt="[SERENA MANDATORY PROTOCOL] 严禁使用原生 glob/grep，仅使用 serena_search_for_pattern 和 serena_find_symbol！\n\n 请搜索代码库中 AST 遍历的模式，并先读取 style_conventions 记忆以符合代码规范..."
)
```

## 3. 全局规范与记忆联动
每次执行代码编写或架构更改，无论是 Orchestrator 还是子 Agent，都必须首先通过 `serena_read_memory` 确认：
- 代码是否符合 `style_conventions`
- 架构是否破坏了 `project_overview`
- 只有确认这些内存记录后，才能执行相应的 `serena_*` 修改工具。
