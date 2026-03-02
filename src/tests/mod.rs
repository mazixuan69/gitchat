use super::*;

fn ok<T>(r: Result<T, GcError<String>>) -> T {
    match r {
        Ok(v) => v,
        Err(_) => panic!("expected Ok(..)")
    }
}

fn make_msg(content: &str) -> Message<String> {
    Message {
        uuid: Uuid::new_v4(),
        content: content.to_string(),
    }
}

fn push_message(root: &mut Root<String>, breach_id: Uuid, content: &str) -> Uuid {
    let msg = make_msg(content);
    let msg_id = msg.uuid;
    let idx = ok(root.find_breach_index_by_uuid(&breach_id));
    root.breaches[idx].messages.push(msg);
    msg_id
}

#[test]
fn create_breach_rejects_duplicate_name() {
    let mut root = Root::<String>::new("repo".to_string());
    assert_eq!(root.name, "repo");
    let id = ok(root.create_breach("main".to_string()));
    assert!(root.find_breach_index_by_uuid(&id).is_ok());

    let err = root.create_breach("main".to_string());
    assert!(matches!(err, Err(GcError::ThingExist)));
}

#[test]
fn fork_breach_by_name_and_by_id() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    let first = push_message(&mut root, main_id, "m1");
    push_message(&mut root, main_id, "m2");

    let dev_id = ok(root.fork_breach(
        StringOrUuid::Name("main".to_string()),
        I64OrUuid::MessageId(first),
        "dev".to_string(),
    ));
    let idx = ok(root.find_breach_index_by_uuid(&dev_id));
    assert_eq!(root.breaches[idx].messages.len(), 1);
    assert!(matches!(
        root.breaches[idx].is_forked,
        IsForked::True(parent, line) if parent == main_id && line == first
    ));

    let feature_id = ok(root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::Index(1),
        "feature".to_string(),
    ));
    let feature_idx = ok(root.find_breach_index_by_uuid(&feature_id));
    assert_eq!(root.breaches[feature_idx].messages.len(), 2);
}

#[test]
fn fork_breach_reports_not_found_errors() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    push_message(&mut root, main_id, "m1");

    let by_name = root.fork_breach(
        StringOrUuid::Name("missing".to_string()),
        I64OrUuid::Index(0),
        "dev".to_string(),
    );
    assert!(matches!(by_name, Err(GcError::StringNotFound)));

    let by_id = root.fork_breach(
        StringOrUuid::BreachId(Uuid::new_v4()),
        I64OrUuid::Index(0),
        "dev2".to_string(),
    );
    assert!(matches!(by_id, Err(GcError::UuidNotFound)));
}

#[test]
fn merge_force_overwrites_target_but_keeps_target_identity() {
    let mut root = Root::<String>::new("repo".to_string());
    let from_id = ok(root.create_breach("from".to_string()));
    let to_id = ok(root.create_breach("to".to_string()));

    push_message(&mut root, from_id, "f1");
    push_message(&mut root, from_id, "f2");
    push_message(&mut root, to_id, "t1");

    ok(root.merge_tool(from_id, to_id, MergeMode::Force));

    let to_idx = ok(root.find_breach_index_by_uuid(&to_id));
    assert_eq!(root.breaches[to_idx].name, "to");
    assert_eq!(root.breaches[to_idx].breach_id, to_id);
    assert_eq!(root.breaches[to_idx].messages.len(), 2);
    assert!(matches!(root.breaches[to_idx].is_forked, IsForked::False));
    assert_eq!(root.breaches[to_idx].messages[0].content, "f1");
}

#[test]
fn merge_human_fast_forward_child_to_parent() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    push_message(&mut root, main_id, "m1");
    let fork_line = push_message(&mut root, main_id, "m2");

    let dev_id = ok(root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::MessageId(fork_line),
        "dev".to_string(),
    ));
    push_message(&mut root, dev_id, "m3-dev");

    ok(root.merge_tool(dev_id, main_id, MergeMode::Human));

    let main_idx = ok(root.find_breach_index_by_uuid(&main_id));
    assert_eq!(root.breaches[main_idx].messages.len(), 3);
    assert_eq!(root.breaches[main_idx].messages[2].content, "m3-dev");
}

#[test]
fn merge_human_conflict_requires_manual_resolution() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    let fork_line = push_message(&mut root, main_id, "m1");
    push_message(&mut root, main_id, "m2-main");

    let dev_id = ok(root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::MessageId(fork_line),
        "dev".to_string(),
    ));
    push_message(&mut root, dev_id, "m2-dev");
    push_message(&mut root, main_id, "m3-main");

    let result = root.merge_tool(dev_id, main_id, MergeMode::Human);
    assert!(matches!(result, Err(GcError::GcMergeHumanError(_, _))));
}

#[test]
fn merge_human_without_relationship_returns_merge_record_not_found() {
    let mut root = Root::<String>::new("repo".to_string());
    let a_id = ok(root.create_breach("a".to_string()));
    let b_id = ok(root.create_breach("b".to_string()));
    push_message(&mut root, a_id, "a1");
    push_message(&mut root, b_id, "b1");

    let result = root.merge_tool(a_id, b_id, MergeMode::Human);
    assert!(matches!(result, Err(GcError::MergeRecordNotFound)));
}

#[test]
fn manual_merge_modes_work_as_expected() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    let fork_line = push_message(&mut root, main_id, "m1");
    let dev_id = ok(root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::MessageId(fork_line),
        "dev".to_string(),
    ));
    push_message(&mut root, dev_id, "m2-dev");

    ok(root.merge_manual(dev_id, main_id, ManualMergeAction::UseTo));
    let main_idx = ok(root.find_breach_index_by_uuid(&main_id));
    assert_eq!(root.breaches[main_idx].messages.len(), 1);

    ok(root.merge_manual(dev_id, main_id, ManualMergeAction::UseFrom));
    let main_idx = ok(root.find_breach_index_by_uuid(&main_id));
    assert_eq!(root.breaches[main_idx].messages.len(), 2);

    let custom = vec![make_msg("custom-1"), make_msg("custom-2"), make_msg("custom-3")];
    ok(root.merge_manual(dev_id, main_id, ManualMergeAction::UseMessages(custom)));
    let main_idx = ok(root.find_breach_index_by_uuid(&main_id));
    assert_eq!(root.breaches[main_idx].messages.len(), 3);
    assert_eq!(root.breaches[main_idx].messages[0].content, "custom-1");
    assert!(matches!(root.breaches[main_idx].is_forked, IsForked::False));
}

#[test]
fn remove_breach_deletes_target() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    let dev_id = ok(root.create_breach("dev".to_string()));

    ok(root.remove_breach(&dev_id));
    assert!(root.find_breach_index_by_uuid(&dev_id).is_err());
    assert!(root.find_breach_index_by_uuid(&main_id).is_ok());
}

#[test]
#[should_panic]
fn fork_on_empty_breach_by_index_panics_currently() {
    let mut root = Root::<String>::new("repo".to_string());
    let _ = ok(root.create_breach("main".to_string()));
    let _ = root.fork_breach(
        StringOrUuid::Name("main".to_string()),
        I64OrUuid::Index(0),
        "dev".to_string(),
    );
}

#[test]
#[should_panic]
fn fork_with_negative_index_panics_currently() {
    let mut root = Root::<String>::new("repo".to_string());
    let main_id = ok(root.create_breach("main".to_string()));
    push_message(&mut root, main_id, "m1");
    let _ = root.fork_breach(
        StringOrUuid::BreachId(main_id),
        I64OrUuid::Index(-1),
        "dev".to_string(),
    );
}
