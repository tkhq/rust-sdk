//! Shared user, credential, and policy operations with typed request preparation.

use std::{
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::{
    external::activity::v1 as activity, immutable::activity::v1 as intent,
    services::coordinator::public::v1 as query,
};
use uuid::Uuid;

use crate::{
    errors::MissingResource,
    shared_auth::ResolvedAuth,
    shared_operations::{OperationOutput, submit},
};

/// User and user-tag commands.
#[derive(Debug, Args)]
pub struct UserArgs {
    #[command(subcommand)]
    command: UserCommand,
}

#[derive(Debug, Subcommand)]
enum UserCommand {
    List,
    Get {
        id: Uuid,
    },
    /// Create one or more users from a CreateUsersIntentV4 parameters object.
    Create(BodyArgs),
    /// Update user name, email, phone, or tag membership.
    Update(BodyArgs),
    Delete {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<Uuid>,
    },
    Tag {
        #[command(subcommand)]
        command: TagCommand,
    },
}

#[derive(Debug, Subcommand)]
enum TagCommand {
    List,
    Create(BodyArgs),
    Update(BodyArgs),
    Delete {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<Uuid>,
    },
}

/// Policy management and server evaluation diagnostics.
#[derive(Debug, Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    command: PolicyCommand,
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    List,
    Get {
        id: Uuid,
    },
    /// Create a policy from a CreatePolicyIntentV3 parameters object.
    Create(BodyArgs),
    /// Create multiple policies from a parameters object containing policies.
    CreateBatch(BodyArgs),
    /// Update with policyEffect/policyCondition/policyConsensus field names.
    Update(BodyArgs),
    Delete {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<Uuid>,
    },
    Evaluations {
        activity_id: Uuid,
    },
}

/// Remote public API-key registration and revocation.
#[derive(Debug, Args)]
pub struct ApiKeyArgs {
    #[command(subcommand)]
    command: ApiKeyCommand,
}

#[derive(Debug, Subcommand)]
pub enum ApiKeyCommand {
    List {
        #[arg(long)]
        user_id: Option<Uuid>,
    },
    /// Register public keys using CreateApiKeysIntentV2 parameters.
    Register(BodyArgs),
    Delete {
        #[arg(long)]
        user_id: Uuid,
        #[arg(required = true, num_args = 1..)]
        ids: Vec<Uuid>,
    },
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct BodyArgs {
    /// Inline JSON parameters (no activity envelope).
    #[arg(long)]
    input_json: Option<String>,
    /// Read JSON parameters from a file, or - for stdin.
    #[arg(long)]
    input_file: Option<PathBuf>,
}

/// A command whose local input has been parsed before loading credentials.
pub struct PreparedResource {
    operation: Operation,
}

enum Operation {
    Users,
    User(Uuid),
    CreateUsers(intent::CreateUsersIntentV4),
    UpdateUser(intent::UpdateUserIntent),
    DeleteUsers(intent::DeleteUsersIntent),
    Tags,
    CreateTag(intent::CreateUserTagIntent),
    UpdateTag(intent::UpdateUserTagIntent),
    DeleteTags(intent::DeleteUserTagsIntent),
    Policies,
    Policy(Uuid),
    CreatePolicy(intent::CreatePolicyIntentV3),
    CreatePolicies(intent::CreatePoliciesIntent),
    UpdatePolicy(intent::UpdatePolicyIntentV2),
    DeletePolicy(intent::DeletePolicyIntent),
    DeletePolicies(intent::DeletePoliciesIntent),
    Evaluations(Uuid),
    ApiKeys(Option<Uuid>),
    RegisterKeys(intent::CreateApiKeysIntentV2),
    DeleteKeys(intent::DeleteApiKeysIntent),
}

impl BodyArgs {
    fn parse<T: DeserializeOwned + Serialize>(self) -> Result<T> {
        let bytes = match (self.input_json, self.input_file) {
            (Some(json), None) => json.into_bytes(),
            (None, Some(path)) if path.as_os_str() == "-" => {
                let mut bytes = Vec::new();
                io::stdin()
                    .read_to_end(&mut bytes)
                    .context("read JSON parameters from stdin")?;
                bytes
            }
            (None, Some(path)) => std::fs::read(&path)
                .with_context(|| format!("read JSON parameters from {}", path.display()))?,
            _ => bail!("provide exactly one of --input-json or --input-file"),
        };
        parse_parameters(&bytes)
    }
}

fn parse_parameters<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    let mut value: Value = serde_json::from_slice(bytes).context("parse JSON parameters")?;
    if !value.is_object() {
        bail!("parameters must be a JSON object");
    }
    normalize_ids(&mut value)?;
    let typed: T = serde_json::from_value(value.clone()).context("invalid operation parameters")?;
    let normalized = serde_json::to_value(&typed)?;
    reject_dropped_fields(&value, &normalized, "parameters")?;
    Ok(typed)
}

