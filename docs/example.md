# Example Workflow

This example demonstrates the current core workflow implemented in `src/lib.rs`:

1. Create branches
2. Add messages
3. Fork from a message checkpoint
4. Try human merge
5. Resolve conflict with manual merge

Note: current core API is private. This example is intended for internal usage (for example, inside crate tests/modules), not as an external dependency API.

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

    // 1) Create main branch
    let main_id = root.create_breach("main".to_string())?;

    // 2) Add two messages to main
    let main_idx = root.find_breach_index_by_uuid(&main_id)?;
    let m1 = msg("hello");
    let m1_id = m1.uuid;
    root.breaches[main_idx].messages.push(m1);
    root.breaches[main_idx].messages.push(msg("how are you"));

    // 3) Fork dev from main at m1
    let dev_id = root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::MessageId(m1_id),
        "dev".to_string(),
    )?;

    // Add different messages to both sides to create divergence
    let main_idx = root.find_breach_index_by_uuid(&main_id)?;
    root.breaches[main_idx].messages.push(msg("main-only message"));

    let dev_idx = root.find_breach_index_by_uuid(&dev_id)?;
    root.breaches[dev_idx].messages.push(msg("dev-only message"));

    // 4) Try human merge: likely conflict in this divergent state
    match root.merge_tool(dev_id, main_id, MergeMode::Human) {
        Ok(()) => {}
        Err(GcError::GcMergeHumanError(_, _)) => {
            // 5) Resolve manually with custom messages
            let resolved = vec![
                msg("hello"),
                msg("resolved: merged main/dev"),
            ];
            root.merge_manual(dev_id, main_id, ManualMergeAction::UseMessages(resolved))?;
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
```

## Minimal Error Handling Pattern

```rust
match root.fork_breach(
    StringOrUuid::Name("main".to_string()),
    I64OrUuid::Index(3),
    "feature".to_string(),
) {
    Ok(id) => println!("forked: {id}"),
    Err(GcError::ThingExist) => println!("branch name already exists"),
    Err(GcError::StringNotFound) => println!("source branch name not found"),
    Err(GcError::UuidNotFound) => println!("fork checkpoint not found"),
    Err(_) => println!("other error"),
}
```
