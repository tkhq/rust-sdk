use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;
use tvc::commands::policy::init::{PolicySubject, TvcResource, build_policy};

fn tvc_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path());
    (temp, cmd)
}

#[test]
fn all_resources_default_uses_in_condition_and_tag_single_approval() {
    let policy = build_policy(
        None,
        &TvcResource::all(),
        1,
        PolicySubject::Tag {
            tag_id: "tag_123".to_string(),
        },
        None,
    )
    .unwrap();

    assert_eq!(
        policy.policy_name,
        "tvc-operators-all-resources-threshold-1"
    );
    assert_eq!(
        policy.condition,
        "activity.resource in ['TVC_APP', 'TVC_DEPLOYMENT', 'TVC_OPERATOR', 'TVC_QUORUM_KEY']"
    );
    assert_eq!(
        policy.consensus,
        "approvers.any(user, user.tags.contains('tag_123'))"
    );
    assert_eq!(policy.notes, "");
}

#[test]
fn single_resource_uses_equality_condition() {
    let policy = build_policy(
        Some("custom-policy".to_string()),
        &[TvcResource::Deployment],
        2,
        PolicySubject::Tag {
            tag_id: "tag_abc".to_string(),
        },
        Some("operators can deploy".to_string()),
    )
    .unwrap();

    assert_eq!(policy.policy_name, "custom-policy");
    assert_eq!(policy.condition, "activity.resource == 'TVC_DEPLOYMENT'");
    assert_eq!(
        policy.consensus,
        "approvers.filter(user, user.tags.contains('tag_abc')).count() >= 2"
    );
    assert_eq!(policy.notes, "operators can deploy");
}

#[test]
fn resource_subset_preserves_requested_order_in_membership_condition() {
    let policy = build_policy(
        None,
        &[TvcResource::Operator, TvcResource::Deployment],
        1,
        PolicySubject::UserIds(vec!["user_1".to_string(), "user_2".to_string()]),
        None,
    )
    .unwrap();

    assert_eq!(
        policy.policy_name,
        "tvc-operators-tvc-operator-tvc-deployment-threshold-1"
    );
    assert_eq!(
        policy.condition,
        "activity.resource in ['TVC_OPERATOR', 'TVC_DEPLOYMENT']"
    );
    assert_eq!(
        policy.consensus,
        "approvers.any(user, user.id in ['user_1', 'user_2'])"
    );
}

#[test]
fn user_id_threshold_consensus_uses_filter_count() {
    let policy = build_policy(
        None,
        &[TvcResource::App],
        2,
        PolicySubject::UserIds(vec!["user_1".to_string(), "user_2".to_string()]),
        None,
    )
    .unwrap();

    assert_eq!(
        policy.consensus,
        "approvers.filter(user, user.id in ['user_1', 'user_2']).count() >= 2"
    );
}

#[test]
fn threshold_must_be_at_least_one() {
    let err = build_policy(
        None,
        &[TvcResource::App],
        0,
        PolicySubject::UserIds(vec!["user_1".to_string()]),
        None,
    )
    .unwrap_err();

    assert!(err.to_string().contains("--threshold must be at least 1"));
}

#[test]
fn dry_run_prints_policy_json_and_planned_calls_without_auth() {
    let (_temp, mut cmd) = tvc_cmd();

    cmd.args([
        "policy",
        "init",
        "--dry-run",
        "--tag-name",
        "TVC Operators",
        "--user-ids",
        "user_1,user_2",
        "--resources",
        "TVC_DEPLOYMENT,TVC_OPERATOR",
        "--threshold",
        "2",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Planned API calls:"))
    .stdout(predicate::str::contains("1. create_user_tag"))
    .stdout(predicate::str::contains("2. create_policies"))
    .stdout(predicate::str::contains("\"effect\": \"EFFECT_ALLOW\""))
    .stdout(predicate::str::contains(
        "activity.resource in ['TVC_DEPLOYMENT', 'TVC_OPERATOR']",
    ))
    .stdout(predicate::str::contains(
        "approvers.filter(user, user.tags.contains('<created tag ID from create_user_tag>')).count() >= 2",
    ))
    .stderr(predicate::str::contains("No active organization").not());
}

#[test]
fn rejects_unknown_resource_name() {
    let (_temp, mut cmd) = tvc_cmd();

    cmd.args([
        "policy",
        "init",
        "--dry-run",
        "--user-ids",
        "user_1",
        "--resources",
        "TVC_DEPLOYMENT,BAD_RESOURCE",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "invalid TVC policy resource: BAD_RESOURCE",
    ));
}

#[test]
fn requires_user_source() {
    let (_temp, mut cmd) = tvc_cmd();

    cmd.args(["policy", "init", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "provide --tag-id, or provide --user-ids with --tag-name for tag mode, or --user-ids alone for tagless mode",
        ));
}
