# 示例流程

这个示例演示 `src/lib.rs` 当前实现的核心流程：

1. 创建分支
2. 写入消息
3. 从某条消息位置 fork
4. 尝试 human merge
5. 冲突后使用 manual merge 解决

注意：当前核心 API 主要是私有的。该示例适合仓库内部使用（例如测试模块），不适合直接当外部依赖调用。

```rust
use crate::{
    GcError, I64OrUuid, ManualMergeAction, MergeMode, Message, Root, StringOrUuid,
};
use uuid::Uuid;

fn msg(text: &str) -> Message<String> {
    Message { uuid: Uuid::new_v4(), content: text.to_string() }
}

fn run_example() -> Result<(), GcError<String>> {
    let mut root = Root::<String>::new("chat-repo".to_string());

    // 1) 创建 main 分支
    let main_id = root.create_breach("main".to_string())?;

    // 2) 给 main 写两条消息
    let main_idx = root.find_breach_index_by_uuid(&main_id)?;
    let m1 = msg("你好");
    let m1_id = m1.uuid;
    root.breaches[main_idx].messages.push(m1);
    root.breaches[main_idx].messages.push(msg("最近怎么样"));

    // 3) 在 m1 位置 fork 出 dev
    let dev_id = root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::MessageId(m1_id),
        "dev".to_string(),
    )?;

    // 双方继续演化，制造分叉
    let main_idx = root.find_breach_index_by_uuid(&main_id)?;
    root.breaches[main_idx].messages.push(msg("main 独有消息"));

    let dev_idx = root.find_breach_index_by_uuid(&dev_id)?;
    root.breaches[dev_idx].messages.push(msg("dev 独有消息"));

    // 4) human 合并：此时大概率冲突
    match root.merge_tool(dev_id, main_id, MergeMode::Human) {
        Ok(()) => {}
        Err(GcError::GcMergeHumanError(_, _)) => {
            // 5) 手动合并：写入自定义结果
            let resolved = vec![
                msg("你好"),
                msg("resolved: main/dev 已合并"),
            ];
            root.merge_manual(dev_id, main_id, ManualMergeAction::UseMessages(resolved))?;
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
```

## 最小错误处理模板

```rust
match root.fork_breach(
    StringOrUuid::Name("main".to_string()),
    I64OrUuid::Index(3),
    "feature".to_string(),
) {
    Ok(id) => println!("fork 成功: {id}"),
    Err(GcError::ThingExist) => println!("分支名已存在"),
    Err(GcError::StringNotFound) => println!("源分支名不存在"),
    Err(GcError::UuidNotFound) => println!("fork 检查点不存在"),
    Err(_) => println!("其他错误"),
}
```
