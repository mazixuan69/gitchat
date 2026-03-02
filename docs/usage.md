# gitchat Usage Guide

This guide documents the behavior implemented in `src/lib.rs` as of this repository state.

## 1. Concept Model

`gitchat` models chat history with Git-like semantics:
- `Root<ChatType>`: repository root, containing multiple branches
- `Breach<ChatType>`: a branch, holding ordered messages
- `Message<ChatType>`: one message node (`uuid` + `content`)
- `IsForked`: whether a branch is original or forked from `(parent_branch_id, message_id)`

The naming uses `breach` in code; functionally it behaves like `branch`.

## 2. Data Structures

### `Message<ChatType>`
- fields:
  - `uuid: Uuid`
  - `content: ChatType`

### `GcError<ChatType>`
- `UuidNotFound`
- `StringNotFound`
- `MergeRecordNotFound`
- `ThingExist`
- `GcMergeHumanError(Breach<ChatType>, Breach<ChatType>)`

### `ManualMergeAction<ChatType>`
- `UseFrom`: overwrite target with source
- `UseTo`: keep target unchanged
- `UseMessages(Vec<Message<ChatType>>)`:
  replace target messages with custom list and reset fork metadata

## 3. Branch Operations

### Create branch
`create_breach(name)`:
- success: creates empty branch and returns new UUID
- duplicate name: returns `ThingExist`

### Fork branch
`fork_breach(source, fork_point, new_name)`:
- source can be branch name or branch UUID
- fork point can be message UUID or index
- resulting branch copies messages from start up to fork point (inclusive)
- resulting branch stores fork metadata as `IsForked::True(parent_id, fork_line)`

Errors:
- source name missing -> `StringNotFound`
- source UUID missing -> `UuidNotFound`
- fork point not found -> `UuidNotFound`
- target name already exists -> `ThingExist`

## 4. Merge Operations

### Force merge
`merge_tool(from, to, MergeMode::Force)`:
- always applies `from -> to`
- target keeps its own branch identity (`to` UUID and name)
- target messages are replaced by source messages
- target `is_forked` is reset to `False`

### Human merge
`merge_tool(from, to, MergeMode::Human)`:
- tries to infer safe merge from fork metadata
- if safe fast-forward-like condition is met, it auto-merges or no-ops
- if both sides diverged after fork line, returns:
  `GcMergeHumanError(from_breach_snapshot, to_breach_snapshot)`
- if no merge relationship record can be inferred, returns `MergeRecordNotFound`

### Manual merge
`merge_manual(from, to, action)`:
- `UseFrom`: same outcome as base merge
- `UseTo`: no changes
- `UseMessages(...)`: set custom messages on target and clear fork state

## 5. Remove Branch

`remove_breach(id)` removes a branch by UUID.

- success: branch deleted
- missing ID: `UuidNotFound`

## 6. Known Edge Cases and Risks

Important for production use:

- `fork_breach` with `Index(0)` on an empty source branch currently panics
  (due to index math on empty vector)
- `fork_breach` with negative index can panic
  (can reach `last().unwrap()` on empty temporary vector)
- many core APIs are private, so this crate is not yet ergonomic as public dependency

These are intentionally documented and covered in tests to avoid hidden surprises.

## 7. Test Coverage

Tests are in `src/tests/mod.rs` and currently verify:
- create + duplicate handling
- fork by name/UUID + error branches
- force/human/manual merge behaviors
- conflict signaling
- remove branch
- panic edge cases

Run:

```bash
cargo test
```

## 8. Practical Integration Advice

Until public API is stabilized, the most practical options are:
- keep this logic as internal module in your workspace
- expose your own safe wrapper API around current internals
- harden fork index validation before using untrusted input


## 9. Examples

- `docs/example.md`
- `docs/example_zh.md`
