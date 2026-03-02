# gitchat

A lightweight Rust branch-and-merge model for chat/message histories.

`gitchat` models conversations like a Git graph:
- create branches (`breach`)
- fork from a message checkpoint
- merge with force/human/manual strategies

## Current Status

This crate is currently **implementation-first**:
- Core logic exists in `src/lib.rs`
- Behavior is covered by tests in `src/tests/mod.rs`
- Most core types/functions are not yet public API for external crates

That means this repository is currently best used as:
- a reference implementation
- an internal module
- a base to evolve into a public crate API

## Quick Start (for contributors)

```bash
cargo test
cargo check
```

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
- Known panic edge cases for index-based forking on invalid index inputs

## Known Limitations

- No public high-level API yet (`Root`, `Breach`, merge helpers are private)
- `GcError::GcMergeHumanError` exposes private `Breach` types (compiler warns)
- Forking with invalid index patterns can panic in current implementation

See `docs/usage.md` for full details, examples, and behavior notes.

- Examples: `docs/example.md` and `docs/example_zh.md`
