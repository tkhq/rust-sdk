use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn skills_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("skills");
    (temp, cmd)
}

fn parse_single_json_line(assert: &assert_cmd::assert::Assert) -> Value {
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected one JSON message, got {stdout:?}");
    serde_json::from_str(lines[0]).unwrap()
}

#[test]
fn save_writes_the_bundled_skill_to_an_explicit_dir() {
    let (temp, mut cmd) = skills_cmd();
    let dest = temp.path().join("my-skills");

    let assert = cmd
        .args(["save", "--dir"])
        .arg(&dest)
        .args(["--message-format", "json"])
        .assert()
        .success();

    let message = parse_single_json_line(&assert);
    assert_eq!(message["reason"], "skills_saved");
    assert_eq!(message["command"], "skills save");
    assert_eq!(message["skills"], serde_json::json!(["tvc-deployments"]));
    assert_eq!(message["filesWritten"], 4);
    assert_eq!(message["filesUnchanged"], 0);

    let skill_md = dest.join("tvc-deployments/SKILL.md");
    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.starts_with("---"),
        "SKILL.md must begin with YAML frontmatter"
    );
    assert!(
        dest.join("tvc-deployments/references/deploy-lifecycle.md")
            .exists()
    );
}

#[test]
fn save_is_idempotent() {
    let (temp, mut first) = skills_cmd();
    let dest = temp.path().join("skills");
    first.args(["save", "--dir"]).arg(&dest).assert().success();

    let mut second = cargo_bin_cmd!("tvc");
    second.env_clear().env("HOME", temp.path());
    let assert = second
        .args(["skills", "save", "--dir"])
        .arg(&dest)
        .args(["--message-format", "json"])
        .assert()
        .success();

    let message = parse_single_json_line(&assert);
    assert_eq!(message["filesWritten"], 0);
    assert_eq!(message["filesUnchanged"], 4);
}

#[test]
fn save_refuses_to_overwrite_a_modified_file_without_force() {
    let (temp, mut first) = skills_cmd();
    let dest = temp.path().join("skills");
    first.args(["save", "--dir"]).arg(&dest).assert().success();

    let skill_md = dest.join("tvc-deployments/SKILL.md");
    std::fs::write(&skill_md, "locally modified").unwrap();

    let mut second = cargo_bin_cmd!("tvc");
    second.env_clear().env("HOME", temp.path());
    let assert = second
        .args(["skills", "save", "--dir"])
        .arg(&dest)
        .args(["--message-format", "json"])
        .assert()
        .failure();

    let message = parse_single_json_line(&assert);
    assert_eq!(message["reason"], "command_error");
    assert_eq!(message["code"], "command_error");
    assert!(
        message["message"]
            .as_str()
            .unwrap()
            .contains("tvc-deployments/SKILL.md")
    );

    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert_eq!(content, "locally modified", "refusal must not clobber");
}

#[test]
fn save_force_replaces_a_modified_file() {
    let (temp, mut first) = skills_cmd();
    let dest = temp.path().join("skills");
    first.args(["save", "--dir"]).arg(&dest).assert().success();

    let skill_md = dest.join("tvc-deployments/SKILL.md");
    std::fs::write(&skill_md, "locally modified").unwrap();

    let mut second = cargo_bin_cmd!("tvc");
    second.env_clear().env("HOME", temp.path());
    let assert = second
        .args(["skills", "save", "--force", "--dir"])
        .arg(&dest)
        .args(["--message-format", "json"])
        .assert()
        .success();

    let message = parse_single_json_line(&assert);
    assert_eq!(message["filesWritten"], 1);
    assert_eq!(message["filesUnchanged"], 3);

    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert!(content.starts_with("---"), "file must be restored");
}

#[test]
fn save_global_targets_the_home_skills_dir() {
    let (temp, mut cmd) = skills_cmd();

    cmd.args(["save", "--global"]).assert().success();

    assert!(
        temp.path()
            .join(".claude/skills/tvc-deployments/SKILL.md")
            .exists()
    );
}

#[test]
fn save_defaults_to_the_project_skills_dir() {
    let (temp, mut cmd) = skills_cmd();
    let project = temp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    cmd.arg("save").current_dir(&project).assert().success();

    assert!(
        project
            .join(".claude/skills/tvc-deployments/SKILL.md")
            .exists()
    );
}

#[test]
fn save_dir_conflicts_with_global() {
    let (_temp, mut cmd) = skills_cmd();

    cmd.args(["save", "--dir", "somewhere", "--global"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn save_human_output_is_not_json() {
    let (temp, mut cmd) = skills_cmd();
    let dest = temp.path().join("skills");

    cmd.args(["save", "--dir"])
        .arg(&dest)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""reason""#).not())
        .stdout(predicate::str::contains("tvc-deployments"));
}
