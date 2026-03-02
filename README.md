# gitchat

A lightweight Rust branch-and-merge model for chat/message histories.

`gitchat` models conversations like a Git graph:
- create branches
- fork from a message checkpoint
- merge with force/human/manual strategies
- export/import to JSON for persistence

## Quick Start

```bash
cargo test
cargo check
```

## Features

- **Serialize/Deserialize**: All core types implement `Serialize` and `Deserialize` via serde
- **Export/Import**: `Root::export()` and `Root::import()` for JSON serialization

## Documentation

- English usage guide: `docs/usage.md`
- 中文使用指南: `docs/usage_zh.md`
- 中文 README: `README_ZH.md`

## Verified Behavior (from tests)

- Branch creation and duplicate-name protection
- Forking by branch name and branch UUID
- Merge modes: `Force`, `Human`, and manual merge actions
- Human-merge conflict detection (`GcMergeHumanError`)
- Branch deletion
- Export/Import roundtrip serialization
- Known panic edge cases for index-based forking on invalid index inputs

## Known Limitations

- Forking with invalid index patterns can panic in current implementation

See `docs/usage.md` for full details, examples, and behavior notes.

- Examples: `docs/example.md` and `docs/example_zh.md`
