# Serena 工具限制和规范

## 当前可用的 Serena 工具

### 文件操作
| 工具 | 功能 | 限制 |
|------|------|------|
| `serena_list_dir` | 列出目录/文件 | ✅ 可用 |
| `serena_find_file` | 查找文件 | ✅ 可用 |
| `serena_read_file` | 读取文件 | ✅ 可用 |
| `serena_replace_content` | 替换文件内容（支持正则） | ✅ 可用 |
| - | 创建新文件 | ❌ **不可用** |
| - | 编辑/插入行 | ❌ **不可用** |

### 符号操作
| 工具 | 功能 | 限制 |
|------|------|------|
| `serena_get_symbols_overview` | 获取文件符号概览 | ✅ 可用 |
| `serena_find_symbol` | 查找符号 | ✅ 可用 |
| `serena_find_referencing_symbols` | 查找引用 | ✅ 可用 |
| `serena_replace_symbol_body` | 替换符号定义 | ✅ 可用 |
| `serena_insert_after_symbol` | 在符号后插入 | ✅ 可用 |
| `serena_insert_before_symbol` | 在符号前插入 | ✅ 可用 |
| `serena_rename_symbol` | 重命名符号 | ✅ 可用 |

### 搜索
| 工具 | 功能 | 限制 |
|------|------|------|
| `serena_search_for_pattern` | 搜索模式 | ✅ 可用 |

### 内存操作
| 工具 | 功能 | 限制 |
|------|------|------|
| `serena_read_memory` | 读取内存 | ✅ 可用 |
| `serena_write_memory` | 写入内存 | ✅ 可用 |
| `serena_edit_memory` | 编辑内存 | ✅ 可用 |
| `serena_list_memories` | 列出内存 | ✅ 可用 |
| `serena_delete_memory` | 删除内存 | ✅ 可用 |
| `serena_rename_memory` | 重命名内存 | ✅ 可用 |

### 项目管理
| 工具 | 功能 |
|------|------|
| `serena_activate_project` | 激活项目 |
| `serena_get_current_config` | 获取当前配置 |
| `serena_check_onboarding_performed` | 检查 onboarding 状态 |
| `serena_onboarding` | 执行 onboarding |
| `serena_initial_instructions` | 获取初始指令 |

## 开发规范（严格遵守）

### ✅ 允许的操作
1. **使用 `bash`** 执行命令（安装、构建、测试等）
2. **使用 `serena_replace_content`** 替换文件内容
3. **使用符号级工具** (`serena_find_symbol`, `serena_replace_symbol_body` 等)
4. **使用 Serena 搜索工具** (`serena_search_for_pattern`, `serena_find_file`) 进行文件和内容搜索
5. **使用 `serena_read_file`** 读取完整文件内容
### ❌ 禁止的操作
1. **禁止使用原生 `read` / `edit` / `write` / `apply_patch`** 工具（除非 Serena 工具无法完成任务）
2. **尽量避免使用原生 `glob` / `grep`** 工具，仅当 `serena_search_for_pattern` 或 `serena_find_file` 无法处理时才作为后备方案使用
3. **禁止使用 `bash + cat > file`** 创建文件
4. **禁止使用 `bash + sed/awk`** 修改文件

### 📝 文件创建/修改策略

由于 `serena_read_file` 和 `serena_write_file` 不可用，需要使用以下策略：

#### 策略 1: 使用 `bash` + `heredoc` 创建文件
```bash
cat > path/to/file << 'EOF'
content here
EOF
```

#### 策略 2: 使用 `serena_replace_content` 修改文件
- 对于已存在的文件，使用正则替换部分内容
- 需要先通过 `read` 工具或 `bash + cat` 获取文件内容

#### 策略 3: 使用符号级工具
- `serena_replace_symbol_body` - 替换整个函数/类定义
- `serena_insert_after_symbol` - 在符号后插入内容
- `serena_insert_before_symbol` - 在符号前插入内容

#### 策略 4: 对于代码实现任务
- 使用 `task()` 委托给 subagent
- subagent 可以使用原生工具（如 read/write/edit）
- **Orchestrator 只负责协调和验证**

## 关键原则

1. **Orchestrator 角色**：协调和验证，不直接实现代码
2. **Subagent 委托**：实现任务时委托给 subagent
3. **Serena 工具优先**：在可能的情况下优先使用 Serena 工具
4. **例外情况**：原生 read/write/edit 等工具不可用时，使用 `bash` 创建/修改文件
5. **验证责任**：Orchestrator 必须验证所有 subagent 的工作

## 当前环境限制

- **原生工具降级**：优先使用 Serena 工具链（`serena_read_file`、`serena_search_for_pattern` 等），原生工具（`read`、`glob`、`grep`）仅作为后备降级方案
- **无直接文件创建工具**：需要通过 bash 创建文件
- **符号级工具受限**：只能用于已解析的符号
## 后续开发建议

对于 Task 2-6 的实现：
1. 使用 `task(category="...", ...)` 委托实现
2. 使用 6-section prompt 结构明确指定要求
3. 验证时使用 `lsp_diagnostics` + `read` + `bash` 测试
4. 更新 `.sisyphus/notepads/rast-ast-tool/` 记录进度
