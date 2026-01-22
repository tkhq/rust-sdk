use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn approve_requires_source() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--dry-run")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest source is required"));
}

#[test]
fn dangerous_approve_with_file() {
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
    // Simulate user typing "yes" or "y" for each of the 5 prompts:
    // namespace, enclave, pivot, manifest set, share set
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
        .stdout(predicate::str::contains("MANIFEST APPROVAL"))
        .stdout(predicate::str::contains("NAMESPACE"))
        .stdout(predicate::str::contains("turnkey-prod"))
        .stdout(predicate::str::contains("ENCLAVE (AWS Nitro)"))
        .stdout(predicate::str::contains("PIVOT BINARY"))
        .stdout(predicate::str::contains("MANIFEST SET"))
        .stdout(predicate::str::contains("operator-alice"))
        .stdout(predicate::str::contains("SHARE SET"))
        .stdout(predicate::str::contains("ALL SECTIONS APPROVED"))
        .stdout(predicate::str::contains("\"signature\""));
}

#[test]
fn approve_interactive_reject() {
    // User rejects at first prompt
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
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--manifest <PATH>' cannot be used with '--deploy-id <DEPLOY_ID>'",
        ));
}

/// Test that --skip-post is required when --manifest-id is not provided
#[test]
fn approve_requires_manifest_id_or_skip_post() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--manifest-id is required to post approval to API",
        ));
}
