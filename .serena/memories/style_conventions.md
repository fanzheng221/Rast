# Rast 代码风格和规范

## 通用规范

### 文件组织
- **Rust**: 使用 `src/lib.rs` 作为库入口，使用模块组织代码
- **TypeScript**: 使用 `src/index.ts` 作为包入口，按功能模块化

### 命名约定
- **Rust**:
  - 函数和变量: `snake_case`
  - 类型和结构体: `PascalCase`
  - 常量: `SCREAMING_SNAKE_CASE`
  
- **TypeScript**:
  - 函数和变量: `camelCase`
  - 类型和接口: `PascalCase`
  - 常量: `UPPER_SNAKE_CASE`
  - 文件名: `kebab-case`

### 代码格式化
- **Rust**: 使用 `rustfmt`（Cargo 默认支持）
- **TypeScript**: 使用 `Prettier` 和 `ESLint`
- **缩进**: 2 空格（TypeScript），4 空格（Rust）

## Rust 特定规范

### 模块组织
```rust
// crates/ast_engine/src/lib.rs
pub mod ast;
pub mod visitor;
pub mod lint;

pub use ast::*;
pub use visitor::*;
pub use lint::*;

#[cfg(test)]
mod tests {
    use super::*;
    // 测试代码
}
```

### 错误处理
- 使用 `Result<T, E>` 进行错误处理
- 定义自定义错误类型时实现 `std::error::Error`
- 使用 `thiserror` crate 简化错误定义（如果需要）

### 注释和文档
- 公共 API 必须有文档注释 `///`
- 复杂逻辑使用内部注释 `//`

## TypeScript 特定规范

### 模块导出
```typescript
// packages/unplugin/src/index.ts
export const rastUnplugin = createUnplugin(() => {
  return {
    name: 'rast-unplugin',
    // ...
  };
});

export default rastUnplugin;
```

### 类型定义
- 为所有公共 API 提供类型定义
- 使用 `interface` 定义对象类型
- 使用 `type` 定义联合类型或类型别名

### 异步处理
- 优先使用 `async/await`
- 明确处理 Promise 错误

## 测试规范

### Rust 测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "...";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### TypeScript 测试
- 使用 `vitest` 进行单元测试
- 测试文件放在 `test/` 目录或 `*.test.ts` 文件中
- 使用 `describe` 和 `it` 组织测试

## Git 规范

### 提交信息格式
```
<type>: <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式化（不影响功能）
- `refactor`: 重构
- `test`: 添加或修改测试
- `chore`: 构建/工具链更新

## Serena 工具使用规范

### 必须使用 Serena 工具
1. **文件读取**: 优先使用 `serena_read_file` 替代原生 `read` 工具
2. **内容搜索**: 优先使用 `serena_search_for_pattern` 或 `serena_find_file` 替代原生 `grep` / `glob`
3. **符号查找**: 优先使用 `serena_find_symbol` 等符号级工具
4. **文件修改**: 使用 `serena_replace_content`、`serena_replace_symbol_body` 等
### 符号级操作优先
- 修改整个函数/类: `serena_replace_symbol_body`
- 在符号后插入: `serena_insert_after_symbol`
- 在符号前插入: `serena_insert_before_symbol`
- 重命名符号: `serena_rename_symbol`

### 文件级操作
- 无法使用符号级操作时: `serena_replace_content`（支持正则）
- 创建新文件: `serena_create_text_file`

## 依赖管理

### Rust
- 使用 workspace 依赖共享公共版本
- 版本号在 `Cargo.toml` workspace 部分统一管理

### Node.js
- 使用 workspace 协议引用本地包: `"@rast/bindings": "workspace:*"`
- 使用 `pnpm` 进行依赖安装
