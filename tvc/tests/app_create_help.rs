use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn app_create_help_documents_operator_reuse_opt_out() {
    cargo_bin_cmd!("tvc")
        .arg("app")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-operator-reuse"))
        .stdout(predicate::str::contains("TVC_NO_OPERATOR_REUSE"))
        .stdout(predicate::str::contains(
            "reusing the most recently created",
        ));
}
