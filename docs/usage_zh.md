# gitchat 使用指南

本指南基于当前仓库 `src/lib.rs` 的实际实现行为编写。

## 1. 概念模型

`gitchat` 使用类似 Git 的语义管理聊天历史：
- `Root<ChatType>`: 根容器，管理多个分支
- `Branch<ChatType>`: 分支，保存有序消息列表
- `Message<ChatType>`: 消息节点（`uuid` + `content`）
- `IsForked`: 标记分支是否由 `(父分支ID, 分叉消息ID)` fork 而来

## 2. 主要数据结构

### `Message<ChatType>`
- 字段：
  - `uuid: Uuid`
  - `content: ChatType`

### `GcError<ChatType>`
- `UuidNotFound`
- `StringNotFound`
- `MergeRecordNotFound`
- `ThingExist`
- `GcMergeHumanError(Branch<ChatType>, Branch<ChatType>)`

### `ManualMergeAction<ChatType>`
- `UseFrom`: 用来源分支覆盖目标分支
- `UseTo`: 保留目标分支不变
- `UseMessages(Vec<Message<ChatType>>)`:
  用给定消息列表替换目标分支并清除 fork 状态

## 3. 分支操作

### 创建分支
`create_branch(name)`：
- 成功：创建空分支并返回 UUID
- 重名：返回 `ThingExist`

### Fork 分支
`fork_branch(source, fork_point, new_name)`：
- `source` 支持按分支名或分支 UUID 指定
- `fork_point` 支持按消息 UUID 或消息索引指定
- 新分支会复制从起点到 fork 点（含 fork 点）的消息
- 新分支会记录 `IsForked::True(parent_id, fork_line)`

错误返回：
- 源分支名不存在 -> `StringNotFound`
- 源分支 UUID 不存在 -> `UuidNotFound`
- fork 点不存在 -> `UuidNotFound`
- 新分支名已存在 -> `ThingExist`

## 4. 合并操作

### 强制合并
`merge_tool(from, to, MergeMode::Force)`：
- 始终按 `from -> to` 方向覆盖
- 目标分支保留自身身份（`to` 的 UUID 与 name）
- 目标消息列表替换为来源消息列表
- 目标 `is_forked` 被清空为 `False`

### Human 合并
`merge_tool(from, to, MergeMode::Human)`：
- 基于 fork 元信息判断是否可自动合并
- 可快进时会自动合并或直接 no-op
- 若 fork 后两侧都产生分叉修改，返回：
  `GcMergeHumanError(from_branch_snapshot, to_branch_snapshot)`
- 若推断不出合并关系，返回 `MergeRecordNotFound`

### 手动合并
`merge_manual(from, to, action)`：
- `UseFrom`: 等效于基础覆盖合并
- `UseTo`: 不做改动
- `UseMessages(...)`: 使用自定义消息替换目标并清空 fork 状态

## 5. 删除分支

`remove_branch(id)` 按 UUID 删除分支。

- 成功：分支被删除
- UUID 不存在：`UuidNotFound`

## 6. 已知边界与风险

对使用者最关键的限制如下：

- 空分支上执行 `Index(0)` fork 目前会 panic
  （空向量参与索引计算导致）
- 负索引 fork 也可能 panic
  （可能触发空向量 `last().unwrap()`）

这些行为已在测试中显式覆盖，目的是防止"文档没写但线上踩坑"。

## 7. 测试覆盖说明

测试文件：`src/tests/mod.rs`。

当前已覆盖：
- 创建分支与重名检查
- 按名称/UUID fork 及错误路径
- Force/Human/Manual 三类合并
- 冲突识别
- 删除分支
- 已知 panic 边界

运行方式：

```bash
cargo test
```

## 8. 示例

- `docs/example.md`
- `docs/example_zh.md`
