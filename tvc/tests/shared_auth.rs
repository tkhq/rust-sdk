use assert_cmd::{Command, cargo::cargo_bin_cmd};
use serde_json::{Value, json};
use std::{fs, path::Path};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

const ORG: &str = "00000000-0000-4000-8000-000000000001";
fn command(temp: &TempDir) -> Command {
    let mut command = cargo_bin_cmd!("tk");
    for name in [
        "TK_CONFIG",
        "TK_PROFILE",
        "TURNKEY_ORGANIZATION_ID",
        "TURNKEY_API_PUBLIC_KEY",
        "TURNKEY_API_PRIVATE_KEY",
        "TURNKEY_API_BASE_URL",
        "TVC_ORG_ID",
        "TVC_API_KEY_PUBLIC",
        "TVC_API_KEY_PRIVATE",
        "TVC_API_BASE_URL",
    ] {
        command.env_remove(name);
    }
    command
        .env("HOME", temp.path())
        .arg("--message-format=json");
    command
}
fn key(path: &Path) -> (String, String) {
    let key = TurnkeyP256ApiKey::generate();
    let public = hex::encode(key.compressed_public_key());
    let private = hex::encode(key.private_key());
    fs::write(
        path,
        serde_json::to_vec(&json!({"public_key": public, "private_key": private, "curve": "p256"}))
            .unwrap(),
    )
    .unwrap();
    (public, private)
}
fn registry(temp: &TempDir) -> (String, String) {
    let directory = temp.path().join(".config/turnkey");
    fs::create_dir_all(&directory).unwrap();
    let (admin, _) = key(&directory.join("admin.json"));
    let (agent, _) = key(&directory.join("agent.json"));
    fs::write(
        directory.join("tk.config.toml"),
        format!(
            r#"version = 1
active_profile = "admin"
[profiles.admin]
organization_id = "{ORG}"
api_base_url = "https://api.turnkey.com"
api_key_file = "{}/admin.json"
[profiles.agent]
organization_id = "{ORG}"
api_base_url = "https://api.turnkey.com"
api_key_file = "{}/agent.json"
"#,
            directory.display(),
            directory.display()
        ),
    )
    .unwrap();
    (admin, agent)
}
fn output(command: &mut Command) -> Value {
    let result = command.assert().success();
    serde_json::from_slice(&result.get_output().stdout).unwrap()
}
#[test]
fn two_identities_share_an_org_and_explicit_selection_ignores_ambient_credentials() {
    let temp = TempDir::new().unwrap();
    let (admin, agent) = registry(&temp);
    let result = output(command(&temp).args(["auth", "status"]));
    assert_eq!(result["data"]["publicKey"], admin);
    let result = output(
        command(&temp)
            .env("TURNKEY_API_PRIVATE_KEY", "unused-secret")
            .env("TVC_ORG_ID", "invalid")
            .args(["--profile", "agent", "auth", "status"]),
    );
    assert_eq!(
        result["data"],
        json!({"ready": true, "profile": "agent", "organizationId": ORG, "apiBaseUrl": "https://api.turnkey.com", "publicKey": agent, "credentialSource": "profile"})
    );
    assert!(!temp.path().join(".config/turnkey/tvc.config.toml").exists());
}
#[test]
fn environment_auth_does_not_require_home_or_read_bad_registry() {
    let temp = TempDir::new().unwrap();
    let (public, private) = key(&temp.path().join("key.json"));
    let config = temp.path().join("bad.toml");
    fs::write(&config, "not valid toml").unwrap();
    let result = output(
        command(&temp)
            .env_remove("HOME")
            .env("TK_CONFIG", &config)
            .env("TURNKEY_ORGANIZATION_ID", ORG)
            .env("TURNKEY_API_PUBLIC_KEY", &public)
            .env("TURNKEY_API_PRIVATE_KEY", private)
            .args(["auth", "status"]),
    );
    assert_eq!(result["data"]["credentialSource"], "environment");
    assert_eq!(result["data"]["publicKey"], public);
}
#[test]
fn partial_and_mixed_bundles_fail_without_leaking_secrets() {
    let temp = TempDir::new().unwrap();
    registry(&temp);
    for mixed in [false, true] {
        let mut cmd = command(&temp);
        cmd.env("TURNKEY_API_PRIVATE_KEY", "never-print-this");
        if mixed {
            cmd.env("TVC_ORG_ID", ORG);
        }
        let result = cmd.args(["auth", "status"]).assert().failure();
        let stdout = String::from_utf8_lossy(&result.get_output().stdout);
        assert!(!stdout.contains("never-print-this"));
        let parsed: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(parsed["schemaVersion"], 1);
    }
}
#[test]
fn malformed_selected_registry_reports_invalid_input_without_source_text() {
    let temp = TempDir::new().unwrap();
    registry(&temp);
    fs::write(
        temp.path().join(".config/turnkey/tk.config.toml"),
        "secret-pasted-on-invalid-line",
    )
    .unwrap();
    let result = command(&temp)
        .args(["--profile", "agent", "auth", "status"])
        .assert()
        .failure();
    let parsed: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(parsed["code"], "invalid_input");
    assert!(
        !parsed["message"]
            .as_str()
            .unwrap()
            .contains("secret-pasted")
    );
}
#[test]
fn profile_delete_and_logout_keep_credentials() {
    let temp = TempDir::new().unwrap();
    registry(&temp);
    output(command(&temp).args(["profile", "use", "agent"]));
    output(command(&temp).args(["auth", "logout"]));
    let list = output(command(&temp).args(["profile", "list"]));
    assert!(list["data"]["activeProfile"].is_null());
    output(command(&temp).args(["profile", "delete", "agent"]));
    assert!(temp.path().join(".config/turnkey/agent.json").exists());
    assert!(temp.path().join(".config/turnkey/admin.json").exists());
}
#[tokio::test]
async fn login_verifies_identity_and_saves_no_operator_state() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("key.json");
    key(&key_path);
    let server = MockServer::start().await;
    let identity = json!({"organizationId": ORG, "organizationName": "test", "userId": "user-1", "username": "alice"});
    Mock::given(method("POST"))
        .and(path("/public/v1/query/whoami"))
        .and(body_json(json!({"organizationId": ORG})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&identity))
        .expect(2)
        .mount(&server)
        .await;
    let login = output(
        command(&temp)
            .args([
                "--profile",
                "admin",
                "--organization-id",
                ORG,
                "--api-base-url",
                &server.uri(),
                "login",
                "--api-key-file",
            ])
            .arg(&key_path),
    );
    assert_eq!(login["data"]["identity"], identity);
    let whoami = output(command(&temp).arg("whoami"));
    assert_eq!(whoami["data"], identity);
    assert!(!temp.path().join(".config/turnkey/tvc.config.toml").exists());
    assert!(!temp.path().join(".config/turnkey/orgs").exists());
}

