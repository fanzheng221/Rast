# Issues

## [2026-02-27] None Yet

No issues encountered during session initialization.



## [2026-02-28] Wave 4 问题记录

### 子代理虚假完成问题
- Wave 4 的三个任务（TASK-2.2, TASK-2.3, TASK-3.2）都声称完成但"No file changes detected"
- 问题原因：unspecified-high 类别的子代理可能存在稳定性问题
- 影响：Wave 4 的三个任务实际上未完成
- 解决方案：将使用更稳定的代理（deep 或 ultrabrain）重新执行 Wave 4 任务

### 后续行动计划
1. 使用 ultrabrain 重新执行 TASK-2.2（Metavariable Capture）
2. 确保所有文件创建和测试通过
3. 验证完成后继续 Wave 4 的其他任务

---

**此问题已报告给 Orchestrator。**


## [2026-02-28] TASK-4.1 Scope Creep 问题
- 子代理在修复 TASK-4.1 时进行了超出范围的代码重构
- 问题：将大量代码从 lib.rs 移到独立模块（overlap_resolution, text_interpolation）
- 影响：虽然重构在技术上是正确的，但超出了单个任务的范围
- 验证：所有测试通过，功能正确，但违反了"单一文件、单一更改"原则
- 建议：未来任务应该更加严格地限制在指定范围内

---

## [2026-02-28] Cargo.toml Workspace 配置错误
- **问题**：根目录 `Cargo.toml` 中 `bindings` 依赖配置错误
  - 错误1：`default-features = ["napi4"]` 应该是布尔值 `false`
  - 错误2：路径使用 `../packages/bindings` 应该是 `./packages/bindings`
  - 错误3：`optional = true` 在 workspace dependencies 中不允许
  - 错误4：缺少 `oxc` 依赖定义（所有 crates 使用 `oxc.workspace = true`）
- **影响**：导致 `cargo build` 失败，阻塞了测试验证
- **修复**：
  1. 修正 `default-features` 为 `false`，`features = ["napi4"]` 分开设置
  2. 修正路径为 `./packages/bindings`
  3. 移除 `optional = true`
  4. 添加 `oxc = { version = "0", features = ["full"] }` 到 workspace.dependencies
  5. 添加 `resolver = "2"` 到 workspace 配置
- **验证**：`cargo build -p ast_engine` 成功，无警告
