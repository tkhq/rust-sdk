//! Wallet discovery and typed signing inputs for the shared CLI.
use crate::{
    shared_auth::ResolvedAuth,
    shared_operations::{OperationOutput, submit},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, to_value};
use std::{
    io::Read,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turnkey_client::generated::{
    GetWalletAccountsRequest, GetWalletRequest, GetWalletsRequest,
    external::activity::v1::{
        CreateWalletAccountsRequest, CreateWalletRequest, SignRawPayloadRequest,
        SignTransactionRequest, UpdateWalletRequest,
    },
    immutable::activity::v1::{
        CreateWalletAccountsIntent, CreateWalletIntent, SignRawPayloadIntentV2,
        SignTransactionIntentV2, UpdateWalletIntent,
    },
};
use uuid::Uuid;

/// Structured parameters are parsed before resolving any credentials.
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct JsonInput {
    #[arg(long)]
    input_json: Option<String>,
    /// JSON parameters file, or - for stdin.
    #[arg(long)]
    input_file: Option<PathBuf>,
}
impl JsonInput {
    fn parse<T: DeserializeOwned + Serialize>(self) -> Result<T> {
        let source = match (self.input_json, self.input_file) {
            (Some(value), None) => value,
            (None, Some(path)) if path.as_os_str() == "-" => {
                let mut value = String::new();
                std::io::stdin()
                    .read_to_string(&mut value)
                    .context("read JSON parameters from stdin")?;
                value
            }
            (None, Some(path)) => std::fs::read_to_string(&path)
                .with_context(|| format!("read JSON parameters from {}", path.display()))?,
            _ => anyhow::bail!("exactly one JSON input is required"),
        };
        let original: Value = serde_json::from_str(&source).context("parse command parameters")?;
        let parsed: T =
            serde_json::from_value(original.clone()).context("parse typed command parameters")?;
        reject_unknown_fields(&original, &to_value(&parsed)?)?;
        Ok(parsed)
    }
}

fn reject_unknown_fields(input: &Value, parsed: &Value) -> Result<()> {
    match (input, parsed) {
        (Value::Object(fields), Value::Object(known)) => {
            for (name, value) in fields {
                let normalized = known
                    .get(name)
                    .with_context(|| format!("unknown parameter {name}"))?;
                reject_unknown_fields(value, normalized)?;
            }
        }
        (Value::Array(values), Value::Array(known)) => {
            for (value, normalized) in values.iter().zip(known) {
                reject_unknown_fields(value, normalized)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Subcommand)]
pub enum WalletCommand {
    List,
    Get {
        id: Uuid,
    },
    Create(JsonInput),
    Update(JsonInput),
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
}
#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    List {
        #[arg(long)]
        wallet_id: Uuid,
    },
    Create(JsonInput),
}
#[derive(Debug, Subcommand)]
pub enum SignCommand {
    /// Sign a payload with explicit encoding and hash function in JSON input.
    Payload(JsonInput),
    /// Sign an already serialized transaction; does not broadcast.
    Transaction(JsonInput),
}

pub enum PreparedWalletCommand {
    List,
    Get(Uuid),
    Create(CreateWalletIntent),
    Update(UpdateWalletIntent),
    Accounts(Uuid),
    CreateAccounts(CreateWalletAccountsIntent),
    Payload(SignRawPayloadIntentV2),
    Transaction(SignTransactionIntentV2),
}
impl WalletCommand {
    pub fn prepare(self) -> Result<PreparedWalletCommand> {
        Ok(match self {
            Self::List => PreparedWalletCommand::List,
            Self::Get { id } => PreparedWalletCommand::Get(id),
            Self::Create(input) => PreparedWalletCommand::Create(input.parse()?),
            Self::Update(input) => {
                let params: UpdateWalletIntent = input.parse()?;
                Uuid::parse_str(&params.wallet_id).context("walletId must be a UUID")?;
                PreparedWalletCommand::Update(params)
            }
            Self::Account {
                command: AccountCommand::List { wallet_id },
            } => PreparedWalletCommand::Accounts(wallet_id),
            Self::Account {
                command: AccountCommand::Create(input),
            } => {
                let params: CreateWalletAccountsIntent = input.parse()?;
                Uuid::parse_str(&params.wallet_id).context("walletId must be a UUID")?;
                PreparedWalletCommand::CreateAccounts(params)
            }
        })
    }
}
impl SignCommand {
    pub fn prepare(self) -> Result<PreparedWalletCommand> {
        Ok(match self {
            Self::Payload(input) => PreparedWalletCommand::Payload(input.parse()?),
            Self::Transaction(input) => PreparedWalletCommand::Transaction(input.parse()?),
        })
    }
}
impl PreparedWalletCommand {
    pub async fn run(self, auth: ResolvedAuth) -> Result<OperationOutput> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let organization_id = auth.org_id.clone();
        let output = match self {
            Self::List => {
                let client = crate::client::build_turnkey_client(auth.stamper, &auth.api_base_url)?;
                let data = client
                    .get_wallets(GetWalletsRequest { organization_id })
                    .await?;
                OperationOutput::result("wallet.list", to_value(data)?)
            }
            Self::Get(id) => {
                let client = crate::client::build_turnkey_client(auth.stamper, &auth.api_base_url)?;
                let data = client
                    .get_wallet(GetWalletRequest {
                        organization_id,
                        wallet_id: id.to_string(),
                    })
                    .await?;
                data.wallet
                    .as_ref()
                    .ok_or_else(|| crate::errors::MissingResource::new("wallet", id.to_string()))?;
                OperationOutput::result("wallet.get", to_value(data)?)
            }
            Self::Accounts(id) => {
                let client = crate::client::build_turnkey_client(auth.stamper, &auth.api_base_url)?;
                let data = client
                    .get_wallet_accounts(GetWalletAccountsRequest {
                        organization_id,
                        wallet_id: Some(id.to_string()),
                        include_wallet_details: None,
                        pagination_options: None,
                    })
                    .await?;
                OperationOutput::result("wallet.account.list", to_value(data)?)
            }
            Self::Create(parameters) => {
                submit(
                    "wallet.create",
                    "/public/v1/submit/create_wallet",
                    &CreateWalletRequest {
                        r#type: "ACTIVITY_TYPE_CREATE_WALLET".into(),
                        timestamp_ms,
                        organization_id,
                        parameters: Some(parameters),
                        generate_app_proofs: None,
                    },
                    &auth.api_base_url,
                    &auth.stamper,
                )
                .await
            }
            Self::Update(parameters) => {
                submit(
                    "wallet.update",
                    "/public/v1/submit/update_wallet",
                    &UpdateWalletRequest {
                        r#type: "ACTIVITY_TYPE_UPDATE_WALLET".into(),
                        timestamp_ms,
                        organization_id,
                        parameters: Some(parameters),
                        generate_app_proofs: None,
                    },
                    &auth.api_base_url,
                    &auth.stamper,
                )
                .await
            }
            Self::CreateAccounts(parameters) => {
                submit(
                    "wallet.account.create",
                    "/public/v1/submit/create_wallet_accounts",
                    &CreateWalletAccountsRequest {
                        r#type: "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS".into(),
                        timestamp_ms,
                        organization_id,
                        parameters: Some(parameters),
                        generate_app_proofs: None,
                    },
                    &auth.api_base_url,
                    &auth.stamper,
                )
                .await
            }
            Self::Payload(parameters) => {
                submit(
                    "sign.payload",
                    "/public/v1/submit/sign_raw_payload",
                    &SignRawPayloadRequest {
                        r#type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2".into(),
                        timestamp_ms,
                        organization_id,
                        parameters: Some(parameters),
                        generate_app_proofs: None,
                    },
                    &auth.api_base_url,
                    &auth.stamper,
                )
                .await
            }
            Self::Transaction(parameters) => {
                submit(
                    "sign.transaction",
                    "/public/v1/submit/sign_transaction",
                    &SignTransactionRequest {
                        r#type: "ACTIVITY_TYPE_SIGN_TRANSACTION_V2".into(),
                        timestamp_ms,
                        organization_id,
                        parameters: Some(parameters),
                        generate_app_proofs: None,
                    },
                    &auth.api_base_url,
                    &auth.stamper,
                )
                .await
            }
        };
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[derive(Parser)]
    struct WalletParser {
        #[command(subcommand)]
        command: WalletCommand,
    }
    #[derive(Parser)]
    struct SignParser {
        #[command(subcommand)]
        command: SignCommand,
    }

    #[test]
    fn conflicting_or_missing_inputs_fail_during_parsing() {
        assert!(WalletParser::try_parse_from(["wallet", "create"]).is_err());
        assert!(
            WalletParser::try_parse_from([
                "wallet",
                "create",
                "--input-json",
                "{}",
                "--input-file",
                "x"
            ])
            .is_err()
        );
    }

    #[test]
    fn wallet_uuid_is_checked_before_authentication() {
        assert!(WalletParser::try_parse_from(["wallet", "get", "not-an-id"]).is_err());
        let parsed = WalletParser::try_parse_from([
            "wallet",
            "update",
            "--input-json",
            r#"{"walletId":"bad","walletName":"next"}"#,
        ])
        .unwrap();
        assert!(parsed.command.prepare().is_err());
    }

    #[test]
    fn signing_requires_explicit_algorithm_inputs() {
        let parsed = SignParser::try_parse_from([
            "sign",
            "payload",
            "--input-json",
            r#"{"signWith":"opaque-key","payload":"00"}"#,
        ])
        .unwrap();
        assert!(parsed.command.prepare().is_err());
    }

    #[test]
    fn signing_preserves_opaque_key_identifiers() {
        let parsed = SignParser::try_parse_from(["sign", "transaction", "--input-json", r#"{"signWith":"opaque-key","unsignedTransaction":"00","type":"TRANSACTION_TYPE_ETHEREUM"}"#]).unwrap();
        let PreparedWalletCommand::Transaction(params) = parsed.command.prepare().unwrap() else {
            panic!("expected prepared transaction")
        };
        assert_eq!(params.sign_with, "opaque-key");
    }
    #[tokio::test]
    async fn transaction_submission_is_typed_and_not_retried_when_pending() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{body_partial_json, method, path},
        };
        #[derive(Parser)]
        struct IdentityParser {
            #[command(flatten)]
            options: crate::shared_auth::AuthOptions,
        }
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let key = turnkey_api_key_stamper::TurnkeyP256ApiKey::generate();
        let key_path = dir.path().join("credential.json");
        let config_path = dir.path().join("config.toml");
        let key_value = crate::config::turnkey::StoredApiKey {
            public_key: hex::encode(key.compressed_public_key()),
            private_key: hex::encode(key.private_key()),
            curve: crate::config::turnkey::KeyCurve::P256,
        };
        std::fs::write(&key_path, serde_json::to_vec(&key_value).unwrap()).unwrap();
        let org = "00000000-0000-4000-8000-000000000001";
        std::fs::write(
            &config_path,
            format!(
                r#"version = 1
active_profile = "test"
[profiles.test]
organization_id = "{org}"
api_base_url = "{}"
api_key_file = "{}"
"#,
                server.uri(),
                key_path.display()
            ),
        )
        .unwrap();
        let parsed = IdentityParser::try_parse_from([
            "identity",
            "--config",
            config_path.to_str().unwrap(),
            "--profile",
            "test",
        ])
        .unwrap();
        let auth = crate::shared_auth::resolve(&parsed.options).await.unwrap();
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/sign_transaction"))
            .and(body_partial_json(serde_json::json!({"organizationId": org, "type": "ACTIVITY_TYPE_SIGN_TRANSACTION_V2", "parameters": {"signWith": "opaque-key", "unsignedTransaction": "00", "type": "TRANSACTION_TYPE_ETHEREUM"}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"activity":{"id":"pending-id","status":"ACTIVITY_STATUS_CONSENSUS_NEEDED"}})))
            .expect(1).mount(&server).await;
        let parsed = SignParser::try_parse_from(["sign", "transaction", "--input-json", r#"{"signWith":"opaque-key","unsignedTransaction":"00","type":"TRANSACTION_TYPE_ETHEREUM"}"#]).unwrap();
        let output = parsed.command.prepare().unwrap().run(auth).await.unwrap();
        assert!(!output.failed());
        let value = serde_json::to_value(output).unwrap();
        assert_eq!(value["status"], "pending");
        assert_eq!(value["activity"]["id"], "pending-id");
        server.verify().await;
        for response in [serde_json::json!({}), serde_json::json!({"wallet": null})] {
            server.reset().await;
            Mock::given(method("POST"))
                .and(path("/public/v1/query/get_wallet"))
                .respond_with(ResponseTemplate::new(200).set_body_json(response))
                .expect(1)
                .mount(&server)
                .await;
            let parsed = IdentityParser::try_parse_from([
                "identity",
                "--config",
                config_path.to_str().unwrap(),
                "--profile",
                "test",
            ])
            .unwrap();
            let auth = crate::shared_auth::resolve(&parsed.options).await.unwrap();
            let error = PreparedWalletCommand::Get(Uuid::parse_str(org).unwrap())
                .run(auth)
                .await
                .unwrap_err();
            assert!(
                error
                    .downcast_ref::<crate::errors::MissingResource>()
                    .is_some()
            );
            server.verify().await;
        }
    }
}