#[test]
fn experimental_import_preserves_source_and_does_not_select_or_leak_credentials() {
    let temp = TempDir::new().unwrap();
    let (public, private) = key(&temp.path().join("original-key.json"));
    let source = temp.path().join("legacy.toml");
    let original = format!(
        r#"[turnkey]
organizationId = "{ORG}"
apiPublicKey = "{public}"
apiPrivateKey = "{private}"
apiBaseUrl = "https://api.turnkey.com"
privateKeyId = "ssh-signing-key"
"#
    );
    fs::write(&source, &original).unwrap();
    let imported = output(
        command(&temp)
            .args([
                "profile",
                "import",
                "--from",
                "experimental-tk",
                "--name",
                "agent",
                "--source",
            ])
            .arg(&source),
    );
    assert_eq!(
        imported["data"],
        json!({"name": "agent", "selected": false})
    );
    let list = output(command(&temp).args(["profile", "list"]));
    assert!(list["data"]["activeProfile"].is_null());
    assert!(!list.to_string().contains(&private));
    let profile = &list["data"]["profiles"]["agent"];
    assert_eq!(profile["ssh_signing_key_id"], "ssh-signing-key");
    let credential = Path::new(profile["api_key_file"].as_str().unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(credential).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(fs::read_to_string(source).unwrap(), original);
    let status = output(command(&temp).args(["--profile", "agent", "auth", "status"]));
    assert_eq!(status["data"]["publicKey"], public);
}

#[test]
fn invalid_key_length_is_an_error_without_panic() {
    let temp = TempDir::new().unwrap();
    let result = command(&temp)
        .env("TURNKEY_ORGANIZATION_ID", ORG)
        .env("TURNKEY_API_PUBLIC_KEY", "00")
        .env("TURNKEY_API_PRIVATE_KEY", "01")
        .args(["auth", "status"])
        .assert()
        .code(1);
    assert!(result.get_output().stderr.is_empty());
    let parsed: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(parsed["reason"], "command_error");
}

#[test]
fn shared_selectors_cannot_silently_use_a_different_tvc_identity() {
    let temp = TempDir::new().unwrap();
    let result = command(&temp)
        .args(["--profile", "agent", "tvc", "app", "list"])
        .assert()
        .failure();
    let parsed: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(parsed["code"], "invalid_input");
    assert!(!temp.path().join(".config").exists());
}

#[test]
fn tvc_import_preserves_source_and_reuses_only_api_identity() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("tvc-key.json");
    let (public, _) = key(&key_path);
    let source = temp.path().join("tvc.config.toml");
    let original = format!(
        r#"version = 1
active_org = "dev"
[orgs.dev]
id = "{ORG}"
api_key_path = "{}"
api_base_url = "https://api.turnkey.com"
"#,
        key_path.display()
    );
    fs::write(&source, &original).unwrap();
    output(
        command(&temp)
            .args([
                "profile", "import", "--from", "tvc", "--name", "imported", "--source",
            ])
            .arg(&source),
    );
    let status = output(command(&temp).args(["--profile", "imported", "auth", "status"]));
    assert_eq!(status["data"]["publicKey"], public);
    assert_eq!(fs::read_to_string(source).unwrap(), original);
    assert!(!temp.path().join(".config/turnkey/orgs").exists());
}

