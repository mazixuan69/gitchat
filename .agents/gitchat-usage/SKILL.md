---
name: gitchat-usage
description: Explain and operationalize how to use the gitchat project for chat-history branching workflows. Use when users ask how to model conversations with branch/fork/merge behavior, how merge modes behave, how to resolve conflicts, how to avoid known edge-case panics, or how to write practical usage examples and integration guidance from this repository.
---

# Gitchat Usage

Use this skill to answer usage questions about this repository's core chat branching model.

## Workflow

1. Read `src/lib.rs` when behavior certainty is required.
2. Map the user's goal to one of: create branch, fork branch, merge branch, manual resolve, remove branch.
3. Explain expected success path first, then enumerate error outcomes and edge cases.
4. Provide an example sequence tailored to the user scenario.
5. Highlight known edge cases and safe integration options.

## Response Contract

- Ground behavior claims in current implementation and tests.
- Prefer concrete operation order over abstract descriptions.
- Include pitfalls that materially affect correctness.
- If proposing production usage, add guardrails for invalid fork indexes.
- Keep examples minimal but runnable.

## Operation Mapping

Load [references/operation-mapping.md](references/operation-mapping.md) when you need exact behavior mapping, merge decision guidance, or answer templates.

## Required Caveats

Always state these when relevant:
- `fork_branch` has known panic edges for empty/negative index paths.
- `MergeMode::Human` may return conflict or missing-record errors and require manual merge.