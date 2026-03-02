# gitchat

一个用 Rust 实现的轻量级“聊天分支与合并”模型。

`gitchat` 把消息历史当作类似 Git 的分支图来处理：
- 创建分支（`breach`）
- 从某条消息位置 fork
- 用 force/human/manual 三种策略合并

## 当前状态

这个仓库目前是 **实现优先**：
- 核心逻辑在 `src/lib.rs`
- 行为已由 `src/tests/mod.rs` 覆盖验证
- 主要核心类型/函数暂未作为对外公开 API

因此当前更适合：
- 作为参考实现
- 作为内部模块使用
- 作为后续抽象公开 API 的基础

## 快速开始（面向贡献者）

```bash
cargo test
cargo check
```

## 文档导航

- English 使用指南: `docs/usage.md`
- 中文使用指南: `docs/usage_zh.md`
- English README: `README.md`

## 已验证行为（基于测试）

- 分支创建与重名保护
- 按分支名/分支 UUID 进行 fork
- 合并模式：`Force`、`Human`、手动合并
- Human 合并冲突识别（`GcMergeHumanError`）
- 分支删除
- 基于索引 fork 的已知 panic 边界场景

## 已知限制

- 暂无公开的高层 API（`Root`、`Breach`、merge helper 均为私有）
- `GcError::GcMergeHumanError` 暴露了私有类型 `Breach`（会有编译器告警）
- 在部分非法索引输入下，fork 逻辑当前会 panic

更多细节和示例见 `docs/usage_zh.md`。

- 示例文件：`docs/example.md` 和 `docs/example_zh.md`