#[test]
fn empty_environment_bundle_does_not_fall_back_to_saved_admin() {
    let temp = TempDir::new().unwrap();
    registry(&temp);
    let result = command(&temp)
        .env("TURNKEY_API_PRIVATE_KEY", "")
        .args(["auth", "status"])
        .assert()
        .code(1);
    let parsed: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(parsed["reason"], "command_error");
    output(command(&temp).env("TURNKEY_API_PRIVATE_KEY", "").args([
        "--profile",
        "agent",
        "auth",
        "status",
    ]));
}

#[cfg(unix)]
#[test]
fn nonunicode_credential_environment_does_not_fall_back() {
    use std::os::unix::ffi::OsStringExt;
    let temp = TempDir::new().unwrap();
    registry(&temp);
    command(&temp)
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            std::ffi::OsString::from_vec(vec![255]),
        )
        .args(["auth", "status"])
        .assert()
        .code(1);
}

#[test]
fn malformed_tvc_import_never_echoes_source_lines() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("bad.toml");
    fs::write(&source, "PRIVATE_MARKER_UNTERMINATED = \"").unwrap();
    let result = command(&temp)
        .args([
            "profile", "import", "--from", "tvc", "--name", "a", "--source",
        ])
        .arg(&source)
        .assert()
        .code(1);
    let text = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(!text.contains("PRIVATE_MARKER"));
    assert!(text.contains("bad.toml"));
}

#[test]
fn experimental_import_defaults_absent_endpoint_and_rejects_bad_explicit_endpoint() {
    let temp = TempDir::new().unwrap();
    let (public, private) = key(&temp.path().join("key.json"));
    let source = temp.path().join("legacy.toml");
    let contents = format!(
        r#"[turnkey]
organizationId = "{ORG}"
apiPublicKey = "{public}"
apiPrivateKey = "{private}"
"#
    );
    fs::write(&source, &contents).unwrap();
    output(
        command(&temp)
            .args([
                "profile",
                "import",
                "--from",
                "experimental-tk",
                "--name",
                "default",
                "--source",
            ])
            .arg(&source),
    );
    let selected = output(command(&temp).args(["--profile", "default", "auth", "status"]));
    assert_eq!(selected["data"]["apiBaseUrl"], "https://api.turnkey.com");
    let credentials = temp.path().join(".config/turnkey/credentials");
    let count = fs::read_dir(&credentials).unwrap().count();
    fs::write(&source, format!("{contents}apiBaseUrl = 'not a URL'\n")).unwrap();
    command(&temp)
        .args([
            "profile",
            "import",
            "--from",
            "experimental-tk",
            "--name",
            "bad",
            "--source",
        ])
        .arg(&source)
        .assert()
        .code(1);
    assert_eq!(fs::read_dir(credentials).unwrap().count(), count);
}
