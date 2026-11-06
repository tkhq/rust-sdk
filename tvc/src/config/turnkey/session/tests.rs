use super::*;
use crate::config::turnkey::{HostedOperatorRecord, OperatorKind, OperatorRecord, OrgConfig};
use std::{
    process::Command,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn absent_load_is_read_only_and_owner_advances_only_its_baseline() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    let mut first = ConfigSession::load(path.clone()).await.unwrap();
    let mut stale = ConfigSession::load(path.clone()).await.unwrap();
    assert!(!path.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    first
        .extra
        .insert("future".into(), toml::Value::String("preserved".into()));
    first.save().await.unwrap();
    first
        .extra
        .insert("owner".into(), toml::Value::Boolean(true));
    first.save().await.unwrap();
    let original = fs::read(&path).unwrap();
    stale.active_org = Some("stale".into());
    assert!(matches!(
        stale.save().await,
        Err(ConfigSaveError::StaleConflict { .. })
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(path.with_file_name("config.toml.lock").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
#[test]
fn pre_replacement_failure_preserves_bytes_and_post_failure_reports_actual_commit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, b"version = 1\n").unwrap();
    let original = fs::read(&path).unwrap();
    let next = b"version = 1\nactive_org = \"new\"\n";
    for failure in [FailurePoint::BeforeWrite, FailurePoint::BeforeReplace] {
        assert!(matches!(
            commit(
                &path,
                Baseline::Bytes(original.clone()),
                next,
                None,
                failure
            ),
            Err(ConfigSaveError::BeforeReplacement { .. })
        ));
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
    assert!(matches!(
        commit(
            &path,
            Baseline::Bytes(original),
            next,
            None,
            FailurePoint::AfterReplace
        ),
        Err(ConfigSaveError::DurabilityUncertain { .. })
    ));
    assert_eq!(fs::read(&path).unwrap(), next);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}
#[tokio::test]
async fn byte_changes_and_missing_versus_empty_are_conflicts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    let mut absent = ConfigSession::load(path.clone()).await.unwrap();
    fs::write(&path, b"").unwrap();
    assert!(matches!(
        absent.save().await,
        Err(ConfigSaveError::StaleConflict { .. })
    ));
    fs::write(&path, b"version = 1\n").unwrap();
    let mut loaded = ConfigSession::load(path.clone()).await.unwrap();
    fs::write(&path, b"version = 1\n# external editor\n").unwrap();
    assert!(matches!(
        loaded.save().await,
        Err(ConfigSaveError::StaleConflict { .. })
    ));
}
fn key(seed: u8) -> String {
    use p256::ecdsa::SigningKey;
    hex::encode(
        SigningKey::from_slice(&[seed; 32])
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes(),
    )
}
fn config_with_record(record: OperatorRecord) -> Config {
    Config {
        active_org: Some("active".into()),
        orgs: std::collections::BTreeMap::from([(
            "active".into(),
            OrgConfig {
                id: "org".into(),
                api_key_path: "api.json".into(),
                api_base_url: "http://localhost".into(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![record],
                extra: toml::Table::new(),
            },
        )]),
        ..Config::default()
    }
}
#[test]
fn legacy_both_role_inventory_and_missing_or_malformed_local_files() {
    let hosted = OperatorRecord {
        name: "legacy".into(),
        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
            operator_id: Uuid::nil(),
            wallet_id: Uuid::nil(),
            path: "m/0".into(),
            encrypt_public_key: key(1),
            sign_public_key: key(2),
            extra: toml::Table::new(),
        }),
    };
    let config = config_with_record(hosted);
    assert_eq!(
        config.operator_role_keys().unwrap(),
        vec![
            RolePublicKey::parse(&key(1)).unwrap(),
            RolePublicKey::parse(&key(2)).unwrap()
        ]
    );
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("operator.json");
    let local = config_with_record(OperatorRecord::local(file.clone()));
    assert!(local.operator_role_keys().is_err());
    fs::write(&file, "{}").unwrap();
    assert!(local.operator_role_keys().is_err());
    fs::write(&file,serde_json::to_vec(&serde_json::json!({"public_key":format!("{}{}",key(3),key(4)),"private_key":"not-used-for-inventory"})).unwrap()).unwrap();
    assert_eq!(local.operator_role_keys().unwrap().len(), 2);
}
#[tokio::test]
async fn insertion_rechecks_inventory_under_lock_including_changed_key_files() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("operator.json");
    let path = dir.path().join("config.toml");
    let write_key = |a, b| {
        fs::write(&file,serde_json::to_vec(&serde_json::json!({"public_key":format!("{}{}",key(a),key(b)),"private_key":"unused"})).unwrap()).unwrap()
    };
    write_key(1, 2);
    let config = config_with_record(OperatorRecord::local(file.clone()));
    fs::write(&path, disk::to_toml(&config).unwrap()).unwrap();
    let mut session = ConfigSession::load(path.clone()).await.unwrap();
    let original = fs::read(&path).unwrap();
    assert_eq!(session.operator_role_keys().unwrap().len(), 2);
    write_key(3, 4);
    let error = session
        .save_verified_roles([
            RolePublicKey::parse(&key(3)).unwrap(),
            RolePublicKey::parse(&key(5)).unwrap(),
        ])
        .await
        .unwrap_err();
    assert!(matches!(error, ConfigSaveError::BeforeReplacement { .. }));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .to_string()
            .contains("reuses a stored")
    );
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn subprocess_config_writer() {
    let Ok(directory) = std::env::var("TVC_CONFIG_SESSION_TEST_DIRECTORY") else {
        return;
    };
    let label = std::env::var("TVC_CONFIG_SESSION_TEST_LABEL").unwrap();
    let directory = PathBuf::from(directory);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let mut session = ConfigSession::load(directory.join("config.toml"))
            .await
            .unwrap();
        session.active_org = Some(label.clone());
        fs::write(directory.join(format!("ready-{label}")), b"").unwrap();
        let started = Instant::now();
        while !directory.join("go").exists() {
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
        let outcome = match session.save().await {
            Ok(()) => "saved",
            Err(ConfigSaveError::StaleConflict { .. }) => "conflict",
            Err(error) => panic!("unexpected save outcome {error}"),
        };
        fs::write(directory.join(format!("result-{label}")), outcome).unwrap();
    });
}
#[test]
fn separate_processes_serialize_on_stable_os_lock_and_stale_writer_loses() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "version = 1\n").unwrap();
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.path().join("config.toml.lock"))
        .unwrap();
    lock.lock().unwrap();
    let mut children = Vec::new();
    for label in ["first", "second"] {
        children.push(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "config::turnkey::session::tests::subprocess_config_writer",
                    "--nocapture",
                ])
                .env("TVC_CONFIG_SESSION_TEST_DIRECTORY", dir.path())
                .env("TVC_CONFIG_SESSION_TEST_LABEL", label)
                .spawn()
                .unwrap(),
        );
    }
    let started = Instant::now();
    while !["first", "second"]
        .iter()
        .all(|label| dir.path().join(format!("ready-{label}")).exists())
    {
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(5));
    }
    fs::write(dir.path().join("go"), b"").unwrap();
    std::thread::sleep(Duration::from_millis(80));
    assert!(!dir.path().join("result-first").exists());
    assert!(!dir.path().join("result-second").exists());
    drop(lock);
    for mut child in children {
        assert!(child.wait().unwrap().success());
    }
    let mut results = [
        fs::read_to_string(dir.path().join("result-first")).unwrap(),
        fs::read_to_string(dir.path().join("result-second")).unwrap(),
    ];
    results.sort();
    assert_eq!(results, ["conflict", "saved"]);
    let actual =
        Config::from_toml(&fs::read_to_string(dir.path().join("config.toml")).unwrap()).unwrap();
    assert!(matches!(
        actual.active_org.as_deref(),
        Some("first" | "second")
    ));
}

#[tokio::test]
async fn marker_and_unknown_fields_survive_without_granting_legacy_provenance() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    let mut hosted = HostedOperatorRecord {
        operator_id: Uuid::nil(),
        wallet_id: Uuid::nil(),
        path: "m/0".into(),
        encrypt_public_key: key(1),
        sign_public_key: key(2),
        extra: toml::Table::new(),
    };
    hosted
        .extra
        .insert("future-field".into(), toml::Value::Integer(42));
    let mut config = config_with_record(OperatorRecord {
        name: "legacy".into(),
        kind: OperatorRecordKind::Hosted(hosted),
    });
    config
        .extra
        .insert("future-top".into(), toml::Value::Boolean(true));
    fs::write(&path, disk::to_toml(&config).unwrap()).unwrap();
    let mut session = ConfigSession::load(path.clone()).await.unwrap();
    session.save().await.unwrap();
    let actual = Config::from_toml(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(actual, config);
    let OperatorRecordKind::Hosted(record) = &actual.orgs["active"].operators[0].kind else {
        panic!("hosted record expected")
    };
    assert!(!record.extra.contains_key("hosted_key_provenance"));
}
