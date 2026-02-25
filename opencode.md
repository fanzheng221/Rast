# 全局工作指南

## 回答风格

- 回答必须使用中文。
- 对总结、计划、任务拆解、以及长内容输出，先做逻辑整理，再优先使用表格呈现。
- 普通简短内容可直接自然语言回答，不强制表格。

# Serena 强制规范（稳定缓存版）

目标：在本项目中统一使用 Serena 工具链，减少规则重复与上下文波动。

## 必须遵守

1. 文件读取/修改仅使用 `serena_*` 工具。
2. 禁止使用原生 `read`、`edit`、`write`、`apply_patch` 及 desktop-commander MCP。
3. 优先符号级操作：`serena_find_symbol`、`serena_replace_symbol_body`、`serena_insert_*`。
4. 无法符号定位时使用 `serena_replace_content` 做文件级正则修改。

## 推荐读取顺序

1. 结构定位：`serena_list_dir`、`serena_find_file`
2. 语义定位：`serena_get_symbols_overview`、`serena_find_symbol`、`serena_find_referencing_symbols`
3. 跨文件搜索：`serena_search_for_pattern`
4. 仅在需要完整行内容时使用 `serena_read_file`

## 会话初始化

1. 先确认项目已激活并检查 onboarding 状态。
2. 开始开发前，按需读取 memories：`project_overview`、`style_conventions`、`suggested_commands`、`task_completion_checklist`。

## Memory 同步（架构/规范变更时）

1. 架构或核心依赖变化 → 更新 `project_overview`
2. 命令/Docker/运维脚本变化 → 更新 `suggested_commands`
3. 代码规范/命名/测试标准变化 → 更新 `style_conventions`

