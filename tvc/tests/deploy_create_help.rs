use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn deploy_create_help_documents_config_and_override_precedence() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use --config-file, flags, env vars, or a mix of them.",
        ))
        .stdout(predicate::str::contains(
            "override env vars; env vars override config file values.",
        ))
        .stdout(predicate::str::contains(
            "--pivot-args replaces the config file's list entirely",
        ));
}

#[test]
fn deploy_create_help_documents_env_only_usage() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TVC_ORG_ID=..."))
        .stdout(predicate::str::contains("TVC_API_KEY_PUBLIC=..."))
        .stdout(predicate::str::contains("TVC_API_KEY_PRIVATE=..."))
        .stdout(predicate::str::contains("TVC_EXPECTED_PIVOT_DIGEST=..."))
        .stdout(predicate::str::contains("# OR"))
        .stdout(predicate::str::contains("tvc deploy create"));
}

#[test]
fn deploy_create_help_leaves_auth_details_to_global_docs() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Authentication:").not());
}

#[test]
fn top_level_help_documents_auth_precedence() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Authentication:"))
        .stdout(predicate::str::contains("Env vars take precedence over"))
        .stdout(predicate::str::contains("TVC_ORG_ID"))
        .stdout(predicate::str::contains("TVC_API_KEY_PUBLIC"))
        .stdout(predicate::str::contains("TVC_API_KEY_PRIVATE"));
}