// Known UUID fields in this resource surface are parsed and canonicalized once.
fn normalize_ids(value: &mut Value) -> Result<()> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(key.as_str(), "userId" | "userTagId" | "policyId") {
                    let id =
                        Uuid::parse_str(value.as_str().context("resource ID must be a string")?)
                            .with_context(|| format!("invalid UUID in {key}"))?;
                    *value = Value::String(id.to_string());
                } else if matches!(
                    key.as_str(),
                    "userIds"
                        | "userTags"
                        | "userTagIds"
                        | "addUserIds"
                        | "removeUserIds"
                        | "apiKeyIds"
                        | "policyIds"
                ) {
                    let ids = value
                        .as_array_mut()
                        .context("resource IDs must be an array")?;
                    for value in ids {
                        let id = Uuid::parse_str(
                            value.as_str().context("resource ID must be a string")?,
                        )
                        .with_context(|| format!("invalid UUID in {key}"))?;
                        *value = Value::String(id.to_string());
                    }
                } else {
                    normalize_ids(value)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                normalize_ids(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// Generated structs accept unknown fields. Reject them before they can silently
// disappear from a submitted policy or credential request.
fn reject_dropped_fields(input: &Value, normalized: &Value, path: &str) -> Result<()> {
    match (input, normalized) {
        (Value::Object(input), Value::Object(normalized)) => {
            for (key, value) in input {
                let Some(known) = normalized.get(key) else {
                    bail!("unsupported field {path}.{key}");
                };
                reject_dropped_fields(value, known, &format!("{path}.{key}"))?;
            }
        }
        (Value::Array(input), Value::Array(normalized)) => {
            for (i, (value, known)) in input.iter().zip(normalized).enumerate() {
                reject_dropped_fields(value, known, &format!("{path}[{i}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl UserArgs {
    pub fn prepare(self) -> Result<PreparedResource> {
        let operation = match self.command {
            UserCommand::List => Operation::Users,
            UserCommand::Get { id } => Operation::User(id),
            UserCommand::Create(body) => {
                let params: intent::CreateUsersIntentV4 = body.parse()?;
                if params.users.is_empty() {
                    bail!("users must contain at least one user");
                }
                Operation::CreateUsers(params)
            }
            UserCommand::Update(body) => Operation::UpdateUser(body.parse()?),
            UserCommand::Delete { ids } => Operation::DeleteUsers(intent::DeleteUsersIntent {
                user_ids: ids.into_iter().map(|id| id.to_string()).collect(),
            }),
            UserCommand::Tag { command } => match command {
                TagCommand::List => Operation::Tags,
                TagCommand::Create(body) => Operation::CreateTag(body.parse()?),
                TagCommand::Update(body) => Operation::UpdateTag(body.parse()?),
                TagCommand::Delete { ids } => Operation::DeleteTags(intent::DeleteUserTagsIntent {
                    user_tag_ids: ids.into_iter().map(|id| id.to_string()).collect(),
                }),
            },
        };
        Ok(PreparedResource { operation })
    }
}

impl PolicyArgs {
    pub fn prepare(self) -> Result<PreparedResource> {
        let operation = match self.command {
            PolicyCommand::List => Operation::Policies,
            PolicyCommand::Get { id } => Operation::Policy(id),
            PolicyCommand::Create(body) => Operation::CreatePolicy(body.parse()?),
            PolicyCommand::CreateBatch(body) => {
                let params: intent::CreatePoliciesIntent = body.parse()?;
                if params.policies.is_empty() {
                    bail!("policies must contain at least one policy");
                }
                Operation::CreatePolicies(params)
            }
            PolicyCommand::Update(body) => Operation::UpdatePolicy(body.parse()?),
            PolicyCommand::Delete { ids } => {
                let mut ids = ids.into_iter().map(|id| id.to_string());
                let first = ids.next().context("at least one policy ID is required")?;
                let remaining: Vec<_> = ids.collect();
                if remaining.is_empty() {
                    Operation::DeletePolicy(intent::DeletePolicyIntent { policy_id: first })
                } else {
                    Operation::DeletePolicies(intent::DeletePoliciesIntent {
                        policy_ids: std::iter::once(first).chain(remaining).collect(),
                    })
                }
            }
            PolicyCommand::Evaluations { activity_id } => Operation::Evaluations(activity_id),
        };
        Ok(PreparedResource { operation })
    }
}

impl From<ApiKeyCommand> for ApiKeyArgs {
    fn from(command: ApiKeyCommand) -> Self {
        Self { command }
    }
}

impl ApiKeyArgs {
    pub fn prepare(self) -> Result<PreparedResource> {
        let operation = match self.command {
            ApiKeyCommand::List { user_id } => Operation::ApiKeys(user_id),
            ApiKeyCommand::Register(body) => {
                let params: intent::CreateApiKeysIntentV2 = body.parse()?;
                if params.api_keys.is_empty() {
                    bail!("apiKeys must contain at least one public key");
                }
                Operation::RegisterKeys(params)
            }
            ApiKeyCommand::Delete { user_id, ids } => {
                Operation::DeleteKeys(intent::DeleteApiKeysIntent {
                    user_id: user_id.to_string(),
                    api_key_ids: ids.into_iter().map(|id| id.to_string()).collect(),
                })
            }
        };
        Ok(PreparedResource { operation })
    }
}

impl PreparedResource {
    pub fn command(&self) -> &'static str {
        match &self.operation {
            Operation::Users => "user.list",
            Operation::User(_) => "user.get",
            Operation::CreateUsers(_) => "user.create",
            Operation::UpdateUser(_) => "user.update",
            Operation::DeleteUsers(_) => "user.delete",
            Operation::Tags => "user.tag.list",
            Operation::CreateTag(_) => "user.tag.create",
            Operation::UpdateTag(_) => "user.tag.update",
            Operation::DeleteTags(_) => "user.tag.delete",
            Operation::Policies => "policy.list",
            Operation::Policy(_) => "policy.get",
            Operation::CreatePolicy(_) => "policy.create",
            Operation::CreatePolicies(_) => "policy.create-batch",
            Operation::UpdatePolicy(_) => "policy.update",
            Operation::DeletePolicy(_) | Operation::DeletePolicies(_) => "policy.delete",
            Operation::Evaluations(_) => "policy.evaluations",
            Operation::ApiKeys(_) => "api-key.list",
            Operation::RegisterKeys(_) => "api-key.register",
            Operation::DeleteKeys(_) => "api-key.delete",
        }
    }

    pub async fn run(self, auth: ResolvedAuth) -> Result<OperationOutput> {
        self.run_with_credentials(auth.org_id, auth.api_base_url, auth.stamper)
            .await
    }

    async fn run_with_credentials(
        self,
        organization_id: String,
        api_base_url: String,
        stamper: TurnkeyP256ApiKey,
    ) -> Result<OperationOutput> {
        let command = self.command();
        macro_rules! submit_operation {
            ($params:expr, $request:ident, $kind:literal, $endpoint:literal) => {{
                let request = activity::$request {
                    r#type: $kind.to_owned(),
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis()
                        .to_string(),
                    organization_id,
                    parameters: Some($params),
                    generate_app_proofs: None,
                };
                submit(
                    command,
                    concat!("/public/v1/submit/", $endpoint),
                    &request,
                    &api_base_url,
                    &stamper,
                )
                .await
            }};
        }
        // Query clients are created only on read paths. Mutation transport always
        // disables redirects and preserves uncertain submission outcomes.
        macro_rules! client {
            () => {
                crate::client::build_turnkey_client(stamper, &api_base_url)?
            };
        }
        let response = match self.operation {
            Operation::Users => OperationOutput::result(
                command,
                serde_json::to_value(
                    client!()
                        .get_users(query::GetUsersRequest { organization_id })
                        .await?,
                )?,
            ),
            Operation::User(id) => {
                let response = client!()
                    .get_user(query::GetUserRequest {
                        organization_id,
                        user_id: id.to_string(),
                    })
                    .await?;
                if response.user.is_none() {
                    return Err(MissingResource::new("user", id.to_string()).into());
                }
                OperationOutput::result(command, serde_json::to_value(response)?)
            }
            Operation::CreateUsers(params) => submit_operation!(
                params,
                CreateUsersRequest,
                "ACTIVITY_TYPE_CREATE_USERS_V4",
                "create_users"
            ),
            Operation::UpdateUser(params) => submit_operation!(
                params,
                UpdateUserRequest,
                "ACTIVITY_TYPE_UPDATE_USER",
                "update_user"
            ),
            Operation::DeleteUsers(params) => submit_operation!(
                params,
                DeleteUsersRequest,
                "ACTIVITY_TYPE_DELETE_USERS",
                "delete_users"
            ),
            Operation::Tags => OperationOutput::result(
                command,
                serde_json::to_value(
                    client!()
                        .list_user_tags(query::ListUserTagsRequest { organization_id })
                        .await?,
                )?,
            ),
            Operation::CreateTag(params) => submit_operation!(
                params,
                CreateUserTagRequest,
                "ACTIVITY_TYPE_CREATE_USER_TAG",
                "create_user_tag"
            ),
            Operation::UpdateTag(params) => submit_operation!(
                params,
                UpdateUserTagRequest,
                "ACTIVITY_TYPE_UPDATE_USER_TAG",
                "update_user_tag"
            ),
            Operation::DeleteTags(params) => submit_operation!(
                params,
                DeleteUserTagsRequest,
                "ACTIVITY_TYPE_DELETE_USER_TAGS",
                "delete_user_tags"
            ),
            Operation::Policies => OperationOutput::result(
                command,
                serde_json::to_value(
                    client!()
                        .get_policies(query::GetPoliciesRequest { organization_id })
                        .await?,
                )?,
            ),
            Operation::Policy(id) => {
                let response = client!()
                    .get_policy(query::GetPolicyRequest {
                        organization_id,
                        policy_id: id.to_string(),
                    })
                    .await?;
                if response.policy.is_none() {
                    return Err(MissingResource::new("policy", id.to_string()).into());
                }
                OperationOutput::result(command, serde_json::to_value(response)?)
            }
            Operation::CreatePolicy(params) => submit_operation!(
                params,
                CreatePolicyRequest,
                "ACTIVITY_TYPE_CREATE_POLICY_V3",
                "create_policy"
            ),
            Operation::CreatePolicies(params) => submit_operation!(
                params,
                CreatePoliciesRequest,
                "ACTIVITY_TYPE_CREATE_POLICIES",
                "create_policies"
            ),
            Operation::UpdatePolicy(params) => submit_operation!(
                params,
                UpdatePolicyRequest,
                "ACTIVITY_TYPE_UPDATE_POLICY_V2",
                "update_policy"
            ),
            Operation::DeletePolicy(params) => submit_operation!(
                params,
                DeletePolicyRequest,
                "ACTIVITY_TYPE_DELETE_POLICY",
                "delete_policy"
            ),
            Operation::DeletePolicies(params) => submit_operation!(
                params,
                DeletePoliciesRequest,
                "ACTIVITY_TYPE_DELETE_POLICIES",
                "delete_policies"
            ),
            Operation::Evaluations(id) => OperationOutput::result(
                command,
                serde_json::to_value(
                    client!()
                        .get_policy_evaluations(query::GetPolicyEvaluationsRequest {
                            organization_id,
                            activity_id: id.to_string(),
                        })
                        .await?,
                )?,
            ),
            Operation::ApiKeys(user_id) => OperationOutput::result(
                command,
                serde_json::to_value(
                    client!()
                        .get_api_keys(query::GetApiKeysRequest {
                            organization_id,
                            user_id: user_id.map(|id| id.to_string()),
                        })
                        .await?,
                )?,
            ),
            Operation::RegisterKeys(params) => submit_operation!(
                params,
                CreateApiKeysRequest,
                "ACTIVITY_TYPE_CREATE_API_KEYS_V2",
                "create_api_keys"
            ),
            Operation::DeleteKeys(params) => submit_operation!(
                params,
                DeleteApiKeysRequest,
                "ACTIVITY_TYPE_DELETE_API_KEYS",
                "delete_api_keys"
            ),
        };
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    const ID: &str = "11111111-1111-4111-8111-111111111111";
    const OTHER: &str = "22222222-2222-4222-8222-222222222222";

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommand,
    }

    #[derive(Subcommand)]
    enum TestCommand {
        User(UserArgs),
        Policy(PolicyArgs),
        ApiKey(ApiKeyArgs),
    }

    fn prepare(args: &[&str]) -> Result<PreparedResource> {
        let cli = TestCli::try_parse_from(std::iter::once("tk").chain(args.iter().copied()))?;
        match cli.command {
            TestCommand::User(args) => args.prepare(),
            TestCommand::Policy(args) => args.prepare(),
            TestCommand::ApiKey(args) => args.prepare(),
        }
    }

    #[test]
    fn malformed_and_unsupported_inputs_fail_before_auth() {
        for args in [
            vec!["user", "get", "not-a-uuid"],
            vec!["user", "delete"],
            vec!["policy", "delete"],
            vec!["policy", "list", "--cursor", "invented"],
            vec![
                "policy",
                "create",
                "--input-json",
                "{}",
                "--input-file",
                "-",
            ],
            vec!["policy", "create"],
            vec!["user", "create", "--input-json", r#"{"users":[]}"#],
            vec![
                "api-key",
                "register",
                "--input-json",
                r#"{"userId":"bad","apiKeys":[]}"#,
            ],
            vec![
                "policy",
                "update",
                "--input-json",
                r#"{"policyId":"11111111-1111-4111-8111-111111111111","effect":"EFFECT_ALLOW"}"#,
            ],
            vec![
                "user",
                "create",
                "--input-json",
                r#"{"users":[{"userName":"agent","apiKeys":[{"apiKeyName":"key","publicKey":"03ab","curveType":"API_KEY_CURVE_P256","privateKey":"secret"}]}]}"#,
            ],
        ] {
            assert!(
                prepare(&args).is_err(),
                "input unexpectedly accepted: {args:?}"
            );
        }
    }

    #[test]
    fn policy_file_preserves_exact_expressions() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let params = json!({
            "policyName": "agent policy",
            "effect": "EFFECT_ALLOW",
            "condition": "activity.action == 'SIGN' &&\n  wallet.id == 'literal'",
            "consensus": "approvers.any(user, user.id == 'literal')",
            "notes": "Reviewed policy",
            "time": null
        });
        std::fs::write(file.path(), serde_json::to_vec(&params).unwrap()).unwrap();
        let body = file.path().display().to_string();
        let command = prepare(&["policy", "create", "--input-file", &body]).unwrap();
        match command.operation {
            Operation::CreatePolicy(actual) => {
                assert_eq!(serde_json::to_value(actual).unwrap(), params)
            }
            _ => panic!("expected prepared policy creation"),
        }
    }

    #[tokio::test]
    async fn all_mutations_use_versioned_envelopes_and_submit_once() {
        let user_body = r#"{"users":[{"userName":"agent","userTags":["11111111-1111-4111-8111-111111111111"]}]}"#;
        let update_user =
            r#"{"userId":"11111111-1111-4111-8111-111111111111","userName":"renamed"}"#;
        let tag_body =
            r#"{"userTagName":"agents","userIds":["11111111-1111-4111-8111-111111111111"]}"#;
        let update_tag = r#"{"userTagId":"11111111-1111-4111-8111-111111111111","addUserIds":["22222222-2222-4222-8222-222222222222"]}"#;
        let policy_body = r#"{"policyName":"agent","effect":"EFFECT_ALLOW","condition":"true","consensus":"true","notes":"test"}"#;
        let policies_body =
            r#"{"policies":[{"policyName":"agent","effect":"EFFECT_ALLOW","notes":"test"}]}"#;
        let update_policy = r#"{"policyId":"11111111-1111-4111-8111-111111111111","policyEffect":"EFFECT_DENY","policyCondition":"true"}"#;
        let keys_body = r#"{"userId":"11111111-1111-4111-8111-111111111111","apiKeys":[{"apiKeyName":"agent","publicKey":"03ab","curveType":"API_KEY_CURVE_P256"}]}"#;
        let cases = [
            (
                vec!["user", "create", "--input-json", user_body],
                "create_users",
                "ACTIVITY_TYPE_CREATE_USERS_V4",
            ),
            (
                vec!["user", "update", "--input-json", update_user],
                "update_user",
                "ACTIVITY_TYPE_UPDATE_USER",
            ),
            (
                vec!["user", "delete", ID],
                "delete_users",
                "ACTIVITY_TYPE_DELETE_USERS",
            ),
            (
                vec!["user", "tag", "create", "--input-json", tag_body],
                "create_user_tag",
                "ACTIVITY_TYPE_CREATE_USER_TAG",
            ),
            (
                vec!["user", "tag", "update", "--input-json", update_tag],
                "update_user_tag",
                "ACTIVITY_TYPE_UPDATE_USER_TAG",
            ),
            (
                vec!["user", "tag", "delete", ID],
                "delete_user_tags",
                "ACTIVITY_TYPE_DELETE_USER_TAGS",
            ),
            (
                vec!["policy", "create", "--input-json", policy_body],
                "create_policy",
                "ACTIVITY_TYPE_CREATE_POLICY_V3",
            ),
            (
                vec!["policy", "create-batch", "--input-json", policies_body],
                "create_policies",
                "ACTIVITY_TYPE_CREATE_POLICIES",
            ),
            (
                vec!["policy", "update", "--input-json", update_policy],
                "update_policy",
                "ACTIVITY_TYPE_UPDATE_POLICY_V2",
            ),
            (
                vec!["policy", "delete", ID],
                "delete_policy",
                "ACTIVITY_TYPE_DELETE_POLICY",
            ),
            (
                vec!["policy", "delete", ID, OTHER],
                "delete_policies",
                "ACTIVITY_TYPE_DELETE_POLICIES",
            ),
            (
                vec!["api-key", "register", "--input-json", keys_body],
                "create_api_keys",
                "ACTIVITY_TYPE_CREATE_API_KEYS_V2",
            ),
            (
                vec!["api-key", "delete", "--user-id", ID, OTHER],
                "delete_api_keys",
                "ACTIVITY_TYPE_DELETE_API_KEYS",
            ),
        ];
        for (args, endpoint, kind) in cases {
            for status in [
                "ACTIVITY_STATUS_PENDING",
                "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "ACTIVITY_STATUS_REJECTED",
                "ACTIVITY_STATUS_FAILED",
                "ACTIVITY_STATUS_COMPLETED",
            ] {
                let server = MockServer::start().await;
                let expected: activity::Activity = serde_json::from_value(json!({
                    "id": OTHER, "organizationId": ID, "type": kind,
                    "status": status, "fingerprint": "fixture"
                }))
                .unwrap();
                Mock::given(method("POST"))
                    .and(path(format!("/public/v1/submit/{endpoint}")))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_json(json!({"activity": expected})),
                    )
                    .expect(1)
                    .mount(&server)
                    .await;
                let result = prepare(&args)
                    .unwrap()
                    .run_with_credentials(
                        ID.to_owned(),
                        server.uri(),
                        TurnkeyP256ApiKey::generate(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(result).unwrap()["data"]["activity"],
                    serde_json::to_value(expected).unwrap()
                );
                let requests = server.received_requests().await.unwrap();
                assert_eq!(requests.len(), 1);
                let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
                assert_eq!(body["type"], kind);
                assert_eq!(body["organizationId"], ID);
                assert!(
                    body["timestampMs"]
                        .as_str()
                        .unwrap()
                        .parse::<u128>()
                        .unwrap()
                        > 0
                );
                assert!(requests[0].headers.contains_key("x-stamp"));
                if endpoint == "update_policy" {
                    assert_eq!(
                        body["parameters"],
                        json!({
                            "policyId": ID, "policyName": null, "policyEffect": "EFFECT_DENY",
                            "policyCondition": "true", "policyConsensus": null, "policyNotes": null, "time": null
                        })
                    );
                }
                server.verify().await;
            }
        }
    }

    #[tokio::test]
    async fn queries_use_supported_shapes_and_preserve_results() {
        let cases = [
            (
                vec!["user", "list"],
                "list_users",
                json!({"organizationId": ID}),
                json!({"users": []}),
            ),
            (
                vec!["user", "tag", "list"],
                "list_user_tags",
                json!({"organizationId": ID}),
                json!({"userTags": []}),
            ),
            (
                vec!["policy", "list"],
                "list_policies",
                json!({"organizationId": ID}),
                json!({"policies": []}),
            ),
            (
                vec!["policy", "evaluations", OTHER],
                "get_policy_evaluations",
                json!({"organizationId": ID,"activityId": OTHER}),
                json!({"policyEvaluations": []}),
            ),
            (
                vec!["api-key", "list", "--user-id", OTHER],
                "get_api_keys",
                json!({"organizationId": ID,"userId": OTHER}),
                json!({"apiKeys": []}),
            ),
        ];
        for (args, endpoint, request, response) in cases {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path(format!("/public/v1/query/{endpoint}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(&response))
                .expect(1)
                .mount(&server)
                .await;
            let result = prepare(&args)
                .unwrap()
                .run_with_credentials(ID.to_owned(), server.uri(), TurnkeyP256ApiKey::generate())
                .await
                .unwrap();
            assert_eq!(serde_json::to_value(result).unwrap()["data"], response);
            let requests = server.received_requests().await.unwrap();
            assert_eq!(
                serde_json::from_slice::<Value>(&requests[0].body).unwrap(),
                request
            );
        }
    }

    #[tokio::test]
    async fn missing_lookup_preserves_typed_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/get_user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"user": null})))
            .mount(&server)
            .await;
        let result = prepare(&["user", "get", ID])
            .unwrap()
            .run_with_credentials(ID.to_owned(), server.uri(), TurnkeyP256ApiKey::generate())
            .await;
        match result {
            Err(error) => assert!(error.downcast_ref::<MissingResource>().is_some()),
            Ok(_) => panic!("missing user was accepted"),
        }
    }
    #[tokio::test]
    async fn mutation_transport_refuses_redirects_and_preserves_uncertain_outcomes() {
        for (template, expected_code) in [
            (
                ResponseTemplate::new(307).insert_header("Location", "/redirect-target"),
                "api_error",
            ),
            (
                ResponseTemplate::new(308).insert_header("Location", "/redirect-target"),
                "api_error",
            ),
            (
                ResponseTemplate::new(200).set_body_string("not JSON"),
                "submission_unknown",
            ),
            (
                ResponseTemplate::new(200).set_body_json(json!({})),
                "submission_unknown",
            ),
            (
                ResponseTemplate::new(401).set_body_json(json!({"message":"denied"})),
                "unauthorized",
            ),
            (
                ResponseTemplate::new(403).set_body_json(json!({"message":"denied"})),
                "unauthorized",
            ),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/public/v1/submit/delete_users"))
                .respond_with(template)
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(path("/redirect-target"))
                .respond_with(ResponseTemplate::new(200))
                .expect(0)
                .mount(&server)
                .await;
            let output = prepare(&["user", "delete", ID])
                .unwrap()
                .run_with_credentials(ID.to_owned(), server.uri(), TurnkeyP256ApiKey::generate())
                .await
                .unwrap();
            assert!(output.failed());
            let record = serde_json::to_value(output).unwrap();
            assert_eq!(record["command"], "user.delete");
            assert_eq!(record["code"], expected_code);
            if expected_code == "submission_unknown" {
                assert_eq!(record["status"], "unknown");
            }
            assert_eq!(server.received_requests().await.unwrap().len(), 1);
            server.verify().await;
        }
    }

    #[tokio::test]
    async fn mutation_timeout_is_unknown_and_never_resubmitted() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/delete_users"))
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(31)))
            .expect(1)
            .mount(&server)
            .await;
        let output = prepare(&["user", "delete", ID])
            .unwrap()
            .run_with_credentials(ID.to_owned(), server.uri(), TurnkeyP256ApiKey::generate())
            .await
            .unwrap();
        let record = serde_json::to_value(output).unwrap();
        assert_eq!(record["code"], "submission_unknown");
        assert_eq!(record["status"], "unknown");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
