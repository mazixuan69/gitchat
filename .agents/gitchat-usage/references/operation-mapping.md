# Operation Mapping

## Goal -> API Mapping

- Create a branch: `create_branch(name)`
- Fork from branch by name: `fork_branch(StringOrUuid::Name(...), fork_point, new_name)`
- Fork from branch by id: `fork_branch(StringOrUuid::BranchId(...), fork_point, new_name)`
- Force merge: `merge_tool(from, to, MergeMode::Force)`
- Human merge: `merge_tool(from, to, MergeMode::Human)`
- Manual resolution: `merge_manual(from, to, ManualMergeAction::...)`
- Delete branch: `remove_branch(id)`

## Fork Point Selection

- Use `I64OrUuid::MessageId(msg_id)` for deterministic checkpointing.
- Use `I64OrUuid::Index(i)` only after validating non-negative and in-range indexes.
- Treat index forking on empty source branches as unsafe in current implementation.

## Merge Behavior Summary

- `Force`: overwrite `to` content with `from` content; keep `to` identity; clear fork metadata.
- `Human`: auto-merge/no-op when fork-line conditions prove safe.
- `Human` conflict path: returns `GcMergeHumanError(...)` when both sides diverge.
- `Human` unknown relation path: returns `MergeRecordNotFound`.

## Recommended Conflict Playbook

1. Attempt `MergeMode::Human`.
2. If `GcMergeHumanError`, choose one:
   - keep source: `ManualMergeAction::UseFrom`
   - keep target: `ManualMergeAction::UseTo`
   - custom synthesis: `ManualMergeAction::UseMessages(messages)`
3. If `MergeRecordNotFound`, confirm branch lineage metadata and choose manual strategy.

## Answer Template

1. State the workflow for the user goal.
2. Provide concrete API call sequence.
3. List probable errors for each step.
4. Add edge-case guardrails.
5. End with a practical next action.