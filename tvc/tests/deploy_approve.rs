use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn approve_requires_source() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--dry-run")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest source is required"));
}

#[test]
fn approve_with_yes_flag() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--yes")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn approve_with_short_y_flag() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("-y")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn approve_with_dangerous_skip_interactive_backward_compat() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn approve_interactive_prompts() {
    let input = "yes\nyes\ny\nyes\ny\n";

    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--skip-post")
        .write_stdin(input)
        .assert()
        .success()
        // Interactive review output goes to stderr
        .stderr(predicate::str::contains("MANIFEST APPROVAL"))
        .stderr(predicate::str::contains("NAMESPACE"))
        .stderr(predicate::str::contains("turnkey-prod"))
        .stderr(predicate::str::contains("ENCLAVE (AWS Nitro)"))
        .stderr(predicate::str::contains("PIVOT BINARY"))
        .stderr(predicate::str::contains("MANIFEST SET"))
        .stderr(predicate::str::contains("operator-alice"))
        .stderr(predicate::str::contains("SHARE SET"))
        .stderr(predicate::str::contains("ALL SECTIONS APPROVED"))
        // Approval JSON goes to stdout
        .stdout(predicate::str::contains("\"signature\""));
}

#[test]
fn approve_interactive_reject() {
    let input = "no\n";

    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("approval cancelled by user"));
}

#[test]
fn manifest_and_deploy_id_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--deploy-id")
        .arg("some-deploy-id")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--manifest <PATH>' cannot be used with '--deploy-id <DEPLOY_ID>'",
        ));
}

#[test]
fn approve_requires_manifest_id_or_skip_post() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--manifest-id is required to post approval to API",
        ));
}

#[test]
fn approve_no_input_skips_interactive() {
    cargo_bin_cmd!("tvc")
        .arg("--no-input")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--skip-post")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"signature\""));
}
